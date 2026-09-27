//! Reconcile interruptions after actions and advance once at the seasonal boundary.

use super::*;
use crate::{
    engine::{actions::record_fact, ActionOutcome},
    state::campaign::DomainFactKind,
};

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
        } else if let Some(reason) = pause_reason(campaign, data, &supply, &settlers, &order) {
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
        if before.last_progress_round == Some(campaign.completed_rounds) {
            continue;
        }
        let mut order = before.clone();
        if !campaign.owns_construction_target(order.owner, order.target) {
            order.status = ConstructionStatus::Cancelled {
                completed_rounds: campaign.completed_rounds,
                reason: CancellationReason::ControlLost,
            };
        } else if let Some(reason) = pause_reason(campaign, data, supply, &settlers, &order) {
            order.status = ConstructionStatus::Paused { reason };
        } else {
            order.progress += 1;
            order.last_progress_round = Some(campaign.completed_rounds);
            order.status = ConstructionStatus::Active;
            if order.progress == order.required_steps {
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
    if order.kind != ConstructionKind::Outpost
        && target_sites
            .iter()
            .any(|site| campaign.site_is_ruined(*site))
    {
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
        && order.progress + 1 == order.required_steps
        && settlers.source(data, supply, order).is_none()
    {
        return Some(ConstructionPause::NoSettlers);
    }
    None
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
