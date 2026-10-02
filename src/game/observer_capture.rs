//! Durable Observer review scenes through the real setup and action dispatcher.

use super::*;
use kestrum::navigation::MapScope;

impl Game {
    pub(super) fn capture_observer(&mut self, scene: &str) -> bool {
        if !scene.starts_with("observer_") {
            return false;
        }
        // Keep the widest roster in every scene so the capture also covers density.
        self.setup.factions = self.data.rules.max_factions;
        self.setup.seed = self.data.production_layout.default_seed;
        if scene == "observer_world" {
            self.capture_campaign();
            self.state.main_menu();
        }
        let original = self.state.campaign.clone();
        let open = self.capture_sheet_tap(vec2(480.0, 614.0));
        assert!(matches!(open, UiAction::OpenObserverSetup));
        self.apply(open);
        if scene == "observer_setup" {
            return true;
        }
        let start = self.capture_sheet_tap(vec2(1328.0, 816.0));
        assert!(matches!(start, UiAction::StartObserver));
        self.apply(start);
        assert!(self.is_observer());
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.navigation.scope(), MapScope::World);
        self.capture_observer_playback();
        let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
        let faction = campaign.factions.keys().next_back().copied().unwrap();
        match scene {
            "observer_world" => {
                self.apply(UiAction::MainMenu);
                assert_eq!(
                    self.state.campaign, original,
                    "Observer preserves the human campaign"
                );
                assert_eq!(self.state.screen, Screen::Title);
                self.apply(UiAction::Continue);
                assert!(!self.is_observer());
                assert_eq!(self.state.screen, Screen::Campaign);
                self.state.main_menu();
                self.apply(UiAction::OpenObserverSetup);
                self.apply(UiAction::StartObserver);
                self.apply(UiAction::ToggleObserverPaused);
                self.apply(UiAction::SetObserverSpeed(4));
            }
            "observer_kingdoms" => self.apply(UiAction::OpenObserverKingdoms),
            "observer_region" => {
                self.apply(UiAction::FocusObserverFaction(faction));
                assert!(matches!(self.navigation.scope(), MapScope::Region(_)));
            }
            "observer_developed" => self.capture_observer_developed(),
            "observer_help" => self.apply(UiAction::Open(Overlay::Help)),
            _ => panic!("Unsupported Observer capture scene: {scene}"),
        }
        true
    }

    fn capture_observer_playback(&mut self) {
        let started = self.state.campaign.clone();
        self.capture = false;
        self.progress_observer(1.0);
        self.capture = true;
        assert_ne!(
            self.state.campaign, started,
            "Observer advances automatically"
        );
        self.refresh_projection();
        self.apply(UiAction::Open(Overlay::Menu));
        let suspended = self.state.campaign.clone();
        self.capture = false;
        self.progress_observer(1.0);
        self.capture = true;
        assert_eq!(self.state.campaign, suspended, "Menu suspends simulation");
        self.apply(UiAction::Back);
        let pause_at = vec2(1786.0, 1032.0);
        assert!(self.capture_sheet_pointer(pause_at, None, true).is_none());
        assert!(self
            .capture_sheet_pointer(pause_at, Some(pause_at), false)
            .is_none());
        let pause = self.capture_sheet_tap(pause_at);
        assert!(matches!(pause, UiAction::ToggleObserverPaused));
        self.apply(pause);
        assert!(self.observer.paused);
        let before = self.state.campaign.clone();
        self.capture = false;
        self.progress_observer(10.0);
        self.capture = true;
        assert_eq!(self.state.campaign, before);
        let speed = self.capture_sheet_tap(vec2(1524.0, 1032.0));
        assert!(matches!(speed, UiAction::SetObserverSpeed(4)));
        self.apply(speed);
        assert_eq!(self.observer.speed, 4);
        let step = self.capture_sheet_tap(vec2(1616.0, 1032.0));
        assert!(matches!(step, UiAction::StepObserver));
        self.apply(step);
        assert_ne!(self.state.campaign, before);
        self.refresh_projection();
        let view = self.projection.as_ref().expect("observer map");
        let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
        assert_eq!(view.world.markers.len(), campaign.world.markers.len());
        assert_eq!(view.factions.len(), campaign.factions.len());
        assert!(!view.player_turn);
    }

    fn capture_observer_developed(&mut self) {
        for _ in 0..1500 {
            let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
            if campaign.completed_rounds >= 3 {
                break;
            }
            self.apply(UiAction::StepObserver);
            assert!(self.error.is_none(), "{:?}", self.error);
        }
        let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
        assert!(campaign.completed_rounds >= 3);
        // Inspect the busiest occupied place after every kingdom has taken turns.
        let mut counts = std::collections::BTreeMap::new();
        for army in campaign.armies.values() {
            *counts.entry(army.site).or_insert(0) += 1;
        }
        if let Some((site, _)) = counts.into_iter().max_by_key(|(_, count)| *count) {
            self.navigation
                .focus_site(&campaign.world, site, &mut self.view);
        }
        self.refresh_projection();
    }
}
