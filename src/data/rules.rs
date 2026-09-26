//! Setup rules consumed by authored scenario validation (P01).

use super::validation::{require, unique};
use serde::{Deserialize, Serialize};

pub const SOURCE: &str = "assets/data/campaign_rules.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Emblem {
    Rose,
    Oak,
    Fern,
    Iris,
    Thistle,
    Ivy,
    Hawthorn,
    Laurel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmblemDefinition {
    pub id: Emblem,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignRules {
    pub schema_version: u32,
    pub content_version: u32,
    pub min_factions: usize,
    pub max_factions: usize,
    pub default_factions: usize,
    pub kingdom_name_max_chars: usize,
    pub difficulty: Difficulty,
    pub ai_income_bonus_percent: u32,
    pub emblems: Vec<EmblemDefinition>,
}

impl CampaignRules {
    pub fn validate(&self) -> Result<(), String> {
        for (field, valid) in [
            ("schema_version", self.schema_version == 1),
            ("content_version", self.content_version == 1),
            ("min_factions", self.min_factions == 4),
            ("max_factions", self.max_factions == 8),
            ("default_factions", self.default_factions == 4),
            ("kingdom_name_max_chars", self.kingdom_name_max_chars == 32),
            ("ai_income_bonus_percent", self.ai_income_bonus_percent == 0),
            ("emblems", self.emblems.len() == 8),
        ] {
            require(SOURCE, field, valid, "unsupported P01 setup value")?;
        }
        unique(
            SOURCE,
            "emblems.id",
            self.emblems.iter().map(|emblem| emblem.id),
        )?;
        for emblem in &self.emblems {
            require(
                SOURCE,
                "emblems.name",
                !emblem.name.trim().is_empty(),
                "must not be empty",
            )?;
        }
        Ok(())
    }

    pub fn valid_kingdom_name(&self, name: &str) -> bool {
        !name.is_empty()
            && name.trim() == name
            && name.chars().count() <= self.kingdom_name_max_chars
    }
}
