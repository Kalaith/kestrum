//! Atlas navigation and destination taps; campaign geography stays authoritative.

use super::*;
use kestrum::{
    data::world::{FactionId, MarkerId, SiteId},
    navigation::MapSelection,
};

impl Game {
    /// Start beside the physical home and its usable local routes.
    pub(super) fn focus_initial_home(&mut self) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let site = campaign.factions[&campaign.player].headquarters;
        self.navigation
            .focus_site(&campaign.world, site, &mut self.view);
        self.navigation.clear_selection();
    }

    pub(super) fn focus_home(&mut self) {
        if self.is_observer() {
            self.navigation.reset(&mut self.view);
            if let Some(view) = &self.projection {
                self.navigation.toggle_overview(&view.world, &mut self.view);
            }
            return;
        }
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let site = self
            .movement
            .site
            .unwrap_or(campaign.factions[&campaign.player].headquarters);
        self.navigation
            .focus_army_site(&campaign.world, site, &mut self.view);
        self.navigation.clear_selection();
    }

    pub(super) fn map_controls_block(&self, point: Vec2) -> bool {
        if !kestrum::navigation::MAP_RECT.contains(point)
            || ui::tutorial_bounds(&self.state).is_some_and(|rect| rect.contains(point))
            || ((self.error.is_some() || self.notice.is_some())
                && !matches!(self.state.overlay, Overlay::Saves | Overlay::SaveRecovery)
                && ui::feedback_bounds(&self.state).contains(point))
        {
            return true;
        }
        if self.is_observer() && ui::observer_controls_contain(point) {
            return true;
        }
        let notifications = self.notification_map_visible();
        let card_open = notifications && self.notifications.is_open;
        if notifications
            && ui::notification_controls_contain(
                point,
                &self.notifications,
                self.notification_projection.as_ref(),
            )
        {
            return true;
        }
        if !self.is_observer()
            && ui::overview_controls_contain(
                point,
                &self.overview_ui,
                card_open
                    || self.navigation.selection().is_some()
                    || self.movement.stage == ui::MoveStage::Map,
                self.overview
                    .as_ref()
                    .map_or(0, |overview| overview.attention.len()),
            )
        {
            return true;
        }
        let world = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .map(|campaign| &campaign.world);
        ui::map_controls_contain(
            point,
            &self.navigation,
            world,
            &self.view,
            &self.movement,
            card_open,
        )
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
        if !self.is_observer() {
            if let engine::AttentionTarget::Army(id) = target {
                self.begin_move(id);
            }
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
        let panel = if self.notification_map_visible() && self.notifications.is_open {
            ui::notification_reserved_rects(
                &self.notifications,
                self.notification_projection.as_ref(),
            )
            .last()
            .copied()
        } else if self.movement.stage == ui::MoveStage::Map {
            Some(ui::movement_panel_bounds(
                &self.movement,
                &self.navigation,
                world,
                &self.view,
            ))
        } else {
            ui::selection_bounds(&self.navigation, world, &self.view)
        };
        if let Some(action) = self.group_focus_action(origin, pointer.position, panel) {
            return Some(action);
        }
        if campaign.player_turn || campaign.observer_mode {
            let attention = if campaign.observer_mode {
                Rect::default()
            } else {
                ui::attention_bounds(
                    &self.overview_ui,
                    panel.is_some(),
                    self.overview
                        .as_ref()
                        .map_or(0, |overview| overview.attention.len()),
                )
            };
            if let Some(target) = self
                .navigation
                .army_targets(world, &self.view, &campaign.armies)
                .into_iter()
                .find(|target| {
                    target.bounds.contains(origin)
                        && target.bounds.contains(pointer.position)
                        && ui::banner_visible(target.bounds, panel, attention)
                        && (!self.notification_map_visible()
                            || !ui::notification_reserved_rects(
                                &self.notifications,
                                self.notification_projection.as_ref(),
                            )
                            .iter()
                            .any(|rect| rect.overlaps(&target.bounds)))
                })
            {
                if let Some(position) = target.focus {
                    return Some(UiAction::FocusMapGroup(position, self.view.working_zoom()));
                }
                if campaign.observer_mode {
                    let id = target.armies.first()?;
                    let army = campaign.armies.iter().find(|army| army.id == *id)?;
                    let selection = match self.navigation.scope() {
                        kestrum::navigation::MapScope::World => {
                            MapSelection::Marker(world.site(army.site)?.marker)
                        }
                        kestrum::navigation::MapScope::Region(_) => MapSelection::Site(army.site),
                    };
                    return Some(UiAction::SelectMap(selection));
                }
                return target.armies.first().copied().map(UiAction::BeginMove);
            }
        }
        self.navigation
            .pick_release(world, &self.view, origin, pointer.position)
            .map(UiAction::SelectMap)
    }

    fn group_focus_action(
        &self,
        origin: Vec2,
        release: Vec2,
        panel: Option<Rect>,
    ) -> Option<UiAction> {
        let world = &self.projection.as_ref()?.world;
        if let Some(group) = self
            .navigation
            .pick_group_release(world, &self.view, origin, release)
            .filter(|group| !panel.is_some_and(|panel| panel.overlaps(&group.bounds())))
        {
            return Some(UiAction::FocusMapGroup(group.focus, group.zoom));
        }
        let selected = self.navigation.selection();
        let groups = self.navigation.place_groups(world, &self.view);
        self.navigation
            .targets(world, &self.view)
            .into_iter()
            .filter(|target| {
                target.bounds().contains(origin)
                    && target.bounds().contains(release)
                    && ui::important_target(self.overview.as_ref(), selected, target.selection)
                    && !panel.is_some_and(|panel| {
                        panel.contains(target.center) && selected != Some(target.selection)
                    })
            })
            .find_map(|target| {
                groups
                    .iter()
                    .find(|group| group.selections.contains(&target.selection))
                    .map(|group| UiAction::FocusMapGroup(group.focus, group.zoom))
            })
    }

    pub(super) fn select_map(&mut self, selection: MapSelection) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        if let Err(error) = self.navigation.select(&campaign.world, selection) {
            self.error = Some(error);
            return;
        }
        self.notifications.clear_card();
        if !self.is_observer() && self.movement.stage == ui::MoveStage::Map {
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
        let visible = match engine::project_map(campaign, campaign.player) {
            Ok(visible) => visible,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        if let Err(error) = self
            .navigation
            .enter_region(&visible.world, region, &mut self.view)
        {
            self.error = Some(error);
            return;
        }
        self.keep_region_army_visible(&visible);
        self.refresh_move_options();
    }

    fn keep_region_army_visible(&mut self, visible: &engine::VisibleCampaign) {
        if self.movement.stage != ui::MoveStage::Map {
            return;
        }
        let Some(site) = self.movement.site.and_then(|id| visible.world.site(id)) else {
            return;
        };
        if self.navigation.scope() != kestrum::navigation::MapScope::Region(site.marker) {
            return;
        }
        let selectable = self
            .navigation
            .army_targets(&visible.world, &self.view, &visible.armies)
            .iter()
            .any(|target| {
                target
                    .armies
                    .iter()
                    .any(|army| self.movement.armies.contains(army))
                    && [
                        vec2(target.bounds.x, target.bounds.y),
                        vec2(target.bounds.right(), target.bounds.y),
                        vec2(target.bounds.x, target.bounds.bottom()),
                        vec2(target.bounds.right(), target.bounds.bottom()),
                    ]
                    .into_iter()
                    .all(|point| !self.map_controls_block(point))
            });
        if !selectable {
            // Region entry already saved the world camera. Only repair a regional
            // view that hides the selected force, retaining useful saved framing.
            self.navigation
                .focus_army_site(&visible.world, site.id, &mut self.view);
            self.navigation.clear_selection();
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
