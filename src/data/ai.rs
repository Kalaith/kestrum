//! Authored, deterministic sovereign planning limits.

use super::economy::TroopKind;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const SOURCE: &str = "assets/data/ai.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiRules {
    pub schema_version: u32,
    pub max_commands_per_phase: u32,
    pub objective_rounds: u32,
    pub retreat_capacity_percent: u32,
    pub unknown_enemy_power: u32,
    pub reserve_upkeep_rounds: u32,
    pub target_armies: usize,
    /// Developed, controlled settlements needed to support one army target step.
    pub sites_per_army: usize,
    /// Hard cap for holdings, relief and threatened-front army targets.
    pub maximum_armies: usize,
    pub minimum_formations: usize,
    /// Seasonal gold income, as a percentage of upkeep, a realm needs before it
    /// stops claiming land beyond its temperament's expansion reach.
    pub surplus_percent: u32,
    pub recruitment_order: Vec<TroopKind>,
    /// Every sovereign temperament's planning profile.
    #[serde(deserialize_with = "super::economy::unique_table")]
    pub personalities: BTreeMap<AiPersonality, AiPersonalityProfile>,
}

/// A sovereign's temperament, fixed when the campaign begins.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum AiPersonality {
    #[default]
    Balanced,
    Aggressive,
    Defensive,
    Peaceful,
    Expansionist,
}

impl AiPersonality {
    pub const ALL: [Self; 5] = [
        Self::Balanced,
        Self::Aggressive,
        Self::Defensive,
        Self::Peaceful,
        Self::Expansionist,
    ];

    /// Presentation text key; `<key>_help` describes how the temperament plays.
    pub fn text_key(self) -> &'static str {
        match self {
            Self::Balanced => "temperament_balanced",
            Self::Aggressive => "temperament_aggressive",
            Self::Defensive => "temperament_defensive",
            Self::Peaceful => "temperament_peaceful",
            Self::Expansionist => "temperament_expansionist",
        }
    }

    /// Distinct temperaments for the kingdoms of one seed, in setup order.
    pub fn for_campaign(seed: u64, index: usize) -> Self {
        let offset =
            macroquad_toolkit::rng::SeededRng::new(seed).next_u64() % Self::ALL.len() as u64;
        Self::ALL[(offset as usize + index) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiPersonalityProfile {
    /// Own observed power, as a percentage of a defender's estimate, needed to attack it.
    pub attack_advantage_percent: u32,
    /// Seasons of peace before this sovereign will open a new war on a neighbor.
    pub war_peace_rounds: u32,
    /// No new war opens while this many neighbors are already enemies; zero never declares.
    pub maximum_wars: usize,
    /// Own aggregate strength, as a percentage of a neighbor's, needed to declare war on it.
    pub war_strength_percent: u32,
    /// Declares war even while unclaimed land still joins the home border.
    pub war_while_land_remains: bool,
    /// Furthest route edges from the capital at which unclaimed land is taken.
    pub expansion_reach: usize,
    /// Weight of capital distance against army travel when ranking new land.
    pub home_distance_percent: u32,
    /// Armies wanted beyond the holdings target, still limited by income.
    pub extra_armies: usize,
    /// Posts armies on threatened borders before claiming or clearing land.
    pub guards_borders: bool,
    /// Offers and accepts peace in any war it is not winning by siege.
    pub seeks_peace: bool,
}

impl AiPersonalityProfile {
    fn valid(&self) -> bool {
        (101..=1000).contains(&self.attack_advantage_percent)
            && (1..=80).contains(&self.war_peace_rounds)
            && self.maximum_wars <= 7
            && (1..=1000).contains(&self.war_strength_percent)
            && (1..=32).contains(&self.expansion_reach)
            && self.home_distance_percent <= 1000
            && self.extra_armies <= 4
    }
}

impl AiRules {
    /// The planning profile for a temperament; validation guarantees every one exists.
    pub fn profile(&self, personality: AiPersonality) -> &AiPersonalityProfile {
        &self.personalities[&personality]
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 2
            || !(1..=64).contains(&self.max_commands_per_phase)
            || !(1..=80).contains(&self.objective_rounds)
            || !(1..=100).contains(&self.retreat_capacity_percent)
            || !(1..=100_000).contains(&self.unknown_enemy_power)
            || !(1..=20).contains(&self.reserve_upkeep_rounds)
            || !(1..=8).contains(&self.target_armies)
            || !(1..=256).contains(&self.sites_per_army)
            || !(self.target_armies..=8).contains(&self.maximum_armies)
            || !(1..=6).contains(&self.minimum_formations)
            || !(100..=1000).contains(&self.surplus_percent)
            || AiPersonality::ALL
                .iter()
                .any(|kind| !self.personalities.get(kind).is_some_and(|p| p.valid()))
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
                "{SOURCE}: invalid planning bounds, personalities or recruitment order"
            ));
        }
        Ok(())
    }
}
