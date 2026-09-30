//! Authored rules for deterministic formation battles and their legal tactic blocks.

use super::{economy::unique_table, economy::TroopKind, world::PersonClass};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SOURCE: &str = "assets/data/battle_tactics.json";
pub const DEFAULT_MAX_TACTIC_ROWS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticTrigger {
    Activation,
    IncomingAttack,
    EndOfRound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleDoctrine {
    DefensiveLine,
    RangedSupport,
    Breakthrough,
}

impl BattleDoctrine {
    pub const ALL: [Self; 3] = [Self::DefensiveLine, Self::RangedSupport, Self::Breakthrough];
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
    Rally,
    HoldTheLine,
    Stabilize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleCapability {
    OfficerRally,
    InfantryHoldTheLine,
    CavalryExploitOpening,
    MedicStabilization,
    SiegeBombardment,
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
    AllyBelowHalfMorale,
    SelfBelowHalfMorale,
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
    AllyLowestMorale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetPriority {
    OwnColumnFirst,
    LowestStrength,
    HighestStrength,
    LowestResistance,
    LowestMorale,
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
    pub rally_morale_restore: u32,
    pub medic_morale_restore: u32,
    pub cavalry_opening_damage_permille: u32,
    pub opening_bombardment_damage_permille: u32,
    pub siege_engine_close_damage_permille: u32,
    #[serde(deserialize_with = "unique_table")]
    pub defaults: BTreeMap<TroopKind, TroopTactics>,
    #[serde(deserialize_with = "unique_table")]
    pub doctrines: BTreeMap<BattleDoctrine, BTreeMap<TroopKind, TroopTactics>>,
}

impl BattleTacticsRules {
    pub fn validate(&self) -> Result<(), String> {
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
            || self.rally_morale_restore == 0
            || self.rally_morale_restore > 100
            || self.medic_morale_restore == 0
            || self.medic_morale_restore > 100
            || [
                self.charge_damage_permille,
                self.breakthrough_damage_permille,
                self.brace_damage_permille,
                self.brace_retaliation_permille,
                self.guard_damage_permille,
                self.cavalry_opening_damage_permille,
                self.opening_bombardment_damage_permille,
                self.siege_engine_close_damage_permille,
            ]
            .iter()
            .any(|factor| *factor > 3000)
            || [
                self.charge_damage_permille,
                self.breakthrough_damage_permille,
                self.brace_damage_permille,
                self.brace_retaliation_permille,
                self.guard_damage_permille,
                self.cavalry_opening_damage_permille,
                self.opening_bombardment_damage_permille,
                self.siege_engine_close_damage_permille,
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
                || tactics.activation.len() > DEFAULT_MAX_TACTIC_ROWS
                || tactics.reaction.len() > DEFAULT_MAX_TACTIC_ROWS
            {
                return Err(format!("{SOURCE}: invalid tactic count for {kind:?}"));
            }
            self.validate_configuration(*kind, tactics)?;
        }
        if self.doctrines.len() != BattleDoctrine::ALL.len()
            || BattleDoctrine::ALL
                .iter()
                .any(|doctrine| !self.doctrines.contains_key(doctrine))
        {
            return Err(format!("{SOURCE}: doctrines must define all three presets"));
        }
        for (doctrine, tactics) in &self.doctrines {
            if tactics.keys().any(|kind| !self.defaults.contains_key(kind)) {
                return Err(format!("{SOURCE}: unknown troop kind in {doctrine:?}"));
            }
            for (kind, rules) in tactics {
                self.validate_configuration(*kind, rules)?;
            }
        }
        Ok(())
    }

    pub fn validate_configuration(
        &self,
        kind: TroopKind,
        tactics: &TroopTactics,
    ) -> Result<(), String> {
        if tactics.activation.len() > self.max_tactic_rows as usize
            || tactics.reaction.len() > self.max_tactic_rows as usize
        {
            return Err(format!("{SOURCE}: too many tactic rows for {kind:?}"));
        }
        validate_rows(kind, &tactics.activation, TacticTrigger::Activation)?;
        validate_rows(kind, &tactics.reaction, TacticTrigger::IncomingAttack)
    }

    pub fn defaults_for(&self, kind: TroopKind) -> Option<&TroopTactics> {
        self.defaults.get(&kind)
    }

    /// Doctrines replace only their authored role blocks; omitted roles use
    /// the legal troop defaults and are still snapshotted on application.
    pub fn doctrine_for(&self, doctrine: BattleDoctrine, kind: TroopKind) -> Option<&TroopTactics> {
        self.doctrines
            .get(&doctrine)
            .and_then(|rules| rules.get(&kind))
            .or_else(|| self.defaults_for(kind))
    }
}

fn validate_rows(
    kind: TroopKind,
    rows: &[TacticRule],
    expected: TacticTrigger,
) -> Result<(), String> {
    use TacticAction::*;
    use TacticTrigger::*;
    let mut ids = std::collections::BTreeSet::new();
    for tactic in rows {
        let action_matches = match (expected, tactic.action) {
            (Activation, Brace) | (IncomingAttack, Attack | Volley | Charge | Breakthrough) => {
                false
            }
            (Activation, _) | (IncomingAttack, Brace | Guard) => true,
            _ => false,
        };
        let target_matches = match tactic.action {
            Attack | Volley | Charge | Breakthrough => {
                tactic.target_filter != TargetFilter::None
                    && tactic.target_filter != TargetFilter::AllyLowestMorale
            }
            Stabilize => tactic.target_filter == TargetFilter::AllyLowestMorale,
            Brace | Guard | Wait | Advance | Rally | HoldTheLine => {
                tactic.target_filter == TargetFilter::None
            }
        };
        if tactic.id.trim().is_empty()
            || tactic.id.len() > 64
            || !ids.insert(tactic.id.as_str())
            || tactic.trigger != expected
            || !action_matches
            || !target_matches
            || (expected == Activation
                && tactic.condition == TacticCondition::IncomingCavalryCharge)
            || (expected == IncomingAttack
                && !matches!(
                    tactic.condition,
                    TacticCondition::Always | TacticCondition::IncomingCavalryCharge
                ))
            || (tactic.action == Breakthrough && tactic.target_filter == TargetFilter::EnemyFront)
            || (tactic.action == Stabilize && kind != TroopKind::Medics)
            || (tactic.action == HoldTheLine
                && !matches!(kind, TroopKind::Warriors | TroopKind::Spearmen))
        {
            return Err(format!(
                "{SOURCE}: invalid tactic {:?} for {kind:?}",
                tactic.id
            ));
        }
    }
    Ok(())
}

/// Leader classes grant one bounded ability to a compatible troop role.
pub fn leader_capabilities(kind: TroopKind, class: Option<PersonClass>) -> Vec<BattleCapability> {
    use BattleCapability::*;
    let mut capabilities = Vec::new();
    match (kind, class) {
        (_, Some(PersonClass::Officer)) => capabilities.push(OfficerRally),
        (TroopKind::Warriors | TroopKind::Spearmen, Some(PersonClass::Infantry)) => {
            capabilities.push(InfantryHoldTheLine)
        }
        (TroopKind::Riders, Some(PersonClass::Cavalry)) => capabilities.push(CavalryExploitOpening),
        _ => {}
    }
    match kind {
        TroopKind::Medics => capabilities.push(MedicStabilization),
        TroopKind::SiegeEngines => capabilities.push(SiegeBombardment),
        _ => {}
    }
    capabilities
}
