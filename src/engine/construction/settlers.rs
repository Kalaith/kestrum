//! A frozen donor budget prevents new arrivals being moved twice in one boundary.

use super::*;
use crate::state::world::CampaignWorld;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
pub(super) struct SettlerTransfer {
    pub(super) source: SiteId,
    pub(super) target: SiteId,
    pub(super) count: u32,
}

pub(super) enum ForecastSource {
    Known(Option<SettlerTransfer>),
    Unknown(BTreeSet<SiteId>),
}

pub(super) struct SettlerSnapshot {
    world: CampaignWorld,
    remaining: BTreeMap<SiteId, u32>,
    threats: BTreeSet<SiteId>,
}

impl SettlerSnapshot {
    pub(super) fn new(campaign: &StrategicCampaign, data: &GameData) -> Self {
        let remaining = campaign
            .world
            .sites
            .iter()
            .map(|site| {
                (
                    site.id,
                    campaign.world.population[&site.id]
                        .saturating_sub(data.construction.population.minimum[&site.habitation]),
                )
            })
            .collect();
        Self {
            world: campaign.world.clone(),
            remaining,
            threats: campaign
                .world
                .sites
                .iter()
                .filter(|site| campaign.active_threat(site.id).is_some())
                .map(|site| site.id)
                .collect(),
        }
    }

    pub(super) fn source(
        &self,
        data: &GameData,
        supply: &SupplySnapshot,
        order: &ConstructionOrder,
    ) -> Option<(SiteId, u32)> {
        let ConstructionTarget::Site(target) = order.target else {
            return None;
        };
        let mut distances = BTreeMap::from([(target, 0_u64)]);
        let mut queue = BTreeSet::from([(0_u64, target)]);
        while let Some((distance, id)) = queue.pop_first() {
            if distance != distances[&id] {
                continue;
            }
            let site = self.world.site(id)?;
            if id != target
                && self.world.is_secure(id, order.owner)
                && site.habitation != Habitation::Unsettled
                && !self.world.development[&id].ruined
                && !self.threats.contains(&id)
                && self.remaining[&id] > 0
            {
                return Some((
                    id,
                    self.remaining[&id].min(data.construction.population.settler_limit),
                ));
            }
            for adjacent in self.world.adjacent_sites(id) {
                if !supply.contains(order.owner, adjacent)
                    || !self.world.is_secure(adjacent, order.owner)
                    || self.threats.contains(&adjacent)
                {
                    continue;
                }
                let edge = self
                    .world
                    .connected_route(id, adjacent)
                    .expect("connected edge");
                let next = distance + u64::from(super::super::route_cost(edge, data));
                if distances
                    .get(&adjacent)
                    .is_none_or(|previous| next < *previous)
                {
                    distances.insert(adjacent, next);
                    queue.insert((next, adjacent));
                }
            }
        }
        None
    }

    pub(super) fn forecast_source(
        &self,
        data: &GameData,
        supply: &SupplySnapshot,
        order: &ConstructionOrder,
        observed: &BTreeSet<SiteId>,
        uncertain_budgets: &BTreeSet<SiteId>,
    ) -> ForecastSource {
        let ConstructionTarget::Site(target) = order.target else {
            return ForecastSource::Known(None);
        };
        let reachable = self.reachable_sites(supply, order.owner, target);
        let potential_donors = reachable
            .iter()
            .copied()
            .filter(|id| *id != target)
            .collect::<BTreeSet<_>>();
        // Do not derive uncertainty from unseen settlement state. Every
        // reachable owned site may affect donor selection until it is observed.
        if reachable.iter().any(|id| !observed.contains(id)) {
            return ForecastSource::Unknown(potential_donors);
        }
        let donors = reachable
            .iter()
            .copied()
            .filter(|id| *id != target)
            .filter(|id| {
                let site = self.world.site(*id).expect("reachable donor");
                site.habitation != Habitation::Unsettled
                    && !self.world.development[id].ruined
                    && self.remaining[id] > 0
            })
            .collect::<BTreeSet<_>>();
        if donors.is_empty() {
            return ForecastSource::Known(None);
        }
        if donors.iter().any(|id| uncertain_budgets.contains(id)) {
            return ForecastSource::Unknown(donors);
        }
        ForecastSource::Known(self.source(data, supply, order).map(|(source, count)| {
            SettlerTransfer {
                source,
                target,
                count,
            }
        }))
    }

    fn reachable_sites(
        &self,
        supply: &SupplySnapshot,
        owner: FactionId,
        target: SiteId,
    ) -> BTreeSet<SiteId> {
        let mut reachable = BTreeSet::from([target]);
        let mut queue = vec![target];
        while let Some(id) = queue.pop() {
            for adjacent in self.world.adjacent_sites(id) {
                if supply.contains(owner, adjacent)
                    && self
                        .world
                        .site(adjacent)
                        .is_some_and(|site| site.controller == Some(owner))
                    && reachable.insert(adjacent)
                {
                    queue.push(adjacent);
                }
            }
        }
        reachable
    }

    pub(super) fn transfer(
        &mut self,
        campaign: &mut StrategicCampaign,
        data: &GameData,
        supply: &SupplySnapshot,
        order: &ConstructionOrder,
    ) -> Result<(), RuleError> {
        let ConstructionTarget::Site(target) = order.target else {
            return Err(blocked(ConstructionBlock::InvalidTarget));
        };
        let (source, count) = self.source(data, supply, order).ok_or_else(|| {
            RuleError::InvalidState("Eligible settlers disappeared during completion.".into())
        })?;
        self.consume(source, count)?;
        let destination = campaign.world.population[&target]
            .checked_add(count)
            .ok_or(RuleError::Overflow {
                field: "settlement population",
            })?;
        *campaign
            .world
            .population
            .get_mut(&source)
            .expect("donor exists") -= count;
        campaign.world.population.insert(target, destination);
        let state = campaign
            .world
            .development
            .get_mut(&source)
            .expect("donor development");
        state.displaced = state.displaced.saturating_sub(count);
        Ok(())
    }

    pub(super) fn consume(&mut self, source: SiteId, count: u32) -> Result<(), RuleError> {
        let remaining = self.remaining.get_mut(&source).expect("donor budget");
        *remaining = remaining.checked_sub(count).ok_or_else(|| {
            RuleError::InvalidState("Construction settler budget was already consumed.".into())
        })?;
        Ok(())
    }
}
