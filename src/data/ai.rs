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
    pub minimum_formations: usize,
    pub war_peace_rounds: u32,
    pub neutral_search_edges: usize,
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
            || !(1..=6).contains(&self.minimum_formations)
            || !(1..=80).contains(&self.war_peace_rounds)
            || !(1..=16).contains(&self.neutral_search_edges)
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
