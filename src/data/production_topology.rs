//! Deterministic country-scale topology derived from the authored production map.

use super::{
    economy::Economy,
    world::{
        AtlasPath, MajorMarker, MarkerId, MarkerLocation, RouteId, Scenario, ScenarioKind, SiteId,
        WorldLayout, LAYOUT_SOURCE,
    },
};
use std::collections::{BTreeMap, BTreeSet};

const COUNTRY_LAYOUT_REVISION: u32 = 4;
const COUNTRY_CLUSTER_RADIUS: f32 = 0.05;

impl WorldLayout {
    /// Flattens nested authored regions without changing physical identities or routes.
    pub(crate) fn flat_country_topology(&self, economy: &Economy) -> Result<Self, String> {
        let headquarters_by_region = self.region_headquarters()?;
        let marker_by_site = self.marker_ids_by_site(&headquarters_by_region)?;
        let mut country = self.clone();
        country.layout_revision = COUNTRY_LAYOUT_REVISION;
        country.sites = self.country_sites(&headquarters_by_region, &marker_by_site)?;
        country.markers = country_markers(&country.sites);
        country.routes = self.country_routes(&marker_by_site)?;
        country.atlas_paths = self.country_atlas_paths(&country)?;
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
            let origin = self
                .site(headquarters)
                .ok_or("production markers: headquarters site disappeared")?;
            let scale = country_cluster_scale(marker.position, origin.position, sites, self)?;
            for site_id in sites {
                let site = self
                    .site(*site_id)
                    .ok_or("production markers: region site disappeared")?;
                let position = [
                    marker.position[0] + (site.position[0] - origin.position[0]) * scale,
                    marker.position[1] + (site.position[1] - origin.position[1]) * scale,
                ];
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
    ) -> Result<BTreeMap<RouteId, AtlasPath>, String> {
        let mut paths = BTreeMap::new();
        for (route_id, path) in &self.atlas_paths {
            let route = self
                .routes
                .iter()
                .find(|route| route.id == *route_id)
                .ok_or("production atlas paths: route disappeared")?;
            let [old_from, old_to] = route
                .major_connection
                .ok_or("production atlas paths: authored route has no major endpoints")?;
            let from_site = self
                .site(route.from)
                .ok_or("production atlas paths: route origin disappeared")?;
            let to_site = self
                .site(route.to)
                .ok_or("production atlas paths: route destination disappeared")?;
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

    fn validate_country_geometry(&self, economy: &Economy) -> Result<(), String> {
        if self.layout_revision != COUNTRY_LAYOUT_REVISION
            || self.markers.len() != self.sites.len()
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
    sites: &[SiteId],
    layout: &WorldLayout,
) -> Result<f32, String> {
    let max_delta = sites
        .iter()
        .filter_map(|id| layout.site(*id))
        .flat_map(|site| {
            [
                (site.position[0] - headquarters[0]).abs(),
                (site.position[1] - headquarters[1]).abs(),
            ]
        })
        .fold(0.0f32, f32::max);
    if max_delta == 0.0 {
        return Err(format!(
            "{LAYOUT_SOURCE}: regional sites need distinct local positions"
        ));
    }
    let mut scale = COUNTRY_CLUSTER_RADIUS / max_delta;
    for site_id in sites {
        let site = layout
            .site(*site_id)
            .ok_or("production topology: region site disappeared")?;
        for axis in 0..2 {
            let offset = site.position[axis] - headquarters[axis];
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
