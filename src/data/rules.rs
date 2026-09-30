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

/// The P11 contribution used by the army inspector and later encounter resolver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeadershipRules {
    pub base_permille: u32,
    pub named_scale_permille: u32,
    pub diminishing_count: u32,
    pub officer_commander_bonus_permille: u32,
    pub field_min_age_years: u32,
    pub officer_movement_allowance: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementRules {
    pub road_bonus: u32,
    pub road_disabled_damage: u32,
    pub minimum_edge_cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FounderRules {
    pub minimum_age_years: u32,
    pub maximum_age_years: u32,
    pub commander_bonus_permille: u32,
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
    pub leadership: LeadershipRules,
    pub movement: MovementRules,
    pub founder: FounderRules,
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
        require(
            SOURCE,
            "leadership",
            self.leadership.base_permille <= 2000
                && self.leadership.named_scale_permille <= 2000
                && (1..=100).contains(&self.leadership.diminishing_count)
                && self.leadership.officer_commander_bonus_permille <= 1000
                && (1..=100).contains(&self.leadership.field_min_age_years)
                && (1..=100).contains(&self.leadership.officer_movement_allowance),
            "invalid contribution or movement rule",
        )?;
        require(
            SOURCE,
            "founder",
            self.founder.minimum_age_years >= self.leadership.field_min_age_years
                && self.founder.minimum_age_years >= 18
                && self.founder.minimum_age_years <= self.founder.maximum_age_years
                && self.founder.maximum_age_years <= 24
                && self.founder.commander_bonus_permille <= 100,
            "founders must be young adults with at most a ten-point command bonus",
        )?;
        require(
            SOURCE,
            "movement",
            self.movement.road_bonus <= 4
                && (1..=100).contains(&self.movement.road_disabled_damage)
                && (1..=4).contains(&self.movement.minimum_edge_cost),
            "invalid road or edge cost rule",
        )?;
        Ok(())
    }

    pub fn valid_kingdom_name(&self, name: &str) -> bool {
        !name.is_empty()
            && name.trim() == name
            && name.chars().count() <= self.kingdom_name_max_chars
    }
}
