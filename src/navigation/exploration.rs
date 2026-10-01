//! Terrain discovery and non-interactive road hints on the irregular atlas.

use super::{MapScope, MapView};
use crate::state::world::CampaignWorld;
use macroquad::prelude::Vec2;

/// Presentation geometry only: hidden sites never become selection or move targets.
pub struct MapExploration {
    revealed: Vec<Vec2>,
    hidden: Vec<Vec2>,
    softness: f32,
    pub connection_hints: Vec<Vec<Vec2>>,
}

impl MapExploration {
    pub fn new(
        world: &CampaignWorld,
        visible: &CampaignWorld,
        scope: MapScope,
        view: &MapView,
    ) -> Self {
        let mut exploration = Self {
            revealed: Vec::new(),
            hidden: Vec::new(),
            softness: 56.0 * view.extent().x / 1280.0,
            connection_hints: Vec::new(),
        };
        let positions: Vec<_> = match scope {
            MapScope::World => world
                .markers
                .iter()
                .map(|marker| (marker.position, visible.marker(marker.id).is_some()))
                .collect(),
            MapScope::Region(region) => world
                .sites
                .iter()
                .filter(|site| site.marker == region)
                .map(|site| (site.position, visible.site(site.id).is_some()))
                .collect(),
        };
        for (position, revealed) in positions {
            let points = if revealed {
                &mut exploration.revealed
            } else {
                &mut exploration.hidden
            };
            points.push(view.normalized_world(position));
        }
        exploration.connection_hints = connection_hints(world, visible, scope, view);
        exploration
    }

    /// Each location reveals its surrounding terrain up to the unknown frontier,
    /// including empty land toward map edges. Distances use atlas pixels so pan,
    /// zoom and viewport size cannot change what has been discovered.
    pub fn opacity(&self, point: Vec2) -> f32 {
        if self.revealed.is_empty() {
            return 1.0;
        }
        if self.hidden.is_empty() {
            return 0.0;
        }
        let nearest = |centers: &[Vec2]| {
            centers
                .iter()
                .map(|center| center.distance_squared(point))
                .fold(f32::INFINITY, f32::min)
                .sqrt()
        };
        let blend = ((nearest(&self.revealed) - nearest(&self.hidden)) / self.softness + 0.5)
            .clamp(0.0, 1.0);
        blend * blend * (3.0 - 2.0 * blend)
    }
}

fn connection_hints(
    world: &CampaignWorld,
    visible: &CampaignWorld,
    scope: MapScope,
    view: &MapView,
) -> Vec<Vec<Vec2>> {
    let mut hints = Vec::new();
    for route in &world.routes {
        if visible.routes.iter().any(|known| known.id == route.id) {
            continue;
        }
        let endpoints = match scope {
            MapScope::World => route.major_connection.and_then(|[from, to]| {
                world
                    .marker(from)
                    .zip(world.marker(to))
                    .filter(|_| visible.marker(from).is_some() || visible.marker(to).is_some())
                    .map(|(a, b)| (a.position, b.position))
            }),
            MapScope::Region(region) => world
                .site(route.from)
                .zip(world.site(route.to))
                .filter(|(a, b)| a.marker == region && b.marker == region)
                .filter(|_| visible.site(route.from).is_some() || visible.site(route.to).is_some())
                .map(|(a, b)| (a.position, b.position)),
        };
        let Some((from, to)) = endpoints else {
            continue;
        };
        let geometry = (scope == MapScope::World)
            .then(|| world.atlas_paths.get(&route.id))
            .flatten();
        hints.push(
            std::iter::once(from)
                .chain(
                    geometry
                        .into_iter()
                        .flat_map(|path| path.waypoints.iter().copied()),
                )
                .chain(std::iter::once(to))
                .map(|point| view.normalized_world(point))
                .collect(),
        );
    }
    hints
}
