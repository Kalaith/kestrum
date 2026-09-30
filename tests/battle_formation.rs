//! Headless contracts for positioning, ordered rules and deterministic playback events.

use kestrum::{
    data::{
        battle_tactics::{
            BattleCapability, BattleTacticsRules, TacticAction, TacticCondition, TacticRule,
            TacticTrigger, TargetFilter, TargetPriority,
        },
        economy::TroopKind,
        world::PersonClass,
        GameData,
    },
    engine::resolve_battle,
    state::{
        battle::{simulation::*, BattleOutcome},
        military::{ArmyId, FormationId},
        people::PersonId,
        threat::ThreatId,
    },
};

#[path = "support/battle_abilities.rs"]
mod abilities;

#[path = "support/battle_review.rs"]
mod review;

fn rules() -> BattleTacticsRules {
    GameData::load().unwrap().battle_tactics
}

fn tactic(
    id: &str,
    trigger: TacticTrigger,
    action: TacticAction,
    condition: TacticCondition,
    target_filter: TargetFilter,
    target_priority: TargetPriority,
) -> TacticRule {
    TacticRule {
        id: id.into(),
        trigger,
        action,
        condition,
        target_filter,
        target_priority,
    }
}

fn wait_rule(id: &str) -> TacticRule {
    tactic(
        id,
        TacticTrigger::Activation,
        TacticAction::Wait,
        TacticCondition::Always,
        TargetFilter::None,
        TargetPriority::OwnColumnFirst,
    )
}

struct UnitProfile {
    id: BattleUnitId,
    kind: Option<TroopKind>,
    headcount: u32,
    attack: u32,
    resistance: u32,
    initiative: u32,
}

fn unit(
    profile: UnitProfile,
    activation_tactics: Vec<TacticRule>,
    reaction_tactics: Vec<TacticRule>,
) -> BattleUnitInput {
    BattleUnitInput {
        id: profile.id,
        kind: profile.kind,
        headcount: profile.headcount,
        capacity: profile.headcount.max(100),
        attack: profile.attack,
        resistance: profile.resistance,
        initiative: profile.initiative,
        leader: None,
        capabilities: profile.kind.map_or_else(Vec::new, |kind| {
            kestrum::data::battle_tactics::leader_capabilities(kind, None)
        }),
        activation_tactics,
        reaction_tactics,
    }
}

fn formation(
    id: u32,
    kind: TroopKind,
    tactics: Vec<TacticRule>,
    initiative: u32,
) -> BattleUnitInput {
    unit(
        UnitProfile {
            id: BattleUnitId::Formation(FormationId(id)),
            kind: Some(kind),
            headcount: 100,
            attack: 10,
            resistance: 10,
            initiative,
        },
        tactics,
        Vec::new(),
    )
}

fn with_leader(mut unit: BattleUnitInput, class: PersonClass) -> BattleUnitInput {
    let kind = unit.kind.expect("formation leader requires a troop role");
    unit.leader = Some(BattleLeaderSnapshot {
        id: PersonId(1),
        name: "Captain Test".into(),
        class,
        active: true,
    });
    unit.capabilities = kestrum::data::battle_tactics::leader_capabilities(kind, Some(class));
    unit
}

fn army(
    id: u32,
    faction: u32,
    side: BattleSide,
    groups: Vec<(usize, BattleUnitInput)>,
) -> BattleArmyInput {
    let mut slots = std::array::from_fn(|_| None);
    for (slot, group) in groups {
        slots[slot] = Some(group);
    }
    BattleArmyInput {
        id: ArmyId(id),
        faction: kestrum::data::world::FactionId(faction),
        name: format!("Host {id}"),
        side,
        slots,
    }
}

fn input(armies: Vec<BattleArmyInput>) -> FormationBattleInput {
    FormationBattleInput {
        terrain_permille: 1000,
        armies,
    }
}

#[test]
fn ordinary_melee_cannot_reach_a_protected_rear_unit() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let rear = BattleUnitId::Formation(FormationId(3));
    let battle = input(vec![
        army(
            1,
            1,
            BattleSide::Attacker,
            vec![(
                0,
                formation(
                    1,
                    TroopKind::Warriors,
                    vec![tactic(
                        "try-rear",
                        TacticTrigger::Activation,
                        TacticAction::Attack,
                        TacticCondition::Always,
                        TargetFilter::EnemyRear,
                        TargetPriority::LowestStrength,
                    )],
                    20,
                ),
            )],
        ),
        army(
            2,
            2,
            BattleSide::Defender,
            vec![
                (
                    0,
                    formation(2, TroopKind::Warriors, vec![wait_rule("hold")], 1),
                ),
                (
                    3,
                    formation(3, TroopKind::Archers, vec![wait_rule("wait")], 1),
                ),
            ],
        ),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    let actor = result
        .events
        .iter()
        .find_map(|event| match event {
            BattleEvent::Activation {
                actor: BattleUnitId::Formation(FormationId(1)),
                action,
                target,
                skipped,
                ..
            } => Some((*action, *target, skipped)),
            _ => None,
        })
        .unwrap();
    assert_eq!(actor.0, TacticAction::Attack);
    assert_eq!(actor.1, Some(BattleUnitId::Formation(FormationId(2))));
    assert_eq!(actor.2[0].reason, TacticSkipReason::NoLegalTarget);
    assert_eq!(
        result
            .units
            .iter()
            .find(|unit| unit.id == rear)
            .unwrap()
            .headcount,
        100
    );
}

#[test]
fn ordinary_melee_can_reach_rear_units_after_an_armys_front_line_is_empty() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let attacker = formation(
        1,
        TroopKind::Warriors,
        vec![tactic(
            "press-through",
            TacticTrigger::Activation,
            TacticAction::Attack,
            TacticCondition::Always,
            TargetFilter::AnyEnemy,
            TargetPriority::OwnColumnFirst,
        )],
        20,
    );
    let rear = formation(2, TroopKind::Archers, vec![wait_rule("wait")], 1);
    let battle = input(vec![
        army(1, 1, BattleSide::Attacker, vec![(0, attacker)]),
        army(2, 2, BattleSide::Defender, vec![(3, rear)]),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::Activation {
            actor: BattleUnitId::Formation(FormationId(1)),
            target: Some(BattleUnitId::Formation(FormationId(2))),
            action: TacticAction::Attack,
            ..
        }
    )));
}

#[test]
fn cavalry_breakthrough_uses_a_lane_after_its_front_guard_falls() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let rider = formation(
        2,
        TroopKind::Riders,
        vec![
            tactic(
                "exploit-gap",
                TacticTrigger::Activation,
                TacticAction::Breakthrough,
                TacticCondition::Always,
                TargetFilter::ExposedEnemyRear,
                TargetPriority::LowestStrength,
            ),
            wait_rule("hold-if-closed"),
        ],
        90,
    );
    let mut warrior = formation(1, TroopKind::Warriors, vec![], 100);
    warrior.headcount = 1;
    warrior.capacity = 1;
    warrior.attack = 500;
    warrior.resistance = 1;
    let mut defender = formation(3, TroopKind::Warriors, vec![wait_rule("hold")], 1);
    defender.headcount = 1;
    defender.capacity = 1;
    defender.resistance = 1;
    let rear = formation(4, TroopKind::Archers, vec![wait_rule("wait")], 0);
    let battle = input(vec![
        army(1, 1, BattleSide::Attacker, vec![(0, warrior), (1, rider)]),
        army(2, 2, BattleSide::Defender, vec![(1, defender), (4, rear)]),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::Activation {
            actor: BattleUnitId::Formation(FormationId(2)),
            action: TacticAction::Breakthrough,
            target: Some(BattleUnitId::Formation(FormationId(4))),
            rule_id: Some(rule),
            ..
        } if rule == "exploit-gap"
    )));
    assert_eq!(
        result
            .units
            .iter()
            .find(|unit| unit.id == BattleUnitId::Formation(FormationId(3)))
            .unwrap()
            .headcount,
        0
    );
}

#[test]
fn cavalry_waits_for_an_exposed_rear_then_uses_its_first_rule() {
    let mut rules = rules();
    rules.max_rounds = 2;
    let rider_rules = rules.defaults[&TroopKind::Riders].activation.clone();
    let mut rider = formation(2, TroopKind::Riders, rider_rules, 60);
    let mut warrior = formation(1, TroopKind::Warriors, vec![], 40);
    warrior.attack = 1000;
    let mut enemy_guard = formation(3, TroopKind::Warriors, vec![wait_rule("hold")], 10);
    enemy_guard.headcount = 1;
    enemy_guard.capacity = 1;
    enemy_guard.resistance = 1;
    let enemy_cavalry = formation(4, TroopKind::Riders, vec![wait_rule("wait")], 20);
    rider.headcount = 30;
    let battle = input(vec![
        army(1, 1, BattleSide::Attacker, vec![(0, warrior), (3, rider)]),
        army(
            2,
            2,
            BattleSide::Defender,
            vec![(0, enemy_guard), (3, enemy_cavalry)],
        ),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    let mut round = 0;
    let rider_actions: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            BattleEvent::RoundStarted { round: current } => {
                round = *current;
                None
            }
            BattleEvent::Activation {
                actor: BattleUnitId::Formation(FormationId(2)),
                action,
                ..
            } => Some((round, *action)),
            _ => None,
        })
        .collect();
    assert_eq!(
        rider_actions,
        [(1, TacticAction::Wait), (2, TacticAction::Breakthrough)]
    );
}

#[test]
fn an_unavailable_higher_row_falls_through_to_the_first_legal_charge() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let rider = formation(
        1,
        TroopKind::Riders,
        vec![
            tactic(
                "rear-if-open",
                TacticTrigger::Activation,
                TacticAction::Breakthrough,
                TacticCondition::Always,
                TargetFilter::ExposedEnemyRear,
                TargetPriority::LowestStrength,
            ),
            tactic(
                "charge-front",
                TacticTrigger::Activation,
                TacticAction::Charge,
                TacticCondition::Always,
                TargetFilter::EnemyFront,
                TargetPriority::OwnColumnFirst,
            ),
        ],
        20,
    );
    let battle = input(vec![
        army(1, 1, BattleSide::Attacker, vec![(0, rider)]),
        army(
            2,
            2,
            BattleSide::Defender,
            vec![
                (
                    0,
                    formation(2, TroopKind::Warriors, vec![wait_rule("hold")], 1),
                ),
                (
                    3,
                    formation(3, TroopKind::Archers, vec![wait_rule("wait")], 1),
                ),
            ],
        ),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::Activation {
            actor: BattleUnitId::Formation(FormationId(1)),
            action: TacticAction::Charge,
            target: Some(BattleUnitId::Formation(FormationId(2))),
            rule_id: Some(rule),
            skipped,
        } if rule == "charge-front" && skipped.len() == 1
            && skipped[0].rule_id == "rear-if-open"
            && skipped[0].reason == TacticSkipReason::NoLegalTarget
    )));
}

#[test]
fn brace_reacts_before_charge_damage_and_cannot_start_a_reaction_chain() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let rider = formation(
        1,
        TroopKind::Riders,
        vec![tactic(
            "charge",
            TacticTrigger::Activation,
            TacticAction::Charge,
            TacticCondition::Always,
            TargetFilter::EnemyFront,
            TargetPriority::OwnColumnFirst,
        )],
        20,
    );
    let spearman = unit(
        UnitProfile {
            id: BattleUnitId::Formation(FormationId(2)),
            kind: Some(TroopKind::Spearmen),
            headcount: 100,
            attack: 10,
            resistance: 10,
            initiative: 1,
        },
        vec![wait_rule("hold")],
        vec![tactic(
            "brace-riders",
            TacticTrigger::IncomingAttack,
            TacticAction::Brace,
            TacticCondition::IncomingCavalryCharge,
            TargetFilter::None,
            TargetPriority::OwnColumnFirst,
        )],
    );
    let second_rider = formation(
        5,
        TroopKind::Riders,
        vec![tactic(
            "second-charge",
            TacticTrigger::Activation,
            TacticAction::Charge,
            TacticCondition::Always,
            TargetFilter::EnemyFront,
            TargetPriority::OwnColumnFirst,
        )],
        19,
    );
    let battle = input(vec![
        army(
            1,
            1,
            BattleSide::Attacker,
            vec![(0, rider), (1, second_rider)],
        ),
        army(2, 2, BattleSide::Defender, vec![(0, spearman)]),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    let reaction = result
        .events
        .iter()
        .position(|event| matches!(event, BattleEvent::Reaction { .. }))
        .unwrap();
    let retaliation = result
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                BattleEvent::Damage {
                    source: BattleUnitId::Formation(FormationId(2)),
                    target: BattleUnitId::Formation(FormationId(1)),
                    ..
                }
            )
        })
        .unwrap();
    let charge_damage = result
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                BattleEvent::Damage {
                    source: BattleUnitId::Formation(FormationId(1)),
                    target: BattleUnitId::Formation(FormationId(2)),
                    ..
                }
            )
        })
        .unwrap();
    assert!(reaction < retaliation && retaliation < charge_damage);
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(event, BattleEvent::Reaction { .. }))
            .count(),
        1
    );
}

#[test]
fn identical_inputs_replay_identically_and_all_waits_end_at_the_round_cap() {
    let mut rules = rules();
    rules.max_rounds = 8;
    let battle = input(vec![
        army(
            1,
            1,
            BattleSide::Attacker,
            vec![(
                0,
                formation(1, TroopKind::Warriors, vec![wait_rule("wait")], 5),
            )],
        ),
        army(
            2,
            2,
            BattleSide::Defender,
            vec![(
                0,
                formation(2, TroopKind::Warriors, vec![wait_rule("wait")], 5),
            )],
        ),
    ]);

    let first = resolve_battle(&battle, &rules).unwrap();
    let second = resolve_battle(&battle, &rules).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.opening_morale, rules.initial_morale);
    assert_eq!(first.completed_rounds, 8);
    assert_eq!(first.outcome, BattleOutcome::Stalemate);
    assert_eq!(first.reason, BattleResolutionReason::RoundLimit);
    assert_eq!(first.events.len(), 25);
}

#[test]
fn morale_rout_clears_positions_and_applies_bounded_ally_shock() {
    let mut rules = rules();
    rules.max_rounds = 1;
    rules.rout_morale = 75;
    rules.ally_rout_morale_loss = 30;
    let mut attacker = formation(
        1,
        TroopKind::Warriors,
        vec![tactic(
            "press-front",
            TacticTrigger::Activation,
            TacticAction::Attack,
            TacticCondition::Always,
            TargetFilter::EnemyFront,
            TargetPriority::OwnColumnFirst,
        )],
        20,
    );
    attacker.attack = 10;
    let mut target = formation(9, TroopKind::Warriors, vec![wait_rule("hold")], 1);
    target.resistance = 4;
    let ally = formation(10, TroopKind::Archers, vec![wait_rule("wait")], 0);
    let battle = input(vec![
        army(1, 1, BattleSide::Attacker, vec![(0, attacker)]),
        army(2, 2, BattleSide::Defender, vec![(0, target), (1, ally)]),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    assert_eq!(result.reason, BattleResolutionReason::Rout);
    assert_eq!(result.outcome, BattleOutcome::AttackerVictory);
    let routed = result
        .units
        .iter()
        .find(|unit| unit.id == BattleUnitId::Formation(FormationId(9)))
        .unwrap();
    assert_eq!(routed.headcount, 75);
    assert_eq!(routed.slot, None);
    assert!(routed.is_routed);
    let shocked = result
        .units
        .iter()
        .find(|unit| unit.id == BattleUnitId::Formation(FormationId(10)))
        .unwrap();
    assert_eq!(shocked.morale, 70);
    assert!(shocked.is_routed);
}

#[test]
fn advance_uses_its_activation_to_occupy_an_empty_front_slot() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let reserve = formation(
        1,
        TroopKind::Spearmen,
        vec![tactic(
            "advance-into-gap",
            TacticTrigger::Activation,
            TacticAction::Advance,
            TacticCondition::Always,
            TargetFilter::None,
            TargetPriority::OwnColumnFirst,
        )],
        20,
    );
    let battle = input(vec![
        army(1, 1, BattleSide::Attacker, vec![(3, reserve)]),
        army(
            2,
            2,
            BattleSide::Defender,
            vec![(
                0,
                formation(2, TroopKind::Warriors, vec![wait_rule("wait")], 1),
            )],
        ),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::PositionChanged {
            unit: BattleUnitId::Formation(FormationId(1)),
            from: BattlePosition { slot: 3, .. },
            to: BattlePosition { slot: 0, .. },
        }
    )));
    assert_eq!(
        result
            .units
            .iter()
            .find(|unit| unit.id == BattleUnitId::Formation(FormationId(1)))
            .unwrap()
            .slot,
        Some(0)
    );
}

#[test]
fn equal_initiative_alternates_side_priority_between_rounds() {
    let mut rules = rules();
    rules.max_rounds = 2;
    let attack = |id| {
        formation(
            id,
            TroopKind::Warriors,
            vec![tactic(
                "attack",
                TacticTrigger::Activation,
                TacticAction::Attack,
                TacticCondition::Always,
                TargetFilter::EnemyFront,
                TargetPriority::OwnColumnFirst,
            )],
            10,
        )
    };
    let mut left = attack(1);
    let mut right = attack(2);
    left.attack = 1;
    left.resistance = 100;
    right.attack = 1;
    right.resistance = 100;
    let battle = input(vec![
        army(1, 1, BattleSide::Attacker, vec![(0, left)]),
        army(2, 2, BattleSide::Defender, vec![(0, right)]),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    let activations: Vec<_> = result
        .events
        .iter()
        .filter_map(|event| match event {
            BattleEvent::Activation { actor, .. } => Some(*actor),
            _ => None,
        })
        .collect();
    assert_eq!(
        activations,
        [
            BattleUnitId::Formation(FormationId(1)),
            BattleUnitId::Formation(FormationId(2)),
            BattleUnitId::Formation(FormationId(2)),
            BattleUnitId::Formation(FormationId(1)),
        ]
    );
}

#[test]
fn multiple_army_boards_keep_tagged_threat_combatants_without_fake_formations() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let threat = unit(
        UnitProfile {
            id: BattleUnitId::Threat(ThreatId(44)),
            kind: None,
            headcount: 400,
            attack: 12,
            resistance: 8,
            initiative: 1,
        },
        vec![wait_rule("wait")],
        Vec::new(),
    );
    let battle = input(vec![
        army(
            1,
            1,
            BattleSide::Attacker,
            vec![(
                0,
                formation(1, TroopKind::Spearmen, vec![wait_rule("wait")], 2),
            )],
        ),
        army(2, 2, BattleSide::Defender, vec![(0, threat)]),
        army(
            3,
            5,
            BattleSide::Attacker,
            vec![(
                0,
                formation(3, TroopKind::Archers, vec![wait_rule("wait")], 1),
            )],
        ),
        army(
            4,
            3,
            BattleSide::Defender,
            vec![(
                0,
                formation(4, TroopKind::Warriors, vec![wait_rule("wait")], 1),
            )],
        ),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    assert_eq!(result.opening.armies.len(), 4);
    assert!(result
        .units
        .iter()
        .any(|unit| unit.id == BattleUnitId::Threat(ThreatId(44))));
    assert!(!result
        .units
        .iter()
        .any(|unit| unit.id == BattleUnitId::Formation(FormationId(44))));
}
