//! Persisted local occupants are distinct from factions, armies and formations.

mod validation;

use super::StrategicCampaign;
use crate::data::{
    economy::Resources,
    threats::InitialThreat,
    threats::ThreatKind,
    world::{FactionId, Scenario, SiteId},
    GameData,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ThreatId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ThreatStatus {
    Active,
    Cleared {
        round: u32,
        by: FactionId,
        payout: Resources,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Threat {
    pub id: ThreatId,
    pub site: SiteId,
    pub kind: ThreatKind,
    pub name: String,
    pub headcount: u32,
    pub status: ThreatStatus,
    /// A per-site ruination transition, absent for authored initial occupants.
    pub ruination: Option<u64>,
}

impl StrategicCampaign {
    pub fn active_threat(&self, site: SiteId) -> Option<&Threat> {
        self.threats
            .values()
            .find(|entry| entry.site == site && entry.status == ThreatStatus::Active)
    }
}

pub fn initialize_threats(
    data: &GameData,
    scenario: &Scenario,
    initial: &[InitialThreat],
) -> Result<BTreeMap<ThreatId, Threat>, String> {
    data.threats.validate_initials(scenario, initial)?;
    initial
        .iter()
        .enumerate()
        .map(|(index, initial)| {
            let id = ThreatId(u32::try_from(index + 1).map_err(|_| "too many initial threats")?);
            Ok((
                id,
                Threat {
                    id,
                    site: initial.site,
                    kind: initial.kind,
                    name: data.threats.definitions[&initial.kind].name.clone(),
                    headcount: data.threats.definitions[&initial.kind].headcount,
                    status: ThreatStatus::Active,
                    ruination: None,
                },
            ))
        })
        .collect()
}
