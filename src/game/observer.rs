//! Separate spectator sessions, playback pacing and read-only atlas navigation.

use super::*;
use kestrum::{data::world::FactionId, state::StrategicCampaign};

pub(super) struct CampaignReturn {
    campaign: Option<Campaign>,
    navigation: MapNavigation,
    view: MapView,
}

impl Game {
    pub(super) fn is_observer(&self) -> bool {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .is_some_and(StrategicCampaign::is_observer)
    }

    pub(super) fn apply_observer_action(&mut self, action: UiAction) -> bool {
        match action {
            UiAction::OpenObserverSetup => {
                self.setup.editing_name = false;
                self.state.overlay = Overlay::ObserverSetup;
                self.error = None;
            }
            UiAction::StartObserver => self.start_observer(),
            UiAction::ToggleObserverPaused if self.is_observer() => {
                self.observer.set_paused(!self.observer.paused);
            }
            UiAction::SetObserverSpeed(speed) if self.is_observer() => {
                self.observer.set_speed(speed);
            }
            UiAction::StepObserver if self.is_observer() && self.observer.paused => {
                self.step_observer();
            }
            UiAction::OpenObserverKingdoms if self.is_observer() => {
                self.kingdom.page = 0;
                self.state.overlay = Overlay::ObserverKingdoms;
            }
            UiAction::FocusObserverFaction(faction) if self.is_observer() => {
                self.focus_observer_faction(faction);
            }
            UiAction::MainMenu if self.is_observer() => self.leave_observer(),
            UiAction::ToggleObserverPaused
            | UiAction::SetObserverSpeed(_)
            | UiAction::StepObserver
            | UiAction::OpenObserverKingdoms
            | UiAction::FocusObserverFaction(_) => {}
            _ => return false,
        }
        true
    }

    fn start_observer(&mut self) {
        if self.state.overlay != Overlay::ObserverSetup {
            return;
        }
        // Human naming choices are deliberately absent from Observer setup.
        let setup = kestrum::data::generation::ProductionSetup {
            kingdom_name: ui::SetupView::default().kingdom_name,
            emblem: ui::SetupView::default().emblem,
            factions: self.setup.factions,
            seed: self.setup.seed,
        };
        let result =
            StrategicCampaign::new_production_observer(&self.data, &setup).and_then(|campaign| {
                campaign.validate(&self.data)?;
                Ok(Campaign::Strategic(Box::new(campaign)))
            });
        let campaign = match result {
            Ok(campaign) => campaign,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        if !self.is_observer() {
            self.observer_return = Some(CampaignReturn {
                campaign: self.state.campaign.take(),
                navigation: self.navigation.clone(),
                view: self.view,
            });
        }
        self.state.campaign = Some(campaign);
        self.state.screen = Screen::Campaign;
        self.state.overlay = Overlay::None;
        self.observer = Default::default();
        self.reset_observer_views();
        self.navigation.reset(&mut self.view);
        self.refresh_projection();
        if let Some(view) = &self.projection {
            self.navigation.toggle_overview(&view.world, &mut self.view);
        }
    }

    fn leave_observer(&mut self) {
        self.state.campaign = None;
        if let Some(previous) = self.observer_return.take() {
            self.state.campaign = previous.campaign;
            self.navigation = previous.navigation;
            self.view = previous.view;
        }
        self.observer = Default::default();
        self.reset_observer_views();
        self.state.main_menu();
    }

    fn reset_observer_views(&mut self) {
        self.portraits.request_reset();
        self.reset_notifications();
        self.army = ui::ArmyView::default();
        self.army_refresh_pending = true;
        self.movement = ui::MoveView::default();
        self.battle = ui::BattleView::default();
        self.battlefield = None;
        self.battlefield_view = ui::BattlefieldView::default();
        self.kingdom = ui::KingdomView::default();
        self.overview_ui = ui::OverviewView::default();
        self.battle_return = None;
        self.reset_history();
        self.diplomacy_seen.clear();
        self.ending_saved = false;
        self.npc_delay = 0.0;
        self.notice = None;
        self.error = None;
        self.invalidate_projection();
    }

    pub(super) fn progress_observer(&mut self, dt: f32) {
        if !self.is_observer() {
            return;
        }
        if self.capture
            || self.error.is_some()
            || self.state.screen != Screen::Campaign
            || self.state.overlay != Overlay::None
        {
            self.observer.reset_clock();
            return;
        }
        if self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .is_some_and(StrategicCampaign::observer_finished)
        {
            self.observer.set_paused(true);
            return;
        }
        if self
            .observer
            .due_step(dt, self.data.presentation.npc_action_delay_seconds)
        {
            self.step_observer();
        }
    }

    fn step_observer(&mut self) {
        if self.state.overlay != Overlay::None || self.error.is_some() {
            return;
        }
        if self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .is_some_and(StrategicCampaign::observer_finished)
        {
            self.observer.set_paused(true);
            return;
        }
        self.observer.reset_clock();
        if let Err(error) = self.state.advance_observer(&self.data) {
            self.observer.set_paused(true);
            self.error = Some(error.to_string());
        }
    }

    fn focus_observer_faction(&mut self, faction: FactionId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let Some(kingdom) = campaign.factions.get(&faction) else {
            return;
        };
        self.navigation
            .focus_site(&campaign.world, kingdom.headquarters, &mut self.view);
        self.state.overlay = Overlay::None;
    }
}
