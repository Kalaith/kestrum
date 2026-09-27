//! Ordinary local occupants have authored strength and rewards, never sovereign turns.

use super::{
    economy::Resources,
    world::{Scenario, SiteId},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE: &str = "assets/data/threats.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThreatKind {
    Bandits,
    Wildlife,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreatDefinition {
    pub name: String,
    pub headcount: u32,
    pub attack: u32,
    pub resistance: u32,
    pub reward: Resources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialThreat {
    pub site: SiteId,
    pub kind: ThreatKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreatRules {
    pub schema_version: u32,
    pub leadership_permille: u32,
    pub lawless_rounds: u32,
    #[serde(deserialize_with = "super::economy::unique_table")]
    pub definitions: BTreeMap<ThreatKind, ThreatDefinition>,
    pub initial: Vec<InitialThreat>,
}

impl ThreatRules {
    pub fn validate(&self, scenario: &Scenario) -> Result<(), String> {
        self.validate_initials(scenario, &self.initial)
    }

    pub fn validate_definitions(&self) -> Result<(), String> {
        let invalid = |reason| format!("{SOURCE}: {reason}");
        if self.schema_version != 1 || self.leadership_permille != 1000 || self.lawless_rounds == 0
        {
            return Err(invalid(
                "invalid schema, neutral leadership or lawless duration",
            ));
        }
        if self.definitions.len() != 2 {
            return Err(invalid("definitions require Bandits and Wildlife"));
        }
        for kind in [ThreatKind::Bandits, ThreatKind::Wildlife] {
            let entry = self
                .definitions
                .get(&kind)
                .ok_or_else(|| invalid("missing definition"))?;
            if entry.name.trim().is_empty()
                || entry.name.chars().count() > 80
                || !(1..=100_000).contains(&entry.headcount)
                || !(1..=10_000).contains(&entry.attack)
                || !(1..=10_000).contains(&entry.resistance)
            {
                return Err(invalid("invalid name, headcount, attack or resistance"));
            }
            entry.reward.validate(SOURCE, "reward")?;
        }
        Ok(())
    }

    pub fn validate_initials(
        &self,
        scenario: &Scenario,
        initial: &[InitialThreat],
    ) -> Result<(), String> {
        self.validate_definitions()?;
        let invalid = |reason| format!("{SOURCE}: {reason}");
        let mut sites = BTreeSet::new();
        for entry in initial {
            if !sites.insert(entry.site)
                || !scenario
                    .sites
                    .iter()
                    .any(|site| site.id == entry.site && site.controller.is_none())
            {
                return Err(invalid(
                    "initial threat requires a distinct uncontrolled physical site",
                ));
            }
        }
        Ok(())
    }
}
