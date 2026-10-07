//! Actual planner choices for threat approaches and useful local careers.

use kestrum::{
    data::{
        economy::{Resources, TroopKind},
        progression::TrainingDiscipline,
        world::{DiplomaticState, FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{advance_npc, ai, apply, Actor, Command},
    state::{
        ai::{AiObjective, AiObjectiveKind},
        construction::Focus,
        military::ArmyId,
        people::{PersonAssignment, PersonId},
        threat::ThreatStatus,
        Campaign, StrategicCampaign,
    },
};

const NPC: FactionId = FactionId(2);

fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    fund(&mut campaign, 0);
    (data, campaign)
}

fn fund(campaign: &mut StrategicCampaign, gold: i64) {
    campaign.factions.get_mut(&NPC).unwrap().resources = Resources {
        gold,
        wood: 0,
        stone: 0,
    };
    for site in campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(NPC))
    {
        campaign.world.focus.insert(site.id, Focus::Gold);
    }
}

fn next_turn(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round || campaign.active_faction() != NPC {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
}

fn reload(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let loaded: Campaign = serde_json::from_str(
        &serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap(),
    )
    .unwrap();
    loaded.validate(data).unwrap();
    loaded.strategic().unwrap().clone()
}

#[test]
fn npc_selects_a_known_adjacent_threat_and_clears_it_once_across_reload() {
    let (data, mut campaign) = fixture();
    for site in [5, 6, 7] {
        campaign
            .set_site_control(&data, SiteId(site), Some(NPC), false)
            .unwrap();
    }
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(7);
    fund(&mut campaign, 0);
    let decision = ai::propose(&campaign, &data, NPC).unwrap();
    let Command::ClearThreat { threat, .. } = decision.command else {
        panic!("{decision:?}");
    };
    let before = campaign.factions[&NPC].resources;
    advance_npc(&mut campaign, &data).unwrap();
    let reward = data.threats.definitions[&campaign.threats[&threat].kind].reward;
    assert_eq!(
        campaign.factions[&NPC].resources.gold,
        before.gold + reward.gold
    );
    let mut loaded = reload(&campaign, &data);
    assert!(apply(&mut loaded, &data, Actor::Npc(NPC), decision.command).is_err());
    assert_eq!(loaded, campaign);
}

#[test]
fn npc_approaches_distant_observed_threat_without_entering_it_or_using_unknown_sites() {
    let (data, mut campaign) = fixture();
    let formation = campaign.armies[&ArmyId(2)].formation_ids().last().unwrap();
    let spotter = apply(
        &mut campaign,
        &data,
        Actor::Npc(NPC),
        Command::SplitArmy { formation },
    )
    .unwrap()
    .split_army
    .unwrap();
    for site in [5, 6, 7] {
        campaign
            .set_site_control(&data, SiteId(site), Some(NPC), false)
            .unwrap();
    }
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(6);
    campaign.armies.get_mut(&spotter).unwrap().site = SiteId(7);
    let entry = campaign.formations.get_mut(&formation).unwrap();
    entry.movement_spent = entry.movement_allowance(&data);
    fund(&mut campaign, 0);
    let decision = ai::propose(&campaign, &data, NPC).unwrap();
    assert!(
        matches!(&decision.command, Command::Move(order) if order.path == [SiteId(6), SiteId(7)])
    );
    assert_eq!(
        decision.objective.unwrap().kind,
        kestrum::state::ai::AiObjectiveKind::Threat
    );
    campaign
        .set_site_control(&data, SiteId(7), Some(campaign.player), false)
        .unwrap();
    let blocked = ai::propose(&campaign, &data, NPC).unwrap();
    assert!(
        !matches!(blocked.objective, Some(objective) if objective.kind == kestrum::state::ai::AiObjectiveKind::Threat)
    );
    campaign.armies.get_mut(&spotter).unwrap().site = SiteId(2);
    let unknown = ai::propose(&campaign, &data, NPC).unwrap();
    assert!(!matches!(unknown.command, Command::ClearThreat { .. }));
}

#[test]
fn border_post_stays_fixed_during_war_and_stops_being_an_objective_at_peace() {
    let (mut data, mut campaign, objective, spotter) = border_post_fixture();
    // Keep this frontier regression from adding unrelated new wars after its scout split.
    data.ai.target_armies = 3;
    set_relation(&mut campaign, FactionId(1), DiplomaticState::War);
    set_last_offer_this_round(&mut campaign, FactionId(1));
    let approach = ai::propose(&campaign, &data, NPC).unwrap();
    assert!(
        matches!(&approach.command, Command::Move(order)
            if order.path == [SiteId(6), SiteId(5)]),
        "{approach:?}"
    );
    assert_eq!(approach.objective, Some(objective.clone()));
    assert_eq!(approach, ai::propose(&campaign, &data, NPC).unwrap());
    advance_npc(&mut campaign, &data).unwrap();
    assert_eq!(campaign.armies[&ArmyId(2)].site, SiteId(5));
    assert_eq!(
        campaign.ai.factions[&NPC].objective,
        Some(objective.clone())
    );
    for formation in campaign.armies[&ArmyId(2)].formation_ids() {
        campaign.formations.get_mut(&formation).unwrap().headcount = 1;
    }
    campaign.armies.get_mut(&spotter).unwrap().site = SiteId(5);

    for _ in 0..=data.ai.objective_rounds * 2 {
        set_zero_resources(&mut campaign, NPC);
        set_last_offer_this_round(&mut campaign, FactionId(1));
        assert_npc_turn_does_not_move(&mut campaign, &data, Some(&objective));
        assert!(!matches!(
            campaign.ai.factions[&NPC].objective.as_ref(),
            Some(retained)
                if retained.kind == AiObjectiveKind::Border && retained.site != objective.site
        ));
        advance_to_next_npc_turn(&mut campaign, &data);
    }

    assert!(campaign.completed_rounds >= data.ai.objective_rounds * 2);
    let mut expired_post = campaign.clone();
    let state = expired_post.ai.factions.entry(NPC).or_default();
    state.objective = Some(objective.clone());
    state.phase_round = expired_post.completed_rounds;
    state.accepted_commands = data.ai.max_commands_per_phase;
    let retained = ai::propose(&expired_post, &data, NPC).unwrap();
    assert_eq!(retained.command, Command::EndTurn);
    assert_eq!(retained.objective, Some(objective.clone()));

    let mut changed = campaign.clone();
    let state = changed.ai.factions.entry(NPC).or_default();
    state.objective = Some(objective.clone());
    state.phase_round = changed.completed_rounds;
    state.accepted_commands = 0;
    changed
        .set_site_control(&data, SiteId(7), None, false)
        .unwrap();
    let blocked = ai::propose(&changed, &data, NPC).unwrap();
    assert!(matches!(blocked.command, Command::InviteApprentice { .. }));
    let before_rejection = changed.clone();
    ai::rejected(&mut changed, &before_rejection, &data, NPC, &blocked).unwrap();
    for formation in changed.armies[&spotter].formation_ids() {
        let formation = changed.formations.get_mut(&formation).unwrap();
        formation.headcount = formation.capacity;
        formation.movement_spent = 0;
    }
    let mut later = ai::propose(&changed, &data, NPC).unwrap();
    for _ in 0..data.ai.max_commands_per_phase {
        if !matches!(later.command, Command::InviteApprentice { .. }) {
            break;
        }
        let before_rejection = changed.clone();
        ai::rejected(&mut changed, &before_rejection, &data, NPC, &later).unwrap();
        later = ai::propose(&changed, &data, NPC).unwrap();
    }
    assert!(
        matches!(&later.command, Command::Move(order)
            if order.path == [SiteId(5), SiteId(6)]),
        "{later:?}"
    );
    assert!(matches!(
        later.objective,
        Some(objective) if objective.kind == AiObjectiveKind::Expand && objective.site == SiteId(7)
    ));

    set_relation(&mut campaign, FactionId(1), DiplomaticState::Peace);
    for _ in 0..=data.ai.objective_rounds * 2 {
        let state = campaign.ai.factions.entry(NPC).or_default();
        state.objective = Some(objective.clone());
        state.phase_round = campaign.completed_rounds;
        state.accepted_commands = 0;
        set_zero_resources(&mut campaign, NPC);
        let decision = ai::propose(&campaign, &data, NPC).unwrap();
        assert!(
            !matches!(&decision.command, Command::Move(_)),
            "{decision:?}"
        );
        assert!(!matches!(
            decision.objective.as_ref(),
            Some(retained) if retained.kind == AiObjectiveKind::Border
        ));
        assert_eq!(decision, ai::propose(&campaign, &data, NPC).unwrap());
        assert_npc_turn_does_not_move(&mut campaign, &data, None);
        advance_to_next_npc_turn(&mut campaign, &data);
    }
}

#[test]
fn established_officers_and_dual_qualified_people_do_not_buy_repeated_career_switches() {
    let (data, mut campaign) = fixture();
    let person = PersonId(2);
    campaign
        .people
        .get_mut(&person)
        .unwrap()
        .evidence
        .counts
        .insert(kestrum::state::evidence::EvidenceKind::Battle, 4);
    campaign
        .people
        .get_mut(&person)
        .unwrap()
        .evidence
        .service_by_troop
        .extend([(TroopKind::Warriors, 2), (TroopKind::Archers, 2)]);
    campaign
        .people
        .get_mut(&person)
        .unwrap()
        .evidence
        .counts
        .insert(
            kestrum::state::evidence::EvidenceKind::MeaningfulEncounter,
            4,
        );
    for class in [PersonClass::Officer, PersonClass::Infantry] {
        campaign.people.get_mut(&person).unwrap().class = class;
        for _ in 0..8 {
            fund(&mut campaign, 20);
            let decision = ai::propose(&campaign, &data, NPC).unwrap();
            assert!(
                !matches!(decision.command, Command::TrainPerson { person: id, .. } if id == person)
            );
            next_turn(&mut campaign, &data);
            assert_eq!(campaign.people[&person].class, class);
        }
    }
}

#[test]
fn planner_invites_teaches_trains_and_deploys_an_unrelated_replacement() {
    command_apprentice_enters_field();
    let (data, mut campaign) = fixture();
    let founder = PersonId(2);
    campaign.people.get_mut(&founder).unwrap().class = PersonClass::Infantry;
    // Real seasonal service qualifies the teacher; no lesson counters are seeded.
    for _ in 0..4 {
        next_turn(&mut campaign, &data);
    }
    assert_eq!(
        campaign.people[&founder].career.discipline_service_seasons[&TrainingDiscipline::Infantry],
        4
    );
    fund(&mut campaign, 20);
    let invitation = ai::propose(&campaign, &data, NPC).unwrap();
    assert!(
        matches!(invitation.command, Command::InviteApprentice { .. }),
        "{invitation:?}"
    );
    let pupil = advance_npc(&mut campaign, &data).unwrap().new_people[0];
    let mut selected_training = false;
    let mut selected_transfer = false;
    for _ in 0..16 {
        fund(&mut campaign, 20);
        for _ in 0..16 {
            let decision = ai::propose(&campaign, &data, NPC).unwrap();
            match decision.command {
                Command::TrainPerson { person, class, .. } if person == pupil => {
                    assert_eq!(class, PersonClass::Infantry);
                    selected_training = true;
                }
                Command::TransferPerson { person, .. } if person == pupil => {
                    selected_transfer = true
                }
                Command::Move(_) | Command::EndTurn | Command::ClearThreat { .. } => break,
                _ => {}
            }
            advance_npc(&mut campaign, &data).unwrap();
        }
        campaign = reload(&campaign, &data);
        if selected_transfer {
            break;
        }
        next_turn(&mut campaign, &data);
    }
    assert!(
        selected_training && selected_transfer,
        "trained={selected_training} transferred={selected_transfer}; pupil={:?}; mentorship={:?}",
        campaign.people[&pupil],
        campaign.mentorships.get(&pupil)
    );
    assert_eq!(
        campaign.people[&pupil].career.mentorship_seasons[&TrainingDiscipline::Infantry],
        4
    );
    assert!(matches!(
        campaign.people[&pupil].assignment,
        PersonAssignment::Formation { .. }
    ));
    assert!(campaign.is_pupil(founder, pupil));
    apply(
        &mut campaign,
        &data,
        Actor::Npc(NPC),
        Command::RetirePerson {
            person: founder,
            site: SiteId(2),
        },
    )
    .unwrap();
    assert_eq!(campaign.armies[&ArmyId(2)].commander, Some(pupil));
    replacement_encounter(&mut campaign, &data, pupil);
    assert_eq!(reload(&campaign, &data), campaign);
}

#[test]
fn known_rivalry_breaks_only_strategic_ties_without_following_hidden_people() {
    use ai::{rank_targets, AiTarget};
    use kestrum::state::{
        battle::BattleId,
        knowledge::{EncounteredPerson, ObservedCondition},
        people::PersonRelationship,
    };
    let (_, mut campaign) = fixture();
    let mut view = kestrum::engine::project(&campaign, NPC).unwrap();
    let sites = [SiteId(7), SiteId(8)];
    view.hostile_presence.extend(sites);
    view.people[0].career.relationships.insert(
        PersonId(1),
        PersonRelationship {
            shared_service_seasons: 0,
            last_shared_service_round: None,
            mutual_combat_rounds: 2,
            last_mutual_combat_round: Some(0),
        },
    );
    campaign
        .knowledge
        .observers
        .entry(NPC)
        .or_default()
        .people
        .insert(
            PersonId(1),
            EncounteredPerson {
                appearance: campaign.people[&PersonId(1)].appearance.clone(),
                id: PersonId(1),
                name: "Witnessed rival".into(),
                class: PersonClass::Officer,
                completed_rounds: 0,
                site: sites[1],
                site_name: "Bridge".into(),
                army: ArmyId(1),
                army_name: "Witnessed army".into(),
                condition: ObservedCondition::Fit,
                battle: BattleId(1),
            },
        );
    let candidates = sites
        .map(|site| AiTarget {
            site,
            travel_cost: 2,
            strategic_priority: 4,
        })
        .to_vec();
    assert_eq!(
        rank_targets(&campaign, &view, candidates.clone())[0].site,
        sites[1]
    );
    let mut urgent = candidates.clone();
    urgent[0].strategic_priority = 0;
    assert_eq!(rank_targets(&campaign, &view, urgent)[0].site, sites[0]);
    // Private movement and names cannot change the retained observation.
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(12);
    campaign.people.get_mut(&PersonId(1)).unwrap().name = "Unobserved rename".into();
    assert_eq!(
        rank_targets(&campaign, &view, candidates.clone())[0].site,
        sites[1]
    );
    view.hostile_presence.remove(&sites[1]);
    assert_eq!(
        rank_targets(&campaign, &view, candidates.clone())[0].site,
        sites[0]
    );
    view.hostile_presence.insert(sites[1]);
    campaign.knowledge.observers.clear();
    assert_eq!(rank_targets(&campaign, &view, candidates)[0].site, sites[0]);
}

fn replacement_encounter(campaign: &mut StrategicCampaign, data: &GameData, pupil: PersonId) {
    apply(
        campaign,
        data,
        Actor::Npc(NPC),
        Command::Move(kestrum::engine::MoveOrder {
            armies: vec![ArmyId(2)],
            path: [2, 5, 6, 7].map(SiteId).to_vec(),
        }),
    )
    .unwrap();
    next_turn(campaign, data);
    fund(campaign, 0);
    let mut fought = false;
    for _ in 0..16 {
        let expedition = ai::propose(campaign, data, NPC).unwrap();
        assert!(
            !matches!(expedition.command, Command::EndTurn | Command::Move(_)),
            "{expedition:?}"
        );
        fought = matches!(expedition.command, Command::ClearThreat { .. });
        advance_npc(campaign, data).unwrap();
        if fought {
            break;
        }
    }
    assert!(fought);
    next_turn(campaign, data);
    assert!(
        campaign.people[&pupil].evidence.counts[&kestrum::state::evidence::EvidenceKind::Battle]
            > 0
    );
}

fn command_apprentice_enters_field() {
    let (data, mut campaign) = fixture();
    for _ in 0..8 {
        next_turn(&mut campaign, &data);
    }
    fund(&mut campaign, 20);
    assert!(matches!(
        ai::propose(&campaign, &data, NPC).unwrap().command,
        Command::InviteApprentice { .. }
    ));
    let pupil = advance_npc(&mut campaign, &data).unwrap().new_people[0];
    let mut deployed = false;
    for _ in 0..8 {
        fund(&mut campaign, 20);
        for _ in 0..16 {
            let decision = ai::propose(&campaign, &data, NPC).unwrap();
            if matches!(
                decision.command,
                Command::Move(_) | Command::EndTurn | Command::ClearThreat { .. }
            ) {
                break;
            }
            if matches!(decision.command, Command::TransferPerson { person, .. } if person == pupil)
            {
                deployed = true;
            }
            advance_npc(&mut campaign, &data).unwrap();
        }
        if deployed {
            break;
        }
        next_turn(&mut campaign, &data);
    }
    assert!(
        deployed,
        "An Officer's pupil must reach the field to earn encounter prerequisites"
    );
    assert_eq!(campaign.people[&pupil].class, PersonClass::Recruit);
    assert_eq!(
        campaign.people[&pupil].career.mentorship_seasons[&TrainingDiscipline::Command],
        4
    );
    assert_eq!(reload(&campaign, &data), campaign);
}

fn border_post_fixture() -> (GameData, StrategicCampaign, AiObjective, ArmyId) {
    let (data, mut campaign) = fixture();
    let completed_rounds = campaign.completed_rounds;
    for threat in campaign.threats.values_mut() {
        threat.headcount = 0;
        threat.status = ThreatStatus::Cleared {
            round: completed_rounds,
            by: NPC,
            payout: Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            },
        };
    }
    for site in campaign.world.sites.clone() {
        let owner = site.controller.unwrap_or(NPC);
        if campaign.world.site(site.id).unwrap().controller != Some(owner) {
            campaign
                .set_site_control(&data, site.id, Some(owner), false)
                .unwrap();
        }
    }
    let formation = campaign.armies[&ArmyId(2)].formation_ids().last().unwrap();
    let spotter = apply(
        &mut campaign,
        &data,
        Actor::Npc(NPC),
        Command::SplitArmy { formation },
    )
    .unwrap()
    .split_army
    .unwrap();
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(6);
    campaign.armies.get_mut(&spotter).unwrap().site = SiteId(7);
    for formation in campaign
        .formations
        .values_mut()
        .filter(|formation| formation.faction == NPC)
    {
        formation.headcount = (formation.capacity * 41 / 100 + 1).min(formation.capacity);
        formation.movement_spent = 0;
    }
    for formation in campaign.armies[&spotter].formation_ids() {
        let formation = campaign.formations.get_mut(&formation).unwrap();
        formation.movement_spent = formation.movement_allowance(&data);
    }
    set_zero_resources(&mut campaign, NPC);
    campaign.validate(&data).unwrap();
    (
        data,
        campaign,
        AiObjective {
            site: SiteId(5),
            chosen_round: 0,
            kind: AiObjectiveKind::Border,
        },
        spotter,
    )
}

fn set_relation(campaign: &mut StrategicCampaign, other: FactionId, state: DiplomaticState) {
    let factions = faction_pair(other);
    let round = campaign.completed_rounds;
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == factions)
        .unwrap()
        .state = state;
    let pair = campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == factions)
        .unwrap();
    pair.peace_since = (state == DiplomaticState::Peace).then_some(round);
    pair.truce_until = None;
}

fn set_last_offer_this_round(campaign: &mut StrategicCampaign, other: FactionId) {
    let factions = faction_pair(other);
    let round = campaign.completed_rounds;
    let pair = campaign
        .diplomacy
        .pairs
        .iter_mut()
        .find(|pair| pair.factions == factions)
        .unwrap();
    pair.last_offer_round = Some(round);
}

fn faction_pair(other: FactionId) -> [FactionId; 2] {
    if NPC.0 < other.0 {
        [NPC, other]
    } else {
        [other, NPC]
    }
}

fn set_zero_resources(campaign: &mut StrategicCampaign, faction: FactionId) {
    campaign.factions.get_mut(&faction).unwrap().resources = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    for site in campaign
        .world
        .sites
        .iter()
        .filter(|site| site.controller == Some(faction))
    {
        campaign.world.focus.insert(site.id, Focus::Gold);
    }
}

fn assert_npc_turn_does_not_move(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    expected_border: Option<&AiObjective>,
) {
    let mut commands = 0;
    while campaign.active_faction() == NPC {
        let decision = ai::propose(campaign, data, NPC).unwrap();
        assert!(
            !matches!(&decision.command, Command::Move(_)),
            "{decision:?}"
        );
        assert_eq!(decision, ai::propose(campaign, data, NPC).unwrap());
        if let Some(expected) = expected_border {
            assert!(
                !matches!(
                    decision.objective.as_ref(),
                    Some(retained)
                        if retained.kind == AiObjectiveKind::Border && retained.site != expected.site
                ),
                "{decision:?}"
            );
        } else {
            assert!(!matches!(
                decision.objective.as_ref(),
                Some(retained) if retained.kind == AiObjectiveKind::Border
            ));
        }
        advance_npc(campaign, data).unwrap();
        commands += 1;
        assert!(commands <= data.ai.max_commands_per_phase);
    }
}

fn advance_to_next_npc_turn(campaign: &mut StrategicCampaign, data: &GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round || campaign.active_faction() != NPC {
        let actor = if campaign.active_faction() == campaign.player {
            Actor::Player
        } else {
            Actor::Npc(campaign.active_faction())
        };
        apply(campaign, data, actor, Command::EndTurn).unwrap();
    }
}
