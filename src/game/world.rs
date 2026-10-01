//! Atlas navigation and destination taps; campaign geography stays authoritative.

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
        if self.movement.site.is_some() {
            self.navigation
                .focus_army_site(&campaign.world, site, &mut self.view);
        } else {
            self.navigation
                .focus_site(&campaign.world, site, &mut self.view);
        }
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
        if ui::overview_controls_contain(
            point,
            &self.overview_ui,
            self.navigation.selection().is_some() || self.movement.stage == ui::MoveStage::Map,
            self.overview
                .as_ref()
                .map_or(0, |overview| overview.attention.len()),
        ) {
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

    /// Warning navigation uses only observer-approved objects and never issues travel.
    pub(super) fn focus_attention(&mut self, target: engine::AttentionTarget) {
        let Some(view) = &self.projection else {
            return;
        };
        let site = match target {
            engine::AttentionTarget::Site(id) => id,
            engine::AttentionTarget::Army(id) => {
                let Some(army) = view.armies.iter().find(|army| army.id == id) else {
                    return;
                };
                army.site
            }
        };
        if view.world.site(site).is_none() {
            return;
        }
        self.movement = ui::MoveView::default();
        self.state.overlay = Overlay::None;
        self.navigation
            .focus_site(&view.world, site, &mut self.view);
        if let engine::AttentionTarget::Army(id) = target {
            self.begin_move(id);
        }
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
            let panel = if self.movement.stage == ui::MoveStage::Map {
                Some(ui::movement_panel_bounds(
                    &self.movement,
                    &self.navigation,
                    world,
                    &self.view,
                ))
            } else {
                ui::selection_bounds(&self.navigation, world, &self.view)
            };
            let attention = ui::attention_bounds(
                &self.overview_ui,
                panel.is_some(),
                self.overview
                    .as_ref()
                    .map_or(0, |overview| overview.attention.len()),
            );
            if let Some(target) = self
                .navigation
                .army_targets(world, &self.view, &campaign.armies)
                .into_iter()
                .find(|target| {
                    target.bounds.contains(origin)
                        && target.bounds.contains(pointer.position)
                        && ui::banner_visible(target.bounds, panel, attention)
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
            match selection {
                MapSelection::Site(id) => self.select_move_destination(id),
                MapSelection::Marker(id) => self.select_world_destination(id),
            }
            if self
                .movement
                .preview
                .as_ref()
                .is_some_and(engine::MovementPreview::can_confirm)
            {
                self.confirm_move();
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
        self.refresh_move_options();
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
