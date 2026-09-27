//! Mentor qualifications and location-specific class opportunities.

use super::*;
use crate::{
    data::{progression::TrainingDiscipline, world::PersonClass},
    engine::person_site,
    state::{
        mentorship::MentorshipPauseReason,
        people::{PersonId, PersonStatus, PersonTrait},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MentorshipOption {
    pub mentor: PersonId,
    pub discipline: TrainingDiscipline,
    pub eligible: bool,
    pub reason: Option<String>,
}

pub(super) fn options(
    campaign: &StrategicCampaign,
    data: &GameData,
    learner_id: PersonId,
) -> Result<Vec<MentorshipOption>, RuleError> {
    let learner = campaign
        .people
        .get(&learner_id)
        .ok_or(RuleError::UnknownPerson { person: learner_id })?;
    let Some(site) = person_site(campaign, learner_id) else {
        return Ok(Vec::new());
    };
    let mut result = Vec::new();
    for mentor in campaign.people.values().filter(|person| {
        person.faction == learner.faction
            && person.id != learner_id
            && person_site(campaign, person.id) == Some(site)
    }) {
        for discipline in disciplines_for(learner.class) {
            if mentor_qualified(campaign, data, mentor.id, discipline) {
                let reason = contact_reason(campaign, data, learner_id, mentor.id, discipline);
                result.push(MentorshipOption {
                    mentor: mentor.id,
                    discipline,
                    eligible: reason.is_none(),
                    reason: reason
                        .map(|reason| super::super::mentorship::pause_reason_text(reason).into()),
                });
            }
        }
    }
    result.sort_by_key(|option| (option.mentor, option.discipline));
    Ok(result)
}

pub(super) fn mentor_qualified(
    campaign: &StrategicCampaign,
    data: &GameData,
    mentor_id: PersonId,
    discipline: TrainingDiscipline,
) -> bool {
    let Some(mentor) = campaign.people.get(&mentor_id) else {
        return false;
    };
    let eligible_identity = match discipline {
        TrainingDiscipline::Infantry => {
            mentor.class == PersonClass::Infantry
                || mentor.career.traits.contains(&PersonTrait::Bold)
                || mentor.career.traits.contains(&PersonTrait::Protective)
        }
        TrainingDiscipline::Archery => {
            mentor.class == PersonClass::Archer || mentor.career.traits.contains(&PersonTrait::Bold)
        }
        TrainingDiscipline::Scouting => mentor.class == PersonClass::Scout,
        TrainingDiscipline::Riding => {
            mentor.class == PersonClass::Cavalry
                || mentor.career.traits.contains(&PersonTrait::Bold)
        }
        TrainingDiscipline::Medicine => {
            mentor.class == PersonClass::Medic
                || mentor.career.traits.contains(&PersonTrait::Protective)
        }
        TrainingDiscipline::Command => {
            mentor.class == PersonClass::Officer
                || mentor
                    .career
                    .traits
                    .contains(&PersonTrait::NaturalCommander)
        }
    };
    mentor.is_alive()
        && mentor.age_years(campaign.completed_rounds) >= data.lifecycle.mentor_minimum_age_years
        && mentor
            .career
            .discipline_service_seasons
            .get(&discipline)
            .copied()
            .unwrap_or(0)
            >= data.lifecycle.mentor_service_seasons
        && eligible_identity
}

pub(super) fn contact_reason(
    campaign: &StrategicCampaign,
    data: &GameData,
    learner_id: PersonId,
    mentor_id: PersonId,
    discipline: TrainingDiscipline,
) -> Option<MentorshipPauseReason> {
    let learner = campaign.people.get(&learner_id)?;
    let mentor = campaign.people.get(&mentor_id)?;
    if !learner.is_alive() || !mentor.is_alive() {
        return Some(MentorshipPauseReason::Apart);
    }
    if learner.status != PersonStatus::Fit || mentor.status != PersonStatus::Fit {
        return Some(
            if matches!(learner.status, PersonStatus::Wounded { .. })
                || matches!(mentor.status, PersonStatus::Wounded { .. })
            {
                MentorshipPauseReason::Wounded
            } else {
                MentorshipPauseReason::Apart
            },
        );
    }
    if !mentor_qualified(campaign, data, mentor_id, discipline) {
        return Some(MentorshipPauseReason::QualificationLost);
    }
    if campaign
        .mentorships
        .iter()
        .any(|(learner, link)| *learner != learner_id && link.mentor == mentor_id)
    {
        return Some(MentorshipPauseReason::MentorAtCapacity);
    }
    let Some(learner_site) = person_site(campaign, learner_id) else {
        return Some(MentorshipPauseReason::Apart);
    };
    if person_site(campaign, mentor_id) != Some(learner_site) {
        return Some(MentorshipPauseReason::Apart);
    }
    if !campaign.is_independent(learner.faction)
        || campaign.world.site(learner_site)?.controller != Some(learner.faction)
    {
        return Some(MentorshipPauseReason::SiteCaptured);
    }
    let Some(site) = campaign.world.site(learner_site) else {
        return Some(MentorshipPauseReason::SiteCaptured);
    };
    let facility = facility_for(discipline);
    if campaign.sieges.contains_key(&learner_site)
        || site.habitation == crate::data::economy::Habitation::Unsettled
        || !site.facilities.contains(&facility)
        || campaign.world.structural_damage(learner_site) >= data.economy.facility_failure_damage
        || campaign
            .supply_path(learner.faction, learner_site)
            .is_none()
    {
        return Some(MentorshipPauseReason::FacilityUnavailable);
    }
    if discipline == TrainingDiscipline::Riding
        && !site
            .tags
            .contains(&crate::data::world::SiteTag::HorseAccess)
    {
        return Some(MentorshipPauseReason::HorseAccessUnavailable);
    }
    None
}

pub(super) fn facility_for(discipline: TrainingDiscipline) -> crate::data::world::Facility {
    use crate::data::world::Facility;
    match discipline {
        TrainingDiscipline::Riding => Facility::Stable,
        TrainingDiscipline::Medicine => Facility::Infirmary,
        _ => Facility::TrainingGround,
    }
}

pub(super) fn disciplines_for(_class: PersonClass) -> Vec<TrainingDiscipline> {
    use TrainingDiscipline::*;
    vec![Infantry, Archery, Scouting, Riding, Medicine, Command]
}
