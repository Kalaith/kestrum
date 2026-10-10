//! Versioned regional authored patches layered over the durable production map.

use super::{
    economy::Economy,
    rules::CampaignRules,
    world::{
        AtlasPath, Geography, MarkerId, MarkerLocation, MilitaryLayer, Road, Route, RouteId,
        SiteId, SiteTag, TerrainFeature, TerrainFeatureKind, WorldLayout,
    },
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SOURCE: &str = "assets/data/regional_geography.json";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegionalGeography {
    schema_version: u32,
    areas: Vec<RegionalArea>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionalArea {
    key: String,
    introduced_revision: u32,
    marker: MarkerId,
    headquarters_site: SiteId,
    sites: Vec<RegionalSitePatch>,
    routes: Vec<AddedRoute>,
    paths: Vec<RegionalPath>,
    terrain_features: Vec<TerrainFeature>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionalSitePatch {
    site: SiteId,
    position: [f32; 2],
    geography: Geography,
    tags: Vec<SiteTag>,
    military: MilitaryLayer,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AddedRoute {
    id: RouteId,
    from: SiteId,
    to: SiteId,
    terrain_cost: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegionalPath {
    route: RouteId,
    waypoints: Vec<[f32; 2]>,
    bridges: Vec<[f32; 2]>,
}

impl RegionalGeography {
    pub(super) fn resolve(
        &self,
        source: &WorldLayout,
        topology_revision: u32,
        rules: &CampaignRules,
        economy: &Economy,
    ) -> Result<WorldLayout, String> {
        if topology_revision != 2 {
            return Err(format!(
                "{SOURCE}: unsupported authored regional topology revision {topology_revision}"
            ));
        }
        self.validate(source)?;

        let mut authored = source.clone();
        for area in self.areas_for_revision(topology_revision) {
            apply_site_patches(&mut authored, area)?;
            for addition in &area.routes {
                authored.routes.push(Route {
                    id: addition.id,
                    from: addition.from,
                    to: addition.to,
                    major_connection: None,
                    terrain_cost: addition.terrain_cost,
                    road: Road {
                        improved: false,
                        damage: 0,
                    },
                });
            }
        }
        authored.validate(rules, economy)?;
        for area in self.areas_for_revision(topology_revision) {
            for path in &area.paths {
                if authored.atlas_paths.contains_key(&path.route) {
                    return Err(format!(
                        "{SOURCE}: route {:?} already has authored atlas geometry",
                        path.route
                    ));
                }
                authored.atlas_paths.insert(
                    path.route,
                    AtlasPath {
                        waypoints: path.waypoints.clone(),
                        bridges: path.bridges.clone(),
                    },
                );
            }
            authored
                .terrain_features
                .extend(area.terrain_features.iter().cloned());
        }
        authored.flat_country_topology_for_revision(economy, 5)
    }

    fn areas_for_revision(&self, revision: u32) -> impl Iterator<Item = &RegionalArea> {
        self.areas
            .iter()
            .filter(move |area| area.introduced_revision <= revision)
    }

    fn validate(&self, source: &WorldLayout) -> Result<(), String> {
        if self.schema_version != 1 {
            return Err(format!("{SOURCE}: unsupported schema_version"));
        }
        let expected = BTreeSet::from([MarkerId(73), MarkerId(74), MarkerId(75)]);
        let mut keys = BTreeSet::new();
        let mut markers = BTreeSet::new();
        let mut route_ids = BTreeSet::new();
        let mut route_pairs = BTreeSet::new();
        let mut feature_keys = BTreeSet::new();
        let max_route_id = source
            .routes
            .iter()
            .map(|route| route.id.0)
            .max()
            .unwrap_or(0);
        for area in &self.areas {
            if area.introduced_revision != 2
                || !keys.insert(area.key.as_str())
                || !markers.insert(area.marker)
                || area.key.trim().is_empty()
            {
                return Err(format!(
                    "{SOURCE}: invalid or duplicate regional area identity"
                ));
            }
            let region_sites = match source.marker(area.marker).map(|marker| &marker.location) {
                Some(MarkerLocation::Region { sites, .. }) => sites,
                _ => {
                    return Err(format!(
                        "{SOURCE}: area marker {:?} is not a region",
                        area.marker
                    ));
                }
            };
            if !region_sites.contains(&area.headquarters_site)
                || !source
                    .headquarters_candidates
                    .contains(&area.headquarters_site)
            {
                return Err(format!(
                    "{SOURCE}: area {:?} must use its reserved headquarters site",
                    area.key
                ));
            }
            let patch_ids: BTreeSet<_> = area.sites.iter().map(|site| site.site).collect();
            if area.sites.len() != region_sites.len()
                || patch_ids.len() != area.sites.len()
                || patch_ids.iter().copied().collect::<Vec<_>>() != {
                    let mut expected_sites = region_sites.clone();
                    expected_sites.sort();
                    expected_sites
                }
            {
                return Err(format!(
                    "{SOURCE}: area {:?} must patch each physical site once",
                    area.key
                ));
            }
            for patch in &area.sites {
                if !normalized(patch.position) || !unique(&patch.tags) {
                    return Err(format!("{SOURCE}: invalid site patch for {:?}", patch.site));
                }
                let original = source
                    .site(patch.site)
                    .ok_or_else(|| format!("{SOURCE}: missing patched site {:?}", patch.site))?;
                if original.marker != area.marker {
                    return Err(format!(
                        "{SOURCE}: patched site {:?} is outside its area",
                        patch.site
                    ));
                }
                if patch.site == area.headquarters_site
                    && !patch.tags.contains(&SiteTag::HorseAccess)
                {
                    return Err(format!("{SOURCE}: reserved start lost horse access"));
                }
            }
            for route in &area.routes {
                let pair = ordered_pair(route.from, route.to);
                if route.id.0 <= max_route_id
                    || !route_ids.insert(route.id)
                    || route.from == route.to
                    || route.terrain_cost < 2
                    || route.terrain_cost > 4
                    || !region_sites.contains(&route.from)
                    || !region_sites.contains(&route.to)
                    || source
                        .routes
                        .iter()
                        .any(|existing| ordered_pair(existing.from, existing.to) == pair)
                    || !route_pairs.insert(pair)
                {
                    return Err(format!(
                        "{SOURCE}: invalid or duplicate added route {:?}",
                        route.id
                    ));
                }
            }
            let area_route_ids: BTreeSet<_> = source
                .routes
                .iter()
                .filter(|route| {
                    region_sites.contains(&route.from) && region_sites.contains(&route.to)
                })
                .map(|route| route.id)
                .chain(area.routes.iter().map(|route| route.id))
                .collect();
            let mut path_ids = BTreeSet::new();
            for path in &area.paths {
                if !area_route_ids.contains(&path.route)
                    || !path_ids.insert(path.route)
                    || path.waypoints.len() > 8
                    || path.bridges.len() > 4
                    || (path.waypoints.is_empty() && path.bridges.is_empty())
                    || path
                        .waypoints
                        .iter()
                        .chain(&path.bridges)
                        .any(|point| !normalized(*point))
                {
                    return Err(format!("{SOURCE}: invalid path for route {:?}", path.route));
                }
            }
            for feature in &area.terrain_features {
                if !feature.key.starts_with(&format!("{}_", area.key))
                    || !feature_keys.insert(feature.key.as_str())
                    || !region_sites.contains(&feature.anchor)
                    || feature.points.len() < 2
                    || feature.points.iter().any(|point| !normalized(*point))
                    || !feature.width.is_finite()
                    || !(0.005..=0.4).contains(&feature.width)
                {
                    return Err(format!(
                        "{SOURCE}: invalid terrain feature {:?}",
                        feature.key
                    ));
                }
            }
            self.validate_river_crossings(source, area)?;
        }
        if markers != expected {
            return Err(format!(
                "{SOURCE}: topology revision 2 requires Northwatch, Alder Vale and Westmere"
            ));
        }
        Ok(())
    }

    fn validate_river_crossings(
        &self,
        source: &WorldLayout,
        area: &RegionalArea,
    ) -> Result<(), String> {
        let positions: BTreeMap<_, _> = area
            .sites
            .iter()
            .map(|site| (site.site, site.position))
            .collect();
        let routes = source
            .routes
            .iter()
            .filter(|route| {
                positions.contains_key(&route.from) && positions.contains_key(&route.to)
            })
            .map(|route| (route.id, route.from, route.to, false))
            .chain(
                area.routes
                    .iter()
                    .map(|route| (route.id, route.from, route.to, true)),
            );
        for (route_id, from_site, to_site, added) in routes {
            let path = area.paths.iter().find(|path| path.route == route_id);
            if added && path.is_none() {
                return Err(format!(
                    "{SOURCE}: added route {:?} requires authored path geometry",
                    route_id
                ));
            }
            let from = *positions
                .get(&from_site)
                .ok_or_else(|| format!("{SOURCE}: path origin {:?} is missing", from_site))?;
            let to = *positions
                .get(&to_site)
                .ok_or_else(|| format!("{SOURCE}: path destination {:?} is missing", to_site))?;
            let road = std::iter::once(from)
                .chain(
                    path.into_iter()
                        .flat_map(|path| path.waypoints.iter().copied()),
                )
                .chain(std::iter::once(to))
                .collect::<Vec<_>>();
            if let Some(path) = path {
                for bridge in &path.bridges {
                    if point_polyline_distance(*bridge, &road) > 0.006
                        || !area.terrain_features.iter().any(|river| {
                            river.kind == TerrainFeatureKind::River
                                && point_polyline_distance(*bridge, &river.points)
                                    <= river.width * 0.5 + 0.006
                        })
                    {
                        return Err(format!(
                            "{SOURCE}: route {:?} bridge must lie on its road and a river",
                            route_id
                        ));
                    }
                }
            }
            for river in area
                .terrain_features
                .iter()
                .filter(|feature| feature.kind == TerrainFeatureKind::River)
            {
                for road_segment in road.windows(2) {
                    for river_segment in river.points.windows(2) {
                        if segment_distance(
                            road_segment[0],
                            road_segment[1],
                            river_segment[0],
                            river_segment[1],
                        ) <= river.width * 0.5
                            && !path.is_some_and(|path| {
                                path.bridges.iter().any(|bridge| {
                                    point_segment_distance(
                                        *bridge,
                                        road_segment[0],
                                        road_segment[1],
                                    ) <= 0.006
                                        && point_segment_distance(
                                            *bridge,
                                            river_segment[0],
                                            river_segment[1],
                                        ) <= river.width * 0.5 + 0.006
                                })
                            })
                        {
                            return Err(format!(
                                "{SOURCE}: road route {:?} crosses river {:?} without an on-road bridge",
                                route_id, river.key
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn apply_site_patches(layout: &mut WorldLayout, area: &RegionalArea) -> Result<(), String> {
    for patch in &area.sites {
        let site = layout
            .sites
            .iter_mut()
            .find(|site| site.id == patch.site)
            .ok_or_else(|| format!("{SOURCE}: missing site {:?}", patch.site))?;
        site.position = patch.position;
        site.geography = patch.geography;
        site.tags = patch.tags.clone();
        site.military = patch.military;
    }
    Ok(())
}

fn normalized(point: [f32; 2]) -> bool {
    point
        .into_iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
}

fn unique<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn ordered_pair(left: SiteId, right: SiteId) -> (SiteId, SiteId) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn point_segment_distance(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let delta = [b[0] - a[0], b[1] - a[1]];
    let length = delta[0] * delta[0] + delta[1] * delta[1];
    if length == 0.0 {
        return (point[0] - a[0]).hypot(point[1] - a[1]);
    }
    let t =
        (((point[0] - a[0]) * delta[0] + (point[1] - a[1]) * delta[1]) / length).clamp(0.0, 1.0);
    (point[0] - a[0] - t * delta[0]).hypot(point[1] - a[1] - t * delta[1])
}

fn point_polyline_distance(point: [f32; 2], line: &[[f32; 2]]) -> f32 {
    line.windows(2)
        .map(|segment| point_segment_distance(point, segment[0], segment[1]))
        .fold(f32::INFINITY, f32::min)
}

fn segment_distance(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> f32 {
    if segments_intersect(a, b, c, d) {
        return 0.0;
    }
    point_segment_distance(a, c, d)
        .min(point_segment_distance(b, c, d))
        .min(point_segment_distance(c, a, b))
        .min(point_segment_distance(d, a, b))
}

fn segments_intersect(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> bool {
    fn orientation(a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> f32 {
        (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
    }
    fn on_segment(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> bool {
        point[0] >= a[0].min(b[0]) - f32::EPSILON
            && point[0] <= a[0].max(b[0]) + f32::EPSILON
            && point[1] >= a[1].min(b[1]) - f32::EPSILON
            && point[1] <= a[1].max(b[1]) + f32::EPSILON
    }
    let ab_c = orientation(a, b, c);
    let ab_d = orientation(a, b, d);
    let cd_a = orientation(c, d, a);
    let cd_b = orientation(c, d, b);
    let opposite = |left: f32, right: f32| {
        (left > f32::EPSILON && right < -f32::EPSILON)
            || (left < -f32::EPSILON && right > f32::EPSILON)
    };
    (opposite(ab_c, ab_d) && opposite(cd_a, cd_b))
        || (ab_c.abs() <= f32::EPSILON && on_segment(c, a, b))
        || (ab_d.abs() <= f32::EPSILON && on_segment(d, a, b))
        || (cd_a.abs() <= f32::EPSILON && on_segment(a, c, d))
        || (cd_b.abs() <= f32::EPSILON && on_segment(b, c, d))
}
