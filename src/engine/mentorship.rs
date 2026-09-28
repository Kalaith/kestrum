//! Qualified, co-located teaching uses the same person commands as class courses.

mod commands;
mod qualification;
mod seasons;

use crate::{
    data::{progression::TrainingDiscipline, GameData},
    engine::RuleError,
    state::{mentorship::MentorshipStatus, people::PersonId, StrategicCampaign},
};

pub(crate) use commands::{execute, validate};
pub use qualification::MentorshipOption;
pub(crate) use seasons::{reconcile, resolve_season};

/// Check live teaching conditions before recovery, including wounds and separation.
pub(crate) fn medical_assistant(
    campaign: &StrategicCampaign,
    data: &GameData,
    learner: PersonId,
) -> Option<PersonId> {
    let link = campaign.mentorships.get(&learner)?;
    (link.discipline == TrainingDiscipline::Medicine
        && qualification::contact_reason(campaign, data, learner, link.mentor, link.discipline)
            .is_none())
    .then_some(link.mentor)
}

pub fn mentorship_options(
    campaign: &StrategicCampaign,
    data: &GameData,
    learner: PersonId,
) -> Result<Vec<MentorshipOption>, RuleError> {
    qualification::options(campaign, data, learner)
}

pub fn mentorship_status_text(status: MentorshipStatus) -> &'static str {
    match status {
        MentorshipStatus::Active => "Learning together",
        MentorshipStatus::Paused { reason } => match reason {
            crate::state::mentorship::MentorshipPauseReason::Apart => {
                "Mentor and learner are apart"
            }
            crate::state::mentorship::MentorshipPauseReason::Wounded => {
                "A wound pauses the lessons"
            }
            crate::state::mentorship::MentorshipPauseReason::SiteCaptured => {
                "The site is no longer friendly"
            }
            crate::state::mentorship::MentorshipPauseReason::FacilityUnavailable => {
                "The required facility is unavailable"
            }
            crate::state::mentorship::MentorshipPauseReason::HorseAccessUnavailable => {
                "The site has no horse access"
            }
            crate::state::mentorship::MentorshipPauseReason::QualificationLost => {
                "The mentor no longer qualifies in this discipline"
            }
            crate::state::mentorship::MentorshipPauseReason::MentorAtCapacity => {
                "This mentor is already teaching another learner"
            }
        },
    }
}

pub(crate) fn pause_reason_text(
    reason: crate::state::mentorship::MentorshipPauseReason,
) -> &'static str {
    match reason {
        crate::state::mentorship::MentorshipPauseReason::Apart => {
            "Mentor and learner must share one physical site"
        }
        crate::state::mentorship::MentorshipPauseReason::Wounded => "A wound pauses the lessons",
        crate::state::mentorship::MentorshipPauseReason::SiteCaptured => {
            "The site is no longer friendly"
        }
        crate::state::mentorship::MentorshipPauseReason::FacilityUnavailable => {
            "The required facility is unavailable"
        }
        crate::state::mentorship::MentorshipPauseReason::HorseAccessUnavailable => {
            "Riding lessons require local horse access"
        }
        crate::state::mentorship::MentorshipPauseReason::QualificationLost => {
            "The mentor no longer qualifies in this discipline"
        }
        crate::state::mentorship::MentorshipPauseReason::MentorAtCapacity => {
            "This mentor is already teaching another learner"
        }
    }
}

pub(crate) fn discipline_for_class(
    class: crate::data::world::PersonClass,
) -> Option<TrainingDiscipline> {
    use crate::data::world::PersonClass;
    match class {
        PersonClass::Infantry => Some(TrainingDiscipline::Infantry),
        PersonClass::Archer => Some(TrainingDiscipline::Archery),
        PersonClass::Scout => Some(TrainingDiscipline::Scouting),
        PersonClass::Cavalry => Some(TrainingDiscipline::Riding),
        PersonClass::Medic => Some(TrainingDiscipline::Medicine),
        PersonClass::Officer => Some(TrainingDiscipline::Command),
        PersonClass::Recruit => None,
    }
}
