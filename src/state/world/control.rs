//! Regional claims derive from secure physical control; claims never capture sites.

use super::RegionControl;
use crate::{
    data::{
        world::{FactionId, MarkerId, MarkerLocation, SiteId},
        GameData,
    },
    state::{FactionStatus, StrategicCampaign},
};
use std::collections::BTreeMap;

impl StrategicCampaign {
    /// Simulation seam for resolved control changes. Player orders still go through
    /// engine commands; no movement or capture order is introduced by this helper.
    pub fn set_site_control(
        &mut self,
        data: &GameData,
        site: SiteId,
        controller: Option<FactionId>,
        contested: bool,
    ) -> Result<(), String> {
        self.validate(data)?;
        if controller.is_some_and(|id| !self.factions.contains_key(&id)) {
            return Err("campaign.world.sites.controller: unknown faction".into());
        }
        let mut candidate = self.clone();
        candidate
            .world
            .sites
            .iter_mut()
            .find(|entry| entry.id == site)
            .ok_or("campaign.world.sites: unknown physical site")?
            .controller = controller;
        if contested {
            candidate.world.contested_sites.insert(site);
        } else {
            candidate.world.contested_sites.remove(&site);
        }
        candidate.reconcile_region_control();
        candidate.validate(data)?;
        *self = candidate;
        Ok(())
    }

    /// Reevaluate after any control, conflict, or HQ change. No random state is used.
    pub fn reconcile_region_control(&mut self) {
        self.world.region_control = self.expected_region_control();
    }

    pub(crate) fn expected_region_control(&self) -> BTreeMap<MarkerId, RegionControl> {
        self.world
            .markers
            .iter()
            .filter_map(|marker| {
                if !matches!(marker.location, MarkerLocation::Region { .. }) {
                    return None;
                }
                let previous = self
                    .world
                    .region_control
                    .get(&marker.id)
                    .and_then(|control| control.political_owner);
                // Current anchor expressions share mandatory sites, so only one side
                // can qualify. For future independent alternatives, retain a qualifying
                // incumbent, otherwise use the lowest faction ID deterministically.
                let qualifies = |id: FactionId| {
                    self.factions.get(&id).is_some_and(|faction| {
                        faction.status != FactionStatus::Eliminated
                            && self
                                .world
                                .anchors_satisfied(marker.id, id, faction.headquarters)
                                == Some(true)
                    })
                };
                let claimant = previous
                    .filter(|id| qualifies(*id))
                    .or_else(|| self.factions.keys().copied().find(|id| qualifies(*id)));
                Some((
                    marker.id,
                    RegionControl {
                        political_owner: claimant.or(previous),
                        contested: claimant.is_none(),
                    },
                ))
            })
            .collect()
    }

    pub(crate) fn validate_region_control(&self) -> Result<(), String> {
        if self
            .world
            .contested_sites
            .iter()
            .any(|id| self.world.site(*id).is_none())
        {
            return Err("campaign.world.contested_sites: unknown physical site".into());
        }
        if self.world.region_control.values().any(|control| {
            control
                .political_owner
                .is_some_and(|id| !self.factions.contains_key(&id))
        }) {
            return Err("campaign.world.region_control: unknown political owner".into());
        }
        if self.world.region_control != self.expected_region_control() {
            return Err("campaign.world.region_control: missing region or claim inconsistent with secure anchors".into());
        }
        Ok(())
    }
}
