//! Observer-safe one-boundary warnings reuse the same development rules as play.

use super::collect::{place, set_warning};
use crate::{
    data::{economy::Habitation, GameData},
    engine::development::development_view,
    state::{
        construction::{ConstructionKind, ConstructionTarget},
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
        let Some(view) = development_view(candidate, data, player, id) else {
            continue;
        };
        // Unknown local safety is not a negative result. Preserve any active
        // warning until the player can observe enough to reassess it.
        if view.safe.is_none() || view.contribution.is_none() {
            continue;
        }
        let place = place(candidate, id).expect("owned site exists");
        let ineligible_boundary = candidate
            .world
            .development
            .get(&id)
            .is_none_or(|state| state.last_resolved_round == Some(candidate.completed_rounds))
            || active_site_work(candidate, id);
        let pressure = view
            .pressure
            .saturating_add(view.contribution.unwrap_or_default());
        let expected_growth = !ineligible_boundary
            && !view.ruined
            && pressure >= data.development.pressure.upgrade_threshold
            && next_tier(view.habitation).is_some_and(|next| {
                next <= view.maximum_habitation
                    && view.population >= data.construction.population.minimum[&next]
            });
        let expected_decline = !ineligible_boundary
            && !view.ruined
            && pressure <= -data.development.pressure.downgrade_threshold
            && previous_tier(view.habitation).is_some();
        let ruin = &data.development.conditions;
        let next_ruin_streak = if view.structural_damage >= ruin.ruin_damage
            && view.population < ruin.ruin_population
        {
            view.ruin_streak.saturating_add(1).min(ruin.ruin_steps)
        } else {
            0
        };
        let imminent_ruin =
            !ineligible_boundary && !view.ruined && next_ruin_streak >= ruin.ruin_steps;
        let conditions = view
            .causes
            .iter()
            .map(|cause| cause_key(&cause.label))
            .collect::<Vec<_>>();
        let forecast_round = candidate.completed_rounds.saturating_add(1);
        for (kind, present, before_tier, after_tier) in [
            (
                NotificationKind::GrowthApproaching,
                expected_growth,
                Some(view.habitation),
                next_tier(view.habitation),
            ),
            (
                NotificationKind::DeclineRisk,
                expected_decline,
                Some(view.habitation),
                previous_tier(view.habitation),
            ),
            (NotificationKind::RuinRisk, imminent_ruin, None, None),
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
                conditions: conditions.clone(),
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
    construction_forecasts(candidate, data, baseline)?;
    Ok(())
}

fn construction_forecasts(
    candidate: &mut StrategicCampaign,
    data: &GameData,
    baseline: bool,
) -> Result<(), String> {
    let eligible = candidate
        .construction
        .values()
        .filter(|work| {
            work.owner == candidate.player
                && work.is_open()
                && matches!(
                    work.status,
                    crate::state::construction::ConstructionStatus::Active
                )
                && work.progress.saturating_add(1) >= work.required_steps
                && work.last_progress_round != Some(candidate.completed_rounds)
                && forecast_target_owned(candidate, work.target)
                && !matches!(work.kind, ConstructionKind::Outpost)
        })
        .map(|work| work.id)
        .collect::<Vec<_>>();
    let active = candidate
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
        })
        .collect::<Vec<_>>();
    for id in active.into_iter().chain(eligible.iter().copied()) {
        let present = eligible.contains(&id);
        let Some(work) = candidate.construction.get(&id) else {
            let existing = candidate
                .notifications
                .warning_episodes
                .iter()
                .find(|episode| {
                    episode.key.subject == WarningSubject::Construction(id)
                        && episode.key.kind == NotificationKind::ConstructionExpected
                })
                .and_then(|episode| episode.active_receipt)
                .and_then(|receipt| candidate.notifications.receipt(receipt))
                .map(|receipt| receipt.detail.clone());
            if let Some(detail) = existing {
                set_warning(
                    candidate,
                    data,
                    WarningKey {
                        subject: WarningSubject::Construction(id),
                        kind: NotificationKind::ConstructionExpected,
                    },
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
            WarningKey {
                subject: WarningSubject::Construction(id),
                kind: NotificationKind::ConstructionExpected,
            },
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

fn forecast_target_owned(campaign: &StrategicCampaign, target: ConstructionTarget) -> bool {
    let owner = campaign.player;
    match target {
        ConstructionTarget::Site(site) => campaign
            .world
            .site(site)
            .is_some_and(|site| site.controller == Some(owner)),
        ConstructionTarget::Route(route) => campaign.world.route(route).is_some_and(|route| {
            campaign
                .world
                .site(route.from)
                .is_some_and(|site| site.controller == Some(owner))
                && campaign
                    .world
                    .site(route.to)
                    .is_some_and(|site| site.controller == Some(owner))
        }),
    }
}

fn active_site_work(campaign: &StrategicCampaign, site: crate::data::world::SiteId) -> bool {
    campaign.construction.values().any(|work| {
        work.owner == campaign.player
            && work.is_open()
            && match work.target {
                ConstructionTarget::Site(target) => target == site,
                ConstructionTarget::Route(route) => campaign
                    .world
                    .route(route)
                    .is_some_and(|route| route.from == site || route.to == site),
            }
    })
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

fn next_tier(value: Habitation) -> Option<Habitation> {
    use Habitation::*;
    match value {
        Unsettled => None,
        Camp => Some(Outpost),
        Outpost => Some(Hamlet),
        Hamlet => Some(Village),
        Village => Some(Town),
        Town => Some(City),
        City => Some(MajorCity),
        MajorCity => None,
    }
}

fn previous_tier(value: Habitation) -> Option<Habitation> {
    use Habitation::*;
    match value {
        Unsettled => None,
        Camp => Some(Unsettled),
        Outpost => Some(Camp),
        Hamlet => Some(Outpost),
        Village => Some(Hamlet),
        Town => Some(Village),
        City => Some(Town),
        MajorCity => Some(City),
    }
}
