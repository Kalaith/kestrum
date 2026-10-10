//! Deterministic country-scale topology derived from the authored production map.

use super::{
    economy::Economy,
    world::{
        AtlasPath, MajorMarker, MarkerId, MarkerLocation, RouteId, Scenario, ScenarioKind, SiteId,
        WorldLayout, LAYOUT_SOURCE,
    },
    GameData,
};
use std::collections::{BTreeMap, BTreeSet};

const COUNTRY_LAYOUT_REVISION: u32 = 4;
const COUNTRY_CLUSTER_RADIUS: f32 = 0.05;
pub(crate) const CURRENT_PRODUCTION_TOPOLOGY_REVISION: u32 = 2;
pub(crate) const CURRENT_PRODUCTION_LAYOUT_REVISION: u32 = 5;

impl GameData {
    /// Resolves a saved production topology and layout to durable authored content.
    pub fn production_topology(
        &self,
        topology_revision: u32,
        layout_revision: u32,
    ) -> Result<WorldLayout, String> {
        match (topology_revision, layout_revision) {
            (1, 1..=3) => {
                let mut layout = self.production_layout.clone();
                layout.layout_revision = layout_revision;
                Ok(layout)
            }
            (1, COUNTRY_LAYOUT_REVISION) => {
                self.production_layout.flat_country_topology(&self.economy)
            }
            (2, CURRENT_PRODUCTION_LAYOUT_REVISION) => self.regional_geography.resolve(
                &self.production_layout,
                topology_revision,
                &self.rules,
                &self.economy,
            ),
            _ => Err(format!(
                "campaign.world: unsupported production topology/layout revision {topology_revision}/{layout_revision}"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CountryRegionProjection {
    center: [f32; 2],
    origin: [f32; 2],
    pub(super) scale: f32,
}

impl CountryRegionProjection {
    pub(super) fn project(self, point: [f32; 2]) -> [f32; 2] {
        [
            self.center[0] + (point[0] - self.origin[0]) * self.scale,
            self.center[1] + (point[1] - self.origin[1]) * self.scale,
        ]
    }
}

pub(super) fn country_region_projection(
    layout: &WorldLayout,
    marker: MarkerId,
    headquarters: SiteId,
    local_positions: &[[f32; 2]],
) -> Result<CountryRegionProjection, String> {
    let center = layout
        .marker(marker)
        .ok_or("production topology: regional marker disappeared")?
        .position;
    let origin = layout
        .site(headquarters)
        .ok_or("production topology: regional headquarters disappeared")?
        .position;
    let scale = country_cluster_scale(center, origin, local_positions)?;
    Ok(CountryRegionProjection {
        center,
        origin,
        scale,
    })
}

impl WorldLayout {
    /// Flattens nested authored regions without changing physical identities or routes.
    pub(crate) fn flat_country_topology(&self, economy: &Economy) -> Result<Self, String> {
        self.flat_country_topology_for_revision(economy, COUNTRY_LAYOUT_REVISION)
    }

    pub(super) fn flat_country_topology_for_revision(
        &self,
        economy: &Economy,
        layout_revision: u32,
    ) -> Result<Self, String> {
        if !matches!(
            layout_revision,
            COUNTRY_LAYOUT_REVISION | CURRENT_PRODUCTION_LAYOUT_REVISION
        ) {
            return Err(format!(
                "{LAYOUT_SOURCE}: unsupported country layout revision"
            ));
        }
        let headquarters_by_region = self.region_headquarters()?;
        let marker_by_site = self.marker_ids_by_site(&headquarters_by_region)?;
        let mut country = self.clone();
        country.layout_revision = layout_revision;
        country.sites = self.country_sites(&headquarters_by_region, &marker_by_site)?;
        country.markers = country_markers(&country.sites);
        country.routes = self.country_routes(&marker_by_site)?;
        country.atlas_paths = self.country_atlas_paths(&country, &headquarters_by_region)?;
        country.terrain_features = self.country_terrain_features(&headquarters_by_region)?;
        country.validate_country_geometry(economy)?;
        Ok(country)
    }

    fn region_headquarters(&self) -> Result<BTreeMap<MarkerId, SiteId>, String> {
        let mut headquarters = BTreeMap::new();
        for candidate in &self.headquarters_candidates {
            let site = self.site(*candidate).ok_or_else(|| {
                format!("{LAYOUT_SOURCE}: headquarters candidate {candidate:?} is missing")
            })?;
            if matches!(self.marker(site.marker).map(|marker| &marker.location), Some(MarkerLocation::Region { sites, .. }) if sites.contains(candidate))
            {
                if headquarters.insert(site.marker, *candidate).is_some() {
                    return Err(format!(
                        "{LAYOUT_SOURCE}: each region needs one designated headquarters site"
                    ));
                }
            } else {
                return Err(format!(
                    "{LAYOUT_SOURCE}: headquarters candidate {candidate:?} must be inside a region"
                ));
            }
        }
        let regions: BTreeSet<_> = self
            .markers
            .iter()
            .filter_map(|marker| {
                matches!(marker.location, MarkerLocation::Region { .. }).then_some(marker.id)
            })
            .collect();
        if headquarters.keys().copied().collect::<BTreeSet<_>>() != regions {
            return Err(format!(
                "{LAYOUT_SOURCE}: every region needs exactly one designated headquarters site"
            ));
        }
        Ok(headquarters)
    }

    fn marker_ids_by_site(
        &self,
        headquarters_by_region: &BTreeMap<MarkerId, SiteId>,
    ) -> Result<BTreeMap<SiteId, MarkerId>, String> {
        let mut marker_by_site = BTreeMap::new();
        let mut used_markers = BTreeSet::new();
        let mut next_marker = self
            .markers
            .iter()
            .map(|marker| marker.id.0)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("production markers: identifier space exhausted")?;

        for marker in &self.markers {
            match &marker.location {
                MarkerLocation::Site { site } => {
                    insert_site_marker(&mut marker_by_site, &mut used_markers, *site, marker.id)?;
                }
                MarkerLocation::Region { sites, .. } => {
                    let headquarters = headquarters_by_region.get(&marker.id).ok_or_else(|| {
                        format!(
                            "{LAYOUT_SOURCE}: region {:?} has no designated headquarters",
                            marker.id
                        )
                    })?;
                    if !sites.contains(headquarters) {
                        return Err(format!(
                            "{LAYOUT_SOURCE}: headquarters {headquarters:?} is outside region {:?}",
                            marker.id
                        ));
                    }
                    insert_site_marker(
                        &mut marker_by_site,
                        &mut used_markers,
                        *headquarters,
                        marker.id,
                    )?;
                    for site in sites.iter().copied().filter(|site| site != headquarters) {
                        while used_markers.contains(&MarkerId(next_marker)) {
                            next_marker = next_marker
                                .checked_add(1)
                                .ok_or("production markers: identifier space exhausted")?;
                        }
                        let id = MarkerId(next_marker);
                        next_marker = next_marker
                            .checked_add(1)
                            .ok_or("production markers: identifier space exhausted")?;
                        insert_site_marker(&mut marker_by_site, &mut used_markers, site, id)?;
                    }
                }
            }
        }
        if marker_by_site.len() != self.sites.len() {
            return Err(format!(
                "{LAYOUT_SOURCE}: each physical site must receive exactly one country marker"
            ));
        }
        Ok(marker_by_site)
    }

    fn country_sites(
        &self,
        headquarters_by_region: &BTreeMap<MarkerId, SiteId>,
        marker_by_site: &BTreeMap<SiteId, MarkerId>,
    ) -> Result<Vec<super::world::Site>, String> {
        let mut positions = BTreeMap::new();
        for marker in &self.markers {
            let MarkerLocation::Region { sites, .. } = &marker.location else {
                continue;
            };
            let headquarters = *headquarters_by_region.get(&marker.id).ok_or_else(|| {
                format!(
                    "{LAYOUT_SOURCE}: region {:?} has no designated headquarters",
                    marker.id
                )
            })?;
            let local_positions = self.region_geometry_positions(marker.id, sites)?;
            let projection =
                country_region_projection(self, marker.id, headquarters, &local_positions)?;
            for site_id in sites {
                let site = self
                    .site(*site_id)
                    .ok_or("production markers: region site disappeared")?;
                let position = projection.project(site.position);
                positions.insert(*site_id, position);
            }
        }

        self.sites
            .iter()
            .map(|site| {
                let mut country_site = site.clone();
                country_site.marker = *marker_by_site
                    .get(&site.id)
                    .ok_or("production markers: physical site has no country marker")?;
                if let Some(position) = positions.get(&site.id) {
                    country_site.position = *position;
                }
                Ok(country_site)
            })
            .collect()
    }

    fn region_geometry_positions(
        &self,
        marker: MarkerId,
        sites: &[SiteId],
    ) -> Result<Vec<[f32; 2]>, String> {
        let mut positions = sites
            .iter()
            .map(|site_id| {
                self.site(*site_id)
                    .map(|site| site.position)
                    .ok_or("production markers: region site disappeared")
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (route_id, path) in &self.atlas_paths {
            let route = self
                .routes
                .iter()
                .find(|route| route.id == *route_id)
                .ok_or("production atlas paths: route disappeared")?;
            if self
                .site(route.from)
                .is_some_and(|site| site.marker == marker)
                && self
                    .site(route.to)
                    .is_some_and(|site| site.marker == marker)
            {
                positions.extend(path.waypoints.iter().copied());
                positions.extend(path.bridges.iter().copied());
            }
        }
        for feature in &self.terrain_features {
            if self
                .site(feature.anchor)
                .is_some_and(|site| site.marker == marker)
            {
                let half_width = feature.width * 0.5;
                for point in &feature.points {
                    positions.extend([
                        [point[0] - half_width, point[1]],
                        [point[0] + half_width, point[1]],
                        [point[0], point[1] - half_width],
                        [point[0], point[1] + half_width],
                    ]);
                }
            }
        }
        Ok(positions)
    }

    fn country_routes(
        &self,
        marker_by_site: &BTreeMap<SiteId, MarkerId>,
    ) -> Result<Vec<super::world::Route>, String> {
        self.routes
            .iter()
            .map(|route| {
                let from = *marker_by_site
                    .get(&route.from)
                    .ok_or("production routes: missing country marker for origin")?;
                let to = *marker_by_site
                    .get(&route.to)
                    .ok_or("production routes: missing country marker for destination")?;
                let mut country_route = route.clone();
                country_route.major_connection = (from != to).then_some([from, to]);
                Ok(country_route)
            })
            .collect()
    }

    fn country_atlas_paths(
        &self,
        country: &WorldLayout,
        headquarters_by_region: &BTreeMap<MarkerId, SiteId>,
    ) -> Result<BTreeMap<RouteId, AtlasPath>, String> {
        let mut paths = BTreeMap::new();
        for (route_id, path) in &self.atlas_paths {
            let route = self
                .routes
                .iter()
                .find(|route| route.id == *route_id)
                .ok_or("production atlas paths: route disappeared")?;
            let from_site = self
                .site(route.from)
                .ok_or("production atlas paths: route origin disappeared")?;
            let to_site = self
                .site(route.to)
                .ok_or("production atlas paths: route destination disappeared")?;
            if route.major_connection.is_none() && from_site.marker == to_site.marker {
                let headquarters = *headquarters_by_region
                    .get(&from_site.marker)
                    .ok_or("production atlas paths: region headquarters disappeared")?;
                let MarkerLocation::Region { sites, .. } = &self
                    .marker(from_site.marker)
                    .ok_or("production atlas paths: region marker disappeared")?
                    .location
                else {
                    return Err("production atlas paths: internal route is outside a region".into());
                };
                let positions = self.region_geometry_positions(from_site.marker, sites)?;
                let projection =
                    country_region_projection(self, from_site.marker, headquarters, &positions)?;
                paths.insert(*route_id, project_path(path, projection));
                continue;
            }
            let [old_from, old_to] = route
                .major_connection
                .ok_or("production atlas paths: authored route has no major endpoints")?;
            let new_from = country
                .marker(country_site_marker(country, from_site.id)?)
                .ok_or("production atlas paths: derived origin marker disappeared")?;
            let new_to = country
                .marker(country_site_marker(country, to_site.id)?)
                .ok_or("production atlas paths: derived destination marker disappeared")?;
            let old_from_position = self
                .marker(old_from)
                .ok_or("production atlas paths: authored origin marker disappeared")?
                .position;
            let old_to_position = self
                .marker(old_to)
                .ok_or("production atlas paths: authored destination marker disappeared")?
                .position;
            paths.insert(
                *route_id,
                reanchor_path(
                    path,
                    old_from_position,
                    old_to_position,
                    new_from.position,
                    new_to.position,
                ),
            );
        }
        Ok(paths)
    }

    fn country_terrain_features(
        &self,
        headquarters_by_region: &BTreeMap<MarkerId, SiteId>,
    ) -> Result<Vec<super::world::TerrainFeature>, String> {
        self.terrain_features
            .iter()
            .map(|feature| {
                let marker = self
                    .site(feature.anchor)
                    .ok_or("production terrain: feature anchor disappeared")?
                    .marker;
                let headquarters = *headquarters_by_region
                    .get(&marker)
                    .ok_or("production terrain: feature is outside a region")?;
                let MarkerLocation::Region { sites, .. } = &self
                    .marker(marker)
                    .ok_or("production terrain: region marker disappeared")?
                    .location
                else {
                    return Err("production terrain: feature is outside a region".into());
                };
                let positions = self.region_geometry_positions(marker, sites)?;
                let projection = country_region_projection(self, marker, headquarters, &positions)?;
                Ok(super::world::TerrainFeature {
                    key: feature.key.clone(),
                    anchor: feature.anchor,
                    kind: feature.kind,
                    points: feature
                        .points
                        .iter()
                        .map(|point| projection.project(*point))
                        .collect(),
                    width: feature.width * projection.scale,
                })
            })
            .collect()
    }

    pub(super) fn validate_country_geometry(&self, economy: &Economy) -> Result<(), String> {
        if !matches!(
            self.layout_revision,
            COUNTRY_LAYOUT_REVISION | CURRENT_PRODUCTION_LAYOUT_REVISION
        ) || self.markers.len() != self.sites.len()
            || self
                .markers
                .iter()
                .any(|marker| !matches!(marker.location, MarkerLocation::Site { .. }))
        {
            return Err(format!(
                "{LAYOUT_SOURCE}: derived country topology needs one simple marker per physical site"
            ));
        }
        let scenario = Scenario {
            schema_version: self.schema_version,
            content_version: self.content_version,
            kind: ScenarioKind::Production,
            name: "Derived production country topology".into(),
            seed: self.default_seed,
            difficulty: super::rules::Difficulty::Normal,
            player: super::world::FactionId(1),
            markers: self.markers.clone(),
            sites: self.sites.clone(),
            routes: self.routes.clone(),
            factions: Vec::new(),
            relations: Vec::new(),
        };
        scenario.validate_geography(economy)?;
        self.validate_country_atlas()
    }
}

fn insert_site_marker(
    marker_by_site: &mut BTreeMap<SiteId, MarkerId>,
    used_markers: &mut BTreeSet<MarkerId>,
    site: SiteId,
    marker: MarkerId,
) -> Result<(), String> {
    if marker_by_site.insert(site, marker).is_some() || !used_markers.insert(marker) {
        return Err(format!(
            "{LAYOUT_SOURCE}: country topology contains a duplicate site or marker"
        ));
    }
    Ok(())
}

fn country_markers(sites: &[super::world::Site]) -> Vec<MajorMarker> {
    let mut markers: Vec<_> = sites
        .iter()
        .map(|site| MajorMarker {
            id: site.marker,
            key: site.key.clone(),
            name: site.name.clone(),
            position: site.position,
            location: MarkerLocation::Site { site: site.id },
        })
        .collect();
    markers.sort_by_key(|marker| marker.id);
    markers
}

fn country_cluster_scale(
    origin: [f32; 2],
    headquarters: [f32; 2],
    local_positions: &[[f32; 2]],
) -> Result<f32, String> {
    let max_delta = local_positions
        .iter()
        .flat_map(|position| {
            [
                (position[0] - headquarters[0]).abs(),
                (position[1] - headquarters[1]).abs(),
            ]
        })
        .fold(0.0f32, f32::max);
    if max_delta == 0.0 {
        return Err(format!(
            "{LAYOUT_SOURCE}: regional sites need distinct local positions"
        ));
    }
    let mut scale = COUNTRY_CLUSTER_RADIUS / max_delta;
    for position in local_positions {
        for axis in 0..2 {
            let offset = position[axis] - headquarters[axis];
            if offset > 0.0 {
                scale = scale.min((1.0 - origin[axis]) / offset);
            } else if offset < 0.0 {
                scale = scale.min(origin[axis] / -offset);
            }
        }
    }
    Ok(scale)
}

fn country_site_marker(country: &WorldLayout, site: SiteId) -> Result<MarkerId, String> {
    country
        .site(site)
        .map(|entry| entry.marker)
        .ok_or_else(|| "production atlas paths: physical site disappeared".to_owned())
}

fn reanchor_path(
    path: &AtlasPath,
    old_from: [f32; 2],
    old_to: [f32; 2],
    new_from: [f32; 2],
    new_to: [f32; 2],
) -> AtlasPath {
    let mut waypoints = Vec::with_capacity(path.waypoints.len() + 2);
    if new_from != old_from {
        waypoints.push(old_from);
    }
    waypoints.extend(path.waypoints.iter().copied());
    if new_to != old_to {
        waypoints.push(old_to);
    }
    AtlasPath {
        waypoints,
        bridges: path.bridges.clone(),
    }
}

fn project_path(path: &AtlasPath, projection: CountryRegionProjection) -> AtlasPath {
    AtlasPath {
        waypoints: path
            .waypoints
            .iter()
            .map(|point| projection.project(*point))
            .collect(),
        bridges: path
            .bridges
            .iter()
            .map(|point| projection.project(*point))
            .collect(),
    }
}
