//! The NPC planner uses the same evidence options and validated commands as the player.

mod continuity;
mod training;
use super::*;
use crate::{
    data::{
        progression::TrainingDiscipline,
        world::{Facility, PersonClass, SiteTag},
    },
    engine::{career_options, mentorship_options},
    state::{
        military::FormationId,
        people::{PersonAssignment, PersonStatus},
    },
};

impl Planner<'_> {
    pub(super) fn progression(&self) -> Option<AiDecision> {
        let mut people = self
            .campaign
            .people
            .values()
            .filter(|person| person.faction == self.owner)
            .collect::<Vec<_>>();
        people.sort_by_key(|person| person.id);
        if let Some(decision) = self.governor(&people) {
            return Some(decision);
        }
        if let Some(decision) = self.continuity(&people) {
            return Some(decision);
        }
        if let Some(decision) = self
            .deploy_learners(&people)
            .or_else(|| self.redistribute_heroes(&people))
            .or_else(|| self.train_people(&people))
        {
            return Some(decision);
        }
        if let Some(decision) = self.mentorship(&people) {
            return Some(decision);
        }
        self.train_formations()
            .or_else(|| self.appoint_commander(&people))
            .or_else(|| self.apprentice(&people))
    }

    fn train_formations(&self) -> Option<AiDecision> {
        let mut formations = self
            .campaign
            .formations
            .values()
            .filter(|formation| formation.faction == self.owner)
            .collect::<Vec<_>>();
        formations.sort_by_key(|formation| formation.id);
        for formation in formations {
            let Some(site) = formation_site(self.campaign, formation.id) else {
                continue;
            };
            if !self.can_train_at(site, Facility::TrainingGround, false) {
                continue;
            }
            if let Ok(options) =
                super::super::specialization_options(self.campaign, self.data, formation.id)
            {
                for option in options.into_iter().filter(|option| option.eligible) {
                    if let Some(decision) = self.choose(
                        Command::SpecializeFormation {
                            formation: formation.id,
                            specialization: option.specialization,
                            site,
                        },
                        None,
                    ) {
                        return Some(decision);
                    }
                }
            }
        }
        None
    }

    fn appoint_commander(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        for army in self
            .campaign
            .armies
            .values()
            .filter(|army| army.faction == self.owner && army.commander.is_none())
        {
            let candidate = people
                .iter()
                .filter(|person| {
                    !person.career.retired
                        && person.is_fit_for_field(
                            self.campaign.completed_rounds,
                            self.data.rules.leadership.field_min_age_years,
                        )
                        && matches!(person.assignment, PersonAssignment::Formation { formation }
                        if army.formation_ids().any(|id| id == formation))
                })
                .min_by_key(|person| (person.class != PersonClass::Officer, person.id));
            if let Some(person) = candidate {
                if let Some(decision) = self.choose(
                    Command::SetCommander {
                        army: army.id,
                        person: Some(person.id),
                    },
                    None,
                ) {
                    return Some(decision);
                }
            }
        }

        None
    }

    fn apprentice(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        let learners = people
            .iter()
            .filter(|person| {
                person.is_alive()
                    && (person.age_years(self.campaign.completed_rounds) < 20
                        || self.campaign.mentorships.contains_key(&person.id))
            })
            .count();
        if learners >= 2 {
            return None;
        }
        for site in self.campaign.world.sites.iter().filter(|site| {
            site.controller == Some(self.owner)
                && site.habitation >= self.data.households.apprentice_minimum_habitation
                && self.campaign.supply_path(self.owner, site.id).is_some()
                && !self.campaign.sieges.contains_key(&site.id)
                && self
                    .campaign
                    .world
                    .population
                    .get(&site.id)
                    .copied()
                    .unwrap_or(0)
                    > 0
                && self.campaign.world.structural_damage(site.id)
                    < self.data.economy.facility_failure_damage
        }) {
            if let Some(decision) = self.choose(Command::InviteApprentice { site: site.id }, None) {
                return Some(decision);
            }
        }
        None
    }

    fn governor(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        for site in self.campaign.world.sites.iter().filter(|site| {
            site.controller == Some(self.owner)
                && site.habitation != crate::data::economy::Habitation::Unsettled
                && !self.campaign.sieges.contains_key(&site.id)
        }) {
            let occupied = people.iter().any(|person| {
                person.career.site_role == Some(crate::state::people::PersonSiteRole::Governor)
                    && person.assignment == (PersonAssignment::Site { site: site.id })
            });
            if occupied {
                continue;
            }
            for person in people.iter().filter(|person| {
                person.is_alive()
                    && person.status == PersonStatus::Fit
                    && person.career.site_role.is_none()
                    && (person.career.retired
                        || person.age_years(self.campaign.completed_rounds)
                            >= self.data.lifecycle.elder_age_years)
                    && person_site(self.campaign, person.id) == Some(site.id)
            }) {
                if let Some(decision) = self.choose(
                    Command::AppointGovernor {
                        person: person.id,
                        site: site.id,
                    },
                    None,
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    fn mentorship(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        let priority = |discipline: TrainingDiscipline| match discipline {
            TrainingDiscipline::Command => 0,
            TrainingDiscipline::Medicine => 1,
            TrainingDiscipline::Infantry => 2,
            TrainingDiscipline::Archery => 3,
            TrainingDiscipline::Riding => 4,
            TrainingDiscipline::Scouting => 5,
        };
        for learner in people.iter().filter(|person| {
            person.status == PersonStatus::Fit
                && (person.class == PersonClass::Recruit
                    || self.campaign.mentorships.contains_key(&person.id))
                && person.is_alive()
                && !person.career.retired
                && person.career.course.is_none()
                && person.age_years(self.campaign.completed_rounds)
                    >= self.data.lifecycle.learner_minimum_age_years
                && (!matches!(person.assignment, PersonAssignment::Formation { .. })
                    || person.age_years(self.campaign.completed_rounds)
                        >= self.data.rules.leadership.field_min_age_years)
        }) {
            let existing = self.campaign.mentorships.get(&learner.id);
            if existing.is_some_and(|mentorship| {
                mentorship.status == crate::state::mentorship::MentorshipStatus::Active
            }) {
                continue;
            }
            let Ok(mut options) = mentorship_options(self.campaign, self.data, learner.id) else {
                continue;
            };
            options.retain(|option| option.eligible);
            if let Some(existing) = existing {
                options.retain(|option| {
                    option.mentor == existing.mentor && option.discipline == existing.discipline
                });
            }
            options.sort_by_key(|option| (priority(option.discipline), option.mentor));
            for option in options {
                if let Some(decision) = self.choose(
                    Command::StartMentorship {
                        mentor: option.mentor,
                        learner: learner.id,
                        discipline: option.discipline,
                    },
                    None,
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    fn can_train_at(
        &self,
        site: crate::data::world::SiteId,
        facility: Facility,
        horse: bool,
    ) -> bool {
        let Some(location) = self.campaign.world.site(site) else {
            return false;
        };
        location.controller == Some(self.owner)
            && location.facilities.contains(&facility)
            && self.campaign.world.structural_damage(site) == 0
            && self.view.supplied_sites.contains(&site)
            && !self.campaign.sieges.contains_key(&site)
            && (!horse || location.tags.contains(&SiteTag::HorseAccess))
    }
}

fn item_link(
    campaign: &StrategicCampaign,
    predecessor: crate::state::people::PersonId,
    successor: crate::state::people::PersonId,
    site: crate::data::world::SiteId,
) -> Option<crate::state::relationships::SuccessorLink> {
    use crate::state::relationships::SuccessorLink;
    let links = [
        (
            SuccessorLink::Martial,
            campaign.is_pupil(predecessor, successor),
        ),
        (
            SuccessorLink::Religious,
            campaign.is_pupil(predecessor, successor)
                && campaign.world.site(site).is_some_and(|place| {
                    place.controller == Some(campaign.people[&predecessor].faction)
                        && place.facilities.contains(&Facility::Temple)
                }),
        ),
        (
            SuccessorLink::Political,
            person_site(campaign, successor) == Some(site)
                && campaign
                    .people
                    .get(&predecessor)
                    .zip(campaign.people.get(&successor))
                    .is_some_and(|(first, second)| {
                        (first.career.site_role.is_some() || second.career.site_role.is_some())
                            && first
                                .career
                                .relationships
                                .get(&successor)
                                .is_some_and(|link| link.shared_service_seasons >= 4)
                    }),
        ),
        (
            SuccessorLink::Adopted,
            campaign.adopted_relation(predecessor, successor),
        ),
        (
            SuccessorLink::Blood,
            campaign.known_blood_relation(predecessor, successor),
        ),
    ];
    links
        .into_iter()
        .find_map(|(link, valid)| valid.then_some(link))
}

fn person_site(
    campaign: &StrategicCampaign,
    id: crate::state::people::PersonId,
) -> Option<crate::data::world::SiteId> {
    match campaign.people.get(&id)?.assignment {
        PersonAssignment::Formation { formation } => formation_site(campaign, formation),
        PersonAssignment::Site { site } => Some(site),
        PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site } => Some(site),
        PersonAssignment::Dead => None,
    }
}

fn formation_site(
    campaign: &StrategicCampaign,
    id: FormationId,
) -> Option<crate::data::world::SiteId> {
    campaign
        .armies
        .values()
        .find(|army| army.formation_ids().any(|formation| formation == id))
        .map(|army| army.site)
}
