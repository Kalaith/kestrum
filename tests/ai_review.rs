//! Actual planner choices for threat approaches and useful local careers.

use kestrum::{
    data::{
        economy::{Resources, TroopKind},
        progression::TrainingDiscipline,
        world::{FactionId, PersonClass, SiteId},
        GameData,
    },
    engine::{advance_npc, ai, apply, Actor, Command},
    state::{
        construction::Focus,
        military::ArmyId,
        people::{PersonAssignment, PersonId},
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
