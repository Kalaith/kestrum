//! Army banners share the same geometry for drawing and release picking.

use super::*;
use crate::state::military::{Army, ArmyId};

#[derive(Debug, Clone, PartialEq)]
pub struct ArmyTarget {
    pub armies: Vec<ArmyId>,
    pub bounds: Rect,
}

impl MapNavigation {
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
