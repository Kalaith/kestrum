//! Persistent dispositions, earned distinctions and active local development.

use super::PersonId;
use crate::{
    data::{
        economy::TroopKind,
        progression::{EpithetFact, TrainingDiscipline},
        world::{PersonClass, SiteId},
    },
    state::military::FormationId,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tendency {
    Negative,
    #[default]
    Neutral,
    Positive,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disposition {
    pub courage: Tendency,
    pub care: Tendency,
    pub curiosity: Tendency,
}

impl Tendency {
    pub fn adjustment(self) -> i32 {
        match self {
            Self::Negative => -1,
            Self::Neutral => 0,
            Self::Positive => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonTrait {
    Bold,
    Protective,
    NaturalCommander,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersonSiteRole {
    Governor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recognition {
    pub completed_rounds: u32,
    pub epithet: String,
    pub cause: EpithetFact,
    pub site: SiteId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmergenceRecord {
    pub completed_rounds: u32,
    pub source_formation: FormationId,
    pub source_troop: TroopKind,
    pub site: SiteId,
    pub distinguishing_deed: Option<EpithetFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PersonCourse {
    Class {
        target: PersonClass,
        site: SiteId,
        steps_completed: u32,
    },
    RidingPractice {
        site: SiteId,
        steps_completed: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonRelationship {
    pub shared_service_seasons: u32,
    pub last_shared_service_round: Option<u32>,
    pub mutual_combat_rounds: u32,
    pub last_mutual_combat_round: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedApprenticeship {
    pub mentor: PersonId,
    pub discipline: TrainingDiscipline,
    pub started_round: u32,
    pub completed_round: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonCareer {
    pub disposition: Disposition,
    #[serde(default)]
    pub notable_sites: BTreeMap<EpithetFact, SiteId>,
    #[serde(default)]
    pub emergence: Option<EmergenceRecord>,
    pub traits: BTreeSet<PersonTrait>,
    pub recognition: Option<Recognition>,
    pub course: Option<PersonCourse>,
    pub riding_practice_seasons: u32,
    #[serde(default)]
    pub discipline_service_seasons: BTreeMap<TrainingDiscipline, u32>,
    #[serde(default)]
    pub mentorship_seasons: BTreeMap<TrainingDiscipline, u32>,
    #[serde(default)]
    pub completed_mentors: BTreeSet<CompletedApprenticeship>,
    #[serde(default)]
    pub site_role: Option<PersonSiteRole>,
    #[serde(default)]
    pub automatic_retirement_round: Option<u32>,
    pub relationships: BTreeMap<PersonId, PersonRelationship>,
    pub retired: bool,
}
