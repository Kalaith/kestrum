//! Observer-safe one-boundary warnings reuse the same development rules as play.

use super::collect::{place, set_warning};
use crate::{
    data::{economy::Habitation, GameData},
    engine::{construction::forecast_projection, development, recovery},
    state::{
        notifications::{NotificationDetail, NotificationKind, WarningKey, WarningSubject},
        StrategicCampaign,
    },
};

pub(super) fn reconcile(
    candidate: &mut StrategicCampaign,
    data: &GameData,
    baseline: bool,
) -> Result<(), String> {
    let player = candidate.player;
    let development_snapshot = development::snapshot(candidate, data);
    let observed = development::observed_safety_sites(candidate, player);
    let supply = recovery::snapshot(candidate);
    let construction = forecast_projection(candidate, data, &supply, &observed);
    let ids = candidate
        .world
        .sites
        .iter()
        .map(|site| site.id)
        .collect::<Vec<_>>();
    for id in ids {
        let owned = candidate
            .world
            .site(id)
            .is_some_and(|site| site.controller == Some(player));
        if !owned {
            let detail = empty_detail(candidate, id);
            for kind in [
                NotificationKind::GrowthApproaching,
                NotificationKind::DeclineRisk,
                NotificationKind::RuinRisk,
            ] {
                set_warning(
                    candidate,
                    data,
                    WarningKey {
                        subject: WarningSubject::Site(id),
                        kind,
                    },
                    false,
                    detail.clone(),
                    baseline,
                )?;
            }
            continue;
        }
        if !observed.contains(&id) || construction.uncertain_sites.contains(&id) {
            continue;
        }
        let Some(conditions) = development_snapshot.conditions.get(&id) else {
            continue;
        };
        let site = candidate.world.site(id).expect("owned site exists");
        let state = construction
            .development
            .get(&id)
            .cloned()
            .unwrap_or_else(|| development::step_state(candidate, id));
        let forecast = development::forecast_step(
            candidate.completed_rounds,
            data,
            site,
            state.clone(),
            conditions,
        )
        .map_err(|error| format!("development forecast for site {}: {error}", id.0))?;
        let place = place(candidate, id).expect("owned site exists");
        let cause_conditions = development::conditions::causes(data, conditions)
            .iter()
            .map(|cause| cause_key(&cause.label))
            .collect::<Vec<_>>();
        let forecast_round = candidate.completed_rounds.saturating_add(1);
        for (kind, present, before_tier, after_tier) in [
            (
                NotificationKind::GrowthApproaching,
                forecast.habitation > state.habitation,
                Some(state.habitation),
                Some(forecast.habitation),
            ),
            (
                NotificationKind::DeclineRisk,
                forecast.habitation < state.habitation,
                Some(state.habitation),
                Some(forecast.habitation),
            ),
            (
                NotificationKind::RuinRisk,
                forecast.ruined && !state.development.ruined,
                None,
                None,
            ),
        ] {
            let detail = NotificationDetail::Place {
                place: place.clone(),
                before: before_tier.map(habitation_key),
                after: after_tier.map(habitation_key),
                cause: Some(if kind == NotificationKind::RuinRisk {
                    "ruin_condition_reaches_final_step".into()
                } else {
                    "pressure_crosses_development_threshold".into()
                }),
                forecast_round: Some(forecast_round),
                conditions: cause_conditions.clone(),
            };
            set_warning(
                candidate,
                data,
                WarningKey {
                    subject: WarningSubject::Site(id),
                    kind,
                },
                present,
                detail,
                baseline,
            )?;
        }
    }
    construction_forecasts(candidate, data, baseline, &construction)?;
    Ok(())
}

fn construction_forecasts(
    candidate: &mut StrategicCampaign,
    data: &GameData,
    baseline: bool,
    projection: &crate::engine::construction::ForecastProjection,
) -> Result<(), String> {
    let mut ids = projection.completed_next.clone();
    ids.extend(
        candidate
            .notifications
            .warning_episodes
            .iter()
            .filter_map(|episode| match episode.key.subject {
                WarningSubject::Construction(id)
                    if episode.key.kind == NotificationKind::ConstructionExpected =>
                {
                    Some(id)
                }
                _ => None,
            }),
    );
    for id in ids {
        if projection.unknown_orders.contains(&id) {
            continue;
        }
        let present = projection.completed_next.contains(&id);
        let Some(work) = candidate.construction.get(&id) else {
            let episode = candidate
                .notifications
                .warning_episodes
                .iter()
                .find(|episode| episode.key == construction_warning_key(id));
            if let Some(episode) = episode {
                let detail = episode
                    .active_receipt
                    .and_then(|receipt| candidate.notifications.receipt(receipt))
                    .map(|receipt| receipt.detail.clone())
                    .unwrap_or_else(|| NotificationDetail::Facts {
                        values: Default::default(),
                    });
                set_warning(
                    candidate,
                    data,
                    construction_warning_key(id),
                    false,
                    detail,
                    baseline,
                )?;
            }
            continue;
        };
        let snapshot = super::collect::construction(candidate, work);
        let status = work.status;
        let forecast_round = candidate.completed_rounds.saturating_add(1);
        set_warning(
            candidate,
            data,
            construction_warning_key(id),
            present,
            NotificationDetail::Work {
                work: snapshot,
                before: Some(status),
                cause: Some("active_work_one_step_from_completion".into()),
                forecast_round: Some(forecast_round),
            },
            baseline,
        )?;
    }
    Ok(())
}

fn construction_warning_key(id: crate::state::construction::OrderId) -> WarningKey {
    WarningKey {
        subject: WarningSubject::Construction(id),
        kind: NotificationKind::ConstructionExpected,
    }
}

fn empty_detail(
    campaign: &StrategicCampaign,
    id: crate::data::world::SiteId,
) -> NotificationDetail {
    let place = place(campaign, id).expect("site exists");
    NotificationDetail::Place {
        place,
        before: None,
        after: None,
        cause: None,
        forecast_round: None,
        conditions: Vec::new(),
    }
}

fn cause_key(label: &str) -> String {
    match label {
        "Safe" => "safe".into(),
        "Connected" => "connected".into(),
        "Trade" => "trade".into(),
        "Food" => "food".into(),
        "Capital" => "capital".into(),
        "Growth focus" => "growth_focus".into(),
        "Battle" => "battle".into(),
        "Governor" => "governor".into(),
        "Cut off" => "cut_off".into(),
        "Damage" => "damage".into(),
        "Occupation" => "occupation".into(),
        _ => "other".into(),
    }
}

fn habitation_key(value: Habitation) -> String {
    format!("{:?}", value).to_lowercase()
}
