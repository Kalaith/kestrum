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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryEntry {
    pub army: ArmyId,
    pub formation: FormationId,
    pub site: SiteId,
    /// Historical rule inputs remain meaningful if this formation is later removed.
    pub kind: TroopKind,
    pub capacity: u32,
    pub headcount_before: u32,
    pub restored: u32,
    pub gold_cost: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecoveryStatement {
    pub completed_rounds: u32,
    /// The post-upkeep balance, before any formation recovers.
    pub opening_gold: i64,
    pub closing_gold: i64,
    pub gold_spent: i64,
    pub restored: u64,
    pub entries: Vec<RecoveryEntry>,
}

impl super::StrategicCampaign {
    /// Every attached member constrains the army; transfers never reset spent movement.
    pub fn army_movement_remaining(
        &self,
        army: ArmyId,
        data: &crate::data::GameData,
    ) -> Option<u32> {
        let army = self.armies.get(&army)?;
        let mut remaining = None;
        for id in army.formation_ids() {
            let formation = self.formations.get(&id)?;
            let allowance = data
                .economy
                .formations
                .get(&formation.kind)?
                .movement_allowance;
            let available = allowance.saturating_sub(formation.movement_spent);
            remaining = Some(remaining.map_or(available, |minimum: u32| minimum.min(available)));
        }
        let mut remaining = remaining?;
        for person in self.people.values() {
            if matches!(person.assignment, super::people::PersonAssignment::Formation { formation }
                if army.formation_ids().any(|id| id == formation))
            {
                remaining = remaining.min(
                    data.rules
                        .leadership
                        .officer_movement_allowance
                        .saturating_sub(person.movement_spent),
                );
            }
        }
        Some(remaining)
    }
}
