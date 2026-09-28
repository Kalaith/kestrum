//! Train toward missing capabilities and move qualified local adults into field service.

use super::*;
use crate::state::people::Person;

impl Planner<'_> {
    pub(super) fn train_people(&self, people: &[&Person]) -> Option<AiDecision> {
        for person in people {
            if person.career.course.is_some()
                || person.career.retired
                || person.career.site_role.is_some()
                || person.status != PersonStatus::Fit
                || !matches!(
                    person.assignment,
                    PersonAssignment::Formation { .. } | PersonAssignment::Site { .. }
                )
            {
                continue;
            }
            let commander = self
                .view
                .armies
                .iter()
                .any(|army| army.commander == Some(person.id));
            // Preserve established roles. Only a current commander has a reason to
            // replace a trained class, and that upgrade is always to Officer.
            if person.class != PersonClass::Recruit
                && (!commander || person.class == PersonClass::Officer)
            {
                continue;
            }
            let site = person_site(self.campaign, person.id)?;
            let mut options = career_options(self.campaign, self.data, person.id).ok()?;
            options.retain(|option| {
                option.eligible
                    && option.class != person.class
                    && (person.class == PersonClass::Recruit
                        || option.class == PersonClass::Officer)
            });
            options.sort_by_key(|option| {
                (
                    self.role_priority(person, option.class, site),
                    option.class as u32,
                )
            });
            for option in options {
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
            if person.class == PersonClass::Recruit
                && person.career.riding_practice_seasons
                    < self.data.progression.careers.riding_seasons
                && matches!(person.assignment, PersonAssignment::Formation { formation }
                    if self.campaign.formations[&formation].kind == crate::data::economy::TroopKind::Riders)
                && self.can_train_at(site, Facility::Stable, true)
            {
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
        None
    }

    fn role_priority(&self, person: &Person, class: PersonClass, site: SiteId) -> (bool, u8) {
        let present = self.view.people.iter().any(|other| {
            other.id != person.id
                && other.class == class
                && !other.career.retired
                && other.is_alive()
                && person_site(self.campaign, other.id) == Some(site)
        });
        let priority = match class {
            PersonClass::Officer => 0,
            PersonClass::Medic => 1,
            PersonClass::Infantry => 2,
            PersonClass::Archer => 3,
            PersonClass::Scout => 4,
            PersonClass::Cavalry => 5,
            PersonClass::Recruit => 6,
        };
        (present, priority)
    }

    pub(super) fn deploy_learners(&self, people: &[&Person]) -> Option<AiDecision> {
        for person in people {
            if !matches!(person.assignment, PersonAssignment::Site { .. })
                || (person.class == PersonClass::Recruit
                    && person.career.completed_mentors.is_empty())
                || person.career.course.is_some()
                || self.campaign.mentorships.contains_key(&person.id)
            {
                continue;
            }
            if let Some(formation) = self.field_formation(person) {
                if let Some(decision) = self.choose(
                    Command::TransferPerson {
                        person: person.id,
                        to_formation: formation,
                    },
                    None,
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    pub(super) fn field_formation(&self, person: &Person) -> Option<FormationId> {
        if person.career.retired
            || person.career.site_role.is_some()
            || !person.is_fit_for_field(
                self.campaign.completed_rounds,
                self.data.households.service_minimum_age_years,
            )
            || person.age_years(self.campaign.completed_rounds)
                >= self.data.lifecycle.elder_age_years
        {
            return None;
        }
        let site = person_site(self.campaign, person.id)?;
        self.view.armies.iter().filter(|army| army.site == site).find_map(|army| {
            let attached = |other: &&Person| matches!(other.assignment,
                PersonAssignment::Formation { formation } if army.formation_ids().any(|id| id == formation));
            let members = self.view.people.iter().filter(|other| other.is_alive()
                && !other.career.retired).collect::<Vec<_>>();
            let replacement = army.commander.is_none_or(|id| {
                (self.campaign.is_pupil(id, person.id)
                    || self.campaign.people[&id].age_years(self.campaign.completed_rounds) >= self.data.lifecycle.elder_age_years)
                    && !members.iter().copied().filter(attached).any(|other| other.id != id
                        && self.campaign.is_pupil(id, other.id)
                        && other.is_fit_for_field(self.campaign.completed_rounds, 17))
            });
            let missing_role = person.class != PersonClass::Recruit
                && !members.iter().copied().filter(attached).any(|other| other.class == person.class);
            (replacement || missing_role).then(|| army.formation_ids().next()).flatten()
        })
    }
}
