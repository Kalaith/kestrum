//! Catalogue readiness and retries retain the exact state being saved.

use super::*;
use kestrum::state::persistence::{PreparedSave, SaveLibrary, SAVE_NAMESPACE};
use macroquad_toolkit::persistence::{IndexedKeyStore, WriterStatus};

pub(super) struct PendingWrite {
    pub snapshot: Campaign,
    pub name: Option<String>,
    pub target: Option<u64>,
    pub prepared: Option<PreparedSave>,
    pub return_overlay: Overlay,
    pub load_after: bool,
}

impl Game {
    pub(super) fn retry_storage(&mut self) {
        if self.capture {
            return;
        }
        self.storage_checked = false;
        self.saves.ready = false;
        self.saves.status = self.data.presentation.text("storage_waiting").into();
        let result = if let Some(store) = &mut self.storage {
            store.retry_writer().map(|_| ())
        } else {
            IndexedKeyStore::new("kestrum", SAVE_NAMESPACE).map(|store| self.storage = Some(store))
        };
        if let Err(error) = result {
            self.storage_checked = true;
            self.saves.status = error;
        }
    }

    pub(super) fn poll_storage(&mut self) {
        if self.capture || self.storage_checked {
            return;
        }
        let Some(store) = &mut self.storage else {
            return;
        };
        let status = match store.poll_writer() {
            Ok(WriterStatus::Pending) => return,
            Ok(status) => status,
            Err(error) => {
                self.storage_checked = true;
                self.saves.status = error;
                return;
            }
        };
        self.storage_checked = true;
        match SaveLibrary::open(store, &self.data) {
            Ok(library) => {
                self.library = Some(library);
                self.saves.ready = status == WriterStatus::Ready;
                self.update_save_rows();
                if !self.saves.ready {
                    self.saves.status = self.data.presentation.text("storage_busy").into();
                }
            }
            Err(error) => self.saves.status = error,
        }
        if self.saves.ready && self.retry_save_when_ready {
            self.retry_save_when_ready = false;
            self.write_pending();
        }
    }

    pub(super) fn save_checkpoint(&mut self) {
        if self.capture {
            return;
        }
        let Some(snapshot) = self.state.campaign.clone() else {
            return;
        };
        let name = snapshot
            .strategic()
            .filter(|campaign| campaign.completed_rounds == 0)
            .map(|_| self.data.presentation.text("founding_save").to_owned());
        self.pending_save = Some(PendingWrite {
            snapshot,
            name,
            target: None,
            prepared: None,
            return_overlay: Overlay::None,
            load_after: false,
        });
        self.write_pending();
    }

    pub(super) fn write_pending(&mut self) {
        let result = (|| {
            let library = self
                .library
                .as_mut()
                .ok_or_else(|| self.saves.status.clone())?;
            let store = self
                .storage
                .as_mut()
                .ok_or_else(|| self.saves.status.clone())?;
            let pending = self.pending_save.as_mut().ok_or("No save is waiting")?;
            if pending.prepared.is_none() {
                pending.prepared = Some(if let Some(name) = &pending.name {
                    library.prepare_manual(
                        store,
                        &self.data,
                        &pending.snapshot,
                        name,
                        pending.target,
                    )?
                } else {
                    library.prepare_checkpoint(store, &self.data, &pending.snapshot)?
                });
            }
            library.write(
                store,
                &self.data,
                pending.prepared.as_ref().ok_or("Save preparation failed")?,
            )
        })();
        match result {
            Ok(receipt) => {
                let Some(pending) = self.pending_save.take() else {
                    return;
                };
                self.state.overlay = pending.return_overlay;
                self.saves.mode = ui::SaveMode::Browse;
                self.save_error.clear();
                self.error = None;
                self.notice = Some((self.data.presentation.text("save_success").into(), 3.0));
                self.update_save_rows();
                self.saves.selected = Some(receipt.id);
                self.saves.page = 0;
                if !receipt.warnings.is_empty() {
                    self.error = Some(receipt.warnings.join(" "));
                }
                if pending.load_after {
                    self.load_indexed(receipt.id);
                }
            }
            Err(error) => {
                self.save_error = error;
                self.error = None;
                self.state.overlay = Overlay::SaveRecovery;
            }
        }
    }

    pub(super) fn retry_save(&mut self) {
        if !self.saves.ready {
            self.retry_save_when_ready = true;
            self.retry_storage();
        } else {
            self.write_pending();
        }
    }

    pub(super) fn continue_unsaved(&mut self) {
        self.retry_save_when_ready = false;
        if let Some(pending) = self.pending_save.take() {
            self.state.overlay = pending.return_overlay;
        }
        self.saves.mode = ui::SaveMode::Browse;
        self.save_error.clear();
    }

    pub(super) fn go_back(&mut self) {
        match self.state.overlay {
            Overlay::SaveRecovery => {}
            Overlay::Saves if self.saves.mode != ui::SaveMode::Browse => {
                self.saves.mode = ui::SaveMode::Browse
            }
            _ => self.state.back(),
        }
    }
}
