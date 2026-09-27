//! Validate and apply local, one-teacher/one-learner mentorship assignments.

use super::qualification::{contact_reason, mentor_qualified};
use crate::{
    data::{progression::TrainingDiscipline, GameData},
    engine::{Command, RuleError},
    state::{
        mentorship::{Mentorship, MentorshipStatus},
        people::{PersonId, PersonStatus},
        StrategicCampaign,
    },
};

pub(crate) fn validate(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    match command {
        Command::StartMentorship {
            mentor,
            learner,
            discipline,
        } => validate_start(campaign, data, owner, *mentor, *learner, *discipline),
        Command::EndMentorship { learner } => {
            let person = campaign
                .people
                .get(learner)
                .ok_or(RuleError::UnknownPerson { person: *learner })?;
            if person.faction != owner {
                return Err(RuleError::PersonNotOwned { person: *learner });
            }
            if !campaign.mentorships.contains_key(learner) {
                return Err(RuleError::Progression(
                    "This learner has no mentor to release.".into(),
                ));
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn validate_start(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    mentor_id: PersonId,
    learner_id: PersonId,
    discipline: TrainingDiscipline,
) -> Result<(), RuleError> {
    if mentor_id == learner_id {
        return Err(RuleError::Progression(
            "A person cannot mentor themself.".into(),
        ));
    }
    let mentor = campaign
        .people
        .get(&mentor_id)
        .ok_or(RuleError::UnknownPerson { person: mentor_id })?;
    let learner = campaign
        .people
        .get(&learner_id)
        .ok_or(RuleError::UnknownPerson { person: learner_id })?;
    if mentor.faction != owner {
        return Err(RuleError::PersonNotOwned { person: mentor_id });
    }
    if learner.faction != owner {
        return Err(RuleError::PersonNotOwned { person: learner_id });
    }
    if !mentor.is_alive()
        || mentor.age_years(campaign.completed_rounds) < data.lifecycle.mentor_minimum_age_years
        || !mentor_qualified(campaign, data, mentor_id, discipline)
    {
        return Err(RuleError::Progression(
            "A mentor must be a living adult with four seasons in this discipline and its class or earned distinction.".into(),
        ));
    }
    if !learner.is_alive()
        || learner.career.retired
        || learner.age_years(campaign.completed_rounds) < data.lifecycle.learner_minimum_age_years
        || matches!(
            learner.assignment,
            crate::state::people::PersonAssignment::Dead
        )
        || learner.career.course.is_some()
    {
        return Err(RuleError::Progression(
            "Choose a living, unretired learner aged thirteen or older with no active course."
                .into(),
        ));
    }
    if matches!(
        learner.assignment,
        crate::state::people::PersonAssignment::Formation { .. }
    ) && learner.age_years(campaign.completed_rounds) < 17
    {
        return Err(RuleError::Progression(
            "A field formation cannot include anyone younger than seventeen.".into(),
        ));
    }
    if !matches!(
        mentor.status,
        PersonStatus::Fit | PersonStatus::Wounded { .. }
    ) || !matches!(
        learner.status,
        PersonStatus::Fit | PersonStatus::Wounded { .. }
    ) {
        return Err(RuleError::Progression(
            "Displaced or dead people cannot begin a mentorship.".into(),
        ));
    }
    if let Some(existing) = campaign.mentorships.get(&learner_id) {
        if existing.mentor != mentor_id || existing.discipline != discipline {
            return Err(RuleError::Progression(
                "End the learner's current mentorship before choosing another.".into(),
            ));
        }
        if existing.status == MentorshipStatus::Active {
            return Err(RuleError::Progression(
                "This mentorship is already active.".into(),
            ));
        }
    }
    if campaign
        .mentorships
        .iter()
        .any(|(learner, assignment)| *learner != learner_id && assignment.mentor == mentor_id)
    {
        return Err(RuleError::Progression(
            "A mentor can teach one learner at a time.".into(),
        ));
    }
    if let Some(reason) = contact_reason(campaign, data, learner_id, mentor_id, discipline) {
        return Err(RuleError::Progression(
            super::pause_reason_text(reason).into(),
        ));
    }
    Ok(())
}

pub(crate) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    match command {
        Command::StartMentorship {
            mentor,
            learner,
            discipline,
        } => {
            if let Some(existing) = campaign.mentorships.get_mut(learner) {
                existing.status = MentorshipStatus::Active;
            } else {
                campaign.mentorships.insert(
                    *learner,
                    Mentorship {
                        mentor: *mentor,
                        discipline: *discipline,
                        started_round: campaign.completed_rounds,
                        seasons_completed: 0,
                        status: MentorshipStatus::Active,
                    },
                );
            }
        }
        Command::EndMentorship { learner } => {
            campaign.mentorships.remove(learner);
        }
        _ => {
            let _ = (data, owner);
            return Err(RuleError::InvalidState(
                "Unexpected mentorship command.".into(),
            ));
        }
    }
    Ok(())
}
