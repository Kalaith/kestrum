//! Immutable encounter receipts. Opening or projecting one never replays combat.

pub mod simulation;
mod validation;

use super::{
    military::{ArmyId, FormationId},
    people::{PersonAssignment, PersonCombatEvent, PersonId, PersonStatus},
};
use crate::data::{
    economy::TroopKind,
    world::{FactionId, FounderClass, RouteId, SiteId},
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
    #[serde(default)]
    pub context: BattleContext,
    #[serde(default = "ordinary_factor")]
    pub wall_permille: u32,
    #[serde(default)]
    pub fort_damage_added: u32,
    #[serde(default)]
    pub road_damage: Option<BattleRoadDamage>,
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
    pub defender: BattleDefender,
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
#[serde(
    tag = "kind",
    content = "side",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum BattleDefender {
    Faction(BattleSideReport),
    Threat(ThreatSideReport),
}

impl BattleDefender {
    pub fn faction(&self) -> Option<FactionId> {
        self.faction_side().map(|side| side.faction)
    }
    pub fn faction_side(&self) -> Option<&BattleSideReport> {
        match self {
            Self::Faction(side) => Some(side),
            Self::Threat(_) => None,
        }
    }
    pub fn faction_side_mut(&mut self) -> Option<&mut BattleSideReport> {
        match self {
            Self::Faction(side) => Some(side),
            Self::Threat(_) => None,
        }
    }
    pub fn armies(&self) -> &[BattleArmyReport] {
        self.faction_side().map_or(&[], |side| &side.armies)
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Faction(side) => &side.name,
            Self::Threat(side) => &side.name,
        }
    }
}

impl BattleReport {
    pub fn faction_sides(&self) -> impl Iterator<Item = &BattleSideReport> {
        std::iter::once(&self.attacker).chain(self.defender.faction_side())
    }
    pub fn participant_factions(&self) -> impl Iterator<Item = FactionId> + '_ {
        self.faction_sides().map(|side| side.faction)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreatSideReport {
    pub id: super::threat::ThreatId,
    pub kind: crate::data::threats::ThreatKind,
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub combat_losses: u32,
    pub encirclement_losses: u32,
    pub attack: u32,
    pub resistance: u32,
    pub payout: crate::data::economy::Resources,
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
    pub threat_losses: u32,
    #[serde(default = "ordinary_factor")]
    pub wall_permille: u32,
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum BattleContext {
    #[default]
    Field,
    Assault {
        siege: super::siege::SiegeId,
    },
    Sortie {
        siege: super::siege::SiegeId,
    },
    Escape {
        siege: super::siege::SiegeId,
        destination: SiteId,
    },
    Relief {
        siege: super::siege::SiegeId,
        garrison: Vec<ArmyId>,
    },
    BesiegerClash {
        siege: super::siege::SiegeId,
        garrison_faction: FactionId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleRoadDamage {
    pub route: RouteId,
    pub added: u32,
}
