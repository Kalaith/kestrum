//! Saved battle facts carry labels independently of later assignments.

use super::{ArmyId, FactionId, PersonAssignment, PersonId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonDeathReason {
    FormationDestroyed,
    NoRefuge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WoundCause {
    FormationDestroyed,
    CommandCasualty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersonCombatOutcome {
    Died {
        reason: PersonDeathReason,
    },
    Wounded {
        assignment: PersonAssignment,
        cause: WoundCause,
    },
    AssumedCommand {
        army: ArmyId,
        previous: PersonId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonCombatEvent {
    pub person: PersonId,
    pub name: String,
    pub faction: FactionId,
    pub outcome: PersonCombatOutcome,
}
