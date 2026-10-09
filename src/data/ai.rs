//! Authored, deterministic sovereign planning limits.

use super::economy::TroopKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SOURCE: &str = "assets/data/ai.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRules {
    pub schema_version: u32,
    pub max_commands_per_phase: u32,
    pub objective_rounds: u32,
    pub retreat_capacity_percent: u32,
    pub attack_advantage_percent: u32,
    pub unknown_enemy_power: u32,
    pub reserve_upkeep_rounds: u32,
    pub target_armies: usize,
    /// Developed, controlled settlements needed to support one army target step.
    pub sites_per_army: usize,
    /// Hard cap for holdings, relief and threatened-front army targets.
    pub maximum_armies: usize,
    pub minimum_formations: usize,
    pub war_peace_rounds: u32,
    /// A sovereign opens no new war while this many neighbors are already enemies.
    pub maximum_wars: usize,
    /// Own aggregate strength, as a percentage of a neighbor's, needed to declare war on it.
    pub war_strength_percent: u32,
    /// Furthest route edges from the capital at which unclaimed land is taken.
    pub expansion_reach: usize,
    /// Weight of capital distance against army travel when ranking new land.
    pub home_distance_percent: u32,
    pub recruitment_order: Vec<TroopKind>,
}

impl AiRules {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || !(1..=64).contains(&self.max_commands_per_phase)
            || !(1..=80).contains(&self.objective_rounds)
            || !(1..=100).contains(&self.retreat_capacity_percent)
            || !(101..=1000).contains(&self.attack_advantage_percent)
            || !(1..=100_000).contains(&self.unknown_enemy_power)
            || !(1..=20).contains(&self.reserve_upkeep_rounds)
            || !(1..=8).contains(&self.target_armies)
            || !(1..=256).contains(&self.sites_per_army)
            || !(self.target_armies..=8).contains(&self.maximum_armies)
            || !(1..=6).contains(&self.minimum_formations)
            || !(1..=80).contains(&self.war_peace_rounds)
            || !(1..=7).contains(&self.maximum_wars)
            || !(1..=1000).contains(&self.war_strength_percent)
            || !(1..=32).contains(&self.expansion_reach)
            || self.home_distance_percent > 1000
            || self.recruitment_order.len() != 6
            || self
                .recruitment_order
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len()
                != 6
        {
            return Err(format!(
                "{SOURCE}: invalid planning bounds or recruitment order"
            ));
        }
        Ok(())
    }
}
