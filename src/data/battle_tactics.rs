//! Authored rules for deterministic formation battles and their legal tactic blocks.

use super::{economy::unique_table, economy::TroopKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SOURCE: &str = "assets/data/battle_tactics.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticTrigger {
    Activation,
    IncomingAttack,
    EndOfRound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticAction {
    Attack,
    Volley,
    Charge,
    Breakthrough,
    Guard,
    Brace,
    Wait,
    Advance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticCondition {
    Always,
    EnemyRearExposed,
    EnemyCavalryPresent,
    FirstActivation,
    SelfBelowHalf,
    AllyInSameRowBelowHalf,
    IncomingCavalryCharge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetFilter {
    None,
    AnyEnemy,
    EnemyFront,
    EnemyRear,
    EnemyCavalry,
    ExposedEnemyRear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetPriority {
    OwnColumnFirst,
    LowestStrength,
    HighestStrength,
    LowestResistance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TacticRule {
    pub id: String,
    pub trigger: TacticTrigger,
    pub action: TacticAction,
    pub condition: TacticCondition,
    pub target_filter: TargetFilter,
    pub target_priority: TargetPriority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TroopTactics {
    pub activation: Vec<TacticRule>,
    pub reaction: Vec<TacticRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleTacticsRules {
    pub schema_version: u32,
    pub rules_revision: String,
    pub max_rounds: u32,
    pub max_armies_per_side: u32,
    pub max_activations_per_round: u32,
    pub max_tactic_rows: u32,
    pub max_reactions_per_unit_round: u32,
    pub victory_margin_permille: u32,
    pub casualty_divisor: u32,
    pub initial_morale: u32,
    pub morale_loss_per_casualty: u32,
    pub rout_morale: u32,
    pub ally_rout_morale_loss: u32,
    pub charge_damage_permille: u32,
    pub breakthrough_damage_permille: u32,
    pub brace_damage_permille: u32,
    pub brace_retaliation_permille: u32,
    pub guard_damage_permille: u32,
    #[serde(deserialize_with = "unique_table")]
    pub defaults: BTreeMap<TroopKind, TroopTactics>,
}

impl BattleTacticsRules {
    pub fn validate(&self) -> Result<(), String> {
        use TacticAction::*;
        use TacticTrigger::*;
        use TroopKind::*;

        if self.schema_version != 1
            || self.rules_revision.trim().is_empty()
            || !(1..=8).contains(&self.max_rounds)
            || !(1..=4).contains(&self.max_armies_per_side)
            || !(1..=48).contains(&self.max_activations_per_round)
            || !(1..=5).contains(&self.max_tactic_rows)
            || self.max_reactions_per_unit_round != 1
            || self.victory_margin_permille > 1000
            || self.casualty_divisor == 0
            || self.initial_morale == 0
            || self.initial_morale > 100
            || self.rout_morale >= self.initial_morale
            || self.morale_loss_per_casualty > 100
            || self.ally_rout_morale_loss > self.initial_morale
            || [
                self.charge_damage_permille,
                self.breakthrough_damage_permille,
                self.brace_damage_permille,
                self.brace_retaliation_permille,
                self.guard_damage_permille,
            ]
            .iter()
            .any(|factor| *factor > 3000)
            || [
                self.charge_damage_permille,
                self.breakthrough_damage_permille,
                self.brace_damage_permille,
                self.brace_retaliation_permille,
                self.guard_damage_permille,
            ]
            .contains(&0)
        {
            return Err(format!("{SOURCE}: invalid battle limits or modifiers"));
        }

        let supported = [Warriors, Spearmen, Archers, Riders, Medics, SiegeEngines];
        if self.defaults.len() != supported.len()
            || supported
                .iter()
                .any(|kind| !self.defaults.contains_key(kind))
        {
            return Err(format!("{SOURCE}: defaults must cover all six troop kinds"));
        }
        for (kind, tactics) in &self.defaults {
            if tactics.activation.is_empty()
                || tactics.activation.len() > self.max_tactic_rows as usize
                || tactics.reaction.len() > self.max_tactic_rows as usize
            {
                return Err(format!("{SOURCE}: invalid tactic count for {kind:?}"));
            }
            for (rows, expected) in [
                (&tactics.activation, Activation),
                (&tactics.reaction, IncomingAttack),
            ] {
                for tactic in rows {
                    let trigger_matches = tactic.trigger == expected;
                    let action_matches = match (expected, tactic.action) {
                        (Activation, _) => tactic.action != Brace,
                        (IncomingAttack, Brace | Guard) => true,
                        _ => false,
                    };
                    let target_matches = match tactic.action {
                        Attack | Volley | Charge | Breakthrough => {
                            tactic.target_filter != TargetFilter::None
                        }
                        Brace | Guard | Wait | Advance => {
                            tactic.target_filter == TargetFilter::None
                        }
                    };
                    if tactic.id.trim().is_empty()
                        || tactic.id.len() > 64
                        || !trigger_matches
                        || !action_matches
                        || !target_matches
                    {
                        return Err(format!(
                            "{SOURCE}: invalid tactic {:?} for {kind:?}",
                            tactic.id
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn defaults_for(&self, kind: TroopKind) -> Option<&TroopTactics> {
        self.defaults.get(&kind)
    }
}
