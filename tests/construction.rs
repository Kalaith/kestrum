//! Five persistent construction contracts through normal candidate commands.

use kestrum::{
    data::{
        economy::{Habitation, OrderKind, Resources, TroopKind},
        world::{Facility, FactionId, MilitaryLayer, RouteId, SiteId},
        GameData,
    },
    engine::{
        advance_npc, apply, construction_options, construction_refund, preview, route_cost, Actor,
        Command, MoveOrder,
    },
    state::{
        construction::{
            CancellationReason, ConstructionKind, ConstructionPause, ConstructionStatus,
            ConstructionTarget, Focus, OrderId,
        },
        military::{ArmyId, FormationId},
        Campaign, CampaignPhase, StrategicCampaign,
    },
};

#[path = "support/construction.rs"]
mod support;
use support::*;

#[test]
fn prepaid_cost_and_three_steps_survive_preview_save_and_round_boundaries() {
    let (data, mut campaign) = builder();
    let before = campaign.clone();
    let command = build(
        ConstructionTarget::Site(SiteId(5)),
        ConstructionKind::Outpost,
        1,
    );
    preview(&campaign, &data, Actor::Player, command.clone()).unwrap();
    assert_eq!(campaign, before);
    let cost = data.economy.orders[&OrderKind::EstablishOutpost].cost;
    apply(&mut campaign, &data, Actor::Player, command).unwrap();
    let order = campaign.next_ids.order.0 - 1;
    assert_eq!(
        campaign.factions[&FactionId(1)].resources,
        subtract(before.factions[&FactionId(1)].resources, cost)
    );
    assert_eq!(campaign.construction[&OrderId(order)].progress, 0);
    apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(campaign.construction[&OrderId(order)].progress, 0);
    finish(&mut campaign, &data);
    assert_eq!(campaign.construction[&OrderId(order)].progress, 1);
    let mut resumed = reload(&campaign, &data);
    for expected in [2, 3] {
        finish(&mut campaign, &data);
        finish(&mut resumed, &data);
        assert_eq!(campaign, resumed);
        assert_eq!(campaign.construction[&OrderId(order)].progress, expected);
    }
    assert_eq!(
        campaign.construction[&OrderId(order)].status,
        ConstructionStatus::Completed {
            completed_rounds: 2
        }
    );
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().habitation,
        Habitation::Outpost
    );
    assert_eq!(campaign.world.population[&SiteId(5)], 50);
    assert_eq!(
        campaign.world.population[&SiteId(1)],
        206,
        "HQ growth remains after the prepaid settler deduction"
    );
    assert_migration(&data);
    assert_invalid(&campaign, &data, OrderId(order));
}

#[test]
fn departure_supply_loss_replacement_and_actual_combat_have_persistent_consequences() {
    let (data, mut campaign) = builder();
    let split = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::SplitArmy {
            formation: FormationId(2),
        },
    )
    .unwrap()
    .split_army
    .unwrap();
    let order = start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(SiteId(5)),
        ConstructionKind::Outpost,
        1,
    );
    apply(&mut campaign, &data, Actor::Player, move_order(1, &[5, 1])).unwrap();
    assert_eq!(
        campaign.construction[&order].status,
        ConstructionStatus::Paused {
            reason: ConstructionPause::BuilderAway
        }
    );
    finish(&mut campaign, &data);
    assert_eq!(campaign.construction[&order].progress, 0);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::ReassignBuilder {
            order,
            builder: split,
        },
    )
    .unwrap();
    finish(&mut campaign, &data);
    assert_eq!(campaign.construction[&order].progress, 1);
    campaign
        .set_site_control(&data, SiteId(1), None, false)
        .unwrap();
    finish(&mut campaign, &data);
    assert_eq!(
        campaign.construction[&order].status,
        ConstructionStatus::Paused {
            reason: ConstructionPause::SupplyLost
        }
    );
    assert_eq!(campaign.construction[&order].progress, 1);
    campaign
        .set_site_control(&data, SiteId(1), Some(FactionId(1)), false)
        .unwrap();
    finish(&mut campaign, &data);
    assert_eq!(campaign.construction[&order].progress, 2);
    assert_combat(false);
    assert_combat(true);
    assert_missing_builder();
}

#[test]
fn refunds_builder_reservations_duplicates_and_focus_are_atomic() {
    let (data, mut campaign) = builder();
    let before = campaign.factions[&FactionId(1)].resources;
    let order = start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(SiteId(5)),
        ConstructionKind::Outpost,
        1,
    );
    for command in [
        build(
            ConstructionTarget::Site(SiteId(5)),
            ConstructionKind::Outpost,
            1,
        ),
        build(
            ConstructionTarget::Route(RouteId(13)),
            ConstructionKind::Road,
            1,
        ),
    ] {
        let stable = campaign.clone();
        assert!(apply(&mut campaign, &data, Actor::Player, command).is_err());
        assert_eq!(campaign, stable);
    }
    assert_eq!(
        construction_refund(&campaign, &data, FactionId(1), order).unwrap(),
        data.economy.orders[&OrderKind::EstablishOutpost].cost
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelConstruction { order },
    )
    .unwrap();
    assert_eq!(campaign.factions[&FactionId(1)].resources, before);
    let second = start(
        &mut campaign,
        &data,
        ConstructionTarget::Site(SiteId(5)),
        ConstructionKind::Outpost,
        1,
    );
    assert!(second > order);
    finish(&mut campaign, &data);
    let balance = campaign.factions[&FactionId(1)].resources;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelConstruction { order: second },
    )
    .unwrap();
    assert_eq!(campaign.factions[&FactionId(1)].resources, balance);
    assert_eq!(campaign.construction.len(), 1);
    let stable = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelConstruction { order: second }
    )
    .is_err());
    assert_eq!(campaign, stable);
    let population = campaign.world.population.clone();
    for focus in [Focus::Gold, Focus::Growth] {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::SetFocus {
                site: SiteId(5),
                focus,
            },
        )
        .unwrap();
    }
    assert_eq!(campaign.world.focus[&SiteId(5)], Focus::Growth);
    assert_eq!(campaign.world.focus.len(), 1);
    assert_eq!(campaign.world.population, population);
    assert_eq!(campaign.factions[&FactionId(1)].resources, balance);
}

#[test]
fn settlers_are_conserved_with_partial_donors_and_frozen_same_round_budgets() {
    for (starting, expected) in [(250, 50), (209, 9), (200, 0)] {
        let (data, mut campaign) = builder();
        campaign.world.population.insert(SiteId(1), starting);
        campaign
            .world
            .sites
            .iter_mut()
            .find(|site| site.id == SiteId(5))
            .unwrap()
            .habitation = Habitation::Camp;
        campaign.world.population.insert(SiteId(5), 20);
        let graph = topology(&campaign);
        let order = start(
            &mut campaign,
            &data,
            ConstructionTarget::Site(SiteId(5)),
            ConstructionKind::Outpost,
            1,
        );
        for _ in 0..2 {
            finish(&mut campaign, &data);
        }
        // Set the final-step donor budget explicitly after K11 natural growth.
        campaign.world.population.insert(SiteId(1), starting);
        campaign.world.population.insert(SiteId(5), 20);
        let total = population(&campaign);
        finish(&mut campaign, &data);
        let camp_growth = u32::from(expected == 0);
        assert_eq!(population(&campaign), total + 8 + u64::from(camp_growth));
        assert_eq!(topology(&campaign), graph);
        assert_eq!(
            campaign.world.population[&SiteId(5)],
            20 + expected + camp_growth
        );
        assert_eq!(
            campaign.world.population[&SiteId(1)],
            starting - expected + 2
        );
        if expected == 0 {
            assert_eq!(
                campaign.construction[&order].status,
                ConstructionStatus::Active, // Natural growth creates eligible settlers for the next season.
            );
            assert_eq!(campaign.construction[&order].progress, 2);
        } else {
            let destination = campaign.world.population[&SiteId(5)];
            let donor = campaign.world.population[&SiteId(1)];
            finish(&mut campaign, &data);
            assert_eq!(
                campaign.world.population[&SiteId(5)],
                destination + 1,
                "only natural growth follows completed work"
            );
            assert_eq!(
                campaign.world.population[&SiteId(1)],
                donor + 2,
                "settlers are deducted only once"
            );
        }
    }
    assert_frozen_donors();
}

#[test]
fn roads_repairs_and_local_facilities_use_the_same_rules_for_player_and_npc() {
    for owner in [1, 2] {
        assert_facilities(owner);
        assert_road(owner);
    }
    assert_income_timing();
}
