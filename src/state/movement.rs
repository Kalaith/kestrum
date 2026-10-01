//! Saved group routes retain physical sites and survive seasonal checkpoints.

use super::{military::ArmyId, StrategicCampaign};
use crate::data::world::SiteId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementPlan {
    pub armies: Vec<ArmyId>,
    /// Starts at the group's current site, followed by the remaining route.
    pub path: Vec<SiteId>,
}

impl StrategicCampaign {
    pub(crate) fn reconcile_movement_plans(&mut self) {
        let armies = &self.armies;
        let factions = &self.factions;
        let sieges = &self.sieges;
        self.movement_plans.retain(|plan| {
            let Some(first) = plan.armies.first().and_then(|id| armies.get(id)) else {
                return false;
            };
            factions
                .get(&first.faction)
                .is_some_and(|faction| faction.status == super::FactionStatus::Independent)
                && !sieges.contains_key(&first.site)
                && plan.path.first() == Some(&first.site)
                && plan.armies.iter().all(|id| {
                    armies.get(id).is_some_and(|army| {
                        army.site == first.site && army.faction == first.faction
                    })
                })
        });
    }

    pub(crate) fn validate_movement_plans(&self) -> Result<(), String> {
        let mut assigned = BTreeSet::new();
        for plan in &self.movement_plans {
            let first = plan.armies.first().and_then(|id| self.armies.get(id));
            if plan.armies.is_empty()
                || !plan.armies.windows(2).all(|pair| pair[0] < pair[1])
                || !plan.armies.iter().all(|id| {
                    assigned.insert(*id)
                        && self.armies.get(id).is_some_and(|army| {
                            first.is_some_and(|origin| {
                                army.site == origin.site && army.faction == origin.faction
                            })
                        })
                })
                || plan.path.len() < 2
                || first.is_none_or(|army| plan.path.first() != Some(&army.site))
                || plan.path.iter().copied().collect::<BTreeSet<_>>().len() != plan.path.len()
                || !plan
                    .path
                    .windows(2)
                    .all(|pair| self.world.connected_route(pair[0], pair[1]).is_some())
            {
                return Err("campaign.movement_plans: invalid group or physical route".into());
            }
        }
        Ok(())
    }
}
