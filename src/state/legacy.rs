//! Stable, mundane heirlooms whose custody is current game state.

use super::{
    people::{PersonAssignment, PersonId, PersonStatus},
    relationships::LegacyCategory,
    StrategicCampaign,
};
use crate::data::world::{FactionId, PersonClass, SiteId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LegacyItemId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegacyItemKind {
    MusterSword,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum LegacyItemCustody {
    Person(PersonId),
    SiteEstate(SiteId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyItem {
    pub id: LegacyItemId,
    pub faction: FactionId,
    pub name: String,
    pub kind: LegacyItemKind,
    pub created_round: u32,
    pub custody: LegacyItemCustody,
}

impl StrategicCampaign {
    pub(crate) fn initialize_legacy_items(&mut self) -> Result<(), String> {
        let officers = self
            .people
            .values()
            .filter(|person| {
                person.class == PersonClass::Officer
                    && person.is_alive()
                    && person.career.emergence.is_none()
            })
            .map(|person| (person.id, person.faction, person.name.clone()))
            .collect::<Vec<_>>();
        for (person, faction, name) in officers {
            let id = self.next_ids.legacy_item;
            self.next_ids.legacy_item = LegacyItemId(
                id.0.checked_add(1)
                    .ok_or("legacy item identifiers overflow")?,
            );
            self.legacy_items.insert(
                id,
                LegacyItem {
                    id,
                    faction,
                    name: format!("{name}'s Muster Sword"),
                    kind: LegacyItemKind::MusterSword,
                    created_round: self.completed_rounds,
                    custody: LegacyItemCustody::Person(person),
                },
            );
        }
        Ok(())
    }

    pub(crate) fn validate_legacy_items(&self) -> Result<(), String> {
        if self.next_ids.legacy_item.0 == 0 {
            return Err("campaign.next_ids.legacy_item: zero next identifier".into());
        }
        for (id, item) in &self.legacy_items {
            if *id != item.id
                || id.0 == 0
                || *id >= self.next_ids.legacy_item
                || item.name.trim().is_empty()
                || item.name.chars().count() > 96
                || item.name.chars().any(char::is_control)
                || item.created_round > self.completed_rounds
                || !self.factions.contains_key(&item.faction)
            {
                return Err(
                    "campaign.legacy_items: invalid identity, name, date or faction".into(),
                );
            }
            match item.custody {
                LegacyItemCustody::Person(person_id) => {
                    let Some(person) = self.people.get(&person_id) else {
                        return Err("campaign.legacy_items: missing living custodian".into());
                    };
                    if !person.is_alive() || person.faction != item.faction {
                        return Err("campaign.legacy_items: custodian is dead or foreign".into());
                    }
                    if person_site(self, person_id).is_none() {
                        return Err(
                            "campaign.legacy_items: custodian has no physical location".into()
                        );
                    }
                }
                LegacyItemCustody::SiteEstate(site) => {
                    if self.world.site(site).is_none() {
                        return Err("campaign.legacy_items: unknown estate site".into());
                    }
                }
            }
        }
        for (site, round) in &self.world.founded_rounds {
            if self.world.site(*site).is_none() || *round > self.completed_rounds {
                return Err("campaign.world.founded_rounds: invalid site or date".into());
            }
        }
        for (person, years) in &self.history.person_last_reminded {
            let Some(entry) = self.people.get(person) else {
                return Err("campaign.history.person_last_reminded: unknown person".into());
            };
            let current_years = self
                .completed_rounds
                .saturating_sub(entry.service_start_round)
                / 4;
            if *years == 0 || years % 10 != 0 || *years > current_years {
                return Err("campaign.history.person_last_reminded: invalid milestone year".into());
            }
        }
        for (site, years) in &self.history.site_last_reminded {
            let Some(founded) = self.world.founded_rounds.get(site) else {
                return Err("campaign.history.site_last_reminded: unknown foundation date".into());
            };
            let current_years = self.completed_rounds.saturating_sub(*founded) / 4;
            if *years == 0 || years % 10 != 0 || *years > current_years {
                return Err("campaign.history.site_last_reminded: invalid milestone year".into());
            }
        }
        Ok(())
    }
}

pub fn person_site(campaign: &StrategicCampaign, person: PersonId) -> Option<SiteId> {
    if let PersonStatus::Dead { site, .. } = campaign.people.get(&person)?.status {
        return Some(site);
    }
    match campaign.people.get(&person)?.assignment {
        PersonAssignment::Site { site }
        | PersonAssignment::Dependent { site }
        | PersonAssignment::Trainee { site } => Some(site),
        PersonAssignment::Formation { formation } => campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation))
            .map(|army| army.site),
        PersonAssignment::Dead => None,
    }
}

impl StrategicCampaign {
    pub(crate) fn settle_departed_heirloom(&mut self, person: PersonId) {
        let Some(site) = person_site(self, person) else {
            return;
        };
        let faction = self.people[&person].faction;
        let heir = self
            .successors
            .get(&person)
            .and_then(|categories| categories.get(&LegacyCategory::Item))
            .map(|designation| designation.successor)
            .filter(|heir| {
                self.people
                    .get(heir)
                    .is_some_and(|entry| entry.is_alive() && entry.faction == faction)
                    && person_site(self, *heir) == Some(site)
            });
        for item in self
            .legacy_items
            .values_mut()
            .filter(|item| item.custody == LegacyItemCustody::Person(person))
        {
            item.custody = heir.map_or(
                LegacyItemCustody::SiteEstate(site),
                LegacyItemCustody::Person,
            );
        }
    }
}
