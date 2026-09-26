//! Report navigation reads committed observations without replaying an encounter.

use super::*;

impl Game {
    pub(super) fn witnessed_battle(&self, battle: kestrum::state::battle::BattleId) -> bool {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| engine::project(campaign, campaign.player).ok())
            .is_some_and(|view| view.battles.iter().any(|report| report.id == battle))
    }

    pub(super) fn battle_report_count(&self) -> usize {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| engine::project(campaign, campaign.player).ok())
            .map_or(0, |view| view.battles.len())
    }

    pub(super) fn open_battle_reports(&mut self) {
        self.battle = ui::BattleView {
            index: self.battle_report_count().saturating_sub(1),
            ..Default::default()
        };
        self.state.overlay = Overlay::Battle;
        self.notice = None;
        self.error = None;
    }

    pub(super) fn change_battle_report(&mut self, delta: i32) {
        self.battle.index = self
            .battle
            .index
            .saturating_add_signed(delta as isize)
            .min(self.battle_report_count().saturating_sub(1));
        self.battle.page = 0;
    }
}
