//! Ordered decisions for local families, service entry and successor designations.
use super::*;
use crate::state::{
    legacy::LegacyItemCustody,
    relationships::{HouseholdStatus, LegacyCategory, SuccessorLink},
};

impl Planner<'_> {
    pub(super) fn continuity(
        &self,
        people: &[&crate::state::people::Person],
    ) -> Option<AiDecision> {
        self.command_successor(people)
            .or_else(|| self.household_successor(people))
            .or_else(|| self.item_successor(people))
            .or_else(|| self.collect_estate(people))
            .or_else(|| self.enter_local_service(people))
            .or_else(|| self.raise_children())
            .or_else(|| self.form_household(people))
    }
    fn command_successor(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        for army in self
            .campaign
            .armies
            .values()
            .filter(|army| army.faction == self.owner)
        {
            let Some(predecessor) = army.commander else {
                continue;
            };
            let already_named = self
                .campaign
                .successors
                .get(&predecessor)
                .is_some_and(|entries| entries.contains_key(&LegacyCategory::Command));
            if already_named {
                continue;
            }
            if let Some(successor) = people.iter().find(|person| {
                self.campaign.is_pupil(predecessor, person.id)
                    && matches!(person.assignment, PersonAssignment::Formation { formation }
                        if army.formation_ids().any(|id| id == formation))
                    && person.is_fit_for_field(
                        self.campaign.completed_rounds,
                        self.data.rules.leadership.field_min_age_years,
                    )
            }) {
                if let Some(decision) = self.choose(
                    Command::DesignateSuccessor {
                        predecessor,
                        successor: successor.id,
                        category: LegacyCategory::Command,
                        link: SuccessorLink::Martial,
                    },
                    None,
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    fn household_successor(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        for predecessor in people {
            if self
                .campaign
                .successors
                .get(&predecessor.id)
                .is_some_and(|entries| entries.contains_key(&LegacyCategory::Household))
            {
                continue;
            }
            let Some(site) = person_site(self.campaign, predecessor.id) else {
                continue;
            };
            let Some((successor, link)) = people.iter().find_map(|successor| {
                if successor.id == predecessor.id
                    || person_site(self.campaign, successor.id) != Some(site)
                    || !self.campaign.shares_household(predecessor.id, successor.id)
                {
                    return None;
                }
                let link = if self
                    .campaign
                    .known_blood_relation(predecessor.id, successor.id)
                {
                    SuccessorLink::Blood
                } else if self.campaign.adopted_relation(predecessor.id, successor.id) {
                    SuccessorLink::Adopted
                } else {
                    return None;
                };
                Some((successor, link))
            }) else {
                continue;
            };
            if let Some(decision) = self.choose(
                Command::DesignateSuccessor {
                    predecessor: predecessor.id,
                    successor: successor.id,
                    category: LegacyCategory::Household,
                    link,
                },
                None,
            ) {
                return Some(decision);
            }
        }
        None
    }

    fn item_successor(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        for item in self
            .campaign
            .legacy_items
            .values()
            .filter(|item| item.faction == self.owner)
        {
            let LegacyItemCustody::Person(predecessor) = item.custody else {
                continue;
            };
            if self
                .campaign
                .successors
                .get(&predecessor)
                .is_some_and(|entries| entries.contains_key(&LegacyCategory::Item))
            {
                continue;
            }
            let Some(site) = person_site(self.campaign, predecessor) else {
                continue;
            };
            for successor in people.iter().filter(|person| {
                person.id != predecessor
                    && person.is_alive()
                    && person_site(self.campaign, person.id) == Some(site)
            }) {
                let Some(link) = item_link(self.campaign, predecessor, successor.id, site) else {
                    continue;
                };
                if let Some(decision) = self.choose(
                    Command::DesignateSuccessor {
                        predecessor,
                        successor: successor.id,
                        category: LegacyCategory::Item,
                        link,
                    },
                    None,
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    fn collect_estate(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        for item in self
            .campaign
            .legacy_items
            .values()
            .filter(|item| item.faction == self.owner)
        {
            let LegacyItemCustody::SiteEstate(site) = item.custody else {
                continue;
            };
            if self
                .campaign
                .world
                .site(site)
                .is_none_or(|place| place.controller != Some(self.owner))
            {
                continue;
            }
            if let Some(person) = people.iter().find(|person| {
                person.is_alive() && person_site(self.campaign, person.id) == Some(site)
            }) {
                if let Some(decision) = self.choose(
                    Command::TransferLegacyItem {
                        item: item.id,
                        to: person.id,
                    },
                    None,
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    fn enter_local_service(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        for person in people {
            let site = match person.assignment {
                PersonAssignment::Dependent { site } => site,
                PersonAssignment::Trainee { .. } => continue,
                _ => continue,
            };
            let age = person.age_years(self.campaign.completed_rounds);
            let command = if age >= self.data.households.service_minimum_age_years {
                Command::EnterService {
                    person: person.id,
                    formation: self.field_formation(person),
                }
            } else if age >= self.data.households.trainee_minimum_age_years {
                Command::AssignTrainee {
                    person: person.id,
                    site,
                }
            } else {
                continue;
            };
            if let Some(decision) = self.choose(command, None) {
                return Some(decision);
            }
        }
        None
    }

    fn raise_children(&self) -> Option<AiDecision> {
        for household in self.campaign.households.values().filter(|household| {
            household.faction == self.owner
                && matches!(household.status, HouseholdStatus::Active)
                && !household.raising_children
        }) {
            let eligible = household.partners.iter().all(|id| {
                self.campaign.people.get(id).is_some_and(|person| {
                    (self.data.households.child_minimum_age_years
                        ..=self.data.households.child_maximum_age_years)
                        .contains(&person.age_years(self.campaign.completed_rounds))
                        && person_site(self.campaign, *id) == Some(household.home)
                })
            });
            if eligible {
                if let Some(decision) = self.choose(
                    Command::SetHouseholdChildraising {
                        household: household.id,
                        enabled: true,
                    },
                    None,
                ) {
                    return Some(decision);
                }
            }
        }
        None
    }

    fn form_household(&self, people: &[&crate::state::people::Person]) -> Option<AiDecision> {
        for (index, first) in people.iter().enumerate() {
            let Some(site) = person_site(self.campaign, first.id) else {
                continue;
            };
            let Some(second) = people.iter().skip(index + 1).find(|second| {
                person_site(self.campaign, second.id) == Some(site)
                    && first
                        .career
                        .relationships
                        .get(&second.id)
                        .is_some_and(|relation| {
                            relation.shared_service_seasons
                                >= self.data.households.partnership_shared_seasons
                        })
            }) else {
                continue;
            };
            if let Some(decision) = self.choose(
                Command::FormHousehold {
                    first: first.id,
                    second: second.id,
                    site,
                },
                None,
            ) {
                return Some(decision);
            }
        }
        None
    }
}
