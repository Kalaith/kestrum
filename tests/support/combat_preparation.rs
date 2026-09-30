use super::*;

#[test]
fn pending_tactics_are_saved_replayed_and_unavailable_actions_use_the_visible_fallback() {
    let (data, mut campaign) = fixture(80, 80);
    apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 10])).unwrap();
    let receipt = campaign.pending_battle.as_ref().unwrap().movement.clone();
    let before_sequence = campaign.accepted_sequence;
    let rng = campaign.rng.clone();
    let mut tactics = data
        .battle_tactics
        .defaults_for(TroopKind::Warriors)
        .unwrap()
        .clone();
    tactics.activation = vec![
        TacticRule {
            id: "unavailable_volley".into(),
            trigger: TacticTrigger::Activation,
            action: TacticAction::Volley,
            condition: TacticCondition::Always,
            target_filter: TargetFilter::AnyEnemy,
            target_priority: TargetPriority::OwnColumnFirst,
        },
        TacticRule {
            id: "visible_wait".into(),
            trigger: TacticTrigger::Activation,
            action: TacticAction::Wait,
            condition: TacticCondition::Always,
            target_filter: TargetFilter::None,
            target_priority: TargetPriority::OwnColumnFirst,
        },
    ];
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetFormationTactics {
            formation: FormationId(1),
            tactics: tactics.clone(),
        },
    )
    .unwrap();
    let pending = campaign.pending_battle.as_ref().unwrap();
    assert_eq!(pending.report.sequence, before_sequence + 1);
    assert_eq!(pending.movement, receipt);
    assert_eq!(
        campaign.formations[&FormationId(1)].tactics.as_ref(),
        Some(&tactics)
    );
    assert_eq!(campaign.rng, rng);
    let resolution = pending.report.simulation.as_ref().unwrap().clone();
    assert!(resolution.events.iter().any(|event| matches!(
        event,
        kestrum::state::battle::simulation::BattleEvent::Activation {
            actor: BattleUnitId::Formation(FormationId(1)),
            action: TacticAction::Wait,
            skipped,
            ..
        } if skipped.iter().any(|entry| {
            entry.rule_id == "unavailable_volley"
                && entry.reason == kestrum::state::battle::simulation::TacticSkipReason::ActionUnavailable
        })
    )));

    let saved = serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let loaded: Campaign = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.strategic().unwrap(), &campaign);
    let accepted = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    let committed = &campaign.battles[&accepted.battle.unwrap()];
    assert_eq!(committed.simulation, Some(resolution));
    let separated = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: FormationId(1),
        },
    )
    .unwrap()
    .split_army
    .unwrap();
    assert_eq!(campaign.armies[&separated].slots[0], Some(FormationId(1)));
    assert_eq!(
        campaign.formations[&FormationId(1)].tactics.as_ref(),
        Some(&tactics)
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn pending_slot_swaps_preserve_formation_identities_and_update_the_witnessed_board() {
    let (data, mut campaign) = fixture(80, 80);
    let extra = campaign.next_ids.formation;
    let mut formation = campaign.formations[&FormationId(1)].clone();
    formation.id = extra;
    formation.tactics = None;
    campaign.formations.insert(extra, formation);
    campaign.next_ids.formation = FormationId(extra.0 + 1);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().slots[1] = Some(extra);
    campaign.validate(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 10])).unwrap();
    let movement = campaign.pending_battle.as_ref().unwrap().movement.clone();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SwapFormationSlots {
            army: ArmyId(1),
            first: 0,
            second: 1,
        },
    )
    .unwrap();
    assert_eq!(campaign.armies[&ArmyId(1)].slots[0], Some(extra));
    assert_eq!(campaign.armies[&ArmyId(1)].slots[1], Some(FormationId(1)));
    let pending = campaign.pending_battle.as_ref().unwrap();
    assert_eq!(pending.movement, movement);
    let roster = &pending.report.attacker.armies[0].formations;
    assert!(roster
        .iter()
        .any(|entry| entry.id == extra && entry.slot == 0));
    assert!(roster
        .iter()
        .any(|entry| entry.id == FormationId(1) && entry.slot == 1));
    let board = pending
        .report
        .simulation
        .as_ref()
        .unwrap()
        .opening
        .armies
        .iter()
        .find(|army| army.id == ArmyId(1))
        .unwrap();
    assert_eq!(
        board.slots[0].as_ref().unwrap().id,
        BattleUnitId::Formation(extra)
    );
    assert_eq!(
        board.slots[1].as_ref().unwrap().id,
        BattleUnitId::Formation(FormationId(1))
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn player_and_npc_preparation_commands_can_edit_only_their_own_battle_side() {
    let (data, mut campaign) = fixture(80, 80);
    apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 10])).unwrap();
    let defender_kind = campaign.formations[&FormationId(7)].kind;
    let tactics = data
        .battle_tactics
        .defaults_for(defender_kind)
        .unwrap()
        .clone();
    apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(3)),
        Command::SetFormationTactics {
            formation: FormationId(7),
            tactics: tactics.clone(),
        },
    )
    .unwrap();
    assert_eq!(campaign.formations[&FormationId(7)].tactics, Some(tactics));

    let before = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(3)),
        Command::SetFormationTactics {
            formation: FormationId(1),
            tactics: data
                .battle_tactics
                .defaults_for(TroopKind::Warriors)
                .unwrap()
                .clone(),
        },
    )
    .is_err());
    assert_eq!(campaign, before);

    assert!(apply(
        &mut campaign,
        &data,
        Actor::Npc(FactionId(2)),
        Command::SwapFormationSlots {
            army: ArmyId(1),
            first: 0,
            second: 1,
        },
    )
    .is_err());
    assert_eq!(campaign, before);
    campaign.validate(&data).unwrap();
}

#[test]
fn invalid_custom_rows_are_atomic_and_saves_without_tactics_still_load() {
    let (data, mut campaign) = fixture(80, 80);
    apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 10])).unwrap();
    let before = campaign.clone();
    let mut tactics = data
        .battle_tactics
        .defaults_for(TroopKind::Warriors)
        .unwrap()
        .clone();
    for index in tactics.activation.len()..6 {
        tactics.activation.push(TacticRule {
            id: format!("overflow_{index}"),
            trigger: TacticTrigger::Activation,
            action: TacticAction::Wait,
            condition: TacticCondition::Always,
            target_filter: TargetFilter::None,
            target_priority: TargetPriority::OwnColumnFirst,
        });
    }
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetFormationTactics {
            formation: FormationId(1),
            tactics,
        },
    )
    .is_err());
    assert_eq!(campaign, before);

    let mut legacy = serde_json::to_value(Campaign::Strategic(Box::new(campaign))).unwrap();
    for formation in legacy["formations"].as_object_mut().unwrap().values_mut() {
        formation.as_object_mut().unwrap().remove("tactics");
    }
    let migrated: Campaign = serde_json::from_value(legacy).unwrap();
    assert!(migrated
        .strategic()
        .unwrap()
        .formations
        .values()
        .all(|formation| formation.tactics.is_none()));
    migrated.validate(&data).unwrap();
}
