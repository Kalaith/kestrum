//! Game-owned save metadata is presented without exposing storage keys.

use super::*;
use kestrum::state::persistence::{ContinueSelection, SaveKind};
use macroquad_toolkit::persistence::{RawSaveStore, SlotSaveStore};

impl Game {
    pub(super) fn capture_saves(&mut self, scene: &str) {
        self.capture_campaign();
        self.state.overlay = Overlay::Saves;
        self.saves.ready = true;
        self.saves.can_save = true;
        self.saves.rows = (1..=17)
            .rev()
            .map(|id| ui::SaveRow {
                id,
                name: if id == 17 {
                    "The Silver Hawthorns at Winter's Crossing".into()
                } else {
                    format!("Rosemarch campaign {id}")
                },
                detail: format!("Manual  ·  Winter, 1001  ·  Round 4004  ·  #{id}"),
            })
            .collect();
        self.saves.selected = Some(17);
        self.saves.status = self.data.game_text.text("storage_ready").into();
        match scene.trim_end_matches("_minimum") {
            "save_name" | "save_symbols" => {
                self.saves.name = "Rosemarch after the long winter".into();
                self.saves.mode = ui::SaveMode::Name { target: None };
                if scene.starts_with("save_symbols") {
                    self.saves.keyboard_page = macroquad_toolkit::ui::text_entry::KeyboardPage::Numbers;
                }
            }
            "save_busy" => {
                self.saves.ready = false;
                self.saves.status = self.data.game_text.text("storage_busy").into();
            }
            "save_invalid" => self.saves.status = "Could not load campaign: Unsupported campaign schema version 999. Your current game and saved bytes are unchanged.".into(),
            "save_delete" => self.saves.mode = ui::SaveMode::ConfirmDelete(17),
            "save_recovery" => {
                self.state.overlay = Overlay::SaveRecovery;
                self.save_error = "Browser storage rejected the write (it may be full or blocked). Older saves are still available.".into();
            }
            _ => {}
        }
    }

    pub(super) fn update_save_rows(&mut self) {
        self.saves.can_save = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .is_some_and(|campaign| {
                matches!(campaign.phase, kestrum::state::CampaignPhase::PlayerTurn)
                    || self.kingdom.is_save_boundary()
            });
        let Some(library) = &mut self.library else {
            return;
        };
        self.save_exists = library.last_successful().is_some();
        let mut entries: Vec<_> = library.entries().iter().collect();
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.metadata.success_order));
        self.saves.rows = entries
            .into_iter()
            .map(|entry| {
                let meta = &entry.metadata;
                let season = &self.data.game_text.seasons[meta.completed_rounds as usize % 4];
                let year = self.data.presentation.start_year + meta.completed_rounds / 4;
                let kind = self.data.game_text.text(match meta.kind {
                    SaveKind::Manual => "manual_save",
                    SaveKind::RoundCheckpoint => "round_save",
                    SaveKind::Imported => "imported_save",
                });
                ui::SaveRow {
                    id: entry.id,
                    name: meta.name.clone(),
                    detail: format!(
                        "{kind}  ·  {season}, {year}  ·  {} {}  ·  #{}",
                        self.data.game_text.text("round"),
                        meta.completed_rounds.saturating_add(1),
                        entry.id
                    ),
                }
            })
            .collect();
        self.saves.page = self.saves.page.min(self.saves.page_count() - 1);
        self.saves.select_visible();
        self.saves.status = self
            .data
            .game_text
            .text(if !self.saves.ready {
                "storage_busy"
            } else if self.saves.can_save {
                "storage_ready"
            } else {
                "save_during_orders"
            })
            .into();
        let warnings = library.take_warnings();
        if !warnings.is_empty() {
            self.saves.status = warnings.join(" ");
            self.error = Some(self.saves.status.clone());
        }
    }

    pub(super) fn open_saves(&mut self, name_new: bool) {
        if let (Some(library), Some(store)) = (&mut self.library, &mut self.storage) {
            if let Err(error) = library.refresh(store, &self.data) {
                self.saves.ready = false;
                self.saves.status = error;
            } else {
                self.update_save_rows();
            }
        }
        self.state.overlay = Overlay::Saves;
        self.saves.mode = ui::SaveMode::Browse;
        if name_new && self.saves.ready && self.saves.can_save {
            self.name_save(None);
        }
    }

    pub(super) fn name_save(&mut self, target: Option<u64>) {
        if !self.saves.ready || !self.saves.can_save {
            return;
        }
        self.saves.name = target
            .and_then(|id| {
                self.saves
                    .rows
                    .iter()
                    .find(|row| row.id == id)
                    .map(|row| row.name.clone())
            })
            .unwrap_or_else(|| {
                format!(
                    "{} {}",
                    self.data.game_text.text("saved_name"),
                    self.state
                        .campaign
                        .as_ref()
                        .map_or(1, Campaign::display_turn)
                )
            });
        self.saves.mode = ui::SaveMode::Name { target };
        self.saves.keyboard_page = Default::default();
        self.saves.name_error = None;
    }

    pub(super) fn edit_save_name(
        &mut self,
        action: macroquad_toolkit::ui::text_entry::TextEntryAction,
    ) {
        use macroquad_toolkit::ui::text_entry::{apply_text_edit, TextEntryAction};
        if self.state.overlay != Overlay::Saves
            || !matches!(self.saves.mode, ui::SaveMode::Name { .. })
        {
            return;
        }
        match action {
            TextEntryAction::SetPage(page) => self.saves.keyboard_page = page,
            TextEntryAction::Edit(edit) => {
                self.saves.name_error = apply_text_edit(
                    &mut self.saves.name,
                    edit,
                    kestrum::state::persistence::MAX_SAVE_NAME_CHARS,
                )
                .err()
                .map(|error| error.to_string());
            }
        }
    }

    pub(super) fn commit_named_save(&mut self) {
        let ui::SaveMode::Name { target } = self.saves.mode else {
            return;
        };
        let Some(snapshot) = self.state.campaign.clone() else {
            return;
        };
        self.pending_save = Some(storage::PendingWrite {
            snapshot,
            name: Some(self.saves.name.clone()),
            target,
            prepared: None,
            return_overlay: Overlay::Saves,
            load_after: false,
        });
        self.write_pending();
    }

    pub(super) fn delete_save(&mut self, id: u64) {
        let result = match (&mut self.library, &mut self.storage) {
            (Some(library), Some(store)) => library.delete(store, &self.data, id),
            _ => Err(self.saves.status.clone()),
        };
        match result {
            Ok(receipt) => {
                self.saves.mode = ui::SaveMode::Browse;
                self.update_save_rows();
                if !receipt.warnings.is_empty() {
                    self.saves.status = receipt.warnings.join(" ");
                }
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn load_selected(&mut self) {
        if let Some(id) = self.saves.selected {
            self.load_indexed(id);
        }
    }

    pub(super) fn load_indexed(&mut self, id: u64) {
        let result = match (&mut self.library, &mut self.storage) {
            (Some(library), Some(store)) => library.load(store, &self.data, id),
            _ => Err(self.saves.status.clone()),
        };
        self.finish_load(result);
    }

    pub(super) fn load(&mut self) {
        match self
            .library
            .as_ref()
            .and_then(|library| library.last_successful())
        {
            Some(ContinueSelection::Indexed(id)) => self.load_indexed(id),
            Some(ContinueSelection::LegacyShell) => self.load_slot(SAVE_SLOT),
            None if self.legacy_save_exists => self.load_slot(SAVE_SLOT),
            None => self.open_saves(false),
        }
    }

    pub(super) fn finish_load(&mut self, result: Result<Campaign, String>) {
        let result = result.and_then(|mut campaign| {
            if let Campaign::Strategic(campaign) = &mut campaign {
                if !campaign.notifications.baseline_complete {
                    engine::notifications::baseline_current_conditions(campaign, &self.data)?;
                }
                campaign
                    .notifications
                    .sync_preferences(&self.preferences.notifications, &self.data.notifications);
            }
            self.state.load_campaign(campaign, &self.data)
        });
        match result {
            Ok(()) => {
                self.portraits.request_reset();
                self.initialize_notification_campaign();
                self.navigation.reset(&mut self.view);
                self.army = ui::ArmyView::default();
                self.movement = ui::MoveView::default();
                self.battle = ui::BattleView::default();
                self.reset_history();
                self.kingdom = ui::KingdomView::default();
                self.diplomacy_seen.clear();
                self.refresh_kingdom();
                self.ending_saved = self.kingdom_ended();
                self.invalidate_projection();
                self.refresh_projection();
                self.focus_initial_home();
                self.npc_delay = 0.0;
                self.error = None;
                self.notice = Some((self.data.game_text.text("load_success").into(), 3.0));
                self.update_save_rows();
            }
            Err(error) => {
                self.error = Some(format!(
                    "{}: {error}",
                    self.data.game_text.text("load_failed")
                ))
            }
        }
    }

    pub(super) fn load_slot(&mut self, slot: &str) {
        let result = self
            .read_old_slot(slot)
            .and_then(|raw| kestrum::state::persistence::load_legacy(&raw, &self.data));
        let shell = matches!(&result, Ok(Campaign::Shell(_)));
        self.finish_load(result);
        if shell {
            if let (Some(library), Some(store)) = (&mut self.library, &mut self.storage) {
                if let Err(error) = library.remember_legacy_shell(store) {
                    self.error = Some(error);
                }
            }
        }
    }

    fn read_old_slot(&self, slot: &str) -> Result<String, String> {
        SlotSaveStore {
            game_name: "kestrum",
        }
        .read(slot)?
        .ok_or_else(|| "The earlier campaign could not be found".into())
    }

    pub(super) fn import_campaign(&mut self) {
        let result = (|| {
            let raw = self.read_old_slot(campaign::STRATEGIC_SLOT)?;
            let library = self
                .library
                .as_mut()
                .ok_or_else(|| self.saves.status.clone())?;
            let store = self
                .storage
                .as_mut()
                .ok_or_else(|| self.saves.status.clone())?;
            let prepared = library.import_legacy(
                store,
                &self.data,
                &raw,
                self.data.game_text.text("imported_save"),
            )?;
            let snapshot = kestrum::state::persistence::load_legacy(&raw, &self.data)?;
            Ok::<_, String>(storage::PendingWrite {
                snapshot,
                name: None,
                target: None,
                prepared: Some(prepared),
                return_overlay: Overlay::None,
                load_after: true,
            })
        })();
        match result {
            Ok(pending) => {
                self.pending_save = Some(pending);
                self.write_pending();
            }
            Err(error) => {
                self.error = Some(error);
                self.open_saves(false);
            }
        }
    }
}
