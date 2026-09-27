//! Read-only record navigation caches observer-filtered queries between actions.

use super::*;
use kestrum::engine::HistoryFilter;
use kestrum::state::{battle::BattleId, history::HistorySubject};
use macroquad_toolkit::ui::text_entry::{apply_text_edit, TextEntryAction};

impl Game {
    pub(super) fn reset_history(&mut self) {
        self.history = ui::HistoryView::default();
        self.history_return = Overlay::None;
        self.history_from_records = false;
        self.battle_return = None;
    }

    pub(super) fn apply_history_action(&mut self, action: UiAction) {
        match action {
            UiAction::OpenRecords => self.open_records(),
            UiAction::SetRecordCategory(category) => {
                self.history.category = category;
                self.history.records_page = 0;
                self.refresh_records();
            }
            UiAction::RecordsPage(delta) => {
                self.history.records_page = self
                    .history
                    .records_page
                    .saturating_add_signed(delta as isize);
                self.refresh_records();
            }
            UiAction::OpenHistory(subject) => self.open_history(subject),
            UiAction::TransferLegacyItem(item, to) => {
                self.apply_campaign_command(Command::TransferLegacyItem { item, to });
                self.refresh_history();
            }
            UiAction::OpenRelatedHistoryEvent(id) => {
                self.history.subject = None;
                self.history.filter = HistoryFilter {
                    event: Some(id),
                    ..Default::default()
                };
                self.history.screen_page = 0;
                self.history.mode = ui::HistoryMode::Events;
                self.refresh_history();
            }
            UiAction::SetHistoryMode(mode) => {
                self.history.mode = mode;
                self.history.status.clear();
                if mode == ui::HistoryMode::Overview {
                    self.history.overview_page = 0;
                }
            }
            UiAction::HistoryPage(delta) => {
                self.history.screen_page = self
                    .history
                    .screen_page
                    .saturating_add_signed(delta as isize);
                self.refresh_history();
            }
            UiAction::HistoryOverviewPage(delta) => {
                self.history.overview_page = self
                    .history
                    .overview_page
                    .saturating_add_signed(delta as isize);
            }
            UiAction::SetHistoryKind(kind) => self.history.filter.kind = kind,
            UiAction::ShiftHistoryFrom(delta) => self.shift_history_date(delta, true),
            UiAction::ShiftHistoryTo(delta) => self.shift_history_date(delta, false),
            UiAction::ResetHistoryFilters => {
                self.history.filter = HistoryFilter {
                    subject: self.history.subject,
                    ..Default::default()
                };
            }
            UiAction::ApplyHistoryFilters => self.apply_history_filters(),
            UiAction::HistoryBack => self.history_back(),
            UiAction::OpenRecordedBattle(id) => self.open_recorded_battle(id),
            UiAction::EditHistorySearch(action) => self.edit_history_search(action),
            UiAction::ApplyHistorySearch => {
                self.history.mode = ui::HistoryMode::Records;
                self.history.records_page = 0;
                self.refresh_records();
            }
            _ => unreachable!("history action dispatch"),
        }
    }

    fn open_records(&mut self) {
        self.history_return = self.state.overlay;
        self.history_from_records = true;
        self.history = ui::HistoryView::default();
        self.state.overlay = Overlay::History;
        self.notice = None;
        self.error = None;
        self.refresh_records();
    }

    pub(super) fn open_history(&mut self, subject: HistorySubject) {
        if self.state.overlay != Overlay::History
            && !(self.state.overlay == Overlay::Battle
                && self.battle_return == Some(Overlay::History))
        {
            self.history_return = self.state.overlay;
            self.history_from_records = false;
        }
        self.history.mode = ui::HistoryMode::Overview;
        self.history.subject = Some(subject);
        self.history.filter = HistoryFilter {
            subject: Some(subject),
            ..Default::default()
        };
        self.history.screen_page = 0;
        self.history.overview_page = 0;
        self.history.status.clear();
        self.state.overlay = Overlay::History;
        self.notice = None;
        self.error = None;
        self.refresh_history();
    }

    fn refresh_records(&mut self) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            self.history.status = self.data.presentation.text("legacy_read_only").into();
            return;
        };
        let per_page = ui::HISTORY_ROWS_PER_SCREEN;
        let total = match self.history.category {
            ui::RecordCategory::People => {
                let result = engine::known_people(
                    campaign,
                    campaign.player,
                    &self.history.search,
                    self.history.records_page / 10,
                );
                let total = result.total;
                self.history.known_people = Some(result);
                total
            }
            ui::RecordCategory::Places => campaign.world.sites.len(),
            ui::RecordCategory::Armies => campaign
                .armies
                .values()
                .filter(|army| army.faction == campaign.player)
                .count(),
            ui::RecordCategory::Items => campaign
                .legacy_items
                .values()
                .filter(|item| item.faction == campaign.player)
                .count(),
        };
        let clamped = self
            .history
            .records_page
            .min(total.div_ceil(per_page).saturating_sub(1));
        if clamped != self.history.records_page {
            self.history.records_page = clamped;
            if self.history.category == ui::RecordCategory::People {
                self.history.known_people = Some(engine::known_people(
                    campaign,
                    campaign.player,
                    &self.history.search,
                    clamped / 10,
                ));
            }
        }
    }

    fn refresh_history(&mut self) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        self.history.person = match self.history.subject {
            Some(HistorySubject::Person(id)) => {
                engine::person_knowledge(campaign, campaign.player, id)
            }
            _ => None,
        };
        self.history.filter.page = self.history.screen_page / 10;
        let result = engine::history_page(campaign, campaign.player, &self.history.filter);
        let page = self.history.screen_page.min(
            result
                .total_entries
                .div_ceil(ui::HISTORY_ROWS_PER_SCREEN)
                .saturating_sub(1),
        );
        if page / 10 != self.history.filter.page {
            self.history.filter.page = page / 10;
            self.history.result = Some(engine::history_page(
                campaign,
                campaign.player,
                &self.history.filter,
            ));
        } else {
            self.history.result = Some(result);
        }
        self.history.screen_page = page;
    }

    fn shift_history_date(&mut self, delta: i32, from: bool) {
        let current = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .map_or(0, |campaign| campaign.completed_rounds);
        let selected = if from {
            &mut self.history.filter.from_round
        } else {
            &mut self.history.filter.to_round
        };
        *selected = Some(
            selected
                .map_or(current, |round| round.saturating_add_signed(delta))
                .min(current),
        );
        self.history.status.clear();
    }

    fn apply_history_filters(&mut self) {
        if self
            .history
            .filter
            .from_round
            .zip(self.history.filter.to_round)
            .is_some_and(|(from, to)| from > to)
        {
            self.history.status = "The first season must be no later than the last season.".into();
            return;
        }
        self.history.screen_page = 0;
        self.history.mode = ui::HistoryMode::Events;
        self.history.status.clear();
        self.refresh_history();
    }

    pub(super) fn history_back(&mut self) {
        self.history.status.clear();
        match self.history.mode {
            ui::HistoryMode::Search => {
                self.history.mode = ui::HistoryMode::Records;
                self.refresh_records();
            }
            ui::HistoryMode::Filters => {
                self.history.mode = ui::HistoryMode::Events;
                self.refresh_history();
            }
            ui::HistoryMode::Overview | ui::HistoryMode::Events if self.history_from_records => {
                self.history.mode = ui::HistoryMode::Records;
                self.refresh_records();
            }
            _ => self.state.overlay = self.history_return,
        }
    }

    pub(super) fn edit_history_search(&mut self, action: TextEntryAction) {
        if self.state.overlay != Overlay::History || self.history.mode != ui::HistoryMode::Search {
            return;
        }
        match action {
            TextEntryAction::SetPage(page) => self.history.keyboard_page = page,
            TextEntryAction::Edit(edit) => {
                self.history.status = apply_text_edit(&mut self.history.search, edit, 80)
                    .err()
                    .map_or_else(String::new, |error| error.to_string());
            }
        }
    }

    fn open_recorded_battle(&mut self, id: BattleId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let reports = engine::battle_reports(campaign, campaign.player);
        let Some(index) = reports.iter().position(|report| report.id == id) else {
            self.history.status = "This encounter's detailed report is no longer retained.".into();
            return;
        };
        self.battle = ui::BattleView {
            index,
            ..Default::default()
        };
        self.battle_return = Some(Overlay::History);
        self.state.overlay = Overlay::Battle;
    }
}
