//! Strategic command pacing and stable save boundaries for the application.

use super::*;

pub use kestrum::state::STRATEGIC_SAVE_SLOT as STRATEGIC_SLOT;

impl Game {
    pub(super) fn capture_campaign(&mut self) {
        if let Err(error) = self.state.new_game(&self.data) {
            self.error = Some(error);
        }
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            campaign.tutorial.dismiss();
        }
    }

    pub(super) fn start_game(&mut self) {
        let result = (|| {
            let mut campaign = kestrum::state::StrategicCampaign::new_production(
                &self.data,
                &self.setup.campaign_setup(),
            )?;
            if self.capture {
                campaign.tutorial.dismiss();
            }
            if !self.capture {
                let library = self
                    .library
                    .as_mut()
                    .ok_or_else(|| self.saves.status.clone())?;
                let store = self
                    .storage
                    .as_mut()
                    .ok_or_else(|| self.saves.status.clone())?;
                library.refresh(store, &self.data)?;
                campaign.campaign_id = library.allocate_campaign_id(store)?;
            }
            self.state
                .load_campaign(Campaign::Strategic(Box::new(campaign)), &self.data)
        })();
        match result {
            Ok(()) => {
                self.navigation.reset(&mut self.view);
                self.army = ui::ArmyView::default();
                self.movement = ui::MoveView::default();
                self.focus_home();
                self.battle = ui::BattleView::default();
                self.reset_history();
                self.kingdom = ui::KingdomView::default();
                self.diplomacy_seen.clear();
                self.ending_saved = false;
                self.invalidate_projection();
                self.error = None;
                self.npc_delay = 0.0;
                self.save_checkpoint();
            }
            Err(error) => {
                self.error = Some(error);
                self.open_saves(false);
            }
        }
    }

    pub(super) fn apply_campaign_command(&mut self, command: Command) {
        let result = self.state.command(&self.data, command);
        self.handle_campaign_result(result);
        self.npc_delay = 0.0;
    }

    pub(super) fn handle_campaign_result(
        &mut self,
        result: Result<engine::ActionOutcome, engine::RuleError>,
    ) {
        match result {
            Ok(outcome) => {
                let messages = self
                    .state
                    .campaign
                    .as_ref()
                    .and_then(Campaign::strategic)
                    .map(|campaign| {
                        engine::action_notices(campaign, &self.data, campaign.player, &outcome)
                    })
                    .unwrap_or_default();
                if !messages.is_empty() {
                    self.notice = Some((messages.join(" "), 5.0));
                }
                if outcome.battle_pending {
                    self.battle_return = None;
                    self.refresh_pending_battlefield();
                }
                if outcome.round_completed
                    && self
                        .state
                        .campaign
                        .as_ref()
                        .and_then(Campaign::strategic)
                        .is_some_and(|campaign| campaign.diplomacy.ending.is_none())
                {
                    self.save_checkpoint();
                }
                if outcome
                    .battle
                    .is_some_and(|battle| self.witnessed_battle(battle))
                    && self.state.overlay != Overlay::SaveRecovery
                {
                    self.battle_return = None;
                    self.open_committed_battlefield(outcome.battle.expect("witnessed battle"));
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
                !campaign.diplomacy.is_blocked()
                    && campaign.pending_battle.is_none()
                    && matches!(
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
        if self.npc_delay >= self.data.presentation.npc_action_delay_seconds {
            self.npc_delay = 0.0;
            let result = self.state.advance_npc(&self.data);
            self.handle_campaign_result(result);
        }
    }
}
