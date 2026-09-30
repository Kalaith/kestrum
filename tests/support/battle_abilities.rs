use super::*;

#[test]
fn officer_rally_restores_morale_once_without_restoring_headcount() {
    let mut rules = rules();
    rules.max_rounds = 2;
    let rally = tactic(
        "rally-under-pressure",
        TacticTrigger::Activation,
        TacticAction::Rally,
        TacticCondition::SelfBelowHalfMorale,
        TargetFilter::None,
        TargetPriority::OwnColumnFirst,
    );
    let rallied = BattleUnitId::Formation(FormationId(1));
    let attacker = formation(2, TroopKind::Warriors, vec![], 20);
    let mut attacker = attacker;
    attacker.attack = 30;
    let mut officer_unit = formation(1, TroopKind::Warriors, vec![rally], 1);
    officer_unit.attack = 10;
    let officer_unit = with_leader(officer_unit, PersonClass::Officer);
    let battle = input(vec![
        army(1, 1, BattleSide::Attacker, vec![(0, officer_unit)]),
        army(2, 2, BattleSide::Defender, vec![(0, attacker)]),
    ]);

    let result = resolve_battle(&battle, &rules).unwrap();
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(event, BattleEvent::AbilityUsed {
                actor,
                ability: BattleCapability::OfficerRally,
                target: None,
            } if *actor == rallied))
            .count(),
        1
    );
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::MoraleChanged { unit, before, after }
            if *unit == rallied && *before < 50 && *after > *before
    )));
    let rallied_final = result.units.iter().find(|unit| unit.id == rallied).unwrap();
    assert!(rallied_final.headcount < 100);
    assert!(rallied_final.morale > 40);
}

#[test]
fn infantry_hold_the_line_advances_into_its_open_front_and_guards_that_round() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let holding = BattleUnitId::Formation(FormationId(1));
    let mut infantry = formation(
        1,
        TroopKind::Warriors,
        vec![
            tactic(
                "hold-line",
                TacticTrigger::Activation,
                TacticAction::HoldTheLine,
                TacticCondition::Always,
                TargetFilter::None,
                TargetPriority::OwnColumnFirst,
            ),
            wait_rule("wait"),
        ],
        50,
    );
    infantry = with_leader(infantry, PersonClass::Infantry);
    let enemy = formation(2, TroopKind::Warriors, vec![], 40);
    let result = resolve_battle(
        &input(vec![
            army(1, 1, BattleSide::Attacker, vec![(3, infantry)]),
            army(2, 2, BattleSide::Defender, vec![(0, enemy)]),
        ]),
        &rules,
    )
    .unwrap();

    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::PositionChanged { unit, from, to }
            if *unit == holding && from.slot == 3 && to.slot == 0
    )));
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::GuardRaised { unit, expires_round: 1 } if *unit == holding
    )));
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::Damage { source: BattleUnitId::Formation(FormationId(2)), target, amount: 7, .. }
            if *target == holding
    )));
}

#[test]
fn medics_stabilize_the_most_shaken_ally_without_returning_casualties() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let ally_low = BattleUnitId::Formation(FormationId(1));
    let medic = BattleUnitId::Formation(FormationId(3));
    let mut low = formation(1, TroopKind::Warriors, vec![wait_rule("wait-low")], 10);
    let mut high = formation(2, TroopKind::Warriors, vec![wait_rule("wait-high")], 10);
    low.resistance = 10;
    high.resistance = 10;
    let medic_rules = rules
        .defaults_for(TroopKind::Medics)
        .unwrap()
        .activation
        .clone();
    let medics = formation(3, TroopKind::Medics, medic_rules, 1);
    let mut enemy_low = formation(4, TroopKind::Warriors, vec![], 50);
    let mut enemy_high = formation(5, TroopKind::Warriors, vec![], 49);
    enemy_low.attack = 60;
    enemy_high.attack = 30;
    let result = resolve_battle(
        &input(vec![
            army(
                1,
                1,
                BattleSide::Attacker,
                vec![(0, low), (1, high), (3, medics)],
            ),
            army(
                2,
                2,
                BattleSide::Defender,
                vec![(0, enemy_low), (1, enemy_high)],
            ),
        ]),
        &rules,
    )
    .unwrap();

    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::AbilityUsed {
            actor,
            ability: BattleCapability::MedicStabilization,
            target: Some(target),
        } if *actor == medic && *target == ally_low
    )));
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::MoraleChanged { unit, before: 40, after: 60 } if *unit == ally_low
    )));
    assert_eq!(
        result
            .units
            .iter()
            .find(|unit| unit.id == ally_low)
            .unwrap()
            .headcount,
        40
    );
}

#[test]
fn cavalry_exploit_opening_hits_an_exposed_rear_before_ordinary_activations() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let rider_id = BattleUnitId::Formation(FormationId(1));
    let target_id = BattleUnitId::Formation(FormationId(2));
    let rider = with_leader(
        formation(1, TroopKind::Riders, vec![], 30),
        PersonClass::Cavalry,
    );
    let mut target = formation(2, TroopKind::Archers, vec![wait_rule("wait")], 1);
    target.headcount = 1000;
    target.capacity = 1000;
    let result = resolve_battle(
        &input(vec![
            army(1, 1, BattleSide::Attacker, vec![(0, rider)]),
            army(2, 2, BattleSide::Defender, vec![(3, target)]),
        ]),
        &rules,
    )
    .unwrap();
    let opening = result
        .events
        .iter()
        .position(|event| matches!(
            event,
            BattleEvent::OpeningAction { actor, ability: BattleCapability::CavalryExploitOpening, target }
                if *actor == rider_id && *target == target_id
        ))
        .unwrap();
    let opening_damage = result
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                BattleEvent::Damage { source, target, amount: 15, .. }
                    if *source == rider_id && *target == target_id
            )
        })
        .unwrap();
    let activation = result
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                BattleEvent::Activation { actor, .. } if *actor == rider_id
            )
        })
        .unwrap();
    assert!(opening < opening_damage && opening_damage < activation);
}

#[test]
fn siege_engines_bombard_once_and_take_extra_close_combat_damage() {
    let mut rules = rules();
    rules.max_rounds = 1;
    let attacker_id = BattleUnitId::Formation(FormationId(1));
    let siege_id = BattleUnitId::Formation(FormationId(2));
    let attacker = formation(1, TroopKind::Warriors, vec![], 30);
    let mut siege = formation(2, TroopKind::SiegeEngines, vec![wait_rule("wait")], 1);
    siege.attack = 1;
    let result = resolve_battle(
        &input(vec![
            army(1, 1, BattleSide::Attacker, vec![(0, attacker)]),
            army(2, 2, BattleSide::Defender, vec![(0, siege)]),
        ]),
        &rules,
    )
    .unwrap();

    assert_eq!(
        result.events.iter().filter(|event| matches!(
            event,
            BattleEvent::OpeningAction { actor, ability: BattleCapability::SiegeBombardment, .. }
                if *actor == siege_id
        )).count(),
        1
    );
    assert!(result.events.iter().any(|event| matches!(
        event,
        BattleEvent::Damage { source, target, amount: 14, .. }
            if *source == attacker_id && *target == siege_id
    )));
}
