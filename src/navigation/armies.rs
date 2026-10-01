//! Army banners share the same geometry for drawing and release picking.

use super::*;
use crate::state::military::{Army, ArmyId};

#[derive(Debug, Clone, PartialEq)]
pub struct ArmyTarget {
    pub armies: Vec<ArmyId>,
    pub bounds: Rect,
}

impl MapNavigation {
    /// Recenter an army in the current map; only explicit entry opens a region.
    pub fn focus_army_site(&mut self, world: &CampaignWorld, site: SiteId, view: &mut MapView) {
        let Some(place) = world.site(site) else {
            return;
        };
        if self.scope() == MapScope::Region(place.marker) {
            view.focus(place.position, 1.5);
            self.selection = Some(MapSelection::Site(site));
        } else {
            self.show_world(view);
            if let Some(marker) = world.marker(place.marker) {
                view.focus(marker.position, 2.5);
                self.selection = Some(MapSelection::Marker(place.marker));
            }
        }
    }

    /// A world banner includes every army in its region, even at different sites.
    pub fn armies_at_selection(
        &self,
        world: &CampaignWorld,
        site: SiteId,
        armies: &[Army],
    ) -> Vec<ArmyId> {
        let marker = world.site(site).map(|site| site.marker);
        let mut ids: Vec<_> = armies
            .iter()
            .filter(|army| match self.scope() {
                MapScope::World => marker.is_some_and(|marker| {
                    world
                        .site(army.site)
                        .is_some_and(|site| site.marker == marker)
                }),
                MapScope::Region(_) => army.site == site,
            })
            .map(|army| army.id)
            .collect();
        ids.sort_unstable();
        ids
    }

    /// Resume with all discovered markers in view; the caller supplies the
    /// observer-filtered world so hidden geography cannot affect the camera.
    pub fn frame_discovered(&mut self, world: &CampaignWorld, view: &mut MapView) {
        let Some(first) = world.markers.first() else {
            return;
        };
        let mut minimum = first.position;
        let mut maximum = first.position;
        for marker in &world.markers {
            for axis in 0..2 {
                minimum[axis] = minimum[axis].min(marker.position[axis]);
                maximum[axis] = maximum[axis].max(marker.position[axis]);
            }
        }
        self.show_world(view);
        let margin = 2.0 * MAP_TAP_SIZE / HEIGHT;
        let extent = (maximum[0] - minimum[0]).max(maximum[1] - minimum[1]);
        let zoom = (1.0 / (extent + margin)).min(2.5);
        view.focus(
            [
                (minimum[0] + maximum[0]) / 2.0,
                (minimum[1] + maximum[1]) / 2.0,
            ],
            zoom,
        );
        self.clear_selection();
    }

    pub fn army_targets(
        &self,
        world: &CampaignWorld,
        view: &MapView,
        armies: &[Army],
    ) -> Vec<ArmyTarget> {
        self.targets(world, view)
            .into_iter()
            .filter_map(|target| {
                let mut local: Vec<_> = armies
                    .iter()
                    .filter(|army| match target.selection {
                        MapSelection::Site(id) => army.site == id,
                        MapSelection::Marker(id) => {
                            world.site(army.site).is_some_and(|site| site.marker == id)
                        }
                    })
                    .map(|army| army.id)
                    .collect();
                local.sort_unstable();
                (!local.is_empty()).then_some(ArmyTarget {
                    armies: local,
                    bounds: Rect::new(
                        (target.center.x + 32.0).min(WIDTH - 100.0),
                        target.center.y - 24.0,
                        96.0,
                        MAP_TAP_SIZE,
                    ),
                })
            })
            .collect()
    }

    /// Frame the physical home/army site, opening its region when needed.
    pub fn focus_site(&mut self, world: &CampaignWorld, site: SiteId, view: &mut MapView) {
        let Some(place) = world.site(site) else {
            return;
        };
        self.show_world(view);
        if world.physical_site(place.marker) == Some(site) {
            if let Some(marker) = world.marker(place.marker) {
                view.focus(marker.position, 2.5);
            }
            self.selection = Some(MapSelection::Marker(place.marker));
        } else {
            if let Some(marker) = world.marker(place.marker) {
                view.focus(marker.position, 2.5);
            }
            if self.enter_region(world, place.marker, view).is_ok() {
                view.focus(place.position, 1.5);
                self.selection = Some(MapSelection::Site(site));
            }
        }
    }
}
