//! Boundary-ordered aging, recovery placement, retirement and settlement roles.

use super::{person_site, Command, RuleError};
use crate::{
    data::{economy::Habitation, world::FactionId, GameData},
    state::{
        people::{PersonAssignment, PersonId, PersonSiteRole, PersonStatus},
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, VecDeque};

pub(super) fn validate(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    match command {
        Command::RecoverPersonAtSite { person, site } => {
            let entry = owned_person(campaign, owner, *person)?;
            if !matches!(entry.status, PersonStatus::Wounded { .. }) {
                return Err(progress("Only a wounded person needs site recovery."));
            }
            validate_local_site(campaign, owner, *person, *site, true)?;
        }
        Command::RetirePerson { person, site } => {
            let entry = owned_person(campaign, owner, *person)?;
            if !entry.is_alive()
                || entry.career.retired
                || entry.age_years(campaign.completed_rounds) < 17
            {
                return Err(progress(
                    "Retirement is available to a living adult who still serves.",
                ));
            }
            validate_local_site(campaign, owner, *person, *site, false)?;
        }
        Command::AppointGovernor { person, site } => {
            let entry = owned_person(campaign, owner, *person)?;
            if !entry.is_alive()
                || entry.status != PersonStatus::Fit
                || entry.age_years(campaign.completed_rounds)
                    < data.rules.leadership.field_min_age_years
                || entry.career.site_role.is_some()
            {
                return Err(progress(
                    "A governor must be a fit, living adult without another site role.",
                ));
            }
            validate_local_site(campaign, owner, *person, *site, false)?;
            if campaign.people.values().any(|other| {
                other.id != *person
                    && other.career.site_role == Some(PersonSiteRole::Governor)
                    && other.assignment == (PersonAssignment::Site { site: *site })
            }) {
                return Err(progress("This settlement already has a governor."));
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    _owner: FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    match command {
        Command::RecoverPersonAtSite { person, site } => {
            campaign
                .people
                .get_mut(person)
                .expect("validated person")
                .assignment = PersonAssignment::Site { site: *site };
            clear_commander(campaign, *person);
        }
        Command::RetirePerson { person, site } => retire(campaign, data, *person, *site, false),
        Command::AppointGovernor { person, site } => {
            let entry = campaign.people.get_mut(person).expect("validated person");
            entry.assignment = PersonAssignment::Site { site: *site };
            entry.career.site_role = Some(PersonSiteRole::Governor);
            clear_commander(campaign, *person);
        }
        _ => {
            return Err(RuleError::InvalidState(
                "Unexpected lifecycle command.".into(),
            ))
        }
    }
    Ok(())
}

fn clear_commander(campaign: &mut StrategicCampaign, person: PersonId) {
    for army in campaign.armies.values_mut() {
        if army.commander == Some(person) {
            army.commander = None;
        }
    }
}

pub(super) fn resolve_boundary(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<Vec<PersonId>, RuleError> {
    let round = campaign.completed_rounds;
    let birthdays = campaign
        .people
        .values()
        .filter(|person| {
            person.is_alive() && round > 0 && person.age_years(round) > person.age_years(round - 1)
        })
        .map(|person| person.id)
        .collect::<Vec<_>>();
    for id in birthdays {
        let age = campaign.people[&id].age_years(round);
        let chance = data.lifecycle.death_chance_permille(age);
        if chance > 0 && campaign.rng.people.below(1000) < chance as usize {
            if let Some(site) = person_site(campaign, id) {
                mark_dead(campaign, data, id, site);
            }
        }
    }

    let retirees = campaign
        .people
        .values()
        .filter(|person| {
            person.is_alive()
                && !person.career.retired
                && matches!(person.assignment, PersonAssignment::Formation { .. })
                && person.age_years(round) >= data.lifecycle.automatic_retirement_age_years
        })
        .map(|person| person.id)
        .collect::<Vec<_>>();
    let mut player_retirements = Vec::new();
    for id in retirees {
        let current = person_site(campaign, id).ok_or_else(|| {
            RuleError::InvalidState("A serving elder has no physical site.".into())
        })?;
        let owner = campaign.people[&id].faction;
        let refuge = nearest_refuge(campaign, owner, current);
        let destination = refuge.unwrap_or(current);
        retire(campaign, data, id, destination, true);
        if owner == campaign.player {
            player_retirements.push(id);
        }
        if refuge.is_none() {
            let person = campaign.people.get_mut(&id).expect("retired person");
            person.status = PersonStatus::Displaced {
                completed_rounds: round,
                site: current,
            };
            person.movement_spent = 0;
        }
    }
    Ok(player_retirements)
}

pub(super) fn reconcile_roles(campaign: &mut StrategicCampaign) {
    let invalid = campaign
        .people
        .values()
        .filter(|person| {
            person.career.site_role == Some(PersonSiteRole::Governor)
                && (!matches!(person.status, PersonStatus::Fit)
                    || !matches!(person.assignment, PersonAssignment::Site { site }
                    if campaign.world.site(site).is_some_and(|entry| {
                        entry.controller == Some(person.faction)
                            && entry.habitation != Habitation::Unsettled
                            && !campaign.sieges.contains_key(&site)
                    })))
        })
        .map(|person| person.id)
        .collect::<Vec<_>>();
    for id in invalid {
        campaign
            .people
            .get_mut(&id)
            .expect("invalid governor")
            .career
            .site_role = None;
    }
}

pub(crate) fn mark_dead(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    id: PersonId,
    site: crate::data::world::SiteId,
) {
    let person = campaign.people.get_mut(&id).expect("dead person");
    person.status = PersonStatus::Dead {
        completed_rounds: campaign.completed_rounds,
        site,
    };
    person.assignment = PersonAssignment::Dead;
    person.movement_spent = 0;
    person.career.course = None;
    person.career.site_role = None;
    campaign.settle_departed_heirloom(id);
    campaign
        .mentorships
        .retain(|learner, mentorship| *learner != id && mentorship.mentor != id);
    super::succession::release_household_after_death(campaign, id);
    super::succession::resolve_person_departure(campaign, data, id);
}

fn retire(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    id: PersonId,
    site: crate::data::world::SiteId,
    automatic: bool,
) {
    let person = campaign.people.get_mut(&id).expect("retired person");
    person.career.retired = true;
    person.career.course = None;
    person.career.site_role = None;
    person.assignment = PersonAssignment::Site { site };
    if automatic {
        person.career.automatic_retirement_round = Some(campaign.completed_rounds);
    }
    campaign.mentorships.remove(&id);
    super::succession::resolve_person_departure(campaign, data, id);
}

fn nearest_refuge(
    campaign: &StrategicCampaign,
    owner: FactionId,
    origin: crate::data::world::SiteId,
) -> Option<crate::data::world::SiteId> {
    let mut distances = BTreeMap::from([(origin, 0_u32)]);
    let mut queue = VecDeque::from([origin]);
    while let Some(site) = queue.pop_front() {
        let next = distances[&site].saturating_add(1);
        for neighbor in campaign.world.adjacent_sites(site) {
            if let std::collections::btree_map::Entry::Vacant(entry) = distances.entry(neighbor) {
                entry.insert(next);
                queue.push_back(neighbor);
            }
        }
    }
    distances
        .into_iter()
        .filter(|(site, _)| retirement_site(campaign, owner, *site))
        .min_by_key(|(site, distance)| (*distance, *site))
        .map(|(site, _)| site)
}

fn retirement_site(
    campaign: &StrategicCampaign,
    owner: FactionId,
    site: crate::data::world::SiteId,
) -> bool {
    campaign.world.site(site).is_some_and(|entry| {
        entry.controller == Some(owner)
            && entry.habitation != Habitation::Unsettled
            && !campaign.sieges.contains_key(&site)
    })
}

fn validate_local_site(
    campaign: &StrategicCampaign,
    owner: FactionId,
    person: PersonId,
    site: crate::data::world::SiteId,
    require_supply: bool,
) -> Result<(), RuleError> {
    let selected = campaign
        .world
        .site(site)
        .ok_or(RuleError::UnknownSite { site })?;
    if selected.controller != Some(owner)
        || selected.habitation == Habitation::Unsettled
        || campaign.sieges.contains_key(&site)
        || require_supply && campaign.supply_path(owner, site).is_none()
    {
        return Err(progress(
            "Choose a friendly, inhabited, unbesieged site; recovery also needs supply.",
        ));
    }
    if person_site(campaign, person) != Some(site) {
        return Err(progress(
            "The person must already be at this local settlement.",
        ));
    }
    Ok(())
}

fn owned_person(
    campaign: &StrategicCampaign,
    owner: FactionId,
    person: PersonId,
) -> Result<&crate::state::people::Person, RuleError> {
    let entry = campaign
        .people
        .get(&person)
        .ok_or(RuleError::UnknownPerson { person })?;
    if entry.faction != owner {
        return Err(RuleError::PersonNotOwned { person });
    }
    Ok(entry)
}

fn progress(reason: &str) -> RuleError {
    RuleError::Progression(reason.into())
}
