//! Read-only playback projection contracts for the committed battle receipt.

use kestrum::{
    data::{
        battle_tactics::{
            TacticAction, TacticCondition, TacticRule, TacticTrigger, TargetFilter, TargetPriority,
        },
        economy::TroopKind,
        GameData,
    },
    engine::resolve_battle,
    state::{
        battle::{playback::battle_presentation_at, simulation::*, BattleOutcome},
        military::{ArmyId, FormationId},
    },
};

fn rule(id: &str, action: TacticAction) -> TacticRule {
    let target_filter = if matches!(
        action,
        TacticAction::Attack
            | TacticAction::Volley
            | TacticAction::Charge
            | TacticAction::Breakthrough
    ) {
        TargetFilter::AnyEnemy
    } else {
        TargetFilter::None
    };
    TacticRule {
        id: id.into(),
        trigger: TacticTrigger::Activation,
        action,
        condition: TacticCondition::Always,
        target_filter,
        target_priority: TargetPriority::OwnColumnFirst,
    }
}

fn unit(
    id: u32,
    kind: TroopKind,
    action: TacticAction,
    attack: u32,
    resistance: u32,
) -> BattleUnitInput {
    BattleUnitInput {
        id: BattleUnitId::Formation(FormationId(id)),
        kind: Some(kind),
        headcount: 100,
        capacity: 100,
        attack,
        resistance,
        initiative: 20,
        activation_tactics: vec![rule("action", action)],
        reaction_tactics: Vec::new(),
    }
}

fn army(id: u32, side: BattleSide, groups: &[(usize, BattleUnitInput)]) -> BattleArmyInput {
    let mut slots = std::array::from_fn(|_| None);
    for (slot, group) in groups {
        slots[*slot] = Some(group.clone());
    }
    BattleArmyInput {
        id: ArmyId(id),
        faction: kestrum::data::world::FactionId(id),
        name: format!("Host {id}"),
        side,
        slots,
    }
}

fn input(left: BattleUnitInput, right: BattleUnitInput) -> FormationBattleInput {
    FormationBattleInput {
        terrain_permille: 1000,
        armies: vec![
            army(1, BattleSide::Attacker, &[(0, left)]),
            army(2, BattleSide::Defender, &[(0, right)]),
        ],
    }
}

fn rules() -> kestrum::data::battle_tactics::BattleTacticsRules {
    GameData::load().unwrap().battle_tactics
}

fn group(frame: &BattlePresentationFrame, id: u32) -> &BattlePresentationGroup {
    frame
        .groups
        .iter()
        .find(|group| group.id == BattleUnitId::Formation(FormationId(id)))
        .unwrap()
}

use kestrum::state::battle::playback::{BattlePresentationFrame, BattlePresentationGroup};

#[test]
fn opening_and_out_of_range_cursors_project_a_bounded_initial_state() {
    let resolution = resolve_battle(
        &input(
            unit(1, TroopKind::Spearmen, TacticAction::Wait, 10, 10),
            unit(2, TroopKind::Warriors, TacticAction::Wait, 10, 10),
        ),
        &rules(),
    )
    .unwrap();

    let opening = battle_presentation_at(&resolution, 0);
    let bounded = battle_presentation_at(&resolution, usize::MAX);
    assert_eq!(opening.completed_events, 0);
    assert_eq!(opening.round, 1);
    assert_eq!(group(&opening, 1).headcount, 100);
    assert_eq!(group(&opening, 2).morale, resolution.opening_morale);
    assert_eq!(bounded.completed_events, resolution.events.len());
    assert_eq!(bounded.active_event, None);
}

#[test]
fn damage_and_morale_events_update_only_the_projected_prefix() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let resolution = resolve_battle(
        &input(
            unit(1, TroopKind::Warriors, TacticAction::Attack, 40, 10),
            unit(2, TroopKind::Warriors, TacticAction::Attack, 1, 1000),
        ),
        &rules,
    )
    .unwrap();
    let original = resolution.clone();
    let (damage_index, target, remaining) = resolution
        .events
        .iter()
        .enumerate()
        .find_map(|(index, event)| match event {
            BattleEvent::Damage {
                target, remaining, ..
            } => Some((index, *target, *remaining)),
            _ => None,
        })
        .unwrap();
    let damage_frame = battle_presentation_at(&resolution, damage_index + 1);
    let damaged = damage_frame
        .groups
        .iter()
        .find(|group| group.id == target)
        .unwrap();
    assert_eq!(damaged.headcount, remaining);

    if let Some((index, id, after)) =
        resolution
            .events
            .iter()
            .enumerate()
            .find_map(|(index, event)| match event {
                BattleEvent::MoraleChanged { unit, after, .. } => Some((index, *unit, *after)),
                _ => None,
            })
    {
        assert_eq!(
            battle_presentation_at(&resolution, index + 1)
                .groups
                .iter()
                .find(|group| group.id == id)
                .unwrap()
                .morale,
            after
        );
    }
    assert_eq!(resolution, original);
}

#[test]
fn advance_event_moves_the_group_into_its_paired_front_slot() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let mut battle = input(
        unit(1, TroopKind::Spearmen, TacticAction::Wait, 1, 100),
        unit(2, TroopKind::Warriors, TacticAction::Wait, 1, 100),
    );
    battle.armies[0].slots[0] = None;
    battle.armies[0].slots[3] = Some(unit(3, TroopKind::Warriors, TacticAction::Advance, 1, 100));
    let resolution = resolve_battle(&battle, &rules).unwrap();
    let (index, id, slot) = resolution
        .events
        .iter()
        .enumerate()
        .find_map(|(index, event)| match event {
            BattleEvent::PositionChanged { unit, to, .. } => Some((index, *unit, to.slot)),
            _ => None,
        })
        .unwrap();
    assert_eq!(slot, 0);
    let frame = battle_presentation_at(&resolution, index + 1);
    assert_eq!(
        frame
            .groups
            .iter()
            .find(|group| group.id == id)
            .unwrap()
            .slot,
        Some(0)
    );
}

#[test]
fn guard_projection_expires_when_the_next_round_begins() {
    let mut rules = rules();
    rules.max_rounds = 2;
    let resolution = resolve_battle(
        &input(
            unit(1, TroopKind::Spearmen, TacticAction::Guard, 1, 1000),
            unit(2, TroopKind::Warriors, TacticAction::Wait, 1, 1000),
        ),
        &rules,
    )
    .unwrap();
    let (guard_index, id) = resolution
        .events
        .iter()
        .enumerate()
        .find_map(|(index, event)| match event {
            BattleEvent::GuardRaised { unit, .. } => Some((index, *unit)),
            _ => None,
        })
        .unwrap();
    assert!(
        group(&battle_presentation_at(&resolution, guard_index + 1), 1)
            .guard_expires_round
            .is_some()
    );
    let next_round = resolution
        .events
        .iter()
        .enumerate()
        .find_map(|(index, event)| match event {
            BattleEvent::RoundStarted { round: 2 } => Some(index),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        battle_presentation_at(&resolution, next_round + 1)
            .groups
            .iter()
            .find(|group| group.id == id)
            .unwrap()
            .guard_expires_round,
        None
    );
}

#[test]
fn routed_survivors_leave_the_line_and_keep_their_headcount() {
    let mut rules = rules();
    rules.max_rounds = 1;
    rules.initial_morale = 10;
    rules.rout_morale = 5;
    rules.morale_loss_per_casualty = 10;
    rules.ally_rout_morale_loss = 5;
    let resolution = resolve_battle(
        &input(
            unit(1, TroopKind::Warriors, TacticAction::Attack, 1, 1000),
            unit(2, TroopKind::Warriors, TacticAction::Attack, 1, 1000),
        ),
        &rules,
    )
    .unwrap();
    let (index, id, survivors) = resolution
        .events
        .iter()
        .enumerate()
        .find_map(|(index, event)| match event {
            BattleEvent::Routed {
                unit, survivors, ..
            } => Some((index, *unit, *survivors)),
            _ => None,
        })
        .unwrap();
    let routed = battle_presentation_at(&resolution, index + 1);
    let routed = routed.groups.iter().find(|group| group.id == id).unwrap();
    assert!(routed.is_routed);
    assert_eq!(routed.slot, None);
    assert_eq!(routed.headcount, survivors);
}

#[test]
fn presentation_never_mutates_the_saved_resolution_or_outcome() {
    let resolution = resolve_battle(
        &input(
            unit(1, TroopKind::Spearmen, TacticAction::Wait, 10, 10),
            unit(2, TroopKind::Warriors, TacticAction::Wait, 10, 10),
        ),
        &rules(),
    )
    .unwrap();
    let before = resolution.clone();
    for cursor in 0..=resolution.events.len() + 2 {
        let _ = battle_presentation_at(&resolution, cursor);
    }
    assert_eq!(resolution, before);
    assert_eq!(resolution.outcome, BattleOutcome::Stalemate);
}
