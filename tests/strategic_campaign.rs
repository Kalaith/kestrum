//! The five K02 contracts: phase order, calendar, skipping, atomicity, and replay.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

use kestrum::{
    data::{
        world::{DiplomaticState, FactionId, Relation},
        GameData,
    },
    engine::{advance_npc, apply, preview, project, Actor, Command, RuleError},
    state::{
        campaign::{DomainFactKind, Faction},
        Campaign, CampaignPhase, FactionStatus, StrategicCampaign,
    },
};
use macroquad_toolkit::rng::SeededRng;

fn fixture(count: u32) -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    for number in 5..=count {
        let id = FactionId(number);
        let founder = campaign.factions[&campaign.player].clone();
        campaign.factions.insert(
            id,
            Faction {
                id,
                name: format!("Kingdom {number}"),
                ..founder
            },
        );
        for previous in 1..number {
            campaign.relations.push(Relation {
                factions: [FactionId(previous), id],
                state: DiplomaticState::Peace,
            });
        }
    }
    for number in 5..=count {
        let site = campaign
            .world
            .sites
            .iter_mut()
            .find(|site| {
                site.controller.is_none()
                    && !campaign
                        .threats
                        .values()
                        .any(|threat| threat.site == site.id)
            })
            .unwrap();
        site.controller = Some(FactionId(number));
        site.habitation = kestrum::data::economy::Habitation::Outpost;
        campaign
            .factions
            .get_mut(&FactionId(number))
            .unwrap()
            .headquarters = site.id;
        campaign
            .factions
            .get_mut(&FactionId(number))
            .unwrap()
            .capital = site.id;
    }
    campaign.relations.sort_by_key(|relation| relation.factions);
    campaign.diplomacy.pairs = campaign
        .relations
        .iter()
        .map(|relation| kestrum::state::diplomacy::PairDiplomacy {
            factions: relation.factions,
            peace_since: (relation.state == DiplomaticState::Peace).then_some(0),
            truce_until: None,
            last_offer_round: None,
            war_started_round: None,
            war_ended_round: None,
        })
        .collect();
    campaign.diplomacy.pairs.sort_by_key(|pair| pair.factions);
    campaign.reconcile_region_control();
    campaign.next_ids.faction = FactionId(count + 1);
    campaign.round_order = campaign.independent_order();
    campaign.validate(&data).unwrap();
    (data, campaign)
}

fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_npc(campaign, data).unwrap();
    }
}

#[test]
fn four_and_eight_factions_act_in_stable_order_before_one_boundary() {
    let mut reordered = GameData::load().unwrap();
    for relation in &mut reordered.scenario.relations {
        relation.factions.reverse();
    }
    reordered.validate().unwrap();
    let reordered_campaign = StrategicCampaign::new(&reordered).unwrap();
    let canonical_campaign = StrategicCampaign::new(&GameData::load().unwrap()).unwrap();
    assert_eq!(reordered_campaign, canonical_campaign);
    for count in [4, 8] {
        let (data, mut campaign) = fixture(count);
        let rng = campaign.rng.clone();
        let resources: Vec<_> = campaign
            .factions
            .values()
            .map(|faction| faction.resources)
            .collect();
        let mut acted = Vec::new();
        for index in 0..count {
            acted.push(campaign.active_faction());
            let actor = if index == 0 {
                Actor::Player
            } else {
                Actor::Npc(campaign.active_faction())
            };
            let outcome = apply(&mut campaign, &data, actor, Command::EndTurn).unwrap();
            assert_eq!(outcome.round_completed, index == count - 1);
            assert_eq!(campaign.completed_rounds, u32::from(index == count - 1));
            if outcome.round_completed {
                assert_eq!(outcome.consumed_facts.len(), count as usize);
                assert!(campaign.pending_facts.is_empty());
                assert_eq!(campaign.consumed_sequence, u64::from(count));
            } else {
                assert_eq!(
                    campaign
                        .factions
                        .values()
                        .map(|faction| faction.resources)
                        .collect::<Vec<_>>(),
                    resources,
                    "income and upkeep wait for the common boundary"
                );
            }
        }
        assert_eq!(acted, (1..=count).map(FactionId).collect::<Vec<_>>());
        assert_eq!(campaign.phase, CampaignPhase::PlayerTurn);
        assert!(campaign.acted.is_empty());
        assert_eq!(campaign.rng, rng);
        for faction in campaign.factions.values() {
            let statement = faction.last_economy.as_ref().unwrap();
            assert_eq!(statement.completed_rounds, 1);
            assert_eq!(statement.closing, faction.resources);
        }
        let player = &campaign.factions[&campaign.player];
        assert_eq!(player.resources.gold, resources[0].gold + 50 - 30);
        assert_eq!(player.resources.wood, resources[0].wood + 19);
        assert_eq!(player.resources.stone, resources[0].stone + 12);
    }
}

#[test]
fn four_completed_rounds_make_one_year_without_faction_dependent_dates() {
    for count in [4, 8] {
        let (data, mut campaign) = fixture(count);
        for completed in 0..=4 {
            assert_eq!(campaign.completed_rounds, completed);
            assert_eq!(campaign.season_index(), (completed % 4) as usize);
            assert_eq!(campaign.year(9), 9 + completed / 4);
            if completed < 4 {
                finish_round(&mut campaign, &data);
            }
        }
        assert_eq!(campaign.next_ids.fact.0, u64::from(count * 4 + 1));
        assert!(campaign.pending_facts.is_empty());
    }
}

#[test]
fn inactive_mid_round_factions_are_skipped_without_replaying_other_phases() {
    let (data, mut campaign) = fixture(8);
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    pass_npc(&mut campaign, &data).unwrap();
    campaign.factions.get_mut(&FactionId(4)).unwrap().status = FactionStatus::Eliminated;
    campaign.factions.get_mut(&FactionId(6)).unwrap().status = FactionStatus::Vassal {
        sovereign: campaign.player,
    };
    for faction in [FactionId(4), FactionId(6)] {
        campaign.armies.retain(|_, army| army.faction != faction);
        campaign
            .formations
            .retain(|_, formation| formation.faction != faction);
        for site in &mut campaign.world.sites {
            if site.controller == Some(faction) {
                site.controller = None;
            }
        }
        for person in campaign
            .people
            .values_mut()
            .filter(|person| person.faction == faction)
        {
            let site = campaign.factions[&faction].headquarters;
            person.status = kestrum::state::people::PersonStatus::Displaced {
                completed_rounds: 0,
                site,
            };
            person.assignment = kestrum::state::people::PersonAssignment::Site { site };
            person.movement_spent = 0;
        }
    }
    campaign.reconcile_region_control();
    let mut remaining = Vec::new();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        remaining.push(campaign.active_faction());
        let result = pass_npc(&mut campaign, &data).unwrap();
        if result.round_completed {
            let passed: Vec<_> = result
                .consumed_facts
                .iter()
                .map(|fact| match fact.kind {
                    DomainFactKind::FactionPassed { faction } => faction,
                    _ => panic!("Passing factions cannot invent military activity"),
                })
                .collect();
            assert_eq!(passed, [1, 2, 3, 5, 7, 8].map(FactionId));
        }
    }
    assert_eq!(remaining, [3, 5, 7, 8].map(FactionId));
    assert_eq!(campaign.round_order, [1, 2, 3, 5, 7, 8].map(FactionId));
    assert_eq!(campaign.completed_rounds, 1);
    finish_round(&mut campaign, &data);
    assert_eq!(campaign.completed_rounds, 2);
    assert_eq!(campaign.accepted_sequence, 12);
}

#[test]
fn rejected_commands_and_read_only_views_preserve_every_authoritative_field() {
    let (data, mut campaign) = fixture(4);
    let initial = campaign.clone();
    for (actor, command) in [
        (Actor::Npc(FactionId(99)), Command::EndTurn),
        (Actor::Npc(campaign.player), Command::EndTurn),
        (Actor::Npc(FactionId(2)), Command::EndTurn),
        (Actor::Player, Command::SetNpcPaused(true)),
        (Actor::Player, Command::StepNpc),
    ] {
        assert!(apply(&mut campaign, &data, actor, command).is_err());
        assert_eq!(campaign, initial);
    }
    preview(&campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign, initial);
    let visible = project(&campaign, campaign.player).unwrap();
    assert!(visible
        .factions
        .iter()
        .filter(|f| f.id != campaign.player)
        .all(|f| f.resources.is_none() && f.headquarters.is_none()));
    let mut hidden_change = campaign.clone();
    hidden_change
        .factions
        .get_mut(&FactionId(2))
        .unwrap()
        .resources
        .gold += 90;
    assert_eq!(visible, project(&hidden_change, campaign.player).unwrap());
    assert_eq!(campaign, initial);
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(
        preview(&campaign, &data, Actor::Player, Command::StepNpc),
        Err(RuleError::PauseRequired)
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetNpcPaused(true),
    )
    .unwrap();
    let paused = campaign.clone();
    assert_eq!(advance_npc(&mut campaign, &data), Err(RuleError::NpcPaused));
    assert!(apply(&mut campaign, &data, Actor::Player, Command::EndTurn).is_err());
    assert_eq!(campaign, paused);
    for overflow in 0..3 {
        let mut exhausted = initial.clone();
        match overflow {
            0 => exhausted.accepted_sequence = u64::MAX,
            1 => exhausted.next_ids.fact.0 = u64::MAX,
            _ => {
                for person in exhausted.people.values_mut() {
                    let age = i64::from(person.age_years(exhausted.completed_rounds));
                    person.birth_round = i64::from(u32::MAX) - 4 * age;
                    person.service_start_round = person.birth_round as u32;
                }
                exhausted.completed_rounds = u32::MAX;
                apply(&mut exhausted, &data, Actor::Player, Command::EndTurn).unwrap();
                pass_npc(&mut exhausted, &data).unwrap();
                pass_npc(&mut exhausted, &data).unwrap();
            }
        }
        let before = exhausted.clone();
        let actor = if overflow == 2 {
            Actor::Npc(FactionId(4))
        } else {
            Actor::Player
        };
        assert!(matches!(
            apply(&mut exhausted, &data, actor, Command::EndTurn),
            Err(RuleError::Overflow { .. })
        ));
        assert_eq!(exhausted, before);
    }
}

#[test]
fn serialized_paused_phase_and_full_rng_streams_resume_identically() {
    let (data, mut campaign) = fixture(4);
    let mut root = SeededRng::new(campaign.seed);
    let expected_states: [u64; 4] =
        std::array::from_fn(|_| SeededRng::new(root.next_u64()).state());
    let states = campaign.rng.states();
    assert_eq!(&states[..3], &expected_states[..3]);
    assert_ne!(states[3], expected_states[3]);
    let (_, replayed_start) = fixture(4);
    assert_eq!(campaign, replayed_start);
    campaign.rng.generation.next_u64();
    campaign.rng.combat.next_u64();
    campaign.rng.development.next_u64();
    campaign.rng.people.next_u64();
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    advance_npc(&mut campaign, &data).unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetNpcPaused(true),
    )
    .unwrap();
    let wrapped = Campaign::Strategic(Box::new(campaign.clone()));
    let encoded = serde_json::to_string(&wrapped).unwrap();
    let decoded: Campaign = serde_json::from_str(&encoded).unwrap();
    assert_eq!(wrapped, decoded);
    let mut restored = decoded.strategic().unwrap().clone();
    restored.validate(&data).unwrap();
    assert_eq!(campaign, restored);
    for command in [Command::StepNpc, Command::SetNpcPaused(false)] {
        let expected = apply(&mut campaign, &data, Actor::Player, command.clone()).unwrap();
        let resumed = apply(&mut restored, &data, Actor::Player, command).unwrap();
        assert_eq!(expected, resumed);
        assert_eq!(campaign, restored);
    }
    let expected = advance_npc(&mut campaign, &data).unwrap();
    let resumed = advance_npc(&mut restored, &data).unwrap();
    assert_eq!(expected, resumed);
    assert_eq!(expected.round_completed, resumed.round_completed);
    let mut commands = 0;
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        assert_eq!(
            advance_npc(&mut campaign, &data).unwrap(),
            advance_npc(&mut restored, &data).unwrap()
        );
        commands += 1;
        assert!(commands <= 195);
    }
    assert_eq!(campaign.completed_rounds, 1);
    assert!(campaign.pending_facts.is_empty());
    let boundary = Campaign::Strategic(Box::new(campaign.clone()));
    let boundary_json = serde_json::to_string(&boundary).unwrap();
    let boundary_restored: Campaign = serde_json::from_str(&boundary_json).unwrap();
    assert_eq!(boundary, boundary_restored);
    for _ in 0..4 {
        finish_round(&mut campaign, &data);
        finish_round(&mut restored, &data);
        assert_eq!(campaign, restored);
        assert_eq!(
            campaign.rng.people.next_u64(),
            restored.rng.people.next_u64()
        );
        assert_eq!(
            campaign.rng.combat.next_u64(),
            restored.rng.combat.next_u64()
        );
    }
    let mut invalid = serde_json::to_value(&campaign).unwrap();
    invalid["rng"]["combat"]["state"] = serde_json::json!(0);
    let invalid_state: StrategicCampaign = serde_json::from_value(invalid).unwrap();
    assert!(invalid_state.validate(&data).unwrap_err().contains("rng"));
}
