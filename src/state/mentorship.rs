//! Active teacher and learner assignments with explainable interruptions.

use super::people::PersonId;
use crate::data::progression::TrainingDiscipline;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MentorshipPauseReason {
    Apart,
    Wounded,
    SiteCaptured,
    FacilityUnavailable,
    HorseAccessUnavailable,
    QualificationLost,
    MentorAtCapacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MentorshipStatus {
    Active,
    Paused { reason: MentorshipPauseReason },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mentorship {
    pub mentor: PersonId,
    pub discipline: TrainingDiscipline,
    pub started_round: u32,
    pub seasons_completed: u32,
    pub status: MentorshipStatus,
}
