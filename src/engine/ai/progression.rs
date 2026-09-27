//! The NPC planner uses the same evidence options and validated commands as the player.

use super::*;
use crate::{
    data::world::{Facility, PersonClass, SiteTag},
    engine::career_options,
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
