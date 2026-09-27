//! Age-ordered service and shared-location mentorship progress at boundaries.

use super::qualification::{contact_reason, facility_for};
use crate::{
    data::{
        economy::{Habitation, TroopKind},
        progression::TrainingDiscipline,
        world::PersonClass,
        GameData,
    },
    state::{
        mentorship::MentorshipStatus,
        people::{PersonAssignment, PersonStatus},
        StrategicCampaign,
    },
};
use std::collections::BTreeSet;

pub(crate) fn reconcile(campaign: &mut StrategicCampaign, data: &GameData) {
    let ids = campaign.mentorships.keys().copied().collect::<Vec<_>>();
    for learner in ids {
        let Some(assignment) = campaign.mentorships.get(&learner).cloned() else {
            continue;
        };
        let dead = campaign
            .people
            .get(&learner)
            .is_none_or(|person| !person.is_alive())
            || campaign
                .people
                .get(&assignment.mentor)
                .is_none_or(|person| !person.is_alive());
        if dead {
            campaign.mentorships.remove(&learner);
            continue;
        }
        let reason = contact_reason(
            campaign,
            data,
            learner,
            assignment.mentor,
            assignment.discipline,
        );
        let status = reason.map_or(MentorshipStatus::Active, |reason| {
            MentorshipStatus::Paused { reason }
        });
        campaign
            .mentorships
            .get_mut(&learner)
            .expect("assignment")
            .status = status;
    }
}

pub(crate) fn resolve_season(campaign: &mut StrategicCampaign, data: &GameData) {
    accrue_discipline_service(campaign, data);
    reconcile(campaign, data);
    let mut completed = Vec::new();
    let learners = campaign.mentorships.keys().copied().collect::<Vec<_>>();
    for learner in learners {
        let Some(active) = campaign.mentorships.get(&learner).cloned() else {
            continue;
        };
        if active.status != MentorshipStatus::Active {
            continue;
        }
        let seasons = active.seasons_completed.saturating_add(1);
        campaign
            .mentorships
            .get_mut(&learner)
            .expect("active")
            .seasons_completed = seasons;
        let total = campaign
            .people
            .get(&learner)
            .expect("validated learner")
            .career
            .mentorship_seasons
            .get(&active.discipline)
            .copied()
            .unwrap_or(0)
            .saturating_add(1)
            .min(data.lifecycle.apprenticeship_seasons);
        campaign
            .people
            .get_mut(&learner)
            .expect("validated learner")
            .career
            .mentorship_seasons
            .insert(active.discipline, total);
        if seasons >= data.lifecycle.apprenticeship_seasons {
            campaign
                .people
                .get_mut(&learner)
                .expect("validated learner")
                .career
                .completed_mentors
                .insert(crate::state::people::CompletedApprenticeship {
                    mentor: active.mentor,
                    discipline: active.discipline,
                    started_round: active.started_round,
                    completed_round: campaign.completed_rounds,
                });
            completed.push(learner);
        }
    }
    for learner in completed {
        campaign.mentorships.remove(&learner);
    }
}

fn accrue_discipline_service(campaign: &mut StrategicCampaign, data: &GameData) {
    let mut service = Vec::new();
    for person in campaign.people.values().filter(|person| {
        person.status == PersonStatus::Fit && !person.career.retired && person.is_alive()
    }) {
        let mut disciplines = Vec::new();
        match person.assignment {
            PersonAssignment::Formation { formation } => {
                let Some(entry) = campaign.formations.get(&formation) else {
                    continue;
                };
                let troop_discipline = match entry.kind {
                    TroopKind::Warriors | TroopKind::Spearmen => Some(TrainingDiscipline::Infantry),
                    TroopKind::Archers => Some(TrainingDiscipline::Archery),
                    TroopKind::Riders => Some(TrainingDiscipline::Riding),
                    TroopKind::Medics => Some(TrainingDiscipline::Medicine),
                    TroopKind::SiegeEngines => None,
                };
                disciplines.extend(troop_discipline);
                match person.class {
                    PersonClass::Scout => disciplines.push(TrainingDiscipline::Scouting),
                    PersonClass::Medic if entry.kind == TroopKind::Medics => {
                        disciplines.push(TrainingDiscipline::Medicine)
                    }
                    _ => {}
                }
            }
            PersonAssignment::Site { site } => {
                if let (Some(at_site), Some(discipline)) = (
                    campaign.world.site(site),
                    super::discipline_for_class(person.class),
                ) {
                    let facility = facility_for(discipline);
                    if at_site.controller == Some(person.faction)
                        && at_site.habitation != Habitation::Unsettled
                        && at_site.facilities.contains(&facility)
                        && campaign.world.structural_damage(site)
                            < data.economy.facility_failure_damage
                        && campaign.supply_path(person.faction, site).is_some()
                        && !campaign.sieges.contains_key(&site)
                    {
                        disciplines.push(discipline);
                    }
                }
            }
            PersonAssignment::Dead => continue,
        }
        for discipline in disciplines {
            service.push((person.id, discipline));
        }
        if campaign
            .armies
            .values()
            .any(|army| army.commander == Some(person.id))
        {
            service.push((person.id, TrainingDiscipline::Command));
        }
    }
    let service = service.into_iter().collect::<BTreeSet<_>>();
    for (person, discipline) in service {
        let total = campaign.people[&person]
            .career
            .discipline_service_seasons
            .get(&discipline)
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
        campaign
            .people
            .get_mut(&person)
            .expect("service person")
            .career
            .discipline_service_seasons
            .insert(discipline, total);
    }
}
