//! Crowded map locations use one visible focus target until zoom separates them.

use super::*;
use macroquad::prelude::vec2;

#[derive(Debug, Clone, PartialEq)]
pub struct MapGroup {
    pub selections: Vec<MapSelection>,
    pub center: Vec2,
    pub focus: [f32; 2],
    pub zoom: f32,
}

impl MapGroup {
    pub fn bounds(&self) -> Rect {
        Rect::new(
            self.center.x - 24.0,
            self.center.y - 24.0,
            MAP_TAP_SIZE,
            MAP_TAP_SIZE,
        )
    }
}

impl MapNavigation {
    pub fn place_groups(&self, world: &CampaignWorld, view: &MapView) -> Vec<MapGroup> {
        if view.camera.zoom() >= view.zoom_limits().1 {
            return Vec::new();
        }
        let mut clusters: Vec<Vec<MapTarget>> = Vec::new();
        for target in self.targets(world, view) {
            let joined: Vec<_> = clusters
                .iter()
                .enumerate()
                .filter_map(|(index, cluster)| {
                    cluster
                        .iter()
                        .any(|other| other.center.distance(target.center) < MAP_TAP_SIZE)
                        .then_some(index)
                })
                .collect();
            let mut combined = vec![target];
            for index in joined.into_iter().rev() {
                combined.append(&mut clusters.remove(index));
            }
            combined.sort_by_key(|target| target.selection);
            clusters.push(combined);
        }
        clusters
            .into_iter()
            .filter(|cluster| cluster.len() > 1)
            .map(|cluster| {
                let center =
                    cluster.iter().map(|target| target.center).sum::<Vec2>() / cluster.len() as f32;
                let mut separation = MAP_TAP_SIZE;
                for (index, target) in cluster.iter().enumerate() {
                    for other in cluster.iter().skip(index + 1) {
                        separation = separation.min(target.center.distance(other.center));
                    }
                }
                let zoom = (view.camera.zoom() * 64.0 / separation.max(1.0))
                    .max(view.working_zoom())
                    .min(view.zoom_limits().1);
                MapGroup {
                    selections: cluster.into_iter().map(|target| target.selection).collect(),
                    center,
                    focus: (view
                        .camera
                        .screen_to_world(MAP_RECT, center)
                        .unwrap_or(vec2(WIDTH * 0.5, HEIGHT * 0.5))
                        / view.extent())
                    .to_array(),
                    zoom,
                }
            })
            .collect()
    }

    pub fn pick_group_release(
        &self,
        world: &CampaignWorld,
        view: &MapView,
        origin: Vec2,
        release: Vec2,
    ) -> Option<MapGroup> {
        if !origin.is_finite() || !release.is_finite() || origin.distance(release) > DRAG_THRESHOLD
        {
            return None;
        }
        self.place_groups(world, view)
            .into_iter()
            .filter(|group| group.bounds().contains(origin) && group.bounds().contains(release))
            .min_by(|a, b| {
                a.center
                    .distance_squared(release)
                    .total_cmp(&b.center.distance_squared(release))
            })
    }
}
