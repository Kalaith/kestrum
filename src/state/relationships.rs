//! Stable household and successor identity; kinship is separate from short-lived service ties.

use super::{
    people::{PersonAssignment, PersonId},
    StrategicCampaign,
};
use crate::{
    data::{
        economy::Habitation,
        world::{FactionId, SiteId},
        GameData,
    },
    state::military::ArmyId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HouseholdId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HouseholdEndReason {
    Chosen,
    PartnerDied,
    SiteCaptured,
    FactionDefeated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HouseholdStatus {
    Active,
    Ended {
        completed_rounds: u32,
        reason: HouseholdEndReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Household {
    pub id: HouseholdId,
    pub faction: FactionId,
    pub partners: [PersonId; 2],
    pub home: SiteId,
    pub formed_round: u32,
    pub status: HouseholdStatus,
    pub raising_children: bool,
    pub last_attempted_year: Option<u32>,
    pub last_child_round: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilyOrigin {
    Birth,
    AdoptedWard,
    LocalApprentice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilyLink {
    BiologicalParent,
    AdoptiveGuardian,
    ApprenticeOf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonFamily {
    pub origin: FamilyOrigin,
    pub origin_site: SiteId,
    #[serde(default)]
    pub household: Option<HouseholdId>,
    #[serde(default)]
    pub links: BTreeMap<PersonId, FamilyLink>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyCategory {
    Command,
    Item,
    Household,
    Institution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuccessorLink {
    Blood,
    Martial,
    Religious,
    Political,
    Adopted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuccessorDesignation {
    pub predecessor: PersonId,
    pub successor: PersonId,
    pub category: LegacyCategory,
    pub link: SuccessorLink,
    pub designated_round: u32,
    #[serde(default)]
    pub shared_seasons: u32,
    #[serde(default)]
    pub political_role_witnessed: bool,
    /// The validated command records transient Martial/Religious links as historical evidence.
    #[serde(default)]
    pub link_witnessed: bool,
    #[serde(default)]
    pub army: Option<ArmyId>,
    #[serde(default)]
    pub site: Option<SiteId>,
}

pub type SuccessorRegister = BTreeMap<PersonId, BTreeMap<LegacyCategory, SuccessorDesignation>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuccessionNotice {
    pub predecessor: PersonId,
    pub successor: Option<PersonId>,
    pub army: ArmyId,
}

impl StrategicCampaign {
    pub(crate) fn validate_relationships(&self, data: &GameData) -> Result<(), String> {
        let mut partnered = std::collections::BTreeSet::new();
        for (id, household) in &self.households {
            require(
                *id == household.id
                    && id.0 > 0
                    && *id < self.next_ids.household
                    && household_faction(self, household.partners) == Some(household.faction)
                    && household.formed_round <= self.completed_rounds
                    && self.world.site(household.home).is_some()
                    && household.partners[0] < household.partners[1],
                "households",
                "invalid identity, faction, chronology, home or partner order",
            )?;
            for partner in household.partners {
                let Some(person) = self.people.get(&partner) else {
                    return Err("campaign.households: unknown partner".into());
                };
                require(
                    person.faction == household.faction
                        && (matches!(household.status, HouseholdStatus::Ended { .. })
                            || person.is_alive() && partnered.insert(partner)),
                    "households.partners",
                    "partner is foreign, duplicated or unavailable",
                )?;
            }
            if matches!(household.status, HouseholdStatus::Active) {
                require(
                    self.world.site(household.home).is_some_and(|site| {
                        site.controller == Some(household.faction)
                            && site.habitation != Habitation::Unsettled
                    }),
                    "households.home",
                    "active household must retain its friendly inhabited seat",
                )?;
            }
            require(
                household
                    .last_attempted_year
                    .is_none_or(|year| year <= self.completed_rounds / 4)
                    && household.last_child_round.is_none_or(|round| {
                        round >= household.formed_round && round <= self.completed_rounds
                    }),
                "households.births",
                "future or pre-household calendar fact",
            )?;
            if let HouseholdStatus::Ended {
                completed_rounds, ..
            } = household.status
            {
                require(
                    completed_rounds >= household.formed_round
                        && completed_rounds <= self.completed_rounds
                        && !household.raising_children,
                    "households.status",
                    "invalid end date or ended household still opting into births",
                )?;
            }
            let dependent_count = self
                .families
                .iter()
                .filter(|(person_id, family)| {
                    family.household == Some(*id)
                        && self.people.get(person_id).is_some_and(|person| {
                            person.is_alive()
                                && person.age_years(self.completed_rounds)
                                    < data.households.service_minimum_age_years
                                && matches!(
                                    person.assignment,
                                    PersonAssignment::Dependent { .. }
                                        | PersonAssignment::Trainee { .. }
                                )
                        })
                })
                .count();
            require(
                dependent_count <= data.households.maximum_dependent_children as usize,
                "households.children",
                "more dependent children than the authored household limit",
            )?;
        }
        for (person_id, family) in &self.families {
            let person = self.people.get(person_id);
            require(
                person_id.0 > 0
                    && *person_id < self.next_ids.person
                    && self.world.site(family.origin_site).is_some()
                    && family
                        .household
                        .is_none_or(|id| self.households.contains_key(&id))
                    && family.links.len() <= 2
                    && family.links.iter().all(|(relative, link)| {
                        relative.0 > 0
                            && *relative < self.next_ids.person
                            && relative != person_id
                            && person.is_none_or(|person| {
                                self.people.get(relative).is_none_or(|other| {
                                    other.faction == person.faction
                                        && other.birth_round < person.birth_round
                                })
                            })
                            && match family.origin {
                                FamilyOrigin::Birth => *link == FamilyLink::BiologicalParent,
                                FamilyOrigin::AdoptedWard => *link == FamilyLink::AdoptiveGuardian,
                                FamilyOrigin::LocalApprentice => *link == FamilyLink::ApprenticeOf,
                            }
                    }),
                "families",
                "invalid identity, origin, household or guardian chronology",
            )?;
            match family.origin {
                FamilyOrigin::Birth => require(
                    family.links.len() == 2
                        && family.household.is_none_or(|id| {
                            self.households.get(&id).is_some_and(|household| {
                                family.links.keys().copied().collect::<Vec<_>>()
                                    == household.partners.to_vec()
                            })
                        })
                        && person.is_none_or(|person| {
                            !matches!(
                                person.assignment,
                                PersonAssignment::Dependent { .. }
                                    | PersonAssignment::Trainee { .. }
                            ) || person.class == crate::data::world::PersonClass::Recruit
                        }),
                    "families.birth",
                    "a birth needs two biological parents; dependents remain untrained recruits",
                )?,
                FamilyOrigin::AdoptedWard => require(
                    (1..=2).contains(&family.links.len())
                        && family
                            .links
                            .values()
                            .all(|link| *link == FamilyLink::AdoptiveGuardian)
                        && family.household.is_none_or(|id| {
                            self.households.get(&id).is_some_and(|household| {
                                family
                                    .links
                                    .keys()
                                    .any(|guardian| household.partners.contains(guardian))
                            })
                        }),
                    "families.adoption",
                    "an adopted ward needs one or two adoptive guardians",
                )?,
                FamilyOrigin::LocalApprentice => require(
                    family.links.len() <= 1
                        && family
                            .links
                            .values()
                            .all(|link| *link == FamilyLink::ApprenticeOf),
                    "families.apprentice",
                    "a local apprentice has at most one real sponsor",
                )?,
            }
            if let Some(household_id) = family.household {
                let household = &self.households[&household_id];
                require(
                    person.is_none_or(|person| household.faction == person.faction),
                    "families.household",
                    "household belongs to another faction",
                )?;
            }
        }
        for person in self.people.values() {
            if matches!(
                person.assignment,
                PersonAssignment::Dependent { .. } | PersonAssignment::Trainee { .. }
            ) {
                require(
                    self.families.get(&person.id).is_some_and(|family| {
                        matches!(
                            family.origin,
                            FamilyOrigin::Birth | FamilyOrigin::AdoptedWard
                        )
                    }),
                    "people.assignment",
                    "only a recorded child or adopted ward can be a dependent or trainee",
                )?;
            }
        }
        for (faction, year) in &self.apprentice_last_invited_year {
            require(
                self.factions.contains_key(faction) && *year <= self.completed_rounds / 4,
                "apprentice_last_invited_year",
                "unknown faction or future year",
            )?;
        }
        self.validate_successors()
    }

    fn validate_successors(&self) -> Result<(), String> {
        for (predecessor, categories) in &self.successors {
            let Some(owner) = self.people.get(predecessor) else {
                return Err("campaign.successors: unknown predecessor".into());
            };
            require(
                !categories.is_empty() && categories.len() <= 4,
                "successors",
                "each predecessor has at most one designation per legacy category",
            )?;
            for (category, designation) in categories {
                let successor = self
                    .people
                    .get(&designation.successor)
                    .ok_or("campaign.successors: unknown successor")?;
                require(
                    designation.predecessor == *predecessor
                        && designation.category == *category
                        && designation.successor != *predecessor
                        && designation.designated_round <= self.completed_rounds
                        && owner.faction == successor.faction
                        && successor.is_alive()
                        && self.successor_link_exists(*predecessor, designation),
                    "successors.designation",
                    "invalid category, identity, faction, date or relationship evidence",
                )?;
                if *category == LegacyCategory::Command {
                    require(
                        designation.army.is_some_and(|army_id| {
                            self.armies.get(&army_id).is_some_and(|army| {
                                army.faction == owner.faction
                                    && successor.is_fit_for_field(self.completed_rounds, 17)
                                    && !successor.career.retired
                                    && army.formation_ids().any(|formation| {
                                        successor.assignment
                                            == (PersonAssignment::Formation { formation })
                                    })
                            })
                        }),
                        "successors.command",
                        "command successor must already serve with the designated army",
                    )?;
                }
                if *category == LegacyCategory::Item {
                    require(
                        designation
                            .site
                            .is_some_and(|site| self.world.site(site).is_some()),
                        "successors.item",
                        "item designation needs its existing estate site",
                    )?;
                }
                if *category == LegacyCategory::Household {
                    require(
                        self.shares_household(*predecessor, designation.successor),
                        "successors.household",
                        "household heir must share the family seat or a known kin link",
                    )?;
                }
                if *category == LegacyCategory::Institution {
                    require(
                        designation
                            .site
                            .is_some_and(|site| self.world.site(site).is_some())
                            && matches!(
                                designation.link,
                                SuccessorLink::Religious
                                    | SuccessorLink::Political
                                    | SuccessorLink::Martial
                            ),
                        "successors.institution",
                        "institution needs a retained physical site and supported local link",
                    )?;
                }
            }
        }
        Ok(())
    }

    fn successor_link_exists(
        &self,
        predecessor: PersonId,
        designation: &SuccessorDesignation,
    ) -> bool {
        match designation.link {
            SuccessorLink::Blood => self.known_blood_relation(predecessor, designation.successor),
            SuccessorLink::Adopted => self.adopted_relation(predecessor, designation.successor),
            SuccessorLink::Martial => {
                designation.link_witnessed || self.is_pupil(predecessor, designation.successor)
            }
            SuccessorLink::Religious => {
                designation.link_witnessed
                    || designation.site.is_some_and(|site| {
                        self.world.site(site).is_some_and(|entry| {
                            entry
                                .facilities
                                .contains(&crate::data::world::Facility::Temple)
                        }) && self.is_pupil(predecessor, designation.successor)
                    })
            }
            SuccessorLink::Political => {
                designation.shared_seasons >= 4
                    && designation.political_role_witnessed
                    && designation
                        .site
                        .is_some_and(|site| self.world.site(site).is_some())
            }
        }
    }

    pub fn is_pupil(&self, mentor: PersonId, learner: PersonId) -> bool {
        self.people.get(&learner).is_some_and(|person| {
            person
                .career
                .completed_mentors
                .iter()
                .any(|record| record.mentor == mentor)
        }) || self
            .mentorships
            .get(&learner)
            .is_some_and(|mentorship| mentorship.mentor == mentor)
    }

    pub fn known_blood_relation(&self, first: PersonId, second: PersonId) -> bool {
        if self.family_ancestors(first).contains(&second)
            || self.family_ancestors(second).contains(&first)
        {
            return true;
        }
        let first_ancestors = self.family_ancestors(first);
        self.family_ancestors(second)
            .iter()
            .any(|ancestor| first_ancestors.contains(ancestor))
    }

    pub fn known_family_relation(&self, first: PersonId, second: PersonId) -> bool {
        let first_ancestors = self.family_ancestors_by_links(first);
        let second_ancestors = self.family_ancestors_by_links(second);
        first_ancestors.contains(&second)
            || second_ancestors.contains(&first)
            || first_ancestors
                .iter()
                .any(|ancestor| second_ancestors.contains(ancestor))
    }

    pub fn known_close_family_relation(&self, first: PersonId, second: PersonId) -> bool {
        let first_guardians = self.family_guardians(first);
        let second_guardians = self.family_guardians(second);
        first_guardians.contains(&second)
            || second_guardians.contains(&first)
            || first_guardians
                .iter()
                .any(|guardian| second_guardians.contains(guardian))
    }

    pub fn adopted_relation(&self, first: PersonId, second: PersonId) -> bool {
        self.families
            .get(&first)
            .is_some_and(|family| family.links.get(&second) == Some(&FamilyLink::AdoptiveGuardian))
            || self.families.get(&second).is_some_and(|family| {
                family.links.get(&first) == Some(&FamilyLink::AdoptiveGuardian)
            })
    }

    pub fn shares_household(&self, first: PersonId, second: PersonId) -> bool {
        if self.known_family_relation(first, second) {
            return true;
        }
        let first_households = self
            .families
            .get(&first)
            .and_then(|family| family.household)
            .into_iter()
            .chain(
                self.households
                    .values()
                    .filter(|household| household.partners.contains(&first))
                    .map(|household| household.id),
            )
            .collect::<std::collections::BTreeSet<_>>();
        let second_households = self
            .families
            .get(&second)
            .and_then(|family| family.household)
            .into_iter()
            .chain(
                self.households
                    .values()
                    .filter(|household| household.partners.contains(&second))
                    .map(|household| household.id),
            )
            .collect::<std::collections::BTreeSet<_>>();
        !first_households.is_disjoint(&second_households)
    }

    fn family_ancestors(&self, person: PersonId) -> std::collections::BTreeSet<PersonId> {
        let mut ancestors = std::collections::BTreeSet::new();
        let mut pending = self
            .families
            .get(&person)
            .map(|family| {
                family
                    .links
                    .iter()
                    .filter(|(_, link)| **link == FamilyLink::BiologicalParent)
                    .map(|(id, _)| *id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        while let Some(parent) = pending.pop() {
            if !ancestors.insert(parent) {
                continue;
            }
            if let Some(family) = self.families.get(&parent) {
                pending.extend(
                    family
                        .links
                        .iter()
                        .filter(|(_, link)| **link == FamilyLink::BiologicalParent)
                        .map(|(id, _)| *id),
                );
            }
        }
        ancestors
    }

    fn family_guardians(&self, person: PersonId) -> std::collections::BTreeSet<PersonId> {
        self.families
            .get(&person)
            .map(|family| {
                family
                    .links
                    .iter()
                    .filter(|(_, link)| **link != FamilyLink::ApprenticeOf)
                    .map(|(id, _)| *id)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn family_ancestors_by_links(&self, person: PersonId) -> std::collections::BTreeSet<PersonId> {
        let mut ancestors = std::collections::BTreeSet::new();
        let mut pending = self
            .families
            .get(&person)
            .map(|family| {
                family
                    .links
                    .iter()
                    .filter(|(_, link)| **link != FamilyLink::ApprenticeOf)
                    .map(|(id, _)| *id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        while let Some(parent) = pending.pop() {
            if !ancestors.insert(parent) {
                continue;
            }
            if let Some(family) = self.families.get(&parent) {
                pending.extend(
                    family
                        .links
                        .iter()
                        .filter(|(_, link)| **link != FamilyLink::ApprenticeOf)
                        .map(|(id, _)| *id),
                );
            }
        }
        ancestors
    }
}

fn household_faction(campaign: &StrategicCampaign, partners: [PersonId; 2]) -> Option<FactionId> {
    let first = campaign.people.get(&partners[0])?;
    let second = campaign.people.get(&partners[1])?;
    (first.faction == second.faction).then_some(first.faction)
}

fn require(valid: bool, field: &str, reason: &str) -> Result<(), String> {
    if valid {
        Ok(())
    } else {
        Err(format!("campaign.{field}: {reason}"))
    }
}
