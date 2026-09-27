//! Persistent people, field fitness, and released assignments after death.

mod career;
mod combat;
mod validation;
pub use career::{
    Disposition, EmergenceRecord, PersonCareer, PersonCourse, PersonRelationship, PersonTrait,
    Recognition, Tendency,
};
pub use combat::{PersonCombatEvent, PersonCombatOutcome, PersonDeathReason, WoundCause};

use super::{
    military::{ArmyId, FormationId},
    StrategicCampaign,
};
use crate::data::{
    world::{FactionId, FounderClass, SiteId},
    GameData,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersonAssignment {
    Formation { formation: FormationId },
    Site { site: SiteId },
    Dead,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersonStatus {
    #[default]
    Fit,
    Displaced {
        completed_rounds: u32,
        site: SiteId,
    },
    Wounded {
        since_round: u32,
        remaining_steps: u32,
    },
    Dead {
        completed_rounds: u32,
        site: SiteId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Person {
    #[serde(default)]
    pub career: PersonCareer,
    pub evidence: super::evidence::EvidenceLedger,
    pub id: PersonId,
    pub faction: FactionId,
    pub name: String,
    /// Campaign-relative season; founding adults were born before season zero.
    pub birth_round: i64,
    pub service_start_round: u32,
    pub class: FounderClass,
    pub assignment: PersonAssignment,
    pub movement_spent: u32,
    /// Earlier saves simulated only living, fit people; no past injury is invented.
    #[serde(default)]
    pub status: PersonStatus,
}

impl Person {
    pub fn is_alive(&self) -> bool {
        !matches!(self.status, PersonStatus::Dead { .. })
    }

    pub fn is_fit_for_field(&self, completed_rounds: u32, minimum_age: u32) -> bool {
        self.status == PersonStatus::Fit && self.age_years(completed_rounds) >= minimum_age
    }

    pub fn age_years(&self, completed_rounds: u32) -> u32 {
        let completed_rounds = match self.status {
            PersonStatus::Dead {
                completed_rounds: death_round,
                ..
            } => completed_rounds.min(death_round),
            _ => completed_rounds,
        };
        i64::from(completed_rounds)
            .saturating_sub(self.birth_round)
            .saturating_div(4)
            .clamp(0, i64::from(u32::MAX)) as u32
    }
}

impl StrategicCampaign {
    /// Only living, fit adults attached to a surviving formation contribute.
    pub fn army_leadership_permille(&self, army: ArmyId, data: &GameData) -> Option<u32> {
        let army = self.armies.get(&army)?;
        let rules = &data.rules.leadership;
        let contributes = |person: &Person| {
            person.faction == army.faction
                && !person.career.retired
                && person.is_fit_for_field(self.completed_rounds, rules.field_min_age_years)
                && matches!(person.assignment, PersonAssignment::Formation { formation }
                    if army.formation_ids().any(|id| id == formation)
                    && self.formations.get(&formation).is_some_and(|entry| entry.headcount > 0))
        };
        let people = self
            .people
            .values()
            .filter(|person| contributes(person))
            .count() as u64;
        let leadership = u64::from(rules.base_permille)
            + u64::from(rules.named_scale_permille) * people
                / (people + u64::from(rules.diminishing_count));
        let commander = army
            .commander
            .and_then(|id| self.people.get(&id))
            .is_some_and(|person| person.class == FounderClass::Officer && contributes(person));
        Some(
            (leadership
                + if commander {
                    u64::from(rules.officer_commander_bonus_permille)
                } else {
                    0
                }) as u32,
        )
    }
}
