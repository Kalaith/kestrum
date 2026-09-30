//! Siege screens submit the same validated commands as every other caller.

mod refresh;

use super::*;
use kestrum::{
    data::world::SiteId,
    state::siege::{SiegeAction, SiegeOrder},
};
use ui::SiegeMode;

impl Game {
    pub(super) fn open_siege(&mut self, site: SiteId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let Some(view) = engine::siege_view(campaign, &self.data, campaign.player, site) else {
            return;
        };
        if matches!(
            campaign.phase,
            kestrum::state::CampaignPhase::NpcTurn { paused: false, .. }
        ) {
            self.apply_campaign_command(Command::SetNpcPaused(true));
        }
        self.siege = ui::SiegePanel {
            site: Some(site),
            selected: view.own_armies.clone(),
            view: Some(view),
            ..Default::default()
        };
        self.state.overlay = Overlay::Siege;
        self.error = None;
        self.notice = None;
        self.refresh_siege();
    }

    pub(super) fn apply_siege_action(&mut self, action: UiAction) {
        self.siege.status.clear();
        match action {
            UiAction::OpenSiege(site) => {
                self.open_siege(site);
                return;
            }
            UiAction::SiegeMode(mode) => {
                self.siege.mode = mode;
                self.siege.page = 0;
            }
            UiAction::SiegePage(delta) => {
                self.siege.page = self.siege.page.saturating_add_signed(delta as isize);
            }
            UiAction::ToggleSiegeArmy(id) => {
                if self.siege.selected.contains(&id) {
                    self.siege.selected.retain(|army| *army != id);
                } else if self
                    .siege
                    .view
                    .as_ref()
                    .is_some_and(|view| view.own_armies.contains(&id))
                {
                    self.siege.selected.push(id);
                    self.siege.selected.sort_unstable();
                }
            }
            UiAction::ChooseSiegeAction(action) => {
                self.siege.action = Some(action);
                self.siege.destination = None;
                self.siege.page = 0;
                self.siege.mode = if matches!(action, SiegeAction::Withdraw | SiegeAction::Escape) {
                    SiegeMode::Exits
                } else {
                    SiegeMode::Review
                };
            }
            UiAction::SiegeDestination(site) => {
                self.siege.destination = Some(site);
                self.siege.mode = SiegeMode::Review;
            }
            UiAction::ConfirmSiege => self.confirm_siege(),
            UiAction::SiegeBack => {
                self.siege_back();
                return;
            }
            _ => unreachable!("siege action dispatch"),
        }
        self.refresh_siege();
    }

    pub(super) fn siege_order(&self) -> Option<SiegeOrder> {
        Some(SiegeOrder {
            site: self.siege.site?,
            action: self.siege.action?,
            armies: self.siege.selected.clone(),
            destination: self.siege.destination,
        })
    }

    fn confirm_siege(&mut self) {
        let Some(order) = self.siege_order() else {
            return;
        };
        match self.state.command(&self.data, Command::Siege(order)) {
            Ok(outcome) => {
                self.invalidate_projection();
                self.siege.mode = SiegeMode::Forces;
                self.siege.action = None;
                self.siege.destination = None;
                self.siege.page = 0;
                self.siege.status = self.data.presentation.text("siege_order_done").into();
                if outcome.battle_pending {
                    self.open_pending_battlefield();
                }
                if let Some(battle) = outcome.battle {
                    self.open_committed_battlefield(battle);
                }
            }
            Err(error) => self.siege.status = error.to_string(),
        }
    }

    pub(super) fn siege_back(&mut self) {
        self.siege.mode = match self.siege.mode {
            SiegeMode::Forces => {
                self.state.overlay = Overlay::None;
                return;
            }
            SiegeMode::Orders => SiegeMode::Forces,
            SiegeMode::Exits => SiegeMode::Orders,
            SiegeMode::Review if self.siege.destination.is_some() => SiegeMode::Exits,
            SiegeMode::Review => SiegeMode::Orders,
        };
        self.siege.page = 0;
        self.siege.status.clear();
        self.refresh_siege();
    }
}
