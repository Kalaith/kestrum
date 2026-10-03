//! Five event forecasts follow shared boundary rules and preserve unseen uncertainty.

#[path = "support/phases.rs"]
mod phases;
use phases::pass_npc;

#[path = "support/relations.rs"]
#[allow(dead_code)] // Shared development fixtures include separate relation scenarios.
mod relations;
use relations::sync_relations;

use kestrum::{
    data::{
        economy::{Habitation, OrderKind, Resources},
        world::{DiplomaticState, Facility, FactionId, Geography, RouteId, SiteId, SiteTag},
        GameData,
    },
    engine::{self, apply, development_view, preview, Actor, Command},
    state::{
        construction::{
            ConstructionKind, ConstructionOrder, ConstructionStatus, ConstructionTarget, OrderId,
        },
        military::ArmyId,
        notifications::{
            NotificationDetail, NotificationKind, WarningEpisode, WarningKey, WarningSubject,
        },
        Campaign, CampaignPhase, StrategicCampaign,
    },
};

#[path = "support/development.rs"]
#[allow(dead_code)] // Reuse the canonical fixture without rerunning unrelated scenario helpers.
mod support;
use support::*;

#[test]
fn development_forecast_uses_the_shared_plan_and_ignores_same_boundary_work() {
    let (data, mut campaign) = fixture();
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(SiteId(5)),
            kind: ConstructionKind::Facility(kestrum::data::world::Facility::TrainingGround),
            builder: ArmyId(1),
        },
    )
    .unwrap();

    let view = development_view(&campaign, &data, FactionId(1), SiteId(5)).unwrap();
    let contribution = view.contribution.unwrap();
    let threshold = data.development.pressure.upgrade_threshold;
    campaign.world.population.insert(
        SiteId(5),
        data.construction.population.minimum[&Habitation::Town],
    );
    let state = campaign.world.development.get_mut(&SiteId(5)).unwrap();
    state.pressure = threshold - contribution;
    state.last_resolved_round = Some(campaign.completed_rounds);
    engine::notifications::baseline_current_conditions(&mut campaign, &data).unwrap();
    assert!(!warning_present(
        &campaign,
        WarningSubject::Site(SiteId(5)),
        NotificationKind::GrowthApproaching
    ));

    campaign
        .world
        .development
        .get_mut(&SiteId(5))
        .unwrap()
        .last_resolved_round = None;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::RenameSite {
            site: SiteId(5),
            name: "Forecast Grove".into(),
        },
    )
    .unwrap();
    let episode = warning_episode(
        &campaign,
        WarningSubject::Site(SiteId(5)),
        NotificationKind::GrowthApproaching,
    )
    .expect("exact one-step growth remains forecast despite active work at the site");
    let receipt = campaign
        .notifications
        .receipt(episode.active_receipt.unwrap())
        .unwrap();
    let NotificationDetail::Place {
        before,
        after,
        forecast_round,
        ..
    } = &receipt.detail
    else {
        panic!("development warnings retain a place forecast");
    };
    assert_eq!(before.as_deref(), Some("village"));
    assert_eq!(after.as_deref(), Some("town"));
    assert_eq!(*forecast_round, Some(campaign.completed_rounds + 1));
    assert_eq!(campaign.construction[&OrderId(1)].progress, 0);

    finish(&mut campaign, &data);
    assert_eq!(
        campaign.world.site(SiteId(5)).unwrap().habitation,
        Habitation::Town,
        "the exact development forecast matches the next real boundary"
    );
    let work_episode = warning_episode(
        &campaign,
        WarningSubject::Construction(OrderId(1)),
        NotificationKind::ConstructionExpected,
    )
    .expect("the remaining facility step is forecast");
    assert!(work_episode.is_present);
    let work_receipt = campaign
        .notifications
        .receipt(work_episode.active_receipt.unwrap())
        .unwrap();
    let NotificationDetail::Work {
        work,
        forecast_round,
        ..
    } = &work_receipt.detail
    else {
        panic!("construction warnings retain a work forecast");
    };
    assert_eq!(work.progress + 1, work.required_steps);
    assert_eq!(*forecast_round, Some(campaign.completed_rounds + 1));

    finish(&mut campaign, &data);
    assert_eq!(
        campaign.construction[&OrderId(1)].status,
        ConstructionStatus::Completed {
            completed_rounds: 1
        }
    );
    assert!(!warning_present(
        &campaign,
        WarningSubject::Construction(OrderId(1)),
        NotificationKind::ConstructionExpected
    ));
}

#[test]
fn decline_and_ruin_warnings_match_recovered_and_unchanged_boundaries() {
    #[derive(Debug, Clone, Copy)]
    enum Risk {
        Decline,
        Ruin,
    }

    for risk in [Risk::Decline, Risk::Ruin] {
        let (data, mut warned) = fixture();
        let site = SiteId(5);
        let round = warned.completed_rounds;
        let kind = match risk {
            Risk::Decline => NotificationKind::DeclineRisk,
            Risk::Ruin => NotificationKind::RuinRisk,
        };
        let expected_before = match risk {
            Risk::Decline => Some("village"),
            Risk::Ruin => None,
        };
        let expected_after = match risk {
            Risk::Decline => Some("hamlet"),
            Risk::Ruin => None,
        };

        // Risk warnings are emitted only for places whose safety the player observes.
        warned
            .armies
            .get_mut(&ArmyId(1))
            .expect("fixture player army")
            .site = site;
        match risk {
            Risk::Decline => {
                site_mut(&mut warned, site.0).habitation = Habitation::Village;
                warned.world.population.insert(
                    site,
                    data.construction.population.minimum[&Habitation::Village],
                );
            }
            Risk::Ruin => {
                site_mut(&mut warned, site.0).habitation = Habitation::Unsettled;
                warned.world.population.insert(site, 0);
            }
        }
        warned
            .world
            .development
            .get_mut(&site)
            .unwrap()
            .last_resolved_round = Some(round);
        engine::notifications::baseline_current_conditions(&mut warned, &data).unwrap();
        assert!(!warning_present(&warned, WarningSubject::Site(site), kind));

        match risk {
            Risk::Decline => {
                let contribution = development_view(&warned, &data, FactionId(1), site)
                    .unwrap()
                    .contribution
                    .unwrap();
                let pressure = &data.development.pressure;
                let expected_pressure = (-pressure.downgrade_threshold - contribution)
                    .clamp(pressure.minimum, pressure.maximum);
                assert!(
                    (expected_pressure + contribution).clamp(pressure.minimum, pressure.maximum)
                        <= -pressure.downgrade_threshold,
                    "fixture must be one shared-rule step into downgrade"
                );
                let state = warned.world.development.get_mut(&site).unwrap();
                state.pressure = expected_pressure;
                state.last_resolved_round = None;
            }
            Risk::Ruin => {
                warned
                    .world
                    .site_damage
                    .insert(site, data.development.conditions.ruin_damage);
                let state = warned.world.development.get_mut(&site).unwrap();
                state.ruin_streak = data.development.conditions.ruin_steps - 1;
                state.last_resolved_round = None;
            }
        }

        let initial_habitation = warned.world.site(site).unwrap().habitation;
        apply(
            &mut warned,
            &data,
            Actor::Player,
            Command::RenameSite {
                site,
                name: format!("{risk:?} Forecast Site"),
            },
        )
        .unwrap();
        let episode = warning_episode(&warned, WarningSubject::Site(site), kind)
            .unwrap_or_else(|| panic!("{risk:?} shared next-season risk is tracked"));
        assert!(episode.is_present);
        let warning_id = episode.active_receipt.unwrap();
        let warning = warned.notifications.receipt(warning_id).unwrap();
        let NotificationDetail::Place {
            before,
            after,
            forecast_round,
            ..
        } = &warning.detail
        else {
            panic!("risk retains event-time place forecast details");
        };
        assert_eq!(before.as_deref(), expected_before);
        assert_eq!(after.as_deref(), expected_after);
        assert_eq!(*forecast_round, Some(round + 1));
        assert_eq!(
            warned.world.site(site).unwrap().habitation,
            initial_habitation
        );
        assert!(!warned.world.development[&site].ruined);

        let mut boundary = warned.clone();
        finish(&mut boundary, &data);
        match risk {
            Risk::Decline => {
                assert_eq!(
                    boundary.world.site(site).unwrap().habitation,
                    Habitation::Hamlet
                );
                assert!(boundary
                    .notifications
                    .receipts
                    .iter()
                    .any(|receipt| receipt.kind == NotificationKind::HabitationChanged));
            }
            Risk::Ruin => {
                assert!(boundary.world.development[&site].ruined);
                assert!(boundary
                    .notifications
                    .receipts
                    .iter()
                    .any(|receipt| receipt.kind == NotificationKind::SiteRuined));
            }
        }

        let mut recovered = warned;
        match risk {
            Risk::Decline => {
                let state = recovered.world.development.get_mut(&site).unwrap();
                state.pressure = 0;
                state.last_resolved_round = None;
            }
            Risk::Ruin => {
                recovered.world.site_damage.insert(site, 0);
                let state = recovered.world.development.get_mut(&site).unwrap();
                state.ruin_streak = 0;
                state.last_resolved_round = None;
            }
        }
        apply(
            &mut recovered,
            &data,
            Actor::Player,
            Command::RenameSite {
                site,
                name: format!("Recovered {risk:?} Site"),
            },
        )
        .unwrap();
        let recovered_episode = warning_episode(&recovered, WarningSubject::Site(site), kind)
            .expect("site episodes remain available after recovery");
        assert!(!recovered_episode.is_present);
        assert!(
            !recovered
                .notifications
                .receipt(warning_id)
                .unwrap()
                .is_active
        );

        finish(&mut recovered, &data);
        match risk {
            Risk::Decline => {
                assert_eq!(
                    recovered.world.site(site).unwrap().habitation,
                    Habitation::Village
                );
                assert!(!recovered
                    .notifications
                    .receipts
                    .iter()
                    .any(|receipt| receipt.kind == NotificationKind::HabitationChanged));
            }
            Risk::Ruin => {
                assert!(!recovered.world.development[&site].ruined);
                assert!(!recovered
                    .notifications
                    .receipts
                    .iter()
                    .any(|receipt| receipt.kind == NotificationKind::SiteRuined));
            }
        }
    }
}

#[test]
fn unseen_outpost_donor_state_preserves_episode_without_leaking_hidden_branch() {
    let with_donor = outpost_observation_case(true);
    let without_donor = outpost_observation_case(false);
    assert_eq!(with_donor, without_donor);
    let episode = with_donor
        .warning_episodes
        .iter()
        .find(|episode| {
            episode.key.subject == WarningSubject::Construction(OrderId(1))
                && episode.key.kind == NotificationKind::ConstructionExpected
        })
        .expect("pre-existing warning episode remains tracked");
    assert!(
        episode.is_present,
        "unknown must not falsely clear the episode"
    );
}

#[test]
fn competing_outposts_consume_the_frozen_donor_budget_in_order_id_order() {
    let (data, mut campaign) = fixture();
    configure_connected_outposts(&mut campaign, &data);
    let donor_minimum = data.construction.population.minimum[&Habitation::Village];
    campaign.world.population.insert(
        SiteId(1),
        donor_minimum + data.construction.population.settler_limit,
    );
    campaign.world.population.insert(SiteId(5), 20);
    campaign.world.population.insert(SiteId(6), 20);
    campaign
        .construction
        .insert(OrderId(2), outpost_order(&data, 2, SiteId(6), ArmyId(2), 2));
    campaign
        .construction
        .insert(OrderId(1), outpost_order(&data, 1, SiteId(5), ArmyId(1), 2));
    campaign.next_ids.order = OrderId(3);

    engine::notifications::baseline_current_conditions(&mut campaign, &data).unwrap();
    assert!(warning_present(
        &campaign,
        WarningSubject::Construction(OrderId(1)),
        NotificationKind::ConstructionExpected
    ));
    assert!(warning_episode(
        &campaign,
        WarningSubject::Construction(OrderId(2)),
        NotificationKind::ConstructionExpected
    )
    .is_none());
}

#[test]
fn reclaimed_outpost_unblocks_later_road_work_in_the_same_projection() {
    let (data, mut campaign) = fixture();
    configure_connected_outposts(&mut campaign, &data);
    campaign.world.population.insert(
        SiteId(1),
        data.construction.population.minimum[&Habitation::Village]
            + data.construction.population.settler_limit,
    );
    campaign.world.population.insert(SiteId(5), 20);
    let ruined = campaign.world.development.get_mut(&SiteId(5)).unwrap();
    ruined.ruined = true;
    ruined.ruined_round = Some(1);
    ruined.ruination = 1;
    campaign.world.site_damage.insert(SiteId(5), 100);
    let route = campaign
        .world
        .routes
        .iter()
        .find(|route| {
            (route.from == SiteId(5) && route.to == SiteId(6))
                || (route.from == SiteId(6) && route.to == SiteId(5))
        })
        .unwrap()
        .id;
    campaign
        .construction
        .insert(OrderId(1), outpost_order(&data, 1, SiteId(5), ArmyId(1), 2));
    campaign
        .construction
        .insert(OrderId(2), road_order(&data, 2, route, ArmyId(2), 2));
    campaign.next_ids.order = OrderId(3);

    engine::notifications::baseline_current_conditions(&mut campaign, &data).unwrap();
    for id in [OrderId(1), OrderId(2)] {
        assert!(warning_present(
            &campaign,
            WarningSubject::Construction(id),
            NotificationKind::ConstructionExpected
        ));
    }
}

#[test]
fn removed_zero_receipt_construction_episode_is_closed() {
    let (data, mut campaign) = fixture();
    campaign.completed_rounds = 2;
    campaign
        .notifications
        .warning_episodes
        .push(WarningEpisode {
            key: WarningKey {
                subject: WarningSubject::Construction(OrderId(1)),
                kind: NotificationKind::ConstructionExpected,
            },
            next_episode: 1,
            active_receipt: None,
            is_present: true,
        });
    engine::notifications::baseline_current_conditions(&mut campaign, &data).unwrap();
    assert!(warning_episode(
        &campaign,
        WarningSubject::Construction(OrderId(1)),
        NotificationKind::ConstructionExpected
    )
    .is_none());
}

fn outpost_observation_case(
    remote_donor: bool,
) -> kestrum::state::notifications::NotificationInbox {
    let (data, mut campaign) = fixture();
    configure_connected_outposts(&mut campaign, &data);
    for id in [SiteId(1), SiteId(6), SiteId(8), SiteId(10), SiteId(12)] {
        let site = campaign.world.site(id).unwrap().clone();
        site_mut(&mut campaign, id.0).habitation = Habitation::Camp;
        campaign
            .world
            .population
            .insert(id, data.construction.population.minimum[&Habitation::Camp]);
        if site.controller != Some(FactionId(1)) {
            site_mut(&mut campaign, id.0).controller = Some(FactionId(1));
        }
    }
    site_mut(&mut campaign, 1).habitation = Habitation::Village;
    campaign.world.population.insert(
        SiteId(1),
        data.construction.population.minimum[&Habitation::Village],
    );
    campaign.world.population.insert(
        SiteId(12),
        data.construction.population.minimum[&Habitation::Camp] + if remote_donor { 1 } else { 0 },
    );
    site_mut(&mut campaign, 5).habitation = Habitation::Camp;
    campaign.world.population.insert(
        SiteId(5),
        data.construction.population.minimum[&Habitation::Camp],
    );
    campaign
        .construction
        .insert(OrderId(1), outpost_order(&data, 1, SiteId(5), ArmyId(1), 2));
    campaign.next_ids.order = OrderId(2);
    campaign
        .notifications
        .warning_episodes
        .push(WarningEpisode {
            key: WarningKey {
                subject: WarningSubject::Construction(OrderId(1)),
                kind: NotificationKind::ConstructionExpected,
            },
            next_episode: 1,
            active_receipt: None,
            is_present: true,
        });
    engine::notifications::baseline_current_conditions(&mut campaign, &data).unwrap();
    campaign.notifications
}

fn configure_connected_outposts(campaign: &mut StrategicCampaign, _data: &GameData) {
    campaign.completed_rounds = 2;
    campaign.armies.get_mut(&ArmyId(1)).unwrap().site = SiteId(5);
    if let Some(second) = campaign.armies.get_mut(&ArmyId(2)) {
        second.faction = FactionId(1);
        second.site = SiteId(6);
    }
    for site in &mut campaign.world.sites {
        site.controller = matches!(site.id.0, 1 | 5 | 6).then_some(FactionId(1));
    }
    for id in [SiteId(1), SiteId(5), SiteId(6)] {
        campaign.world.development.get_mut(&id).unwrap().ruined = false;
        campaign.world.site_damage.insert(id, 0);
    }
    site_mut(campaign, 5).habitation = Habitation::Camp;
    site_mut(campaign, 6).habitation = Habitation::Camp;
}

fn outpost_order(
    data: &GameData,
    id: u64,
    target: SiteId,
    builder: ArmyId,
    round: u32,
) -> ConstructionOrder {
    ConstructionOrder {
        id: OrderId(id),
        owner: FactionId(1),
        kind: ConstructionKind::Outpost,
        target: ConstructionTarget::Site(target),
        builder: Some(builder),
        created_round: 0,
        progress: data.construction.outpost_steps - 1,
        required_steps: data.construction.outpost_steps,
        paid: data.economy.orders[&OrderKind::EstablishOutpost].cost,
        status: ConstructionStatus::Active,
        last_progress_round: Some(round - 1),
    }
}

fn road_order(
    data: &GameData,
    id: u64,
    target: RouteId,
    builder: ArmyId,
    round: u32,
) -> ConstructionOrder {
    ConstructionOrder {
        id: OrderId(id),
        owner: FactionId(1),
        kind: ConstructionKind::Road,
        target: ConstructionTarget::Route(target),
        builder: Some(builder),
        created_round: 0,
        progress: data.construction.road_steps - 1,
        required_steps: data.construction.road_steps,
        paid: data.economy.orders[&OrderKind::ImproveRoad].cost,
        status: ConstructionStatus::Active,
        last_progress_round: Some(round - 1),
    }
}

fn warning_present(
    campaign: &StrategicCampaign,
    subject: WarningSubject,
    kind: NotificationKind,
) -> bool {
    warning_episode(campaign, subject, kind).is_some_and(|episode| episode.is_present)
}

fn warning_episode(
    campaign: &StrategicCampaign,
    subject: WarningSubject,
    kind: NotificationKind,
) -> Option<&WarningEpisode> {
    campaign
        .notifications
        .warning_episodes
        .iter()
        .find(|episode| episode.key == WarningKey { subject, kind })
}
