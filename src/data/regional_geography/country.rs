//! Revision-three country-sector geometry and its resolved-map invariants.

use super::super::{
    economy::{Economy, Habitation},
    regional_geography::{AddedRoute, RegionalPath, RegionalSitePatch, SOURCE},
    world::{
        AtlasPath, Geography, MarkerLocation, Road, Route, RouteId, SiteId, SiteTag,
        TerrainFeature, TerrainFeatureKind, WorldLayout,
    },
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SECTOR_KEYS: [&str; 8] = [
    "northwatch",
    "alder_vale",
    "westmere",
    "riverfold",
    "lakeward",
    "rosevale",
    "eastfold",
    "southplain",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CountryGeography {
    introduced_revision: u32,
    sectors: Vec<CountrySector>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CountrySector {
    key: String,
    sites: Vec<RegionalSitePatch>,
    routes: Vec<AddedRoute>,
    paths: Vec<RegionalPath>,
    terrain_features: Vec<TerrainFeature>,
}

impl CountryGeography {
    pub(super) fn validate(
        &self,
        source: &WorldLayout,
        regional_route_ids: &BTreeSet<RouteId>,
        regional_route_pairs: &BTreeSet<(SiteId, SiteId)>,
    ) -> Result<(), String> {
        if self.introduced_revision != 3 || self.sectors.len() != SECTOR_KEYS.len() {
            return Err(format!(
                "{SOURCE}: revision 3 requires all eight country sectors"
            ));
        }
        let mut all_route_ids: BTreeSet<_> = source.routes.iter().map(|route| route.id).collect();
        all_route_ids.extend(regional_route_ids.iter().copied());
        let mut route_pairs: BTreeSet<_> = source
            .routes
            .iter()
            .map(|route| ordered_pair(route.from, route.to))
            .collect();
        route_pairs.extend(regional_route_pairs.iter().copied());
        let minimum_new_route_id = all_route_ids.iter().map(|id| id.0).max().unwrap_or(0) + 1;
        let mut feature_keys = BTreeSet::new();
        let mut all_site_ids = BTreeSet::new();
        let mut all_path_ids = BTreeSet::new();

        for (sector_index, sector) in self.sectors.iter().enumerate() {
            let first_site = sector_index as u32 * 9 + 1;
            let expected_sites: BTreeSet<_> = (first_site..first_site + 9).map(SiteId).collect();
            let patched_sites: BTreeSet<_> = sector.sites.iter().map(|site| site.site).collect();
            if sector.key != SECTOR_KEYS[sector_index]
                || sector.sites.len() != 9
                || patched_sites != expected_sites
                || patched_sites.iter().any(|site| !all_site_ids.insert(*site))
            {
                return Err(format!(
                    "{SOURCE}: invalid site coverage for sector {:?}",
                    sector.key
                ));
            }
            for patch in &sector.sites {
                if !normalized(patch.position) || !unique(&patch.tags) {
                    return Err(format!(
                        "{SOURCE}: invalid country site patch {:?}",
                        patch.site
                    ));
                }
                let source_site = source
                    .site(patch.site)
                    .ok_or_else(|| format!("{SOURCE}: missing country site {:?}", patch.site))?;
                if source.marker(source_site.marker).is_none() {
                    return Err(format!(
                        "{SOURCE}: country site {:?} lost its marker",
                        patch.site
                    ));
                }
            }
            for required in [
                SiteTag::WoodSource,
                SiteTag::HorseAccess,
                SiteTag::StoneSource,
            ] {
                if !sector
                    .sites
                    .iter()
                    .any(|site| site.tags.contains(&required))
                {
                    return Err(format!(
                        "{SOURCE}: sector {:?} lacks {required:?}",
                        sector.key
                    ));
                }
            }
            if sector.routes.len() != 2 {
                return Err(format!(
                    "{SOURCE}: sector {:?} needs two authored branches",
                    sector.key
                ));
            }
            for route in &sector.routes {
                let pair = ordered_pair(route.from, route.to);
                if route.id.0 < minimum_new_route_id
                    || !all_route_ids.insert(route.id)
                    || route.from == route.to
                    || route.terrain_cost < 2
                    || route.terrain_cost > 4
                    || !expected_sites.contains(&route.from)
                    || !expected_sites.contains(&route.to)
                    || !route_pairs.insert(pair)
                {
                    return Err(format!("{SOURCE}: invalid country branch {:?}", route.id));
                }
            }
            let valid_routes: BTreeSet<_> = all_route_ids
                .iter()
                .copied()
                .filter(|id| {
                    source
                        .routes
                        .iter()
                        .find(|route| route.id == *id)
                        .is_some_and(|route| {
                            expected_sites.contains(&route.from)
                                || expected_sites.contains(&route.to)
                        })
                        || sector.routes.iter().any(|route| route.id == *id)
                })
                .collect();
            validate_paths(
                &sector.key,
                &sector.paths,
                &valid_routes,
                &sector.routes,
                &mut all_path_ids,
            )?;
            for feature in &sector.terrain_features {
                if !feature.key.starts_with(&format!("{}_", sector.key))
                    || !feature_keys.insert(feature.key.as_str())
                    || !expected_sites.contains(&feature.anchor)
                    || feature.points.len() < 2
                    || feature.points.iter().any(|point| !normalized(*point))
                    || !feature.width.is_finite()
                    || !(0.005..=0.4).contains(&feature.width)
                {
                    return Err(format!(
                        "{SOURCE}: invalid country terrain {:?}",
                        feature.key
                    ));
                }
            }
            validate_neighborhoods(source, sector)?;
        }
        if all_site_ids.len() != 72 {
            return Err(format!(
                "{SOURCE}: country patch must cover all 72 major sites"
            ));
        }
        let new_route_ids: BTreeSet<_> = self
            .sectors
            .iter()
            .flat_map(|sector| sector.routes.iter().map(|route| route.id))
            .collect();
        if new_route_ids.len() != 16
            || new_route_ids.iter().map(|id| id.0).collect::<Vec<_>>()
                != (207..=222).collect::<Vec<_>>()
        {
            return Err(format!(
                "{SOURCE}: country branch IDs must be 207 through 222"
            ));
        }
        Ok(())
    }

    pub(super) fn apply(&self, layout: &mut WorldLayout) -> Result<(), String> {
        let old_positions: BTreeMap<_, _> = layout
            .sites
            .iter()
            .map(|site| (site.id, site.position))
            .collect();
        for sector in &self.sectors {
            for patch in &sector.sites {
                let site = layout
                    .sites
                    .iter_mut()
                    .find(|site| site.id == patch.site)
                    .ok_or_else(|| format!("{SOURCE}: missing resolved site {:?}", patch.site))?;
                site.position = patch.position;
                site.geography = patch.geography;
                site.tags = patch.tags.clone();
                site.military = patch.military;
                let marker = layout
                    .markers
                    .iter_mut()
                    .find(|marker| marker.id == site.marker)
                    .ok_or_else(|| format!("{SOURCE}: missing site marker {:?}", site.marker))?;
                if !matches!(marker.location, MarkerLocation::Site { site: id } if id == patch.site)
                {
                    return Err(format!(
                        "{SOURCE}: marker {:?} does not match site",
                        marker.id
                    ));
                }
                marker.position = patch.position;
            }
        }
        reanchor_existing_paths(layout, &old_positions)?;
        for sector in &self.sectors {
            for addition in &sector.routes {
                let from = layout.site(addition.from).ok_or_else(|| {
                    format!("{SOURCE}: missing branch origin {:?}", addition.from)
                })?;
                let to = layout.site(addition.to).ok_or_else(|| {
                    format!("{SOURCE}: missing branch destination {:?}", addition.to)
                })?;
                layout.routes.push(Route {
                    id: addition.id,
                    from: addition.from,
                    to: addition.to,
                    major_connection: Some([from.marker, to.marker]),
                    terrain_cost: addition.terrain_cost,
                    road: Road {
                        improved: false,
                        damage: 0,
                    },
                });
            }
            for path in &sector.paths {
                layout.atlas_paths.insert(
                    path.route,
                    AtlasPath {
                        waypoints: path.waypoints.clone(),
                        bridges: path.bridges.clone(),
                    },
                );
            }
            layout
                .terrain_features
                .extend(sector.terrain_features.iter().cloned());
        }
        validate_resolved_river_crossings(layout)
    }

    pub(super) fn validate_resolved_layout(
        &self,
        layout: &WorldLayout,
        economy: &Economy,
    ) -> Result<(), String> {
        layout.validate_country_geometry(economy)?;
        validate_country_semantics(layout)
    }
}

fn validate_paths(
    key: &str,
    paths: &[RegionalPath],
    valid_routes: &BTreeSet<RouteId>,
    additions: &[AddedRoute],
    all_path_ids: &mut BTreeSet<RouteId>,
) -> Result<(), String> {
    let mut path_ids = BTreeSet::new();
    for path in paths {
        if !valid_routes.contains(&path.route)
            || !path_ids.insert(path.route)
            || !all_path_ids.insert(path.route)
            || path.waypoints.len() > 8
            || path.bridges.len() > 4
            || (path.waypoints.is_empty() && path.bridges.is_empty())
            || path
                .waypoints
                .iter()
                .chain(&path.bridges)
                .any(|point| !normalized(*point))
        {
            return Err(format!(
                "{SOURCE}: invalid country path {:?} in {key}",
                path.route
            ));
        }
    }
    if additions.iter().any(|route| !path_ids.contains(&route.id)) {
        return Err(format!(
            "{SOURCE}: each new route in {key} needs authored geometry"
        ));
    }
    Ok(())
}

fn validate_neighborhoods(source: &WorldLayout, sector: &CountrySector) -> Result<(), String> {
    let positions: BTreeMap<_, _> = sector
        .sites
        .iter()
        .map(|site| (site.site, site.position))
        .collect();
    let mut neighbors = BTreeSet::new();
    let mut record_neighbor = |from_site, to_site| {
        let (Some(from), Some(to)) = (positions.get(&from_site), positions.get(&to_site)) else {
            return;
        };
        if distance(*from, *to) <= 0.06 {
            neighbors.insert(from_site);
            neighbors.insert(to_site);
        }
    };
    for route in &source.routes {
        record_neighbor(route.from, route.to);
    }
    for route in &sector.routes {
        record_neighbor(route.from, route.to);
    }
    if neighbors.len() != sector.sites.len() {
        return Err(format!(
            "{SOURCE}: every site in {:?} needs a connected neighbor within 0.06",
            sector.key
        ));
    }
    Ok(())
}

fn reanchor_existing_paths(
    layout: &mut WorldLayout,
    old_positions: &BTreeMap<SiteId, [f32; 2]>,
) -> Result<(), String> {
    let new_positions: BTreeMap<_, _> = layout
        .sites
        .iter()
        .map(|site| (site.id, site.position))
        .collect();
    for (route_id, path) in &mut layout.atlas_paths {
        let route = layout
            .routes
            .iter()
            .find(|route| route.id == *route_id)
            .ok_or_else(|| format!("{SOURCE}: atlas path references a missing route"))?;
        let old_from = *old_positions
            .get(&route.from)
            .ok_or_else(|| format!("{SOURCE}: old route origin disappeared"))?;
        let old_to = *old_positions
            .get(&route.to)
            .ok_or_else(|| format!("{SOURCE}: old route destination disappeared"))?;
        let new_from = *new_positions
            .get(&route.from)
            .ok_or_else(|| format!("{SOURCE}: route origin disappeared"))?;
        let new_to = *new_positions
            .get(&route.to)
            .ok_or_else(|| format!("{SOURCE}: route destination disappeared"))?;
        if old_from == new_from && old_to == new_to {
            continue;
        }
        let transform = |point| reanchor_point(point, old_from, old_to, new_from, new_to);
        path.waypoints = path.waypoints.iter().copied().map(transform).collect();
        path.bridges = path.bridges.iter().copied().map(transform).collect();
    }
    Ok(())
}

fn reanchor_point(
    point: [f32; 2],
    old_from: [f32; 2],
    old_to: [f32; 2],
    new_from: [f32; 2],
    new_to: [f32; 2],
) -> [f32; 2] {
    let axis = [old_to[0] - old_from[0], old_to[1] - old_from[1]];
    let length = axis[0] * axis[0] + axis[1] * axis[1];
    let offset = [new_from[0] - old_from[0], new_from[1] - old_from[1]];
    let end_offset = [new_to[0] - old_to[0], new_to[1] - old_to[1]];
    let along = (((point[0] - old_from[0]) * axis[0] + (point[1] - old_from[1]) * axis[1])
        / length)
        .clamp(0.0, 1.0);
    [
        point[0] + offset[0] * (1.0 - along) + end_offset[0] * along,
        point[1] + offset[1] * (1.0 - along) + end_offset[1] * along,
    ]
}

fn validate_resolved_river_crossings(layout: &WorldLayout) -> Result<(), String> {
    let rivers: Vec<_> = layout
        .terrain_features
        .iter()
        .filter(|feature| feature.kind == TerrainFeatureKind::River)
        .collect();
    for river in rivers {
        for route in &layout.routes {
            let from = layout.site(route.from).unwrap().position;
            let to = layout.site(route.to).unwrap().position;
            let path = layout.atlas_paths.get(&route.id);
            let road: Vec<_> = std::iter::once(from)
                .chain(
                    path.into_iter()
                        .flat_map(|path| path.waypoints.iter().copied()),
                )
                .chain(std::iter::once(to))
                .collect();
            if let Some(path) = path {
                for bridge in &path.bridges {
                    let on_river = point_polyline_distance(*bridge, &river.points)
                        <= river.width * 0.5 + 0.006;
                    if on_river && point_polyline_distance(*bridge, &road) > 0.006 {
                        return Err(format!(
                            "{SOURCE}: route {:?} bridge must sit on its road and a river",
                            route.id
                        ));
                    }
                }
            }
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
                                point_segment_distance(*bridge, road_segment[0], road_segment[1])
                                    <= 0.006
                                    && point_segment_distance(
                                        *bridge,
                                        river_segment[0],
                                        river_segment[1],
                                    ) <= river.width * 0.5 + 0.006
                            })
                        })
                    {
                        return Err(format!(
                            "{SOURCE}: road route {:?} crosses river {:?} without a bridge",
                            route.id, river.key
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_country_semantics(layout: &WorldLayout) -> Result<(), String> {
    if layout.sites.len() != 152 || layout.markers.len() != layout.sites.len() {
        return Err(format!(
            "{SOURCE}: resolved country must contain 152 physical sites"
        ));
    }
    let start = layout
        .sites
        .first()
        .map(|site| site.id)
        .ok_or_else(|| format!("{SOURCE}: resolved country has no sites"))?;
    if layout.reachable_sites(start).len() != layout.sites.len() {
        return Err(format!(
            "{SOURCE}: every resolved country site must be reachable"
        ));
    }
    if layout.headquarters_candidates.len() != 8
        || layout
            .headquarters_candidates
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != layout.headquarters_candidates.len()
    {
        return Err(format!(
            "{SOURCE}: resolved country needs eight unique starts"
        ));
    }
    for candidate in &layout.headquarters_candidates {
        let site = layout
            .site(*candidate)
            .ok_or_else(|| format!("{SOURCE}: missing headquarters site {candidate:?}"))?;
        if !matches!(site.habitation, Habitation::Village)
            || site.controller.is_some()
            || !site.tags.contains(&SiteTag::HorseAccess)
        {
            return Err(format!(
                "{SOURCE}: headquarters {candidate:?} must be a neutral village with horse access"
            ));
        }
        let neighbors: BTreeSet<_> = layout
            .routes
            .iter()
            .filter_map(|route| route.other_endpoint(*candidate))
            .collect();
        if neighbors.len() < 2
            || neighbors.iter().any(|neighbor| {
                layout
                    .site(*neighbor)
                    .is_none_or(|site| site.controller.is_some())
            })
        {
            return Err(format!(
                "{SOURCE}: headquarters {candidate:?} needs two neutral neighboring sites"
            ));
        }
        for resource in [SiteTag::WoodSource, SiteTag::StoneSource] {
            if !layout.sites.iter().any(|site| {
                site.tags.contains(&resource)
                    && country_route_distance(layout, *candidate, site.id)
                        .is_some_and(|distance| distance <= 3)
            }) {
                return Err(format!(
                    "{SOURCE}: headquarters {candidate:?} needs accessible {resource:?}"
                ));
            }
        }
    }
    for (index, left) in layout.headquarters_candidates.iter().enumerate() {
        for right in layout.headquarters_candidates.iter().skip(index + 1) {
            if !country_route_distance(layout, *left, *right).is_some_and(|distance| distance >= 4)
            {
                return Err(format!(
                    "{SOURCE}: starts {left:?} and {right:?} are not sufficiently separated"
                ));
            }
        }
    }
    let geographies: BTreeSet<_> = layout.sites.iter().map(|site| site.geography).collect();
    if ![
        Geography::Valley,
        Geography::Coast,
        Geography::Pass,
        Geography::Plains,
        Geography::Forest,
        Geography::Hill,
        Geography::River,
        Geography::Marsh,
    ]
    .into_iter()
    .all(|geography| geographies.contains(&geography))
    {
        return Err(format!(
            "{SOURCE}: resolved country lacks required terrain coverage"
        ));
    }
    Ok(())
}

fn country_route_distance(layout: &WorldLayout, start: SiteId, target: SiteId) -> Option<usize> {
    let mut seen = BTreeSet::from([start]);
    let mut pending = vec![(start, 0usize)];
    while let Some((site, distance)) = pending.pop() {
        if site == target {
            return Some(distance);
        }
        for next in layout
            .routes
            .iter()
            .filter_map(|route| route.other_endpoint(site))
        {
            if seen.insert(next) {
                pending.push((next, distance + 1));
            }
        }
    }
    None
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

fn distance(left: [f32; 2], right: [f32; 2]) -> f32 {
    (left[0] - right[0]).hypot(left[1] - right[1])
}

fn point_segment_distance(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f32 {
    let delta = [b[0] - a[0], b[1] - a[1]];
    let length = delta[0] * delta[0] + delta[1] * delta[1];
    if length == 0.0 {
        return distance(point, a);
    }
    let t =
        (((point[0] - a[0]) * delta[0] + (point[1] - a[1]) * delta[1]) / length).clamp(0.0, 1.0);
    distance(point, [a[0] + t * delta[0], a[1] + t * delta[1]])
}

fn point_polyline_distance(point: [f32; 2], line: &[[f32; 2]]) -> f32 {
    line.windows(2)
        .map(|segment| point_segment_distance(point, segment[0], segment[1]))
        .fold(f32::INFINITY, f32::min)
}

fn segment_distance(a: [f32; 2], b: [f32; 2], c: [f32; 2], d: [f32; 2]) -> f32 {
    if segments_intersect(a, b, c, d) {
        0.0
    } else {
        point_segment_distance(a, c, d)
            .min(point_segment_distance(b, c, d))
            .min(point_segment_distance(c, a, b))
            .min(point_segment_distance(d, a, b))
    }
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
    let (ab_c, ab_d, cd_a, cd_b) = (
        orientation(a, b, c),
        orientation(a, b, d),
        orientation(c, d, a),
        orientation(c, d, b),
    );
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
