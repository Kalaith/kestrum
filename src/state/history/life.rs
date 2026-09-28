//! Dated life transitions share the ordinary narrative retention budget.
use crate::{
    data::{
        economy::TroopKind,
        progression::{EpithetFact, TrainingDiscipline},
        world::PersonClass,
    },
    state::{
        people::PersonId,
        relationships::{FamilyOrigin, HouseholdEndReason, HouseholdId},
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LifeEvent {
    Emerged {
        troop: TroopKind,
    },
    Arrived {
        origin: FamilyOrigin,
    },
    Recognized {
        epithet: String,
        cause: EpithetFact,
    },
    ClassCompleted {
        class: PersonClass,
    },
    MentorshipStarted {
        mentor: PersonId,
        discipline: TrainingDiscipline,
    },
    MentorshipCompleted {
        mentor: PersonId,
        discipline: TrainingDiscipline,
    },
    HouseholdFormed {
        household: HouseholdId,
    },
    HouseholdEnded {
        household: HouseholdId,
        reason: HouseholdEndReason,
    },
    ServiceEntered,
    Retired,
    NaturalDeath,
}
