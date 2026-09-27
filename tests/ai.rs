//! P09 acceptance through ordinary commands, persisted intent and observer views.
#[path = "support/ai.rs"]
mod support;
use kestrum::{
    data::{
        economy::{Resources, TroopKind},
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{advance_npc, ai, apply, preview, Actor, Command},
    state::{
        ai::{AiFactionState, AiObjective, AiObjectiveKind},
        military::ArmyId,
        CampaignPhase, StrategicCampaign,
    },
};
use support::*;

#[test]
fn legal_roster_growth_pays_prices_and_keeps_two_rounds_upkeep() {
    let (data, mut campaign) = fixture();
    campaign.factions.get_mut(&FactionId(2)).unwrap().resources = Resources {
        gold: 5000,
        wood: 5000,
        stone: 5000,
    };
    let mut recruited = Vec::new();
    let mut commands = 0;
    while campaign.active_faction() == FactionId(2) {
        let decision = ai::propose(&campaign, &data, FactionId(2)).unwrap();
        preview(
            &campaign,
            &data,
            Actor::Npc(FactionId(2)),
            decision.command.clone(),
        )
        .unwrap();
        let before = campaign.clone();
        let result = advance_npc(&mut campaign, &data).unwrap();
        assert_eq!(result.accepted_sequence, before.accepted_sequence + 1);
        if let Command::Recruit { army, kind, .. } = decision.command {
            let fresh = &campaign.formations[&result.recruited.unwrap().formation];
            assert_eq!(fresh.headcount, fresh.capacity);
            assert_eq!(
                fresh.movement_spent,
                data.economy.formations[&kind].movement_allowance
            );
            assert_eq!(
                campaign.factions[&FactionId(2)].resources.gold,
                before.factions[&FactionId(2)].resources.gold
                    - data.economy.formations[&kind].recruit_cost.gold
            );
            let upkeep: i64 = campaign
                .formations
                .values()
                .filter(|f| f.faction == FactionId(2))
                .map(|f| data.economy.formations[&f.kind].upkeep_gold)
                .sum();
            assert!(campaign.factions[&FactionId(2)].resources.gold >= upkeep * 2);
            if army.is_none() {
                assert!(before
                    .armies
                    .values()
                    .filter(|army| army.faction == FactionId(2))
                    .all(|army| army.formation_ids().count() >= 3));
            }
            recruited.push(kind);
        }
        commands += 1;
        assert!(commands <= 65);
    }
    assert!(recruited.len() >= 3);
    assert_eq!(
        recruited[..3],
        [TroopKind::Warriors, TroopKind::Spearmen, TroopKind::Archers]
    );
    let armies: Vec<_> = campaign
        .armies
        .values()
        .filter(|army| army.faction == FactionId(2))
        .collect();
    assert_eq!(armies.len(), 2);
    assert!(armies.iter().all(|army| army.formation_ids().count() == 6));
    assert!(campaign
        .construction
        .values()
        .any(|order| order.owner == FactionId(2)));
    invalid_rules_and_state(&data, &campaign);
}

#[test]
fn private_enemy_changes_do_not_grant_strength_and_empty_hostile_land_is_captured() {
    let (data, mut campaign) = fixture();
    no_resources(&mut campaign);
    let before = campaign.clone();
    let expected = ai::propose(&campaign, &data, FactionId(2)).unwrap();
    campaign.armies.get_mut(&ArmyId(3)).unwrap().name = "Unobserved name".into();
    for formation in campaign
        .formations
        .values_mut()
        .filter(|f| f.faction == FactionId(3))
    {
        formation.headcount = formation.capacity / 2;
    }
    campaign.validate(&data).unwrap();
    assert_eq!(
        expected,
        ai::propose(&campaign, &data, FactionId(2)).unwrap()
    );
    assert_eq!(campaign.rng.states(), before.rng.states());
    unknown_enemy_and_empty_capture(&data);
    actual_last_known_attack(&data);
}

#[test]
fn objectives_survive_boundaries_expire_and_yield_to_headquarters_emergency() {
    let (data, mut campaign) = fixture();
    no_resources(&mut campaign);
    let objective = AiObjective {
        site: SiteId(8),
        chosen_round: 0,
        kind: AiObjectiveKind::Expand,
    };
    campaign.ai.factions.insert(
        FactionId(2),
        AiFactionState {
            objective: Some(objective.clone()),
            ..Default::default()
        },
    );
    assert_eq!(
        ai::propose(&campaign, &data, FactionId(2))
            .unwrap()
            .objective,
        Some(objective.clone())
    );
    advance_npc(&mut campaign, &data).unwrap();
    assert_eq!(
        campaign.ai.factions[&FactionId(2)].objective,
        Some(objective.clone())
    );
    for round in 1..=4 {
        next_oak_turn(&mut campaign, &data);
        no_resources(&mut campaign);
        assert_eq!(campaign.completed_rounds, round);
        let decision = ai::propose(&campaign, &data, FactionId(2)).unwrap();
        if round < 4 {
            assert_eq!(decision.objective, Some(objective.clone()));
        } else {
            assert_ne!(decision.objective, Some(objective.clone()));
        }
    }
    headquarters_emergency(&data);
}

#[test]
fn failed_intents_are_bounded_and_phase_budget_ends_without_retry_loops() {
    let (mut data, mut campaign) = fixture();
    let before = campaign.clone();
    let decision = ai::propose(&before, &data, FactionId(2)).unwrap();
    ai::rejected(&mut campaign, &before, &data, FactionId(2), &decision).unwrap();
    assert_eq!(campaign.accepted_sequence, before.accepted_sequence);
    assert_eq!(campaign.rng, before.rng);
    assert_ne!(
        ai::propose(&campaign, &data, FactionId(2)).unwrap().intent,
        decision.intent
    );
    for index in 0..100 {
        let mut rejected = decision.clone();
        rejected.intent.targets = vec![index];
        ai::rejected(&mut campaign, &before, &data, FactionId(2), &rejected).unwrap();
    }
    assert_eq!(campaign.ai.factions[&FactionId(2)].rejected.len(), 64);
    apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        Command::EndTurn,
    )
    .unwrap();
    assert_ne!(campaign.active_faction(), FactionId(2));
    data.ai.max_commands_per_phase = 1;
    let mut limited = before;
    advance_npc(&mut limited, &data).unwrap();
    assert_eq!(limited.ai.factions[&FactionId(2)].accepted_commands, 1);
    assert_eq!(
        ai::propose(&limited, &data, FactionId(2)).unwrap().command,
        Command::EndTurn
    );
    advance_npc(&mut limited, &data).unwrap();
    assert_ne!(limited.active_faction(), FactionId(2));
}

#[test]
fn seeded_phase_replay_matches_paused_steps_and_arbitrary_observation_frames() {
    let (data, base) = fixture();
    let mut automatic = base.clone();
    let mut stepped = base;
    apply(
        &mut stepped,
        &data,
        Actor::Player,
        Command::SetNpcPaused(true),
    )
    .unwrap();
    let pause_sequence_offset = stepped.accepted_sequence - automatic.accepted_sequence;
    let mut commands = 0;
    while matches!(automatic.phase, CampaignPhase::NpcTurn { .. })
        && !automatic.diplomacy.is_blocked()
    {
        let owner = automatic.active_faction();
        let decision = ai::propose(&automatic, &data, owner).unwrap();
        for _ in 0..commands % 7 {
            assert_eq!(ai::propose(&automatic, &data, owner).unwrap(), decision);
        }
        let before = stepped.accepted_sequence;
        advance_npc(&mut automatic, &data).unwrap();
        apply(&mut stepped, &data, Actor::Player, Command::StepNpc).unwrap();
        assert_eq!(stepped.accepted_sequence, before + 1);
        assert!(matches!(
            stepped.phase,
            CampaignPhase::NpcTurn { paused: true, .. } | CampaignPhase::PlayerTurn
        ));
        assert_eq!(automatic.rng, stepped.rng);
        assert_eq!(automatic.armies, stepped.armies);
        assert_eq!(automatic.formations, stepped.formations);
        commands += 1;
        assert!(commands <= 3 * 65);
    }
    assert!(commands > 3, "NPCs performed real strategic actions");
    assert_eq!(automatic.completed_rounds, 1);
    assert_eq!(automatic.factions, stepped.factions);
    assert_eq!(automatic.world, stepped.world);
    assert_eq!(
        stepped.accepted_sequence,
        automatic.accepted_sequence + pause_sequence_offset
    );
    assert_eq!(
        automatic.ai.factions.keys().collect::<Vec<_>>(),
        stepped.ai.factions.keys().collect::<Vec<_>>()
    );
}
