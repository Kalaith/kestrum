//! Immutable encounter receipts. Opening or projecting one never replays combat.

mod validation;

use super::{
    military::{ArmyId, FormationId},
    people::{PersonAssignment, PersonCombatEvent, PersonId, PersonStatus},
};
use crate::data::{
    economy::TroopKind,
    world::{FactionId, FounderClass, SiteId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BattleId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleOutcome {
    AttackerVictory,
    DefenderVictory,
    Stalemate,
    MutualDestruction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleEndReason {
    Annihilation,
    Rout,
    ExchangeLimit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleReport {
    pub id: BattleId,
    pub completed_rounds: u32,
    pub sequence: u64,
    pub site: SiteId,
    pub site_name: String,
    pub origin: SiteId,
    pub outcome: BattleOutcome,
    pub reason: BattleEndReason,
    pub exchanges: Vec<BattleExchange>,
    pub attacker: BattleSideReport,
    pub defender: BattleSideReport,
    pub terrain_permille: u32,
    pub counters: Vec<CounterUse>,
    pub control_before: Option<FactionId>,
    pub control_after: Option<FactionId>,
    pub structural_damage_added: u32,
    pub occupation_after: u32,
    pub person_events: Vec<PersonCombatEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleSideReport {
    pub faction: FactionId,
    pub name: String,
    pub armies: Vec<BattleArmyReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleArmyReport {
    pub id: ArmyId,
    pub name: String,
    pub leadership_permille: u32,
    pub commander: Option<BattleCommander>,
    pub people: Vec<BattlePersonReport>,
    pub formations: Vec<BattleFormationReport>,
    pub final_site: Option<SiteId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleCommander {
    pub id: PersonId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattlePersonReport {
    pub id: PersonId,
    pub starting_formation: FormationId,
    /// Earlier reports did not record starting fitness; absence grants no treatment evidence.
    #[serde(default)]
    pub starting_status: Option<PersonStatus>,
    pub name: String,
    pub class: FounderClass,
    pub status: PersonStatus,
    pub assignment: PersonAssignment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleFormationReport {
    #[serde(default = "ordinary_factor")]
    pub veterancy_permille: u32,
    pub id: FormationId,
    pub slot: usize,
    pub kind: TroopKind,
    pub start: u32,
    pub end: u32,
    pub combat_losses: u32,
    pub encirclement_losses: u32,
}

fn ordinary_factor() -> u32 {
    1000
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleExchange {
    pub number: u32,
    pub losses: Vec<FormationLoss>,
    pub leadership: Vec<ArmyLeadership>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmyLeadership {
    pub army: ArmyId,
    pub permille: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormationLoss {
    pub formation: FormationId,
    pub amount: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterUse {
    pub source: TroopKind,
    pub target: TroopKind,
    pub permille: u32,
}
