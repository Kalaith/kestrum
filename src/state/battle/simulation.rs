//! Serializable opening snapshots, runtime deltas and outcomes for battle playback.

use crate::{
    data::{battle_tactics::TacticAction, economy::TroopKind, world::FactionId},
    state::{
        military::{ArmyId, FormationId},
        threat::ThreatId,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleSide {
    Attacker,
    Defender,
}

impl BattleSide {
    pub fn opposing(self) -> Self {
        match self {
            Self::Attacker => Self::Defender,
            Self::Defender => Self::Attacker,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum BattleUnitId {
    Formation(FormationId),
    Threat(ThreatId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattlePosition {
    pub army: ArmyId,
    pub slot: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormationBattleInput {
    pub terrain_permille: u32,
    pub armies: Vec<BattleArmyInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleArmyInput {
    pub id: ArmyId,
    pub faction: FactionId,
    pub side: BattleSide,
    pub slots: [Option<BattleUnitInput>; 6],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleUnitInput {
    pub id: BattleUnitId,
    pub kind: Option<TroopKind>,
    pub headcount: u32,
    pub capacity: u32,
    pub attack: u32,
    pub resistance: u32,
    pub initiative: u32,
    pub activation_tactics: Vec<crate::data::battle_tactics::TacticRule>,
    pub reaction_tactics: Vec<crate::data::battle_tactics::TacticRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleFinalUnit {
    pub id: BattleUnitId,
    pub army: ArmyId,
    pub opening_slot: u8,
    pub slot: Option<u8>,
    pub headcount: u32,
    pub morale: u32,
    pub is_routed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleResolutionReason {
    Annihilation,
    Rout,
    RoundLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TacticSkipReason {
    ConditionFalse,
    NoLegalTarget,
    ActionUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkippedTactic {
    pub rule_id: String,
    pub reason: TacticSkipReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BattleEvent {
    RoundStarted {
        round: u32,
    },
    Activation {
        actor: BattleUnitId,
        action: TacticAction,
        rule_id: Option<String>,
        target: Option<BattleUnitId>,
        skipped: Vec<SkippedTactic>,
    },
    Reaction {
        actor: BattleUnitId,
        against: BattleUnitId,
        rule_id: String,
        damage_reduction_permille: u32,
        retaliation: u32,
    },
    Damage {
        source: BattleUnitId,
        target: BattleUnitId,
        amount: u32,
        remaining: u32,
    },
    MoraleChanged {
        unit: BattleUnitId,
        before: u32,
        after: u32,
    },
    GuardRaised {
        unit: BattleUnitId,
        expires_round: u32,
    },
    PositionChanged {
        unit: BattleUnitId,
        from: BattlePosition,
        to: BattlePosition,
    },
    Routed {
        unit: BattleUnitId,
        position: BattlePosition,
        survivors: u32,
    },
    BattleEnded {
        outcome: super::BattleOutcome,
        reason: BattleResolutionReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleResolution {
    pub resolver_version: u32,
    pub rules_revision: String,
    pub opening: FormationBattleInput,
    pub opening_morale: u32,
    pub completed_rounds: u32,
    pub outcome: super::BattleOutcome,
    pub reason: BattleResolutionReason,
    pub units: Vec<BattleFinalUnit>,
    pub events: Vec<BattleEvent>,
}
