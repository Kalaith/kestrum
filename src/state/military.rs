//! Stable army and formation identities with exactly six optional formation slots.

mod cleanup;
mod setup;
mod validation;

use super::people::PersonId;
use crate::data::{
    economy::{Resources, TroopKind},
    world::{FactionId, SiteId},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ArmyId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormationId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Army {
    pub id: ArmyId,
    pub faction: FactionId,
    pub site: SiteId,
    pub name: String,
    pub slots: [Option<FormationId>; 6],
    pub commander: Option<PersonId>,
}

impl Army {
    pub fn formation_ids(&self) -> impl Iterator<Item = FormationId> + '_ {
        self.slots.iter().flatten().copied()
    }

    pub fn first_empty_slot(&self) -> Option<usize> {
        self.slots.iter().position(Option::is_none)
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Formation {
    pub id: FormationId,
    pub faction: FactionId,
    pub kind: TroopKind,
    pub headcount: u32,
    pub capacity: u32,
    pub movement_spent: u32,
    pub created_round: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EconomyStatement {
    pub completed_rounds: u32,
    pub income: Resources,
    pub upkeep_due: i64,
    pub upkeep_paid: i64,
    pub shortfall: i64,
    pub closing: Resources,
}
