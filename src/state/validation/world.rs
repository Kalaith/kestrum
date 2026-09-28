//! Saved geography is validated against its own supported authored revision.
use super::*;
use crate::data::world::{MajorMarker, Route, Site};

impl StrategicCampaign {
    pub(super) fn validate_world(&self, data: &GameData) -> Result<(), String> {
        let (markers, sites, routes): (&[_], &[_], &[_]) = match self.scenario_kind {
            ScenarioKind::RosemarchPrototype => (
                &data.scenario.markers,
                &data.scenario.sites,
                &data.scenario.routes,
            ),
            ScenarioKind::Production => (
                &data.production_layout.markers,
                &data.production_layout.sites,
                &data.production_layout.routes,
            ),
        };
        self.validate_layout_revision(data)?;
        require(
            self.world.sites.len() == sites.len()
                && self.world.routes.len() == routes.len()
                && self.world.markers.len() == markers.len(),
            "world",
            "fixed topology counts changed",
        )?;
        require(
            self.world
                .sites
                .windows(2)
                .all(|pair| pair[0].id < pair[1].id)
                && self
                    .world
                    .routes
                    .windows(2)
                    .all(|pair| pair[0].id < pair[1].id)
                && self
                    .world
                    .markers
                    .windows(2)
                    .all(|pair| pair[0].id < pair[1].id),
            "world",
            "IDs must be unique and ordered",
        )?;
        for marker in &self.world.markers {
            let mut original = markers
                .iter()
                .find(|authored| authored.id == marker.id)
                .cloned()
                .ok_or("campaign.world.markers: unknown marker")?;
            if matches!(
                marker.location,
                crate::data::world::MarkerLocation::Site { .. }
            ) {
                original.name = marker.name.clone();
            }
            if self.scenario_kind == ScenarioKind::Production && self.world.layout_revision == 1 {
                if let Some(position) = data
                    .production_layout
                    .legacy_marker_positions
                    .get(&marker.id)
                {
                    original.position = *position;
                }
            }
            require(
                original == *marker && !marker.name.trim().is_empty(),
                "world.markers",
                "fixed marker or entrance mapping changed",
            )?;
        }
        self.validate_sites(data, markers, sites)?;
        self.validate_routes(routes)?;
        self.validate_region_control()
    }

    fn validate_sites(
        &self,
        data: &GameData,
        markers: &[MajorMarker],
        sites: &[Site],
    ) -> Result<(), String> {
        for site in &self.world.sites {
            let original = sites
                .iter()
                .find(|authored| authored.id == site.id)
                .ok_or("campaign.world.sites: unknown site")?;
            let mut position = original.position;
            let mut tags = original.tags.clone();
            // Revision 3 reserves regional starts with horse access. Older
            // worlds keep their saved terrain and original founding locations.
            if self.scenario_kind == ScenarioKind::Production
                && self.world.layout_revision < 3
                && data
                    .production_layout
                    .headquarters_candidates
                    .contains(&site.id)
            {
                tags.retain(|tag| *tag != crate::data::world::SiteTag::HorseAccess);
            }
            if self.scenario_kind == ScenarioKind::Production && self.world.layout_revision == 1
                && markers.iter().any(|m| matches!(m.location, crate::data::world::MarkerLocation::Site { site: id } if id == site.id)) {
                position = data.production_layout.legacy_marker_positions.get(&site.marker).copied().unwrap_or(position);
            }
            require(
                site.marker == original.marker
                    && site.position == position
                    && site.geography == original.geography
                    && site.key == original.key
                    && site.tags == tags,
                "world.sites",
                "fixed site geography changed",
            )?;
            require(
                !site.name.trim().is_empty(),
                "world.sites.name",
                "empty site name",
            )?;
            require(
                site.controller
                    .is_none_or(|controller| self.factions.contains_key(&controller)),
                "world.sites.controller",
                "unknown faction",
            )?;
            let facilities: BTreeSet<_> = site.facilities.iter().collect();
            require(
                facilities.len() == site.facilities.len(),
                "world.sites.facilities",
                "duplicate facility",
            )?;
        }
        Ok(())
    }

    fn validate_routes(&self, routes: &[Route]) -> Result<(), String> {
        for route in &self.world.routes {
            let original = routes
                .iter()
                .find(|authored| authored.id == route.id)
                .ok_or("campaign.world.routes: unknown route")?;
            require(
                route.from == original.from
                    && route.to == original.to
                    && route.major_connection == original.major_connection
                    && route.terrain_cost == original.terrain_cost
                    && route.road.damage <= 100
                    && (route.road.improved || route.road.damage == 0),
                "world.routes",
                "fixed route changed or invalid road damage",
            )?;
        }
        Ok(())
    }

    fn validate_layout_revision(&self, data: &GameData) -> Result<(), String> {
        let valid = match (self.scenario_kind, self.world.layout_revision) {
            (ScenarioKind::RosemarchPrototype, 1) | (ScenarioKind::Production, 1) => {
                self.world.atlas_paths.is_empty()
            }
            (ScenarioKind::Production, revision)
                if revision == 2 || revision == data.production_layout.layout_revision =>
            {
                self.world.atlas_paths == data.production_layout.atlas_paths
            }
            _ => false,
        };
        require(
            valid,
            "world.layout_revision",
            "unsupported or changed authored atlas geometry",
        )
    }
}
