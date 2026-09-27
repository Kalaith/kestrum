//! The NPC planner uses the same evidence options and validated commands as the player.

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
        for person in &people {
            if person.career.course.is_some()
                || !matches!(person.assignment, PersonAssignment::Formation { .. })
            {
                continue;
            }
            if person.status != PersonStatus::Fit {
                continue;
            }
            if let Ok(options) = career_options(self.campaign, self.data, person.id) {
                for option in options.into_iter().filter(|option| option.eligible) {
                    let Some(site) = person_site(self.campaign, person.id) else {
                        continue;
                    };
                    if self.can_train_at(
                        site,
                        option.course.facility,
                        option.course.requires_horse_access,
                    ) {
                        if let Some(decision) = self.choose(
                            Command::TrainPerson {
                                person: person.id,
                                class: option.class,
                                site,
                            },
                            None,
                        ) {
                            return Some(decision);
                        }
                    }
                }
            }
            let cavalry = self.data.progression.careers.courses[&PersonClass::Cavalry].facility;
            let needs_riding = person.career.riding_practice_seasons
                < self.data.progression.careers.riding_seasons;
            if needs_riding && person.career.course.is_none() {
                if let Some(site) = person_site(self.campaign, person.id) {
                    if self.can_train_at(site, cavalry, true) {
                        if let Some(decision) = self.choose(
                            Command::PracticeRiding {
                                person: person.id,
                                site,
                            },
                            None,
                        ) {
                            return Some(decision);
                        }
                    }
                }
            }
        }
        if let Some(decision) = self.mentorship(&people) {
            return Some(decision);
        }
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
        for army in self
            .campaign
            .armies
            .values()
            .filter(|army| army.faction == self.owner && army.commander.is_none())
        {
            if let Some(person) = people.iter().find(|person| !person.career.retired && person.is_fit_for_field(self.campaign.completed_rounds, self.data.rules.leadership.field_min_age_years) && matches!(person.assignment, PersonAssignment::Formation { formation } if army.formation_ids().any(|id| id == formation))) {
                if let Some(decision) = self.choose(Command::SetCommander { army: army.id, person: Some(person.id) }, None) { return Some(decision); }
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

fn person_site(
    campaign: &StrategicCampaign,
    id: crate::state::people::PersonId,
) -> Option<crate::data::world::SiteId> {
    match campaign.people.get(&id)?.assignment {
        PersonAssignment::Formation { formation } => formation_site(campaign, formation),
        PersonAssignment::Site { site } => Some(site),
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
