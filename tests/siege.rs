//! Five persistent siege contracts through actual movement, combat and boundaries.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

#[path = "support/relations.rs"]
mod relations;
use relations::sync_relations;

use kestrum::{
    data::{
        economy::TroopKind,
        world::{DiplomaticState, FactionId, MilitaryLayer, SiteId},
        GameData,
    },
    engine::{apply, preview, reconcile_sieges, recovery_preview, Actor, Command, MoveOrder},
    state::{
        battle::{BattleContext, BattleOutcome, BattleReport},
        military::{ArmyId, FormationId},
        siege::{SiegeAction, SiegeOrder},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
#[path = "support/siege.rs"]
mod support;
use support::*;

#[test]
fn defended_arrival_partitions_real_armies_and_empty_forts_capture_immediately() {
    let (data, mut campaign) = fixture(false, 100, 100);
    let before = campaign.clone();
    preview(&campaign, &data, Actor::Player, enter(&[1], 8, 9)).unwrap();
    assert_eq!(campaign, before);
    let result = apply(&mut campaign, &data, Actor::Player, enter(&[1], 8, 9)).unwrap();
    assert!(result.battle.is_none());
    let siege = &campaign.sieges[&SiteId(9)];
    assert_eq!(siege.defending, vec![ArmyId(3)]);
    assert_eq!(siege.besieging, vec![ArmyId(1)]);
    assert_eq!(
        campaign.armies[&ArmyId(1)].site,
        campaign.armies[&ArmyId(3)].site
    );
    assert_eq!(campaign.army_movement_remaining(ArmyId(1), &data), Some(0));
    assert!(!campaign.army_is_supplied(ArmyId(3)));
    assert!(campaign.army_is_supplied(ArmyId(1)));
    assert_eq!(
        campaign.world.site(SiteId(9)).unwrap().controller,
        Some(FactionId(3))
    );
    assert_eq!(reload(&campaign, &data), campaign);
    assert_rejected(&mut campaign, &data, Actor::Player, enter(&[1], 9, 8));
    assert_rejected(
        &mut campaign,
        &data,
        Actor::Player,
        order(SiegeAction::Assault, &[1], None),
    );
    let (data, mut empty) = fixture(false, 100, 100);
    empty.remove_formation(FormationId(7)).unwrap();
    apply(&mut empty, &data, Actor::Player, enter(&[1], 8, 9)).unwrap();
    assert!(empty.sieges.is_empty() && empty.battles.is_empty());
    assert_eq!(
        empty.world.site(SiteId(9)).unwrap().controller,
        Some(FactionId(1))
    );
    assert_eq!(
        empty.world.structural_damage(SiteId(9)),
        data.combat.capture_damage
    );
    assert_eq!(empty.world.occupation[&SiteId(9)], 100);
    assert_admission();
}

#[test]
fn maintenance_weakens_once_per_boundary_and_endpoint_supply_never_relays() {
    let (data, mut campaign) = fixture(false, 50, 50);
    establish(&mut campaign, &data, false);
    let starting = campaign.clone();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        order(SiegeAction::Maintain, &[1], None),
    )
    .unwrap();
    assert_eq!(campaign.sieges, starting.sieges);
    assert_eq!(campaign.rng, starting.rng);
    let defender = campaign.formations[&FormationId(7)].headcount;
    let mut restored = reload(&campaign, &data);
    finish(&mut campaign, &data);
    finish(&mut restored, &data);
    assert_eq!(campaign, restored);
    assert_eq!(campaign.sieges[&SiteId(9)].elapsed_steps, 1);
    assert_eq!(campaign.world.fort_damage[&SiteId(9)], 5);
    assert_eq!(
        campaign.siege_wall_permille(SiteId(9), &data.siege),
        Some(1410)
    );
    assert_eq!(campaign.formations[&FormationId(7)].headcount, defender);
    assert!(campaign.formations[&FormationId(1)].headcount > 50);
    for _ in 0..20 {
        finish(&mut campaign, &data);
    }
    assert_eq!(campaign.world.fort_damage[&SiteId(9)], 100);
    assert_eq!(
        campaign.siege_wall_permille(SiteId(9), &data.siege),
        Some(1000)
    );
    assert_eq!(campaign.formations[&FormationId(7)].headcount, defender);
    assert!(campaign.sieges.contains_key(&SiteId(9)));
    assert_adjacent_supply();
    assert_civilian_supply();
}

#[test]
fn assaults_apply_real_wall_arithmetic_engine_advantage_and_lasting_damage() {
    let (data, mut campaign) = fixture(false, 100, 100);
    add(
        &mut campaign,
        &data,
        Force {
            id: 5,
            owner: 1,
            site: 8,
            formation: 13,
            kind: TroopKind::SiegeEngines,
            headcount: 20,
        },
    );
    establish_group(&mut campaign, &data, &[1, 5]);
    finish(&mut campaign, &data);
    let mut low_bonus = data.clone();
    low_bonus.siege.engine_wall_reduction_permille = 1;
    let mut comparison = campaign.clone();
    let ordinary = fight(
        &mut comparison,
        &low_bonus,
        Actor::Player,
        SiegeAction::Assault,
        &[1, 5],
        None,
    );
    let report = fight(
        &mut campaign,
        &data,
        Actor::Player,
        SiegeAction::Assault,
        &[1, 5],
        None,
    );
    assert!(matches!(report.context, BattleContext::Assault { .. }));
    assert_eq!(report.wall_permille, 1410);
    assert_eq!(report.exchanges[0].wall_permille, 1210);
    let resistance = |report: &kestrum::state::battle::BattleReport| {
        report
            .simulation
            .as_ref()
            .unwrap()
            .opening
            .armies
            .iter()
            .flat_map(|army| army.slots.iter().flatten())
            .find(|unit| {
                unit.id
                    == kestrum::state::battle::simulation::BattleUnitId::Formation(FormationId(7))
            })
            .unwrap()
            .resistance
    };
    assert!(resistance(&report) < resistance(&ordinary));
    assert_eq!(report.fort_damage_added, 10);
    assert_eq!(campaign.world.fort_damage[&SiteId(9)], 15);
    assert!(report.structural_damage_added >= 10);
    assert_eq!(reload(&campaign, &data), campaign);
    assert_rejected(
        &mut campaign,
        &data,
        Actor::Player,
        order(SiegeAction::Assault, &[1, 5], None),
    );
    assert_assault_capture_and_roads();
    assert_opening_engine_wall_factor();
}

#[test]
fn sortie_and_escape_use_explicit_fallback_without_reviving_casualties() {
    let (data, mut campaign) = fixture(true, 100, 100);
    establish(&mut campaign, &data, true);
    finish(&mut campaign, &data);
    let report = fight(
        &mut campaign,
        &data,
        Actor::Player,
        SiegeAction::Sortie,
        &[1],
        None,
    );
    assert!(matches!(
        report.outcome,
        BattleOutcome::Stalemate | BattleOutcome::DefenderVictory
    ));
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(9));
    assert!(campaign.formations[&FormationId(1)].headcount < 100);
    assert!(campaign.sieges.contains_key(&SiteId(9)));
    assert_eq!(report.exchanges[0].wall_permille, 1000);
    assert_escape(false);
    assert_escape(true);
    let (data, mut trapped) = fixture(true, 100, 100);
    for site in [14, 10] {
        trapped
            .set_site_control(&data, SiteId(site), Some(FactionId(2)), false)
            .unwrap();
    }
    establish(&mut trapped, &data, true);
    finish(&mut trapped, &data);
    assert_rejected(
        &mut trapped,
        &data,
        Actor::Player,
        order(SiegeAction::Escape, &[1], Some(14)),
    );
}

#[test]
fn relief_third_factions_withdrawal_and_peace_reconcile_surviving_ids() {
    assert_relief(true);
    assert_relief(false);
    assert_third_faction();
    let (data, mut campaign) = fixture(false, 100, 100);
    establish(&mut campaign, &data, false);
    finish(&mut campaign, &data);
    let damage = campaign.world.fort_damage[&SiteId(9)];
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        order(SiegeAction::Withdraw, &[1], Some(8)),
    )
    .unwrap();
    assert!(campaign.sieges.is_empty());
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(8));
    assert_eq!(campaign.world.fort_damage[&SiteId(9)], damage);
    finish(&mut campaign, &data);
    assert_eq!(campaign.world.fort_damage[&SiteId(9)], damage);
    assert_peace(false);
    assert_peace(true);
    assert_elimination();
}
