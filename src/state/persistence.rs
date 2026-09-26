//! Kestrum save metadata, validated loading, and immutable save intents.

mod compatibility;
mod requests;
mod selection;

pub use compatibility::load_legacy;

use super::{Campaign, CampaignId, GameState};
use crate::data::GameData;
use macroquad_toolkit::persistence::{
    IndexedCatalogue, IndexedEntry, IndexedSaveStore, SaveCommit,
};
use serde::{Deserialize, Serialize};

pub const SAVE_NAMESPACE: &str = "campaign_catalogue";
pub const MAX_SAVE_NAME_CHARS: usize = 40;
const CONTINUE_KEY: &str = "campaign_catalogue_continue";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveKind {
    Manual,
    RoundCheckpoint,
    Imported,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveMetadata {
    pub name: String,
    pub kind: SaveKind,
    pub campaign_id: CampaignId,
    pub completed_rounds: u32,
    pub schema_version: u32,
    pub content_version: u32,
    /// Monotonic catalogue allocation, independent of simulation RNG and dates.
    pub success_order: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContinueSelection {
    Indexed(u64),
    LegacyShell,
}

/// Retries use these exact bytes and operation identity, even after play continues.
/// New intents commit in preparation order. An older uncommitted intent must be
/// prepared again after a newer save succeeds; completed same-intent retries stay valid.
#[derive(Debug, Clone)]
pub struct PreparedSave {
    pub metadata: SaveMetadata,
    operation: String,
    target: Option<u64>,
    expected_target_order: Option<u64>,
    raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveReceipt {
    pub id: u64,
    pub replayed: bool,
    pub warnings: Vec<String>,
}

pub struct SaveLibrary {
    catalogue: IndexedCatalogue<SaveMetadata>,
    last_successful: Option<ContinueSelection>,
    acknowledged_order: u64,
    warnings: Vec<String>,
}

impl SaveLibrary {
    pub fn open(store: &mut impl IndexedSaveStore, data: &GameData) -> Result<Self, String> {
        let mut library = Self {
            catalogue: IndexedCatalogue::new(SAVE_NAMESPACE)?,
            last_successful: None,
            acknowledged_order: 0,
            warnings: Vec::new(),
        };
        library.refresh(store, data)?;
        Ok(library)
    }

    pub fn refresh(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
    ) -> Result<(), String> {
        let report = self
            .catalogue
            .refresh(store, |metadata, raw| validate_payload(metadata, raw, data))?;
        self.warnings.extend(report.warnings);
        if !report.recovered.is_empty() {
            self.warnings
                .push("An interrupted save operation was recovered.".into());
        }
        if !report.abandoned.is_empty() {
            self.warnings.push(
                "An interrupted save had no valid payload. Existing saves were preserved.".into(),
            );
        }
        if report.pending.is_some() {
            self.warnings
                .push("An interrupted save is waiting for the storage writer connection.".into());
        }
        self.refresh_selection(store, &report.committed_writes);
        Ok(())
    }

    pub fn entries(&self) -> &[IndexedEntry<SaveMetadata>] {
        self.catalogue.entries()
    }

    pub fn last_successful(&self) -> Option<ContinueSelection> {
        self.last_successful
    }

    pub fn take_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }

    pub fn allocate_campaign_id(
        &mut self,
        store: &mut impl IndexedSaveStore,
    ) -> Result<CampaignId, String> {
        self.catalogue.reserve_identity(store).map(CampaignId)
    }

    pub fn write(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
        prepared: &PreparedSave,
    ) -> Result<SaveReceipt, String> {
        validate_payload(&prepared.metadata, &prepared.raw, data)?;
        self.refresh(store, data)?;
        self.validate_intent_order(prepared)?;
        self.validate_overwrite_target(prepared)?;
        let commit = self.catalogue.write(
            store,
            &prepared.operation,
            prepared.target,
            prepared.metadata.clone(),
            &prepared.raw,
            |metadata, raw| validate_payload(metadata, raw, data),
        )?;
        let mut receipt = receipt(commit);
        if let Err(error) = self.remember(store, Some(ContinueSelection::Indexed(receipt.id))) {
            receipt.warnings.push(format!(
                "The save is stored, but Continue could not be updated: {error}"
            ));
        }
        Ok(receipt)
    }

    fn validate_intent_order(&self, prepared: &PreparedSave) -> Result<(), String> {
        let order = prepared.metadata.success_order;
        let committed = self
            .entries()
            .iter()
            .any(|entry| entry.metadata.success_order == order);
        let newer = self.acknowledged_order > order
            || self
                .entries()
                .iter()
                .any(|entry| entry.metadata.success_order > order);
        if newer && !committed {
            return Err(
                "A newer save has already completed. Prepare this save again before writing it."
                    .into(),
            );
        }
        Ok(())
    }

    fn validate_overwrite_target(&self, prepared: &PreparedSave) -> Result<(), String> {
        let Some(target) = prepared.target else {
            return Ok(());
        };
        let current = self
            .entries()
            .iter()
            .find(|entry| entry.id == target)
            .map(|entry| entry.metadata.success_order);
        if current != prepared.expected_target_order
            && current != Some(prepared.metadata.success_order)
        {
            return Err("The selected save changed after this overwrite was prepared. Select it again to prepare a new overwrite.".into());
        }
        Ok(())
    }

    pub fn load(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
        id: u64,
    ) -> Result<Campaign, String> {
        self.refresh(store, data)?;
        let metadata = self
            .entries()
            .iter()
            .find(|entry| entry.id == id)
            .ok_or("That save is no longer in the catalogue.")?
            .metadata
            .clone();
        let raw = self.catalogue.load(store, id)?;
        let candidate = decode_payload(&metadata, &raw, data)?;
        if let Err(error) = self.remember(store, Some(ContinueSelection::Indexed(id))) {
            self.warnings.push(format!(
                "The campaign loaded, but Continue could not be updated: {error}"
            ));
        }
        Ok(candidate)
    }

    /// A convenient application seam proving that failed loads leave play intact.
    pub fn load_into(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
        id: u64,
        state: &mut GameState,
    ) -> Result<(), String> {
        let candidate = self.load(store, data, id)?;
        state.load_campaign(candidate, data)
    }

    pub fn delete(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
        id: u64,
    ) -> Result<SaveReceipt, String> {
        self.refresh(store, data)?;
        let commit = self.catalogue.delete(store, &format!("delete_{id}"), id)?;
        let mut receipt = receipt(commit);
        if self.last_successful == Some(ContinueSelection::Indexed(id)) {
            let next = self.latest_entry();
            if let Err(error) = self.remember(store, next) {
                receipt.warnings.push(format!(
                    "The save was deleted, but Continue could not be updated: {error}"
                ));
                self.last_successful = next;
            }
        }
        Ok(receipt)
    }

    pub fn remember_legacy_shell(
        &mut self,
        store: &mut impl IndexedSaveStore,
    ) -> Result<(), String> {
        self.remember(store, Some(ContinueSelection::LegacyShell))
    }
}

fn receipt(commit: SaveCommit) -> SaveReceipt {
    SaveReceipt {
        id: commit.entry_id,
        replayed: commit.replayed,
        warnings: commit.warnings,
    }
}

fn validate_payload(metadata: &SaveMetadata, raw: &str, data: &GameData) -> Result<(), String> {
    decode_payload(metadata, raw, data).map(|_| ())
}

fn decode_payload(metadata: &SaveMetadata, raw: &str, data: &GameData) -> Result<Campaign, String> {
    let campaign: Campaign = serde_json::from_str(raw)
        .map_err(|error| format!("The saved campaign is invalid: {error}"))?;
    campaign.validate(data)?;
    let strategic = campaign
        .strategic()
        .ok_or("Empty-atlas campaigns stay in their original read-only slot.")?;
    if requests::normalize_name(&metadata.name)? != metadata.name
        || metadata.campaign_id != strategic.campaign_id
        || metadata.completed_rounds != strategic.completed_rounds
        || metadata.schema_version != strategic.version
        || metadata.content_version != strategic.content_version
        || metadata.success_order == 0
        || metadata.campaign_id.0 == 0
    {
        return Err("The saved campaign and its catalogue metadata do not agree.".into());
    }
    if metadata.kind == SaveKind::RoundCheckpoint {
        requests::require_checkpoint(strategic)?;
    }
    Ok(campaign)
}
