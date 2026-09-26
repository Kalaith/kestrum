//! Instantiated fixed geography with campaign-owned mutable site contents.

use crate::data::world::{MajorMarker, Route, Scenario, Site, SiteId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignWorld {
    pub markers: Vec<MajorMarker>,
    pub sites: Vec<Site>,
    pub routes: Vec<Route>,
}

impl CampaignWorld {
    pub fn from_scenario(scenario: &Scenario) -> Self {
        let mut world = Self {
            markers: scenario.markers.clone(),
            sites: scenario.sites.clone(),
            routes: scenario.routes.clone(),
        };
        world.markers.sort_by_key(|marker| marker.id);
        world.sites.sort_by_key(|site| site.id);
        world.routes.sort_by_key(|route| route.id);
        world
    }

    pub fn site(&self, id: SiteId) -> Option<&Site> {
        self.sites.iter().find(|site| site.id == id)
    }
}
