//! Regressions from the fresh review of reaction and round-end boundaries.

use super::*;

fn reaction(action: TacticAction) -> TacticRule {
    tactic(
        "react",
        TacticTrigger::IncomingAttack,
        action,
        TacticCondition::Always,
        TargetFilter::None,
        TargetPriority::OwnColumnFirst,
    )
}

#[test]
fn lethal_brace_retaliation_cancels_the_chargers_damage() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let charge = tactic(
        "charge",
        TacticTrigger::Activation,
        TacticAction::Charge,
        TacticCondition::Always,
        TargetFilter::AnyEnemy,
        TargetPriority::OwnColumnFirst,
    );
    let mut rider = formation(1, TroopKind::Riders, vec![charge], 60);
    rider.headcount = 1;
    rider.resistance = 1;
    let mut spear = formation(2, TroopKind::Spearmen, vec![wait_rule("hold")], 1);
    spear.reaction_tactics = vec![reaction(TacticAction::Brace)];
    let result = resolve_battle(
        &input(vec![
            army(1, 1, BattleSide::Attacker, vec![(0, rider)]),
            army(2, 2, BattleSide::Defender, vec![(0, spear)]),
        ]),
        &rules,
    )
    .unwrap();
    assert_eq!(result.units[0].headcount, 0);
    assert_eq!(result.units[1].headcount, 100);
    assert!(!result.events.iter().any(|event| matches!(
        event,
        BattleEvent::Damage {
            source: BattleUnitId::Formation(FormationId(1)),
            ..
        }
    )));
}

#[test]
fn incoming_guard_reduces_each_attack_once_and_brace_skips_ordinary_attacks() {
    for (action, kind, slot, expected) in [
        (TacticAction::Attack, TroopKind::Warriors, 0, 7),
        (TacticAction::Volley, TroopKind::Archers, 0, 7),
        (TacticAction::Charge, TroopKind::Riders, 0, 10),
        (TacticAction::Breakthrough, TroopKind::Riders, 3, 9),
    ] {
        let mut rules = rules();
        rules.max_rounds = 1;
        let attack = tactic(
            "attack",
            TacticTrigger::Activation,
            action,
            TacticCondition::Always,
            TargetFilter::AnyEnemy,
            TargetPriority::OwnColumnFirst,
        );
        let attacker = formation(1, kind, vec![attack], 60);
        let mut defender = formation(2, TroopKind::Warriors, vec![wait_rule("hold")], 1);
        defender.reaction_tactics = vec![reaction(TacticAction::Guard)];
        if matches!(action, TacticAction::Attack | TacticAction::Volley) {
            let mut brace = reaction(TacticAction::Brace);
            brace.id = "brace-only-charges".into();
            defender.reaction_tactics.insert(0, brace);
        }
        let result = resolve_battle(
            &input(vec![
                army(1, 1, BattleSide::Attacker, vec![(0, attacker)]),
                army(2, 2, BattleSide::Defender, vec![(slot, defender)]),
            ]),
            &rules,
        )
        .unwrap();
        assert!(
            result.events.iter().any(|event| matches!(
                event,
                BattleEvent::Reaction { rule_id, .. } if rule_id == "react"
            )),
            "missing Guard against {action:?}"
        );
        assert_eq!(result.units[1].headcount, 100 - expected, "{action:?}");
    }
}

#[test]
fn rout_shock_clears_earlier_units_before_the_next_round() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let mut first_attacker = formation(4, TroopKind::Warriors, vec![], 60);
    first_attacker.attack = 90;
    let mut second_attacker = formation(5, TroopKind::Warriors, vec![], 50);
    second_attacker.attack = 100;
    let defenders = (1..=3)
        .map(|id| {
            let mut defender = formation(id, TroopKind::Warriors, vec![wait_rule("hold")], 1);
            defender.headcount = 1000;
            defender.capacity = 1000;
            ((id - 1) as usize, defender)
        })
        .collect();
    let result = resolve_battle(
        &input(vec![
            army(
                1,
                1,
                BattleSide::Attacker,
                vec![(0, first_attacker), (1, second_attacker)],
            ),
            army(2, 2, BattleSide::Defender, defenders),
        ]),
        &rules,
    )
    .unwrap();
    for id in [1, 2] {
        let unit = result
            .units
            .iter()
            .find(|unit| unit.id == BattleUnitId::Formation(FormationId(id)))
            .unwrap();
        assert!(
            unit.is_routed && unit.slot.is_none(),
            "unit {id} remained in the line at zero morale"
        );
    }
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(event, BattleEvent::Routed { .. }))
            .count(),
        2
    );
}

#[test]
fn round_limit_power_excludes_routed_survivors() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let mut attacker = formation(1, TroopKind::Warriors, vec![], 60);
    attacker.attack = 100;
    let mut routed = formation(2, TroopKind::Warriors, vec![wait_rule("hold")], 1);
    routed.headcount = 1000;
    routed.capacity = 1000;
    let survivor = formation(3, TroopKind::Warriors, vec![wait_rule("hold")], 1);
    let result = resolve_battle(
        &input(vec![
            army(1, 1, BattleSide::Attacker, vec![(0, attacker)]),
            army(2, 2, BattleSide::Defender, vec![(0, routed), (1, survivor)]),
        ]),
        &rules,
    )
    .unwrap();
    assert_eq!(result.reason, BattleResolutionReason::RoundLimit);
    assert_eq!(result.outcome, BattleOutcome::AttackerVictory);
    assert_eq!(result.units[1].headcount, 900, "routed troops remain alive");
}
