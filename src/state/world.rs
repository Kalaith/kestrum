//! Instantiated fixed geography with campaign-owned mutable site contents.

mod control;

use crate::data::world::{
    FactionId, MajorMarker, MarkerId, MarkerLocation, Route, RouteId, Scenario, Site, SiteId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionControl {
    /// Historical claim remains when no faction currently satisfies the anchors.
    pub political_owner: Option<FactionId>,
    pub contested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundaryCrossing {
    pub route: RouteId,
    pub region: MarkerId,
    pub entrance: SiteId,
    pub external_site: SiteId,
    pub external_marker: MarkerId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignWorld {
    pub population: BTreeMap<SiteId, u32>,
    pub focus: BTreeMap<SiteId, super::construction::Focus>,
    pub markers: Vec<MajorMarker>,
    pub sites: Vec<Site>,
    pub routes: Vec<Route>,
    pub region_control: BTreeMap<MarkerId, RegionControl>,
    /// Public conflict state; these sites cannot provide secure control or supply.
    pub contested_sites: BTreeSet<SiteId>,
    /// Sparse structural damage; absent entries have zero damage.
    pub site_damage: BTreeMap<SiteId, u32>,
    /// Sparse lasting occupation pressure; absent entries are zero.
    pub occupation: BTreeMap<SiteId, u32>,
}

impl CampaignWorld {
    pub fn from_scenario(scenario: &Scenario) -> Self {
        let mut world = Self {
            population: BTreeMap::new(),
            focus: BTreeMap::new(),
            markers: scenario.markers.clone(),
            sites: scenario.sites.clone(),
            routes: scenario.routes.clone(),
            region_control: BTreeMap::new(),
            contested_sites: BTreeSet::new(),
            site_damage: BTreeMap::new(),
            occupation: BTreeMap::new(),
        };
        world.markers.sort_by_key(|marker| marker.id);
        world.sites.sort_by_key(|site| site.id);
        world.routes.sort_by_key(|route| route.id);
        world
    }

    pub fn site(&self, id: SiteId) -> Option<&Site> {
        self.sites.iter().find(|site| site.id == id)
    }

    pub fn structural_damage(&self, site: SiteId) -> u32 {
        self.site_damage.get(&site).copied().unwrap_or(0)
    }

    pub fn marker(&self, id: MarkerId) -> Option<&MajorMarker> {
        self.markers.iter().find(|marker| marker.id == id)
    }

    pub fn route(&self, id: RouteId) -> Option<&Route> {
        self.routes.iter().find(|route| route.id == id)
    }

    /// Resolve only a simple marker. A region has no duplicate physical location.
    pub fn physical_site(&self, marker: MarkerId) -> Option<SiteId> {
        let MarkerLocation::Site { site } = self.marker(marker)?.location else {
            return None;
        };
        self.site(site)
            .filter(|site| site.marker == marker)
            .map(|site| site.id)
    }

    pub fn region_control(&self, marker: MarkerId) -> Option<&RegionControl> {
        self.region_control.get(&marker)
    }

    pub fn controller_counts(&self, marker: MarkerId) -> BTreeMap<Option<FactionId>, usize> {
        let mut counts = BTreeMap::new();
        for site in self.sites.iter().filter(|site| site.marker == marker) {
            *counts.entry(site.controller).or_insert(0) += 1;
        }
        counts
    }

    /// The same boundary supplies both the arrival site and the reverse exit.
    pub fn entrance(&self, region: MarkerId, route: RouteId) -> Option<BoundaryCrossing> {
        let MarkerLocation::Region { entrances, .. } = &self.marker(region)?.location else {
            return None;
        };
        let entrance = entrances.iter().find(|entry| entry.route == route)?.site;
        let edge = self.route(route)?;
        let external_site = edge.other_endpoint(entrance)?;
        let external_marker = self.site(external_site)?.marker;
        if self.site(entrance)?.marker != region || external_marker == region {
            return None;
        }
        let pair = edge.major_connection?;
        if pair != [region, external_marker] && pair != [external_marker, region] {
            return None;
        }
        Some(BoundaryCrossing {
            route,
            region,
            entrance,
            external_site,
            external_marker,
        })
    }

    pub fn adjacent_sites(&self, site: SiteId) -> BTreeSet<SiteId> {
        if self.site(site).is_none() {
            return BTreeSet::new();
        }
        self.routes
            .iter()
            .filter_map(|route| route.other_endpoint(site))
            .filter(|other| self.site(*other).is_some())
            .collect()
    }

    pub fn connected_route(&self, from: SiteId, to: SiteId) -> Option<&Route> {
        if self.site(from).is_none() || self.site(to).is_none() {
            return None;
        }
        self.routes
            .iter()
            .find(|route| route.other_endpoint(from) == Some(to))
    }

    /// Topology reachability does not imply legal movement or supply.
    pub fn reachable_sites(&self, start: SiteId) -> BTreeSet<SiteId> {
        self.paths_from(start, |site| self.site(site).is_some())
            .into_keys()
            .collect()
    }

    pub fn is_secure(&self, site: SiteId, faction: FactionId) -> bool {
        self.site(site)
            .is_some_and(|site| site.controller == Some(faction))
            && !self.contested_sites.contains(&site)
    }

    pub fn secure_sites(&self, faction: FactionId) -> BTreeSet<SiteId> {
        self.sites
            .iter()
            .filter(|site| self.is_secure(site.id, faction))
            .map(|site| site.id)
            .collect()
    }

    /// Foreign sites block supply even during peace. Roads never alter connectivity.
    pub fn supplied_sites(&self, faction: FactionId, headquarters: SiteId) -> BTreeSet<SiteId> {
        self.paths_from(headquarters, |site| self.is_secure(site, faction))
            .into_keys()
            .collect()
    }

    /// A deterministic shortest-edge supply path for explanation, not movement cost.
    pub fn supply_path(
        &self,
        faction: FactionId,
        headquarters: SiteId,
        destination: SiteId,
    ) -> Option<Vec<SiteId>> {
        let paths = self.paths_from(headquarters, |site| self.is_secure(site, faction));
        let mut cursor = destination;
        let mut path = vec![cursor];
        while let Some(previous) = paths.get(&cursor)? {
            cursor = *previous;
            path.push(cursor);
        }
        path.reverse();
        Some(path)
    }

    pub fn anchors_satisfied(
        &self,
        region: MarkerId,
        faction: FactionId,
        headquarters: SiteId,
    ) -> Option<bool> {
        let MarkerLocation::Region { anchors, .. } = &self.marker(region)?.location else {
            return None;
        };
        Some(anchors.is_satisfied(
            &self.secure_sites(faction),
            &self.supplied_sites(faction, headquarters),
        ))
    }

    fn paths_from(
        &self,
        start: SiteId,
        usable: impl Fn(SiteId) -> bool,
    ) -> BTreeMap<SiteId, Option<SiteId>> {
        let mut previous = BTreeMap::new();
        if self.site(start).is_none() || !usable(start) {
            return previous;
        }
        previous.insert(start, None);
        let mut pending = VecDeque::from([start]);
        while let Some(site) = pending.pop_front() {
            for adjacent in self.adjacent_sites(site) {
                if !previous.contains_key(&adjacent) && usable(adjacent) {
                    previous.insert(adjacent, Some(site));
                    pending.push_back(adjacent);
                }
            }
        }
        previous
    }
}
