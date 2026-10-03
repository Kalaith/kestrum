//! Compact force targets and explicit place/force focus without travel changes.

use super::*;
use crate::state::military::{Army, ArmyId};
use macroquad::prelude::vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmyGrouping {
    Site,
    Region,
    Nearby,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArmyTarget {
    pub armies: Vec<ArmyId>,
    pub bounds: Rect,
    pub grouping: ArmyGrouping,
    /// A nearby group focuses geography before the player chooses an exact force.
    pub focus: Option<[f32; 2]>,
}

impl MapNavigation {
    pub fn focus_army_site(&mut self, world: &CampaignWorld, site: SiteId, view: &mut MapView) {
        let Some(place) = world.site(site) else {
            return;
        };
        if let MapScope::Region(region) = self.scope() {
            if let Some(position) = world.region_site_position(region, site) {
                view.focus(position, view.working_zoom());
                self.selection = Some(MapSelection::Site(site));
                return;
            }
        }
        if let Some(marker) = world.marker(place.marker) {
            self.show_world(view);
            view.focus(marker.position, view.working_zoom());
            self.selection = Some(MapSelection::Marker(place.marker));
        }
    }

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

    /// Explicit overview framing for capture tools and the Overview action.
    pub fn frame_discovered(&mut self, world: &CampaignWorld, view: &mut MapView) {
        self.show_world(view);
        let positions: Vec<_> = world.markers.iter().map(|marker| marker.position).collect();
        view.frame_positions(&positions);
    }

    pub fn army_targets(
        &self,
        world: &CampaignWorld,
        view: &MapView,
        armies: &[Army],
    ) -> Vec<ArmyTarget> {
        let targets = self.targets(world, view);
        let occupied: Vec<_> = targets
            .iter()
            .map(MapTarget::bounds)
            .chain(self.place_groups(world, view).iter().map(MapGroup::bounds))
            .collect();
        let mut groups: Vec<(Vec<ArmyId>, Vec<SiteId>, Vec2, usize)> = Vec::new();
        for target in &targets {
            let local: Vec<_> = armies
                .iter()
                .filter(|army| match target.selection {
                    MapSelection::Site(id) => army.site == id,
                    MapSelection::Marker(id) => {
                        world.site(army.site).is_some_and(|site| site.marker == id)
                    }
                })
                .collect();
            if local.is_empty() {
                continue;
            }
            let close = (view.band() == MapScaleBand::Overview)
                .then(|| {
                    groups.iter().position(|group| {
                        group.2.distance(target.center) < view.settings.army_group_distance
                    })
                })
                .flatten();
            if let Some(index) = close {
                let group = &mut groups[index];
                group.0.extend(local.iter().map(|army| army.id));
                group.1.extend(local.iter().map(|army| army.site));
                group.2 = (group.2 * group.3 as f32 + target.center) / (group.3 + 1) as f32;
                group.3 += 1;
            } else {
                groups.push((
                    local.iter().map(|army| army.id).collect(),
                    local.iter().map(|army| army.site).collect(),
                    target.center,
                    1,
                ));
            }
        }
        let mut placed = Vec::new();
        groups
            .into_iter()
            .map(|(mut ids, sites, center, places)| {
                ids.sort_unstable();
                let regional = self.scope() == MapScope::World
                    && sites.iter().any(|site| {
                        world
                            .site(*site)
                            .and_then(|place| world.marker(place.marker))
                            .is_some_and(|marker| {
                                matches!(marker.location, MarkerLocation::Region { .. })
                            })
                    });
                let grouping = if places > 1 {
                    ArmyGrouping::Nearby
                } else if regional {
                    ArmyGrouping::Region
                } else {
                    ArmyGrouping::Site
                };
                let focus = (grouping == ArmyGrouping::Nearby).then(|| {
                    (view
                        .camera
                        .screen_to_world(MAP_RECT, center)
                        .unwrap_or(center)
                        / view.extent())
                    .to_array()
                });
                let bounds = compact_bounds(center, &occupied, &placed);
                placed.push(bounds);
                ArmyTarget {
                    armies: ids,
                    bounds,
                    grouping,
                    focus,
                }
            })
            .collect()
    }

    /// Exact site attention may enter a region; ordinary Recenter retains scope.
    pub fn focus_site(&mut self, world: &CampaignWorld, site: SiteId, view: &mut MapView) {
        let Some(place) = world.site(site) else {
            return;
        };
        self.show_world(view);
        if let Some(marker) = world.marker(place.marker) {
            view.focus(marker.position, view.working_zoom());
        }
        if world.physical_site(place.marker) == Some(site) {
            self.selection = Some(MapSelection::Marker(place.marker));
        } else if self.enter_region(world, place.marker, view).is_ok() {
            if let Some(position) = world.region_site_position(place.marker, site) {
                view.focus(position, view.working_zoom());
                self.selection = Some(MapSelection::Site(site));
            }
        }
    }
}

fn compact_bounds(center: Vec2, targets: &[Rect], placed: &[Rect]) -> Rect {
    let options = [
        vec2(50.0, 0.0),
        vec2(-50.0, 0.0),
        vec2(0.0, 58.0),
        vec2(0.0, -58.0),
        vec2(50.0, 58.0),
        vec2(-50.0, 58.0),
    ];
    options
        .into_iter()
        .map(|offset| {
            let at = center + offset - Vec2::splat(MAP_TAP_SIZE * 0.5);
            Rect::new(
                at.x.clamp(12.0, WIDTH - MAP_TAP_SIZE - 12.0),
                at.y.clamp(96.0, HEIGHT - MAP_TAP_SIZE - 136.0),
                MAP_TAP_SIZE,
                MAP_TAP_SIZE,
            )
        })
        .min_by_key(|rect| {
            targets
                .iter()
                .filter(|target| target.overlaps(rect))
                .count()
                * 2
                + placed.iter().filter(|other| other.overlaps(rect)).count() * 4
        })
        .expect("compact force offsets are nonempty")
}
