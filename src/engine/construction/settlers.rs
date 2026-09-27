//! A frozen donor budget prevents new arrivals being moved twice in one boundary.

use super::*;
use crate::state::world::CampaignWorld;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct SettlerSnapshot {
    world: CampaignWorld,
    remaining: BTreeMap<SiteId, u32>,
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
            if id != target && site.habitation != Habitation::Unsettled && self.remaining[&id] > 0 {
                return Some((
                    id,
                    self.remaining[&id].min(data.construction.population.settler_limit),
                ));
            }
            for adjacent in self.world.adjacent_sites(id) {
                if !supply.contains(order.owner, adjacent) {
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
        *self.remaining.get_mut(&source).expect("donor budget") -= count;
        campaign.world.population.insert(target, destination);
        Ok(())
    }
}
