//! Bounded atlas navigation and stable selection in shared logical pixels.

mod armies;
pub use armies::{ArmyGrouping, ArmyTarget};
mod view;
pub use view::{MapScaleBand, MapView};
mod groups;
pub use groups::MapGroup;
mod exploration;
pub use exploration::MapExploration;
mod labels;
pub use labels::{place_map_labels, MapLabelCandidate, PlacedMapLabel};

use crate::{
    data::world::{MarkerId, MarkerLocation, SiteId},
    state::world::CampaignWorld,
};
use macroquad::prelude::{Rect, Vec2};

use macroquad_toolkit::input::gestures::DRAG_THRESHOLD;
use std::collections::BTreeMap;

pub const WIDTH: f32 = 1920.0;
pub const HEIGHT: f32 = 1080.0;
pub const MAP_RECT: Rect = Rect::new(0.0, 0.0, WIDTH, HEIGHT);

pub const MAP_TAP_SIZE: f32 = 48.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MapScope {
    #[default]
    World,
    Region(MarkerId),
}

/// A world marker is always a marker, including one that contains physical sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapSelection {
    Marker(MarkerId),
    Site(SiteId),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapTarget {
    pub selection: MapSelection,
    /// Logical screen position after the camera transform, also used for drawing.
    pub center: Vec2,
}

impl MapTarget {
    /// Zoom changes geography spacing, while tap targets retain their usable size.
    pub fn bounds(&self) -> Rect {
        Rect::new(
            self.center.x - MAP_TAP_SIZE * 0.5,
            self.center.y - MAP_TAP_SIZE * 0.5,
            MAP_TAP_SIZE,
            MAP_TAP_SIZE,
        )
    }
}

/// Transient presentation state: scope changes never move a campaign entity.
#[derive(Debug, Clone, Default)]
pub struct MapNavigation {
    scope: MapScope,
    selection: Option<MapSelection>,
    world_view: MapView,
    region_views: BTreeMap<MarkerId, MapView>,
}

impl MapNavigation {
    pub fn configured(settings: &crate::data::MapCameraSettings) -> Self {
        Self {
            world_view: MapView::configured(settings, MapScope::World),
            ..Self::default()
        }
    }

    /// Overview is explicit and restores the working camera independently per scope.
    pub fn toggle_overview(&mut self, world: &CampaignWorld, view: &mut MapView) {
        let positions: Vec<_> = match self.scope {
            MapScope::World => world.markers.iter().map(|marker| marker.position).collect(),
            MapScope::Region(region) => world
                .sites
                .iter()
                .filter(|site| site.marker == region)
                .map(|site| site.position)
                .collect(),
        };
        view.toggle_overview(&positions);
    }

    pub fn scope(&self) -> MapScope {
        self.scope
    }

    pub fn selection(&self) -> Option<MapSelection> {
        self.selection
    }

    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    pub fn reset(&mut self, view: &mut MapView) {
        *self = Self::configured(&view.settings);
        *view = self.world_view;
    }

    /// Invalid or off-scope identities leave the current selection intact.
    pub fn select(&mut self, world: &CampaignWorld, selection: MapSelection) -> Result<(), String> {
        let valid = match (self.scope, selection) {
            (MapScope::World, MapSelection::Marker(id)) => {
                world.markers.iter().any(|marker| marker.id == id)
            }
            (MapScope::Region(region), MapSelection::Site(id)) => {
                self.region_sites(world, region).contains(&id)
                    && world.site(id).is_some_and(|site| site.marker == region)
            }
            _ => false,
        };
        if !valid {
            return Err("That location is not available on the current map.".into());
        }
        self.selection = Some(selection);
        Ok(())
    }

    pub fn enter_region(
        &mut self,
        world: &CampaignWorld,
        id: MarkerId,
        view: &mut MapView,
    ) -> Result<(), String> {
        if self.scope != MapScope::World {
            return Err("Return to the World Map before entering another region.".into());
        }
        let Some(marker) = world.marker(id) else {
            return Err("That world location does not contain a regional map.".into());
        };
        let MarkerLocation::Region {
            sites, entrances, ..
        } = &marker.location
        else {
            return Err("That world location does not contain a regional map.".into());
        };
        self.world_view = *view;
        *view = self.region_views.get(&id).copied().unwrap_or_else(|| {
            let mut initial = MapView::configured(&self.world_view.settings, MapScope::Region(id));
            // The caller supplies known geography. A new region can have only
            // one visible entrance outside the middle of its larger extent.
            if let Some(site) = entrances
                .iter()
                .map(|entry| entry.site)
                .chain(sites.iter().copied())
                .find_map(|site| world.site(site))
            {
                initial.focus(site.position, initial.working_zoom());
            }
            initial
        });
        self.scope = MapScope::Region(id);
        self.selection = None;
        Ok(())
    }

    /// Restore the exact world camera and identify the region just left.
    pub fn show_world(&mut self, view: &mut MapView) {
        if let MapScope::Region(id) = self.scope {
            self.region_views.insert(id, *view);
            *view = self.world_view;
            self.scope = MapScope::World;
            self.selection = Some(MapSelection::Marker(id));
        }
    }

    /// Draw these centers and use `pick` with pointer coordinates converted by
    /// the toolkit viewport. World and region positions never share a scope.
    pub fn targets(&self, world: &CampaignWorld, view: &MapView) -> Vec<MapTarget> {
        let mut targets: Vec<_> = match self.scope {
            MapScope::World => world
                .markers
                .iter()
                .map(|marker| MapTarget {
                    selection: MapSelection::Marker(marker.id),
                    center: view.project_normalized(marker.position),
                })
                .collect(),
            MapScope::Region(region) => self
                .region_sites(world, region)
                .iter()
                .filter_map(|id| world.site(*id))
                .filter(|site| site.marker == region)
                .map(|site| MapTarget {
                    selection: MapSelection::Site(site.id),
                    center: view.project_normalized(site.position),
                })
                .collect(),
        };
        targets.retain(|target| target.center.is_finite() && MAP_RECT.contains(target.center));
        targets.sort_by_key(|target| target.selection);
        targets
    }

    /// Overlapping 48-pixel targets choose the nearest center, then stable ID.
    /// UI occlusion and suppression of drag/pinch releases belong to the caller.
    pub fn pick(
        &self,
        world: &CampaignWorld,
        view: &MapView,
        logical_point: Vec2,
    ) -> Option<MapSelection> {
        if !logical_point.is_finite() || !MAP_RECT.contains(logical_point) {
            return None;
        }
        let grouped = self.place_groups(world, view);
        self.targets(world, view)
            .into_iter()
            .filter(|target| {
                !grouped
                    .iter()
                    .any(|group| group.selections.contains(&target.selection))
            })
            .filter(|target| target.bounds().contains(logical_point))
            .min_by(|a, b| {
                a.center
                    .distance_squared(logical_point)
                    .total_cmp(&b.center.distance_squared(logical_point))
                    .then_with(|| a.selection.cmp(&b.selection))
            })
            .map(|target| target.selection)
    }

    /// Check the whole contact, including movement first reported on release.
    /// The caller still suppresses gestures claimed by a pan or pinch earlier.
    pub fn pick_release(
        &self,
        world: &CampaignWorld,
        view: &MapView,
        origin: Vec2,
        release: Vec2,
    ) -> Option<MapSelection> {
        if !origin.is_finite() || !release.is_finite() || origin.distance(release) > DRAG_THRESHOLD
        {
            return None;
        }
        let pressed = self.pick(world, view, origin)?;
        let released = self.pick(world, view, release)?;
        (pressed == released).then_some(released)
    }

    fn region_sites<'a>(&self, world: &'a CampaignWorld, region: MarkerId) -> &'a [SiteId] {
        world
            .markers
            .iter()
            .find_map(|marker| match &marker.location {
                MarkerLocation::Region { sites, .. } if marker.id == region => {
                    Some(sites.as_slice())
                }
                _ => None,
            })
            .unwrap_or_default()
    }
}
