//! Validated user save intents and unique checkpoint identities.

use super::*;
use crate::state::{CampaignPhase, StrategicCampaign};

impl SaveLibrary {
    pub fn prepare_manual(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
        campaign: &Campaign,
        name: &str,
        overwrite: Option<u64>,
    ) -> Result<PreparedSave, String> {
        campaign.validate(data)?;
        let strategic = campaign
            .strategic()
            .ok_or("Empty-atlas campaigns are read-only.")?;
        if strategic.phase != CampaignPhase::PlayerTurn
            && strategic.diplomacy.ending.is_none()
            && !strategic.diplomacy.has_pending_decision()
        {
            return Err("Manual saves are available during your faction's turn.".into());
        }
        self.prepare(store, data, campaign, SaveKind::Manual, name, overwrite)
    }

    pub fn prepare_checkpoint(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
        campaign: &Campaign,
    ) -> Result<PreparedSave, String> {
        campaign.validate(data)?;
        let strategic = campaign
            .strategic()
            .ok_or("Empty-atlas campaigns are read-only.")?;
        require_checkpoint(strategic)?;
        let season = &data.presentation.seasons[strategic.season_index()];
        let name = format!(
            "{season}, Year {}",
            strategic.year(data.presentation.start_year)
        );
        self.prepare(
            store,
            data,
            campaign,
            SaveKind::RoundCheckpoint,
            &name,
            None,
        )
    }

    pub fn import_legacy(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
        raw: &str,
        name: &str,
    ) -> Result<PreparedSave, String> {
        let mut candidate = load_legacy(raw, data)?;
        let Campaign::Strategic(strategic) = &mut candidate else {
            return Err(
                "The empty-atlas campaign remains available in its original read-only slot.".into(),
            );
        };
        normalize_name(name)?;
        self.refresh(store, data)?;
        strategic.campaign_id = self.allocate_campaign_id(store)?;
        self.prepare(store, data, &candidate, SaveKind::Imported, name, None)
    }

    fn prepare(
        &mut self,
        store: &mut impl IndexedSaveStore,
        data: &GameData,
        campaign: &Campaign,
        kind: SaveKind,
        name: &str,
        target: Option<u64>,
    ) -> Result<PreparedSave, String> {
        let name = normalize_name(name)?;
        let strategic = campaign
            .strategic()
            .ok_or("Empty-atlas campaigns are read-only.")?;
        if strategic.campaign_id.0 == 0 {
            return Err("A strategic campaign requires an allocated storage identity.".into());
        }
        self.refresh(store, data)?;
        let expected_target_order = target
            .map(|id| {
                self.entries()
                    .iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| entry.metadata.success_order)
                    .ok_or("The save selected for overwrite no longer exists.")
            })
            .transpose()?;
        let raw = serde_json::to_string(campaign).map_err(|error| error.to_string())?;
        let identity = self.catalogue.reserve_identity(store)?;
        Ok(PreparedSave {
            metadata: SaveMetadata {
                name,
                kind,
                campaign_id: strategic.campaign_id,
                completed_rounds: strategic.completed_rounds,
                schema_version: strategic.version,
                content_version: strategic.content_version,
                success_order: identity,
            },
            operation: format!("save_{identity}"),
            target,
            expected_target_order,
            raw,
        })
    }
}

pub(super) fn normalize_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > MAX_SAVE_NAME_CHARS
        || trimmed.chars().any(char::is_control)
    {
        return Err(format!(
            "Enter a save name of 1–{MAX_SAVE_NAME_CHARS} characters without line breaks."
        ));
    }
    Ok(trimmed.to_owned())
}

pub(super) fn require_checkpoint(campaign: &StrategicCampaign) -> Result<(), String> {
    if campaign.diplomacy.ending.is_some() && campaign.pending_facts.is_empty() {
        return Ok(());
    }
    if campaign.completed_rounds == 0
        || campaign.phase != CampaignPhase::PlayerTurn
        || !campaign.acted.is_empty()
        || !campaign.pending_facts.is_empty()
    {
        return Err("Round checkpoints require a fully completed seasonal boundary.".into());
    }
    Ok(())
}
