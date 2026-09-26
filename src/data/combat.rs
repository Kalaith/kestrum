//! Authored battle arithmetic; troop capacities and prices stay in economy data.

use super::{
    economy::TroopKind,
    world::{Geography, Site, SiteTag},
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TroopCombat {
    pub attack: u32,
    pub resistance: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Troops {
    pub formations: BTreeMap<TroopKind, TroopCombat>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counter {
    pub source: TroopKind,
    pub target: TroopKind,
    pub permille: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatRules {
    pub max_exchanges: u32,
    pub casualty_divisor: u32,
    pub rout_permille: u32,
    pub victory_margin_permille: u32,
    pub ordinary_terrain_permille: u32,
    pub forest_hill_permille: u32,
    pub bridge_pass_permille: u32,
    pub counters: Vec<Counter>,
    pub field_damage: u32,
    pub capture_damage: u32,
    pub capture_occupation: u32,
    pub wipe_death_percent: u32,
    pub commander_loss_percent: u32,
    pub commander_wound_percent: u32,
    pub wound_recovery_steps: u32,
}

impl Troops {
    pub fn validate(&self) -> Result<(), String> {
        use TroopKind::*;
        if self.formations.len() != 6
            || [Warriors, Spearmen, Archers, Riders, Medics, SiegeEngines]
                .iter()
                .any(|kind| !self.formations.contains_key(kind))
        {
            return Err("troops.json: requires every supported troop type".into());
        }
        if self.formations.values().any(|value| {
            value.attack == 0
                || value.resistance == 0
                || value.attack > 1_000_000
                || value.resistance > 1_000_000
        }) {
            return Err("troops.json: attack/resistance must be 1..=1000000".into());
        }
        Ok(())
    }
}

impl CombatRules {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=8).contains(&self.max_exchanges)
            || self.casualty_divisor == 0
            || self.casualty_divisor > 1_000_000
            || self.rout_permille > 1000
            || self.victory_margin_permille > 1000
        {
            return Err("combat_rules.json: invalid exchange, loss or victory rule".into());
        }
        if [
            self.ordinary_terrain_permille,
            self.forest_hill_permille,
            self.bridge_pass_permille,
        ]
        .iter()
        .any(|value| !(1..=10_000).contains(value))
        {
            return Err("combat_rules.json: terrain factors must be 1..=10000".into());
        }
        let mut pairs = std::collections::BTreeSet::new();
        for counter in &self.counters {
            if !(1..=10_000).contains(&counter.permille)
                || !pairs.insert((counter.source, counter.target))
            {
                return Err("combat_rules.json: invalid or duplicate troop counter".into());
            }
        }
        if [
            self.field_damage,
            self.capture_damage,
            self.capture_occupation,
            self.wipe_death_percent,
            self.commander_loss_percent,
            self.commander_wound_percent,
        ]
        .iter()
        .any(|value| *value > 100)
            || self.wound_recovery_steps == 0
        {
            return Err(
                "combat_rules.json: invalid damage, wound chance or recovery duration".into(),
            );
        }
        Ok(())
    }

    pub fn counter(&self, source: TroopKind, target: TroopKind) -> u32 {
        self.counters
            .iter()
            .find(|entry| entry.source == source && entry.target == target)
            .map_or(1000, |entry| entry.permille)
    }

    pub fn terrain(&self, site: &Site) -> u32 {
        if site.tags.contains(&SiteTag::Bridge) || site.tags.contains(&SiteTag::Pass) {
            self.bridge_pass_permille
        } else if matches!(site.geography, Geography::Forest | Geography::Hill) {
            self.forest_hill_permille
        } else {
            self.ordinary_terrain_permille
        }
    }
}
