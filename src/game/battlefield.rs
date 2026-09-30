//! Presentation-only playback controls over an already resolved battle.

use super::*;

impl Game {
    pub(super) fn advance_battlefield(&mut self, delta_seconds: f32) {
        if let Some(resolution) = &self.battlefield {
            self.battlefield_view
                .advance(delta_seconds, resolution.events.len());
        }
    }

    pub(super) fn apply_battlefield_action(&mut self, action: ui::BattlefieldAction) {
        match action {
            ui::BattlefieldAction::StartPendingBattle => {
                self.apply_campaign_command(Command::StartPendingBattle);
                return;
            }
            ui::BattlefieldAction::Close => {
                self.state.overlay = self.battle_return.take().unwrap_or(Overlay::None);
                self.battlefield = None;
                self.battlefield_view = ui::BattlefieldView::default();
                return;
            }
            _ => {}
        }
        let event_count = self
            .battlefield
            .as_ref()
            .map_or(0, |resolution| resolution.events.len());
        self.battlefield_view.apply(action, event_count);
    }

    pub(super) fn open_pending_battlefield(&mut self) {
        let resolution = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| campaign.pending_battle.as_ref())
            .and_then(|pending| pending.report.simulation.clone());
        let Some(resolution) = resolution else {
            return;
        };
        self.battlefield = Some(resolution);
        self.battlefield_view = ui::BattlefieldView {
            is_paused: true,
            ..Default::default()
        };
        self.state.overlay = Overlay::Battlefield;
        self.error = None;
    }

    pub(super) fn open_committed_battlefield(&mut self, id: kestrum::state::battle::BattleId) {
        let resolution = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| campaign.battles.get(&id))
            .and_then(|report| report.simulation.clone());
        let Some(resolution) = resolution else {
            return;
        };
        self.battlefield = Some(resolution);
        self.battlefield_view = ui::BattlefieldView::default();
        self.state.overlay = Overlay::Battlefield;
        self.error = None;
    }

    pub(super) fn sync_pending_battle(&mut self) {
        if self.state.overlay == Overlay::Battlefield {
            return;
        }
        let pending = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .is_some_and(|campaign| campaign.pending_battle.is_some());
        if pending {
            self.open_pending_battlefield();
        }
    }
}
