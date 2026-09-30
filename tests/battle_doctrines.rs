//! Doctrine snapshots, campaign templates and deterministic rival preparation.

use kestrum::{
    data::{
        battle_tactics::{
            BattleDoctrine, TacticAction, TacticCondition, TacticRule, TacticTrigger, TargetFilter,
            TargetPriority,
        },
        economy::TroopKind,
        world::SiteId,
        GameData,
    },
    engine::{self, apply, Actor, Command, MoveOrder, ObservedTroopPosition},
    state::{
        battle::simulation::BattleUnitId,
        military::{ArmyId, FormationId},
        Campaign, StrategicCampaign,
    },
};

#[path = "support/battle_doctrines.rs"]
mod support;

use support::*;

#[test]
fn each_doctrine_resolves_legal_rules_for_all_six_troop_roles() {
    let data = GameData::load().unwrap();
    data.battle_tactics.validate().unwrap();
    let kinds = [
        TroopKind::Warriors,
        TroopKind::Spearmen,
        TroopKind::Archers,
        TroopKind::Riders,
        TroopKind::Medics,
        TroopKind::SiegeEngines,
    ];
    for doctrine in BattleDoctrine::ALL {
        for kind in kinds {
            let tactics = data.battle_tactics.doctrine_for(doctrine, kind).unwrap();
            assert!(!tactics.activation.is_empty());
            data.battle_tactics
                .validate_configuration(kind, tactics)
                .unwrap();
        }
    }
}

#[test]
fn applying_a_doctrine_snapshots_every_unit_and_preserves_explicit_overrides() {
    let (data, mut campaign) = fixture(80, 80);
    let non_override = campaign.next_ids.formation;
    add_spearmen(&mut campaign, &data, ArmyId(1), 100);
    apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 10])).unwrap();
    let movement = campaign.pending_battle.as_ref().unwrap().movement.clone();
    let mut custom = data
        .battle_tactics
        .defaults_for(TroopKind::Warriors)
        .unwrap()
        .clone();
    custom.activation = vec![TacticRule {
        id: "hold-my-order".into(),
        trigger: TacticTrigger::Activation,
        action: TacticAction::Wait,
        condition: TacticCondition::Always,
        target_filter: TargetFilter::None,
        target_priority: TargetPriority::OwnColumnFirst,
    }];
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetFormationTactics {
            formation: FormationId(1),
            tactics: custom.clone(),
        },
    )
    .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetBattleDoctrine {
            army: ArmyId(1),
            doctrine: BattleDoctrine::RangedSupport,
        },
    )
    .unwrap();
    assert_eq!(campaign.formations[&FormationId(1)].tactics, Some(custom));
    assert_eq!(
        campaign.formations[&FormationId(1)].tactics_override,
        Some(true)
    );
    assert_eq!(
        campaign.formations[&non_override].tactics,
        Some(
            data.battle_tactics
                .doctrine_for(BattleDoctrine::RangedSupport, TroopKind::Spearmen)
                .unwrap()
                .clone()
        )
    );
    assert_eq!(
        campaign.formations[&non_override].tactics_override,
        Some(false)
    );
    let pending = campaign.pending_battle.as_ref().unwrap();
    assert_eq!(
        pending.report.attacker.armies[0].battle_doctrine,
        Some(BattleDoctrine::RangedSupport)
    );
    assert_eq!(pending.movement, movement);
    assert_eq!(
        campaign.armies[&ArmyId(1)].battle_doctrine,
        Some(BattleDoctrine::RangedSupport)
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn saved_templates_roundtrip_without_person_or_formation_identity() {
    let (data, mut campaign) = fixture(80, 80);
    add_spearmen(&mut campaign, &data, ArmyId(1), 100);
    apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 10])).unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetBattleDoctrine {
            army: ArmyId(1),
            doctrine: BattleDoctrine::DefensiveLine,
        },
    )
    .unwrap();
    let mut custom = data
        .battle_tactics
        .defaults_for(TroopKind::Warriors)
        .unwrap()
        .clone();
    custom.activation[0].action = TacticAction::Wait;
    custom.activation[0].target_filter = TargetFilter::None;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetFormationTactics {
            formation: FormationId(1),
            tactics: custom.clone(),
        },
    )
    .unwrap();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SaveBattleTemplate {
            army: ArmyId(1),
            name: "North Watch".into(),
        },
    )
    .unwrap();
    let template_json = serde_json::to_string(&campaign.battle_templates).unwrap();
    assert!(!template_json.contains("PersonId"));
    assert!(!template_json.contains("FormationId"));
    let saved = serde_json::to_string(&Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    let loaded: Campaign = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.strategic().unwrap(), &campaign);

    let mut later_custom = custom.clone();
    later_custom.activation[0].action = TacticAction::Guard;
    later_custom.activation[0].target_filter = TargetFilter::None;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetFormationTactics {
            formation: FormationId(1),
            tactics: later_custom.clone(),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.formations[&FormationId(1)].tactics,
        Some(later_custom)
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::ApplyBattleTemplate {
            army: ArmyId(1),
            name: "North Watch".into(),
        },
    )
    .unwrap();
    assert_eq!(
        campaign.armies[&ArmyId(1)].battle_doctrine,
        Some(BattleDoctrine::DefensiveLine)
    );
    assert_eq!(campaign.formations[&FormationId(1)].tactics, Some(custom));
    assert_eq!(
        campaign.formations[&FormationId(1)].tactics_override,
        Some(true)
    );
    campaign.validate(&data).unwrap();
}

#[test]
fn rival_plan_uses_observed_roles_and_fills_front_and_support_slots_deterministically() {
    let own = [
        ObservedTroopPosition {
            army_index: 0,
            kind: TroopKind::Riders,
            slot: 5,
        },
        ObservedTroopPosition {
            army_index: 0,
            kind: TroopKind::Warriors,
            slot: 3,
        },
        ObservedTroopPosition {
            army_index: 0,
            kind: TroopKind::Archers,
            slot: 4,
        },
    ];
    let gap = [ObservedTroopPosition {
        army_index: 0,
        kind: TroopKind::Spearmen,
        slot: 3,
    }];
    let first = engine::plan_rival_battle(&own, &gap);
    let second = engine::plan_rival_battle(&own, &gap);
    assert_eq!(first, second);
    assert_eq!(first.doctrine, BattleDoctrine::Breakthrough);
    assert!(first.slot_moves.contains(&engine::RivalSlotMove {
        army_index: 0,
        from: 5,
        to: 0
    }));
    assert!(first.slot_moves.contains(&engine::RivalSlotMove {
        army_index: 0,
        from: 4,
        to: 3
    }));

    let cavalry = [ObservedTroopPosition {
        army_index: 0,
        kind: TroopKind::Riders,
        slot: 0,
    }];
    let spearmen = [ObservedTroopPosition {
        army_index: 0,
        kind: TroopKind::Spearmen,
        slot: 4,
    }];
    let counter = engine::plan_rival_battle(&spearmen, &cavalry);
    assert_eq!(counter.doctrine, BattleDoctrine::DefensiveLine);
    assert!(counter.slot_moves.contains(&engine::RivalSlotMove {
        army_index: 0,
        from: 4,
        to: 0
    }));
}

#[test]
fn npc_breakthrough_targets_an_observed_rear_gap_and_prepared_spears_brace() {
    let (data, mut campaign) = fixture(80, 80);
    kind(&mut campaign, &data, 1, TroopKind::Spearmen, 80);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().slots =
        [None, None, None, Some(FormationId(1)), None, None];
    let rider_count = 80.min(data.economy.formations[&TroopKind::Riders].capacity);
    kind(&mut campaign, &data, 7, TroopKind::Riders, rider_count);
    campaign.validate(&data).unwrap();
    apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 10])).unwrap();
    let pending = campaign.pending_battle.as_ref().unwrap();
    let rival = pending
        .report
        .defender
        .faction_side()
        .unwrap()
        .armies
        .first()
        .unwrap();
    assert_eq!(rival.battle_doctrine, Some(BattleDoctrine::Breakthrough));
    assert!(rival.ai_prepared);
    assert_eq!(
        pending.report.defender.faction_side().unwrap().armies[0].formations[0].slot,
        0
    );
    let rider_rules = &campaign.formations[&FormationId(7)]
        .tactics
        .as_ref()
        .unwrap()
        .activation;
    assert_eq!(rider_rules[0].action, TacticAction::Breakthrough);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetBattleDoctrine {
            army: ArmyId(1),
            doctrine: BattleDoctrine::DefensiveLine,
        },
    )
    .unwrap();
    let events = &campaign
        .pending_battle
        .as_ref()
        .unwrap()
        .report
        .simulation
        .as_ref()
        .unwrap()
        .events;
    assert!(events.iter().any(|event| matches!(
        event,
        kestrum::state::battle::simulation::BattleEvent::Activation {
            actor: BattleUnitId::Formation(FormationId(7)),
            action: TacticAction::Breakthrough,
            target: Some(BattleUnitId::Formation(FormationId(1))),
            ..
        }
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        kestrum::state::battle::simulation::BattleEvent::Reaction {
            actor: BattleUnitId::Formation(FormationId(1)),
            against: BattleUnitId::Formation(FormationId(7)),
            ..
        }
    )));
    campaign.validate(&data).unwrap();
}

#[test]
fn doctrine_application_keeps_legacy_saved_tactics_explicit() {
    let (data, mut campaign) = fixture(80, 80);
    let mut custom = data
        .battle_tactics
        .defaults_for(TroopKind::Warriors)
        .unwrap()
        .clone();
    custom.activation[0].action = TacticAction::Wait;
    custom.activation[0].target_filter = TargetFilter::None;
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .tactics = Some(custom);
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .tactics_override = Some(true);
    let mut value = serde_json::to_value(Campaign::Strategic(Box::new(campaign))).unwrap();
    value["formations"]["1"]
        .as_object_mut()
        .unwrap()
        .remove("tactics_override");
    let loaded: Campaign = serde_json::from_value(value).unwrap();
    let formation = &loaded.strategic().unwrap().formations[&FormationId(1)];
    assert!(formation.has_explicit_tactics());
    assert_eq!(formation.tactics_override, None);
}

fn add_spearmen(campaign: &mut StrategicCampaign, data: &GameData, army: ArmyId, count: u32) {
    let id = campaign.next_ids.formation;
    let mut formation = campaign.formations[&FormationId(1)].clone();
    formation.id = id;
    formation.kind = TroopKind::Spearmen;
    formation.capacity = data.economy.formations[&TroopKind::Spearmen].capacity;
    formation.headcount = count;
    formation.movement_spent = 0;
    formation.battle_leader = None;
    formation.tactics = None;
    formation.tactics_override = Some(false);
    campaign.formations.insert(id, formation);
    campaign.armies.get_mut(&army).unwrap().slots[1] = Some(id);
    campaign.next_ids.formation = FormationId(id.0 + 1);
}
