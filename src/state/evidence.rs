//! Gameplay evidence survives narrative pruning and belongs to its real subject.

mod validation;

use super::{military::FormationId, people::PersonId};
use crate::data::{
    economy::TroopKind,
    progression::FormationSpecialization,
    progression::ProgressionRules,
    world::{FactionId, RouteId, SiteId},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Veterancy {
    #[default]
    Ordinary,
    Seasoned,
    Veteran,
}

impl Veterancy {
    pub fn from_xp(xp: u32, rules: &ProgressionRules) -> Self {
        if xp >= rules.veteran_xp {
            Self::Veteran
        } else if xp >= rules.seasoned_xp {
            Self::Seasoned
        } else {
            Self::Ordinary
        }
    }
    pub fn permille(self, rules: &ProgressionRules) -> u32 {
        match self {
            Self::Ordinary => rules.ordinary_permille,
            Self::Seasoned => rules.seasoned_permille,
            Self::Veteran => rules.veteran_permille,
        }
    }
}
impl fmt::Display for Veterancy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Ordinary => "Ordinary",
            Self::Seasoned => "Seasoned",
            Self::Veteran => "Veteran",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Battle,
    MeaningfulEncounter,
    Victory,
    Defeat,
    SurvivedOutnumbered,
    DefendedAnchor,
    CapturedAnchor,
    Retreated,
    TreatedWounded,
    CommanderWounded,
    AssumedCommand,
    CommandedVictory,
    RetreatingEnemyVictory,
    AssaultedFort,
    DefendedFort,
    Sortie,
    EscapeAttempt,
    Relief,
    EncounteredBandits,
    EncounteredWildlife,
    ClearedThreat,
}

impl fmt::Display for EvidenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Battle => "Battles witnessed",
            Self::MeaningfulEncounter => "Meaningful encounters",
            Self::Victory => "Victories",
            Self::Defeat => "Defeats",
            Self::SurvivedOutnumbered => "Survived outnumbered",
            Self::DefendedAnchor => "Defended an anchor",
            Self::CapturedAnchor => "Captured an anchor",
            Self::Retreated => "Retreated",
            Self::TreatedWounded => "Treated wounded",
            Self::CommanderWounded => "Commander wounded",
            Self::AssumedCommand => "Assumed command",
            Self::CommandedVictory => "Commanded victory",
            Self::RetreatingEnemyVictory => "Victories over retreating enemies",
            Self::AssaultedFort => "Assaulted a fort",
            Self::DefendedFort => "Defended a fort",
            Self::Sortie => "Fought a sortie",
            Self::EscapeAttempt => "Attempted a siege escape",
            Self::Relief => "Fought to relieve a siege",
            Self::EncounteredBandits => "Encountered bandits",
            Self::EncounteredWildlife => "Encountered wildlife",
            Self::ClearedThreat => "Cleared a local threat",
        })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceLedger {
    pub counts: BTreeMap<EvidenceKind, u32>,
    pub encountered_troops: BTreeSet<TroopKind>,
    pub meaningful_against: BTreeMap<TroopKind, u32>,
    pub service_by_troop: BTreeMap<TroopKind, u32>,
    pub traversed_routes: BTreeSet<RouteId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MovementService {
    pub formations: Vec<FormationId>,
    pub people: Vec<PersonId>,
    pub routes: Vec<RouteId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormationService {
    #[serde(default)]
    pub specialization: Option<FormationSpecialization>,
    #[serde(default)]
    pub course: Option<FormationCourse>,
    pub xp: u32,
    pub tier: Veterancy,
    pub ledger: EvidenceLedger,
    pub recent: Vec<SeasonService>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormationCourse {
    pub target: FormationSpecialization,
    pub site: SiteId,
    pub steps_completed: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeasonService {
    pub completed_rounds: u32,
    pub xp: u32,
    pub encounters: Vec<EncounterService>,
    #[serde(default)]
    pub routes: BTreeSet<RouteId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounterService {
    pub site: SiteId,
    pub opponent: EncounterOpponent,
    pub tags: BTreeSet<EvidenceKind>,
    pub enemy_types: BTreeSet<TroopKind>,
    pub meaningful: bool,
    pub xp: u32,
}

/// Earlier sovereign-opponent numbers retain their representation on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum EncounterOpponent {
    Faction(FactionId),
    Threat { threat: super::threat::ThreatId },
}
