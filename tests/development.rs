//! Five P19 contracts cover real seasonal development and administration.

use kestrum::{
    data::{
        economy::{Habitation, Resources},
        world::{DiplomaticState, Facility, FactionId, Geography, MilitaryLayer, SiteId, SiteTag},
        GameData,
    },
    engine::{advance_npc, apply, development_view, preview, Actor, Command},
    state::{
        construction::{ConstructionKind, ConstructionStatus, ConstructionTarget, Focus},
        military::ArmyId,
        Campaign, CampaignPhase, StrategicCampaign,
    },
};
#[path = "support/development.rs"]
mod support;
use support::*;

#[test]
fn pressure_uses_real_population_one_tier_limits_and_all_geography_caps() {
    let (data, mut campaign) = fixture();
    campaign.world.population.insert(SiteId(5), 600);
    site_mut(&mut campaign, 6).habitation = Habitation::Unsettled;
    campaign.world.population.insert(SiteId(6), 0);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    let first = development_view(&campaign, &data, FactionId(1), SiteId(5)).unwrap();
    assert_eq!(first.contribution, Some(4));
    assert!(development_view(&campaign, &data, FactionId(2), SiteId(5)).is_none());
    for _ in 0..5 {
        finish(&mut campaign, &data);
        assert_eq!(
            campaign.world.site(SiteId(5)).unwrap().habitation,
            Habitation::Village
        );
    }
    let mut resumed = reload(&campaign, &data);
    finish(&mut campaign, &data);
    finish(&mut resumed, &data);
    assert_eq!(campaign, resumed);
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().habitation,
        Habitation::Town
    );
    assert_eq!(campaign.world.development[&SiteId(5)].pressure, 0);
    assert_eq!(campaign.world.population[&SiteId(5)], 636);
    assert_eq!(
        campaign.world.site(SiteId(1)).unwrap().habitation,
        Habitation::Village
    );
    assert_eq!(
        campaign.world.development[&SiteId(1)].pressure,
        24,
        "blocked positive transition retains pressure"
    );
    assert_eq!(
        (
            campaign.world.site(SiteId(6)).unwrap().habitation,
            campaign.world.population[&SiteId(6)]
        ),
        (Habitation::Unsettled, 0)
    );
    grow_capacity_cases();
    assert_builder_population_survives_snapshot();
    assert_remote_forecast_privacy();
}

#[test]
fn decline_ruin_and_reclamation_preserve_population_identity_and_fort_layer() {
    let (data, mut campaign) = fixture();
    site_mut(&mut campaign, 5).military = MilitaryLayer::Fort;
    campaign.world.fort_damage.insert(SiteId(5), 70);
    campaign.world.site_damage.insert(SiteId(5), 100);
    campaign.world.population.insert(SiteId(5), 19);
    campaign
        .world
        .development
        .get_mut(&SiteId(5))
        .unwrap()
        .pressure = -20;
    // An actual hostile adjacent army prevents repair and supplies the unsafe condition.
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [FactionId(1), FactionId(2)])
        .unwrap()
        .state = DiplomaticState::War;
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(6);
    for expected in 1..=4 {
        finish(&mut campaign, &data);
        assert_eq!(campaign.world.development[&SiteId(5)].ruin_streak, expected);
        assert_eq!(campaign.site_is_ruined(SiteId(5)), expected == 4);
    }
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().military,
        MilitaryLayer::Fort
    );
    assert_eq!(campaign.world.population[&SiteId(5)], 19);
    assert_eq!(campaign.world.fort_damage[&SiteId(5)], 70);
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(2);
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [FactionId(1), FactionId(2)])
        .unwrap()
        .state = DiplomaticState::Peace;
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    let id = campaign.next_ids.order;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(SiteId(5)),
            kind: ConstructionKind::Outpost,
            builder: ArmyId(1),
        },
    )
    .unwrap();
    for _ in 0..3 {
        finish(&mut campaign, &data);
    }
    assert!(matches!(
        campaign.construction[&id].status,
        ConstructionStatus::Completed { .. }
    ));
    assert!(!campaign.site_is_ruined(SiteId(5)));
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().habitation,
        Habitation::Outpost
    );
    assert_eq!(campaign.world.structural_damage(SiteId(5)), 50);
    assert_eq!(
        campaign.world.fort_damage[&SiteId(5)],
        66,
        "first two safe boundaries repair; completion preserves remaining fort damage"
    );
    assert_eq!(campaign.world.development[&SiteId(5)].ruination, 1);
    assert_eq!(reload(&campaign, &data), campaign);
}

#[test]
fn occupation_repairs_and_matching_focus_use_one_final_income_floor() {
    let (data, mut campaign) = fixture();
    campaign.world.site_damage.insert(SiteId(5), 51);
    campaign.world.occupation.insert(SiteId(5), 50);
    campaign.world.focus.insert(SiteId(5), Focus::Gold);
    site_mut(&mut campaign, 5).military = MilitaryLayer::Fort;
    campaign.world.fort_damage.insert(SiteId(5), 20);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    let base = data.economy.settlement_income[&Habitation::Village];
    finish(&mut campaign, &data);
    assert_income(
        &campaign,
        &data,
        5,
        Resources {
            gold: base.gold * 75 * 50 * 125 / 1_000_000,
            wood: base.wood * 75 * 50 / 10_000,
            stone: base.stone * 75 * 50 / 10_000,
        },
    );
    assert_eq!(campaign.world.occupation[&SiteId(5)], 40);
    assert_eq!(campaign.world.structural_damage(SiteId(5)), 46);
    assert_eq!(campaign.world.fort_damage[&SiteId(5)], 18);
    reject(
        &mut campaign,
        &data,
        Command::SetFocus {
            site: SiteId(5),
            focus: Focus::Wood,
        },
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SetFocus {
            site: SiteId(5),
            focus: Focus::Fortification,
        },
    )
    .unwrap();
    finish(&mut campaign, &data);
    assert_eq!(campaign.world.fort_damage[&SiteId(5)], 11);
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().military,
        MilitaryLayer::Fort
    );
    let mut unknown = campaign.clone();
    unknown
        .factions
        .get_mut(&FactionId(1))
        .unwrap()
        .last_economy
        .as_mut()
        .unwrap()
        .settlement_inputs = None;
    assert_eq!(
        reload(&unknown, &data),
        unknown,
        "old receipts explicitly have unknown site operands"
    );
}

#[test]
fn simultaneous_migration_and_resettlement_conserve_people_and_destination_capacity() {
    let (data, mut campaign) = fixture();
    // Fill every destination to capacity, leaving one shared space of three people.
    for id in [1, 5, 6, 8, 10, 11, 12] {
        site_mut(&mut campaign, id).habitation = Habitation::Village;
        let capacity = development_view(&campaign, &data, FactionId(1), SiteId(id))
            .unwrap()
            .capacity;
        campaign.world.population.insert(SiteId(id), capacity);
    }
    campaign.world.population.insert(SiteId(1), 2697);
    for id in [5, 6] {
        campaign.world.site_damage.insert(SiteId(id), 80);
    }
    let before = total(&campaign);
    finish(&mut campaign, &data);
    // HQ naturally grows into the three free places before migration budgets are reserved.
    assert_eq!(
        total(&campaign),
        before + 9,
        "three local arrivals are natural growth; the other HQs grow two each"
    );
    assert!(campaign.world.development[&SiteId(5)].displaced > 0);
    assert!(campaign.world.development[&SiteId(6)].displaced > 0);
    campaign.world.population.insert(SiteId(1), 2600);
    let count = total(&campaign);
    let gold = campaign.factions[&FactionId(1)].resources.gold;
    let before_state = campaign.clone();
    preview(
        &campaign,
        &data,
        Actor::Player,
        Command::Resettle {
            from: SiteId(5),
            to: SiteId(1),
        },
    )
    .unwrap();
    assert_eq!(campaign, before_state);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Resettle {
            from: SiteId(5),
            to: SiteId(1),
        },
    )
    .unwrap();
    assert_eq!(total(&campaign), count);
    assert_eq!(campaign.world.population[&SiteId(1)], 2650);
    assert_eq!(campaign.factions[&FactionId(1)].resources.gold, gold - 10);
    assert_eq!(reload(&campaign, &data), campaign);
    reject(
        &mut campaign,
        &data,
        Command::Resettle {
            from: SiteId(5),
            to: SiteId(2),
        },
    );
    assert_shared_arrival_budget();
    assert_threat_blocks_migration();
}

#[test]
fn renaming_capital_and_lost_headquarters_recovery_are_distinct_atomic_orders() {
    let (data, mut campaign) = fixture();
    let original = campaign.world.site(SiteId(5)).unwrap().clone();
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RenameSite {
            site: SiteId(5),
            name: "  New Ashford  ".into(),
        },
    )
    .unwrap();
    assert_eq!(campaign.world.site(SiteId(5)).unwrap().id, original.id);
    assert_eq!(campaign.world.site(SiteId(5)).unwrap().name, "New Ashford");
    reject(
        &mut campaign,
        &data,
        Command::RenameSite {
            site: SiteId(5),
            name: "x".repeat(41),
        },
    );
    reject(
        &mut campaign,
        &data,
        Command::RenameSite {
            site: SiteId(5),
            name: "bad\nname".into(),
        },
    );
    site_mut(&mut campaign, 5).habitation = Habitation::Town;
    campaign.world.population.insert(SiteId(5), 500);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::MoveCapital { site: SiteId(5) },
    )
    .unwrap();
    assert_eq!(
        (
            campaign.factions[&FactionId(1)].capital,
            campaign.factions[&FactionId(1)].headquarters
        ),
        (SiteId(5), SiteId(1))
    );
    site_mut(&mut campaign, 5)
        .facilities
        .push(Facility::TrainingGround);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    campaign
        .set_site_control(&data, SiteId(1), Some(FactionId(2)), false)
        .unwrap();
    assert!(campaign.supplied_sites(FactionId(1)).is_empty());
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RelocateHeadquarters { site: SiteId(5) },
    )
    .unwrap();
    assert_eq!(campaign.factions[&FactionId(1)].headquarters, SiteId(5));
    assert!(campaign.supplied_sites(FactionId(1)).contains(&SiteId(6)));
    site_mut(&mut campaign, 6).habitation = Habitation::Village;
    site_mut(&mut campaign, 6)
        .facilities
        .push(Facility::TrainingGround);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(6);
    reject(
        &mut campaign,
        &data,
        Command::RelocateHeadquarters { site: SiteId(6) },
    );
    for _ in 0..4 {
        finish(&mut campaign, &data);
    }
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RelocateHeadquarters { site: SiteId(6) },
    )
    .unwrap();
    assert_eq!(campaign.factions[&FactionId(1)].capital, SiteId(5));
    assert_eq!(reload(&campaign, &data), campaign);
    assert_npc_administration();
}
