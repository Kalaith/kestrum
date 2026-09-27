//! Persistent wall resistance and assault damage from the P14/P15 rules.

use super::validation::require;
use serde::{Deserialize, Serialize};

pub const SOURCE: &str = "assets/data/siege_rules.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiegeRules {
    pub schema_version: u32,
    pub wall_base_permille: u32,
    pub wall_minimum_permille: u32,
    pub wall_step_reduction_permille: u32,
    pub wall_damage_reduction_permille: u32,
    pub engine_wall_reduction_permille: u32,
    pub fort_damage_per_step: u32,
    pub assault_structural_damage: u32,
    pub assault_fort_damage: u32,
    pub assault_road_damage: u32,
    pub assault_road_loss_permille: u32,
}

impl SiegeRules {
    pub fn validate(&self) -> Result<(), String> {
        require(
            SOURCE,
            "schema_version",
            self.schema_version == 1,
            "must be 1",
        )?;
        require(
            SOURCE,
            "wall_minimum_permille",
            self.wall_minimum_permille == 1000,
            "must be neutral resistance (1000)",
        )?;
        require(
            SOURCE,
            "wall_base_permille",
            (self.wall_minimum_permille..=10_000).contains(&self.wall_base_permille),
            "must be between neutral resistance and 10000",
        )?;
        for (field, value) in [
            (
                "wall_step_reduction_permille",
                self.wall_step_reduction_permille,
            ),
            (
                "wall_damage_reduction_permille",
                self.wall_damage_reduction_permille,
            ),
            (
                "engine_wall_reduction_permille",
                self.engine_wall_reduction_permille,
            ),
        ] {
            require(
                SOURCE,
                field,
                (1..=10_000).contains(&value),
                "must be within 1..=10000",
            )?;
        }
        require(
            SOURCE,
            "fort_damage_per_step",
            (1..=100).contains(&self.fort_damage_per_step),
            "must be within 1..=100",
        )?;
        for (field, value) in [
            ("assault_structural_damage", self.assault_structural_damage),
            ("assault_fort_damage", self.assault_fort_damage),
            ("assault_road_damage", self.assault_road_damage),
        ] {
            require(
                SOURCE,
                field,
                value <= 100,
                "must be a percentage within 0..=100",
            )?;
        }
        require(
            SOURCE,
            "assault_road_loss_permille",
            (1..=1000).contains(&self.assault_road_loss_permille),
            "must be within 1..=1000",
        )
    }

    /// The engine flag is a single surviving formation condition, never a count.
    pub fn wall_permille(&self, elapsed_steps: u32, fort_damage: u32, has_engines: bool) -> u32 {
        if fort_damage >= 100 {
            return self.wall_minimum_permille;
        }
        let reduction = u64::from(elapsed_steps) * u64::from(self.wall_step_reduction_permille)
            + u64::from(fort_damage) * u64::from(self.wall_damage_reduction_permille)
            + if has_engines {
                u64::from(self.engine_wall_reduction_permille)
            } else {
                0
            };
        u64::from(self.wall_base_permille)
            .saturating_sub(reduction)
            .max(u64::from(self.wall_minimum_permille)) as u32
    }
}
