//! Campaign supply includes local occupants without inventing political owners.

use crate::{
    data::world::{FactionId, MarkerId, MarkerLocation, SiteId},
    state::{threat::ThreatStatus, StrategicCampaign},
};
use std::collections::BTreeSet;

impl StrategicCampaign {
    pub fn supplied_sites(&self, faction: FactionId) -> BTreeSet<SiteId> {
        let Some(owner) = self.factions.get(&faction) else {
            return BTreeSet::new();
        };
        self.world
            .supplied_sites_avoiding(faction, owner.headquarters, &self.supply_blockers())
    }

    pub fn supply_path(&self, faction: FactionId, destination: SiteId) -> Option<Vec<SiteId>> {
        let owner = self.factions.get(&faction)?;
        self.world.supply_path_avoiding(
            faction,
            owner.headquarters,
            destination,
            &self.supply_blockers(),
        )
    }

    pub(crate) fn regional_anchors_satisfied(
        &self,
        region: MarkerId,
        faction: FactionId,
    ) -> Option<bool> {
        let MarkerLocation::Region { anchors, .. } = &self.world.marker(region)?.location else {
            return None;
        };
        let blocked = self.supply_blockers();
        let secure = self
            .world
            .secure_sites(faction)
            .difference(&blocked)
            .copied()
            .collect();
        Some(anchors.is_satisfied(&secure, &self.supplied_sites(faction)))
    }

    fn supply_blockers(&self) -> BTreeSet<SiteId> {
        self.threats
            .values()
            .filter(|threat| threat.status == ThreatStatus::Active)
            .map(|threat| threat.site)
            .collect()
    }
}
