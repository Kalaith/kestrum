//! Strategic command pacing and stable save boundaries for the application.

use super::*;
use macroquad_toolkit::persistence::{load_from_slot, save_to_slot_with_version};

pub use kestrum::state::STRATEGIC_SAVE_SLOT as STRATEGIC_SLOT;

impl Game {
    pub(super) fn capture_campaign(&mut self) {
        if let Err(error) = self.state.new_game(&self.data) {
            self.error = Some(error);
        }
    }

    pub(super) fn start_game(&mut self) {
        match self.state.new_game(&self.data) {
            Ok(()) => {
                self.view.reset();
                self.error = None;
                self.npc_delay = 0.0;
                self.save(false);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn apply_campaign_command(&mut self, command: Command) {
        let result = self.state.command(&self.data, command);
        self.handle_campaign_result(result);
        self.npc_delay = 0.0;
    }

    fn handle_campaign_result(&mut self, result: Result<engine::ActionOutcome, engine::RuleError>) {
        match result {
            Ok(outcome) => {
                if outcome.round_completed {
                    self.save(false);
                }
            }
            Err(error) => self.error = Some(error.to_string()),
        }
    }

    pub(super) fn progress_npcs(&mut self, dt: f32) {
        if self.capture
            || self.error.is_some()
            || self.state.screen != Screen::Campaign
            || self.state.overlay != Overlay::None
        {
            self.npc_delay = 0.0;
            return;
        }
        let ready = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .is_some_and(|campaign| {
                matches!(
                    campaign.phase,
                    kestrum::state::campaign::CampaignPhase::NpcTurn { paused: false, .. }
                )
            });
        if !ready {
            self.npc_delay = 0.0;
            return;
        }
        self.npc_delay += dt;
        // Presentation pacing only; each engine command remains an atomic, deterministic step.
        if self.npc_delay >= 1.2 {
            self.npc_delay = 0.0;
            let result = self.state.advance_npc(&self.data);
            self.handle_campaign_result(result);
        }
    }

    pub(super) fn save(&mut self, announce: bool) {
        if self.capture {
            return;
        }
        let Some(campaign) = &self.state.campaign else {
            return;
        };
        let Some(strategic) = campaign.strategic() else {
            self.error = Some(self.data.presentation.text("legacy_read_only").into());
            return;
        };
        if !matches!(
            strategic.phase,
            kestrum::state::campaign::CampaignPhase::PlayerTurn
        ) {
            self.error = Some(self.data.presentation.text("save_player_only").into());
            return;
        }
        match save_to_slot_with_version(
            &self.data.presentation.game_id,
            STRATEGIC_SLOT,
            campaign,
            "2",
        ) {
            Ok(()) => {
                self.save_exists = true;
                self.error = None;
                if announce {
                    self.notice = Some((self.data.presentation.text("save_success").into(), 3.0));
                }
            }
            Err(error) => {
                self.error = Some(format!(
                    "{}: {error}",
                    self.data.presentation.text("save_failed")
                ))
            }
        }
    }

    pub(super) fn load(&mut self) {
        self.load_slot(if self.save_exists {
            STRATEGIC_SLOT
        } else {
            SAVE_SLOT
        });
    }

    pub(super) fn load_slot(&mut self, slot: &str) {
        let loaded: Result<Campaign, String> =
            load_from_slot(&self.data.presentation.game_id, slot);
        match loaded.and_then(|campaign| self.state.load_campaign(campaign, &self.data)) {
            Ok(()) => {
                self.view.reset();
                self.npc_delay = 0.0;
                self.error = None;
                self.notice = Some((self.data.presentation.text("load_success").into(), 3.0));
            }
            Err(error) => {
                self.error = Some(format!(
                    "{}: {error}",
                    self.data.presentation.text("load_failed")
                ))
            }
        }
    }
}
