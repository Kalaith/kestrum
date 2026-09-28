//! Map navigation changes presentation only; campaign geography stays authoritative.

use super::*;
use kestrum::{
    data::world::{FactionId, MarkerId, SiteId},
    navigation::MapSelection,
};

impl Game {
    pub(super) fn focus_home(&mut self) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let site = self
            .movement
            .site
            .unwrap_or(campaign.factions[&campaign.player].headquarters);
        self.navigation
            .focus_site(&campaign.world, site, &mut self.view);
        self.navigation.clear_selection();
    }

    pub(super) fn map_controls_block(&self, point: Vec2) -> bool {
        if !kestrum::navigation::MAP_RECT.contains(point)
            || ui::tutorial_bounds(&self.state).is_some_and(|rect| rect.contains(point))
            || ((self.error.is_some() || self.notice.is_some())
                && !matches!(self.state.overlay, Overlay::Saves | Overlay::SaveRecovery)
                && ui::FEEDBACK.contains(point))
        {
            return true;
        }
        let world = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .map(|campaign| &campaign.world);
        ui::map_controls_contain(point, &self.navigation, world, &self.view, &self.movement)
    }

    pub(super) fn map_selection_action(&self, pointer: Pointer) -> Option<UiAction> {
        if !pointer.released
            || !self.map_gesture
            || self.state.screen != Screen::Campaign
            || self.state.overlay != Overlay::None
            || self.map_controls_block(pointer.position)
        {
            return None;
        }
        let campaign = self.projection.as_ref()?;
        let world = &campaign.world;
        let origin = self.origin?;
        if origin.distance(pointer.position) > macroquad_toolkit::input::gestures::DRAG_THRESHOLD {
            return None;
        }
        if campaign.player_turn {
            if let Some(target) = self
                .navigation
                .army_targets(world, &self.view, &campaign.armies)
                .into_iter()
                .find(|target| {
                    target.bounds.contains(origin)
                        && target.bounds.contains(pointer.position)
                        && !(self.movement.stage == ui::MoveStage::Map
                            && ui::movement_panel_bounds(
                                &self.movement,
                                &self.navigation,
                                world,
                                &self.view,
                            )
                            .overlaps(&target.bounds))
                })
            {
                return target.armies.first().copied().map(UiAction::BeginMove);
            }
        }
        self.navigation
            .pick_release(world, &self.view, origin, pointer.position)
            .map(UiAction::SelectMap)
    }

    pub(super) fn select_map(&mut self, selection: MapSelection) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        if let Err(error) = self.navigation.select(&campaign.world, selection) {
            self.error = Some(error);
            return;
        }
        if self.movement.stage == ui::MoveStage::Map {
            let site = match selection {
                MapSelection::Site(id) => Some(id),
                MapSelection::Marker(id) => campaign.world.physical_site(id),
            };
            if let Some(site) = site {
                self.select_move_destination(site);
            } else {
                self.movement.destination = None;
                self.movement.preview = None;
                self.movement.status.clear();
            }
        }
    }

    pub(super) fn enter_region(&mut self, region: MarkerId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        if let Err(error) = self
            .navigation
            .enter_region(&campaign.world, region, &mut self.view)
        {
            self.error = Some(error);
        }
    }

    pub(super) fn capture_world(&mut self, scene: &str) {
        self.capture_campaign();
        let region = MarkerId(5);
        if scene.starts_with("headquarters") {
            self.select_map(MapSelection::Marker(MarkerId(1)));
            return;
        }
        if scene.starts_with("world_selected") {
            self.select_map(MapSelection::Marker(region));
            return;
        }
        self.enter_region(region);
        if scene.starts_with("region_partial") || scene.starts_with("region_contested") {
            if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                // A real anchor configuration with a surviving hostile pocket.
                for site in [5, 6, 8, 9, 10] {
                    campaign
                        .set_site_control(&self.data, SiteId(site), Some(campaign.player), false)
                        .expect("capture uses valid authored sites");
                }
                campaign
                    .set_site_control(&self.data, SiteId(7), Some(FactionId(3)), false)
                    .expect("capture hostile pocket is a valid site");
                if scene.starts_with("region_contested") {
                    campaign
                        .set_site_control(&self.data, SiteId(9), Some(campaign.player), true)
                        .expect("capture contested anchor is a valid site");
                }
            }
            self.select_map(MapSelection::Site(SiteId(9)));
        } else if scene.starts_with("region_selected") {
            self.select_map(MapSelection::Site(SiteId(11)));
        } else if scene.starts_with("region_long_name") {
            if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                if let Some(site) = campaign
                    .world
                    .sites
                    .iter_mut()
                    .find(|site| site.id == SiteId(10))
                {
                    site.name = "The Riverward Settlement of Silver Hawthorns".into();
                }
            }
            self.select_map(MapSelection::Site(SiteId(10)));
        }
    }
}
