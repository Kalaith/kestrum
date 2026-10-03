//! Reconcile interruptions after actions and advance once at the seasonal boundary.

use super::*;
use crate::{
    engine::{actions::record_fact, ActionOutcome},
    state::campaign::DomainFactKind,
};
use std::collections::{BTreeMap, BTreeSet};

use super::ForecastProjection;

pub(in crate::engine) fn reconcile(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let supply = super::super::recovery::snapshot(campaign);
    let settlers = settlers::SettlerSnapshot::new(campaign, data);
    let ids: Vec<_> = campaign
        .construction
        .values()
        .filter(|order| order.is_open())
        .map(|order| order.id)
        .collect();
    for id in ids {
        let before = campaign.construction[&id].clone();
        let mut order = before.clone();
        if order
            .builder
            .is_some_and(|id| !campaign.armies.contains_key(&id))
        {
            order.builder = None;
        }
        order.status = if !campaign.owns_construction_target(order.owner, order.target) {
            ConstructionStatus::Cancelled {
                completed_rounds: campaign.completed_rounds,
                reason: CancellationReason::ControlLost,
            }
        } else if let Some(reason) =
            pause_reason(campaign, data, &supply, &settlers, &order, |id| {
                campaign.site_is_ruined(id)
            })
        {
            ConstructionStatus::Paused { reason }
        } else {
            ConstructionStatus::Active
        };
        if order != before {
            campaign.construction.insert(id, order.clone());
            record_fact(
                campaign,
                outcome,
                DomainFactKind::ConstructionChanged { order },
            )?;
        }
    }
    campaign.trim_terminal_orders();
    Ok(())
}

pub(in crate::engine) fn resolve(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    supply: &SupplySnapshot,
    outcome: &mut ActionOutcome,
) -> Result<(), RuleError> {
    let mut settlers = settlers::SettlerSnapshot::new(campaign, data);
    let ids: Vec<_> = campaign
        .construction
        .values()
        .filter(|order| order.is_open())
        .map(|order| order.id)
        .collect();
    for id in ids {
        let before = campaign.construction[&id].clone();
        if !progress_due(&before, campaign.completed_rounds) {
            continue;
        }
        let mut order = before.clone();
        if !campaign.owns_construction_target(order.owner, order.target) {
            order.status = ConstructionStatus::Cancelled {
                completed_rounds: campaign.completed_rounds,
                reason: CancellationReason::ControlLost,
            };
        } else if let Some(reason) = pause_reason(campaign, data, supply, &settlers, &order, |id| {
            campaign.site_is_ruined(id)
        }) {
            order.status = ConstructionStatus::Paused { reason };
        } else {
            let completes = completes_on_next_progress(&order);
            order.progress += 1;
            order.last_progress_round = Some(campaign.completed_rounds);
            order.status = ConstructionStatus::Active;
            if completes {
                complete(campaign, data, supply, &mut settlers, &order)?;
                order.status = ConstructionStatus::Completed {
                    completed_rounds: campaign.completed_rounds,
                };
            }
        }
        if order != before {
            campaign.construction.insert(id, order.clone());
            record_fact(
                campaign,
                outcome,
                DomainFactKind::ConstructionChanged { order },
            )?;
        }
    }
    campaign.trim_terminal_orders();
    Ok(())
}

fn pause_reason(
    campaign: &StrategicCampaign,
    data: &GameData,
    supply: &SupplySnapshot,
    settlers: &settlers::SettlerSnapshot,
    order: &ConstructionOrder,
    is_ruined: impl Fn(SiteId) -> bool,
) -> Option<ConstructionPause> {
    let target_sites = sites(campaign, order.target).ok()?;
    if !matches!(order.kind, ConstructionKind::Facility(_))
        && order
            .builder
            .and_then(|id| campaign.armies.get(&id))
            .is_none()
    {
        return Some(ConstructionPause::BuilderMissing);
    }
    if target_sites
        .iter()
        .any(|site| campaign.active_threat(*site).is_some())
    {
        return Some(ConstructionPause::Threat);
    }
    if order.kind != ConstructionKind::Outpost && target_sites.iter().any(|site| is_ruined(*site)) {
        return Some(ConstructionPause::Ruined);
    }
    if target_sites
        .iter()
        .any(|id| campaign.world.contested_sites.contains(id))
    {
        return Some(ConstructionPause::Contested);
    }
    if target_sites
        .iter()
        .any(|site| !supply.contains(order.owner, *site))
    {
        return Some(ConstructionPause::SupplyLost);
    }
    if !matches!(order.kind, ConstructionKind::Facility(_)) {
        let Some(builder) = order.builder.and_then(|id| campaign.armies.get(&id)) else {
            return Some(ConstructionPause::BuilderMissing);
        };
        if !target_sites.contains(&builder.site) {
            return Some(ConstructionPause::BuilderAway);
        }
    }
    if campaign
        .battles
        .values()
        .filter(|report| report.completed_rounds == campaign.completed_rounds)
        .any(|report| {
            target_sites.contains(&report.site)
                || report
                    .attacker
                    .armies
                    .iter()
                    .chain(report.defender.armies())
                    .any(|army| Some(army.id) == order.builder)
        })
    {
        return Some(ConstructionPause::Combat);
    }
    if order.kind == ConstructionKind::Outpost
        && completes_on_next_progress(order)
        && settlers.source(data, supply, order).is_none()
    {
        return Some(ConstructionPause::NoSettlers);
    }
    None
}

fn progress_due(order: &ConstructionOrder, round: u32) -> bool {
    order.is_open() && order.last_progress_round != Some(round)
}

fn completes_on_next_progress(order: &ConstructionOrder) -> bool {
    order.progress.checked_add(1) == Some(order.required_steps)
}

pub(in crate::engine) fn forecast_projection(
    campaign: &StrategicCampaign,
    data: &GameData,
    supply: &SupplySnapshot,
    observed: &BTreeSet<SiteId>,
) -> ForecastProjection {
    let mut projection = ForecastProjection {
        completed_next: BTreeSet::new(),
        unknown_orders: BTreeSet::new(),
        uncertain_sites: BTreeSet::new(),
        development: BTreeMap::new(),
    };
    let mut settlers = settlers::SettlerSnapshot::new(campaign, data);
    let mut ruined = campaign
        .world
        .development
        .iter()
        .filter_map(|(id, state)| state.ruined.then_some(*id))
        .collect::<BTreeSet<_>>();
    let mut uncertain_budgets = BTreeSet::new();
    let mut uncertain_reclamation_sites = BTreeSet::new();

    for order in campaign.construction.values() {
        if order.owner != campaign.player
            || !progress_due(order, campaign.completed_rounds)
            || !completes_on_next_progress(order)
            || !campaign.owns_construction_target(order.owner, order.target)
        {
            continue;
        }
        let Ok(target_sites) = sites(campaign, order.target) else {
            continue;
        };
        if target_sites
            .iter()
            .any(|id| uncertain_reclamation_sites.contains(id))
        {
            projection.unknown_orders.insert(order.id);
            if order.kind == ConstructionKind::Outpost {
                if let settlers::ForecastSource::Unknown(donors) =
                    settlers.forecast_source(data, supply, order, observed, &uncertain_budgets)
                {
                    projection.uncertain_sites.extend(donors.iter().copied());
                    uncertain_budgets.extend(donors);
                }
            }
            continue;
        }
        if target_sites.iter().any(|id| !observed.contains(id)) {
            projection.unknown_orders.insert(order.id);
            if let ConstructionTarget::Site(target) = order.target {
                if order.kind == ConstructionKind::Outpost {
                    projection.uncertain_sites.insert(target);
                    uncertain_reclamation_sites.insert(target);
                    let settlers::ForecastSource::Unknown(donors) =
                        settlers.forecast_source(data, supply, order, observed, &uncertain_budgets)
                    else {
                        continue;
                    };
                    projection.uncertain_sites.extend(donors.iter().copied());
                    uncertain_budgets.extend(donors);
                }
            }
            continue;
        }

        let transfer = if order.kind == ConstructionKind::Outpost {
            match settlers.forecast_source(data, supply, order, observed, &uncertain_budgets) {
                settlers::ForecastSource::Known(transfer) => transfer,
                settlers::ForecastSource::Unknown(donors) => {
                    if let ConstructionTarget::Site(target) = order.target {
                        projection.uncertain_sites.insert(target);
                        uncertain_reclamation_sites.insert(target);
                    }
                    projection.uncertain_sites.extend(donors.iter().copied());
                    uncertain_budgets.extend(donors);
                    projection.unknown_orders.insert(order.id);
                    continue;
                }
            }
        } else {
            None
        };
        if pause_reason(campaign, data, supply, &settlers, order, |id| {
            ruined.contains(&id)
        })
        .is_some()
        {
            continue;
        }
        projection.completed_next.insert(order.id);
        if order.kind == ConstructionKind::Outpost {
            let Some(transfer) = transfer else {
                projection.completed_next.remove(&order.id);
                continue;
            };
            let Some((source, target)) = projected_transfer(campaign, data, transfer, &projection)
            else {
                projection.completed_next.remove(&order.id);
                projection.unknown_orders.insert(order.id);
                if let ConstructionTarget::Site(target) = order.target {
                    projection.uncertain_sites.insert(target);
                    uncertain_reclamation_sites.insert(target);
                }
                continue;
            };
            if settlers.consume(transfer.source, transfer.count).is_err() {
                projection.completed_next.remove(&order.id);
                projection.unknown_orders.insert(order.id);
                if let ConstructionTarget::Site(target) = order.target {
                    projection.uncertain_sites.insert(target);
                    uncertain_reclamation_sites.insert(target);
                }
                continue;
            }
            projection.development.insert(transfer.source, source);
            projection.development.insert(transfer.target, target);
            ruined.remove(&transfer.target);
        }
    }
    projection
}

fn projected_transfer(
    campaign: &StrategicCampaign,
    data: &GameData,
    transfer: settlers::SettlerTransfer,
    projection: &ForecastProjection,
) -> Option<(
    super::super::development::DevelopmentStepState,
    super::super::development::DevelopmentStepState,
)> {
    let mut source = projection
        .development
        .get(&transfer.source)
        .cloned()
        .unwrap_or_else(|| super::super::development::step_state(campaign, transfer.source));
    let mut target = projection
        .development
        .get(&transfer.target)
        .cloned()
        .unwrap_or_else(|| super::super::development::step_state(campaign, transfer.target));
    let target_population = target.population.checked_add(transfer.count)?;
    let source_population = source.population.checked_sub(transfer.count)?;
    source.population = source_population;
    source.development.displaced = source.development.displaced.saturating_sub(transfer.count);
    target.population = target_population;
    if target.development.ruined {
        target.development.ruined = false;
        target.development.ruined_round = None;
        target.development.ruin_streak = 0;
        target.development.lawless_rounds = 0;
        target.development.pressure = 0;
        target.damage = data.development.conditions.reclamation_damage;
    }
    target.habitation = Habitation::Outpost;
    Some((source, target))
}

fn complete(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    supply: &SupplySnapshot,
    settlers: &mut settlers::SettlerSnapshot,
    order: &ConstructionOrder,
) -> Result<(), RuleError> {
    match (order.target, order.kind) {
        (ConstructionTarget::Site(id), ConstructionKind::Outpost) => {
            settlers.transfer(campaign, data, supply, order)?;
            if campaign.site_is_ruined(id) {
                let state = campaign
                    .world
                    .development
                    .get_mut(&id)
                    .expect("site development");
                state.ruined = false;
                state.ruined_round = None;
                state.ruin_streak = 0;
                state.lawless_rounds = 0;
                state.pressure = 0;
                campaign
                    .world
                    .site_damage
                    .insert(id, data.development.conditions.reclamation_damage);
            }
            campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == id)
                .expect("valid site")
                .habitation = Habitation::Outpost;
            campaign
                .world
                .founded_rounds
                .entry(id)
                .or_insert(campaign.completed_rounds);
        }
        (ConstructionTarget::Site(id), ConstructionKind::Fort) => {
            campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == id)
                .expect("valid site")
                .military = MilitaryLayer::Fort;
        }
        (ConstructionTarget::Site(id), ConstructionKind::Facility(facility)) => {
            let site = campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == id)
                .expect("valid site");
            if !site.facilities.contains(&facility) {
                site.facilities.push(facility);
                site.facilities.sort();
            }
        }
        (ConstructionTarget::Route(id), ConstructionKind::Road | ConstructionKind::RoadRepair) => {
            let road = &mut campaign
                .world
                .routes
                .iter_mut()
                .find(|route| route.id == id)
                .expect("valid route")
                .road;
            road.improved = true;
            road.damage = 0;
        }
        _ => return Err(blocked(ConstructionBlock::InvalidTarget)),
    }
    Ok(())
}
