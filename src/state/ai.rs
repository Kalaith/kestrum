//! Durable planning intent; no private foreign military information is stored here.

use super::StrategicCampaign;
use crate::data::{
    world::{FactionId, SiteId},
    GameData,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiState {
    #[serde(deserialize_with = "crate::data::economy::unique_table")]
    pub factions: BTreeMap<FactionId, AiFactionState>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiFactionState {
    pub phase_round: u32,
    pub accepted_commands: u32,
    pub objective: Option<AiObjective>,
    pub rejected_at_sequence: u64,
    pub rejected: BTreeSet<AiIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiObjective {
    pub site: SiteId,
    pub chosen_round: u32,
    pub kind: AiObjectiveKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiObjectiveKind {
    Defend,
    Expand,
    Threat,
    Attack,
    Border,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiIntent {
    pub kind: AiIntentKind,
    pub targets: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiIntentKind {
    Pass,
    Move,
    Recruit,
    Build,
    Reassign,
    CancelConstruction,
    Focus,
    Threat,
    Siege,
    Peace,
    War,
    Headquarters,
    Resettle,
    CityDevelopment,
    Progression,
}

impl StrategicCampaign {
    pub fn validate_ai(&self, data: &GameData) -> Result<(), String> {
        data.ai.validate()?;
        for (faction, state) in &self.ai.factions {
            if !self.factions.contains_key(faction)
                || (*faction == self.player && !self.observer_mode)
                || state.phase_round > self.completed_rounds
                || state.accepted_commands > data.ai.max_commands_per_phase
                || state.rejected_at_sequence > self.accepted_sequence
                || state.rejected.len() > data.ai.max_commands_per_phase as usize
                || state
                    .rejected
                    .iter()
                    .any(|intent| intent.targets.len() > 128)
                || state.objective.as_ref().is_some_and(|objective| {
                    objective.chosen_round > self.completed_rounds
                        || self.world.site(objective.site).is_none()
                })
            {
                return Err("campaign.ai: invalid faction, phase budget or objective".into());
            }
        }
        Ok(())
    }
}
