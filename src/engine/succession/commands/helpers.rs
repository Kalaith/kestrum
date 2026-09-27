//! Shared validation helpers for household and succession commands.

use super::super::family::safe_owned_site;
use super::reason;
use crate::{
    data::{
        economy::Habitation,
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{person_site, RuleError},
    state::{
        campaign::StrategicCampaign,
        people::{Person, PersonAssignment, PersonId},
        relationships::{Household, HouseholdId, HouseholdStatus},
    },
};

fn validate_local(
    campaign: &StrategicCampaign,
    owner: FactionId,
    person: PersonId,
    site: SiteId,
    safe: bool,
) -> Result<(), RuleError> {
    if person_site(campaign, person) != Some(site) {
        return Err(RuleError::NotColocated);
    }
    if safe {
        safe_owned_site(campaign, owner, site)
    } else if campaign.world.site(site).is_some_and(|entry| {
        entry.controller == Some(owner)
            && entry.habitation != Habitation::Unsettled
            && !campaign.sieges.contains_key(&site)
    }) {
        Ok(())
    } else {
        Err(reason("Choose a friendly, inhabited, unbesieged site."))
    }
}

pub(super) fn require_local(
    campaign: &StrategicCampaign,
    owner: FactionId,
    person: PersonId,
    site: SiteId,
    safe: bool,
) -> Result<(), RuleError> {
    validate_local(campaign, owner, person, site, safe)
}

pub(super) fn active_household_for(
    campaign: &StrategicCampaign,
    person: PersonId,
) -> Option<&Household> {
    campaign.households.values().find(|household| {
        household.partners.contains(&person) && matches!(household.status, HouseholdStatus::Active)
    })
}

pub(super) fn has_active_partner(campaign: &StrategicCampaign, person: PersonId) -> bool {
    active_household_for(campaign, person).is_some()
}

pub(super) fn dependent_children(
    campaign: &StrategicCampaign,
    data: &GameData,
    household: HouseholdId,
) -> usize {
    campaign
        .families
        .iter()
        .filter(|(id, family)| {
            family.household == Some(household)
                && campaign.people.get(id).is_some_and(|person| {
                    person.is_alive()
                        && person.age_years(campaign.completed_rounds)
                            < data.households.service_minimum_age_years
                        && matches!(
                            person.assignment,
                            PersonAssignment::Dependent { .. } | PersonAssignment::Trainee { .. }
                        )
                })
        })
        .count()
}

pub(super) fn dependent_children_for_guardian(
    campaign: &StrategicCampaign,
    data: &GameData,
    guardian: PersonId,
) -> usize {
    campaign
        .families
        .iter()
        .filter(|(id, family)| {
            family.links.contains_key(&guardian)
                && campaign.people.get(id).is_some_and(|person| {
                    person.is_alive()
                        && person.age_years(campaign.completed_rounds)
                            < data.households.service_minimum_age_years
                        && matches!(
                            person.assignment,
                            PersonAssignment::Dependent { .. } | PersonAssignment::Trainee { .. }
                        )
                })
        })
        .count()
}

pub(super) fn owned_household(
    campaign: &StrategicCampaign,
    owner: FactionId,
    household: HouseholdId,
) -> Result<&Household, RuleError> {
    campaign
        .households
        .get(&household)
        .filter(|entry| entry.faction == owner)
        .ok_or_else(|| reason("That household is unavailable."))
}

pub(super) fn owned_person(
    campaign: &StrategicCampaign,
    owner: FactionId,
    id: PersonId,
) -> Result<&Person, RuleError> {
    let person = campaign
        .people
        .get(&id)
        .ok_or(RuleError::UnknownPerson { person: id })?;
    if person.faction != owner {
        return Err(RuleError::PersonNotOwned { person: id });
    }
    Ok(person)
}

pub(super) fn eligible_for_army(
    campaign: &StrategicCampaign,
    data: &GameData,
    person: &Person,
    army_id: crate::state::military::ArmyId,
) -> bool {
    campaign.armies.get(&army_id).is_some_and(|army| {
        person.is_alive()
            && !person.career.retired
            && person.is_fit_for_field(
                campaign.completed_rounds,
                data.rules.leadership.field_min_age_years,
            )
            && person.faction == army.faction
            && matches!(person.assignment, PersonAssignment::Formation { formation }
                if army.formation_ids().any(|id| id == formation))
    })
}
