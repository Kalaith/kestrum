//! Spatial review scenes exercise the same application actions used in play.

use super::*;
use kestrum::navigation::{MapScope, MapSelection};

impl Game {
    pub(super) fn capture_spatial(&mut self, scene: &str) -> bool {
        if !matches!(scene, "spatial_overview" | "spatial_navigation") {
            return false;
        }
        self.capture_setup_pointer_actions();
        assert!(self.capture_midgame("midgame_map"));
        self.refresh_projection();
        let mut original = self.state.campaign.clone();
        let home = original.as_ref().unwrap().strategic().unwrap();
        let marker = home
            .world
            .site(home.factions[&home.player].headquarters)
            .unwrap()
            .marker;
        assert_eq!(self.navigation.scope(), MapScope::World);
        assert_eq!(self.view.camera.zoom(), self.view.working_zoom());
        self.view.pan(vec2(-280.0, 80.0));
        let working = self.view;
        self.capture_menu_pointer_actions();
        self.apply(UiAction::SelectMap(MapSelection::Marker(marker)));
        self.apply(UiAction::CloseSelection);
        self.apply(UiAction::MapKey);
        self.apply(UiAction::Back);
        assert_eq!(
            self.state.overlay,
            Overlay::None,
            "Map Key returns to the map"
        );
        self.apply(UiAction::Open(Overlay::Menu));
        self.apply(UiAction::Open(Overlay::Help));
        self.apply(UiAction::Back);
        assert_eq!(
            self.state.overlay,
            Overlay::Menu,
            "menu help retains its return path"
        );
        self.apply(UiAction::Back);
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(
            self.view, working,
            "selection and help retain the working view"
        );
        self.apply(UiAction::Overview);
        assert!(self.view.overview_active());
        self.apply(UiAction::Overview);
        assert_eq!(self.view, working, "Overview returns to the exact camera");
        self.apply(UiAction::EnterRegion(marker));
        self.view.pan(vec2(100.0, -60.0));
        let regional = self.view;
        self.apply(UiAction::WorldMap);
        assert_eq!(self.view, working);
        self.apply(UiAction::EnterRegion(marker));
        assert_eq!(
            self.view, regional,
            "regions retain their independent camera"
        );
        self.apply(UiAction::WorldMap);
        // These existing teaching receipts may be earned while exploring.
        if let Some(Campaign::Strategic(campaign)) = &mut original {
            campaign
                .tutorial
                .record(kestrum::state::tutorial::TutorialStep::Region);
            campaign
                .tutorial
                .record(kestrum::state::tutorial::TutorialStep::WorldMap);
        }
        assert_eq!(
            self.state.campaign, original,
            "navigation only records existing tutorial visits"
        );
        self.navigation.clear_selection();
        if scene == "spatial_overview" {
            self.apply(UiAction::Overview);
        } else {
            self.apply(UiAction::Recenter);
            self.apply(UiAction::SelectMap(MapSelection::Marker(marker)));
        }
        self.notice = None;
        true
    }

    fn capture_menu_pointer_actions(&mut self) {
        self.apply(UiAction::Open(Overlay::Menu));
        let settings = self.capture_sheet_tap(vec2(1079.0, 568.0));
        assert!(matches!(settings, UiAction::Open(Overlay::Settings)));
        self.apply(settings);
        let back = vec2(960.0, 752.0);
        assert!(self
            .capture_sheet_pointer(back, Some(back), false)
            .is_none());
        assert!(self.capture_sheet_pointer(back, None, true).is_none());
        assert!(self
            .capture_sheet_pointer(back, Some(vec2(500.0, 752.0)), true)
            .is_none());
        assert!(self
            .capture_sheet_pointer(vec2(640.0, 572.0), Some(vec2(640.0, 572.0)), true)
            .is_none());
        let action = self.capture_sheet_tap(back);
        assert!(matches!(action, UiAction::Back));
        self.apply(action);
        assert_eq!(self.state.overlay, Overlay::Menu);
        let action = self.capture_sheet_tap(back);
        assert!(matches!(action, UiAction::Back));
        self.apply(action);
        assert_eq!(self.state.overlay, Overlay::None);
    }

    fn capture_setup_pointer_actions(&mut self) {
        use macroquad_toolkit::ui::text_entry::keyboard_keys;

        // Capture isolation preserves the caller's setup without touching saves.
        let setup = std::mem::take(&mut self.setup);
        self.apply(UiAction::NewGame);
        assert_eq!(self.state.overlay, Overlay::Setup);
        let action = self.capture_sheet_tap(vec2(1256.0, 377.0));
        assert!(matches!(action, UiAction::OpenSetupName));
        self.apply(action);
        assert!(self.setup.editing_name);
        for (label, expected) in [
            ("Clear", ""),
            ("q", "q"),
            ("ABC", "q"),
            ("R", "qR"),
            ("Backspace", "q"),
            ("123", "q"),
            ("1", "q1"),
        ] {
            let key = keyboard_keys(
                Rect::new(120.0, 264.0, 1048.0, 282.0),
                self.setup.keyboard_page,
            )
            .unwrap()
            .into_iter()
            .find(|key| key.label == label)
            .unwrap();
            let action = self.capture_sheet_tap(key.rect.center() + vec2(320.0, 180.0));
            assert!(matches!(action, UiAction::EditSetupName(edit) if edit == key.action));
            self.apply(action);
            assert_eq!(self.setup.kingdom_name, expected);
        }
        let action = self.capture_sheet_tap(vec2(1378.0, 816.0));
        assert!(matches!(action, UiAction::SetupNameDone));
        self.apply(action);
        assert!(!self.setup.editing_name);
        let action = self.capture_sheet_tap(vec2(555.0, 808.0));
        assert!(matches!(action, UiAction::Back));
        self.apply(action);
        assert_eq!(self.state.overlay, Overlay::None);
        self.setup = setup;
    }

    pub(in crate::game) fn capture_sheet_tap(&self, screen_position: Vec2) -> UiAction {
        self.capture_sheet_pointer(screen_position, Some(screen_position), true)
            .expect("displayed sheet control must accept its screen-coordinate release")
    }

    /// Exercise the real draw/action path, including both viewport and sheet
    /// translation. This injects a pointer frame, not an operating-system event.
    pub(in crate::game) fn capture_sheet_pointer(
        &self,
        screen_position: Vec2,
        screen_origin: Option<Vec2>,
        released: bool,
    ) -> Option<UiAction> {
        assert_eq!(vec2(screen_width(), screen_height()), vec2(WIDTH, HEIGHT));
        let viewport = begin_virtual_ui_frame(WIDTH, HEIGHT);
        let ctx = ui::Context {
            observer: &self.observer,
            overview: self.overview.as_ref(),
            overview_ui: &self.overview_ui,
            kingdom: &self.kingdom,
            settlement: &self.settlement,
            siege: &self.siege,
            threat: &self.threat,
            data: &self.data.presentation,
            battle_tactics: &self.data.battle_tactics,
            economy: &self.data.economy,
            rules: &self.data.rules,
            lifecycle: &self.data.lifecycle,
            household_rules: &self.data.households,
            progression: &self.data.progression,
            history: &self.history,
            army: &self.army,
            movement: &self.movement,
            battle: &self.battle,
            battlefield: &self.battlefield_view,
            pending_battle: self
                .state
                .campaign
                .as_ref()
                .and_then(Campaign::strategic)
                .is_some_and(|campaign| campaign.pending_battle.is_some()),
            battle_resolution: self.battlefield.as_ref(),
            help_page: self.help_page,
            state: &self.state,
            preferences: &self.preferences,
            view: &self.view,
            navigation: &self.navigation,
            assets: &self.assets,
            pointer: Pointer {
                position: viewport.screen_to_ui(screen_position),
                released,
                down: !released,
                hovering: false,
            },
            origin: screen_origin.map(|position| viewport.screen_to_ui(position)),
            save_exists: self.save_exists,
            legacy_save_exists: self.legacy_save_exists,
            import_save_exists: self.import_save_exists,
            saves: &self.saves,
            save_error: &self.save_error,
            setup: &self.setup,
            campaign_view: self.projection.as_ref(),
        };
        ui::prepare_dynamic_text(&ctx, None);
        let action = ui::draw(&ctx);
        end_virtual_ui_frame();
        action
    }
}
