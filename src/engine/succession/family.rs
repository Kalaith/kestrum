//! Spring household attempts, sparse dependents and local capture cleanup.

use crate::{
    data::{
        economy::Habitation,
        world::{DiplomaticState, FactionId, SiteId},
        GameData,
    },
    engine::{person_site, RuleError},
    state::{
        campaign::StrategicCampaign,
        people::{Person, PersonAssignment, PersonId},
        relationships::{
            FamilyLink, FamilyOrigin, HouseholdEndReason, HouseholdId, HouseholdStatus,
            PersonFamily,
        },
    },
};
use std::collections::BTreeMap;

pub(super) fn shared_seasons(first: &Person, second: &Person) -> u32 {
    first
        .career
        .relationships
        .get(&second.id)
        .map_or(0, |relationship| relationship.shared_service_seasons)
}

pub(super) fn safe_owned_site(
    campaign: &StrategicCampaign,
    owner: FactionId,
    site_id: SiteId,
) -> Result<(), RuleError> {
    let site = campaign
        .world
        .site(site_id)
        .ok_or(RuleError::UnknownSite { site: site_id })?;
    if site.controller != Some(owner)
        || site.habitation == crate::data::economy::Habitation::Unsettled
        || campaign.sieges.contains_key(&site_id)
        || campaign.active_threat(site_id).is_some()
        || campaign.battles.values().any(|battle| {
            battle.site == site_id && battle.completed_rounds == campaign.completed_rounds
        })
        || hostile_at_or_near(campaign, owner, site_id)
    {
        return Err(RuleError::Progression(
            "Household decisions need a safe, friendly inhabited settlement.".into(),
        ));
    }
    Ok(())
}

fn hostile_at_or_near(campaign: &StrategicCampaign, owner: FactionId, site: SiteId) -> bool {
    let mut range = campaign
        .world
        .adjacent_sites(site)
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    range.insert(site);
    campaign
        .armies
        .values()
        .any(|army| range.contains(&army.site) && at_war(campaign, owner, army.faction))
}

fn at_war(campaign: &StrategicCampaign, first: FactionId, second: FactionId) -> bool {
    let mut pair = [first, second];
    pair.sort();
    campaign
        .relations
        .iter()
        .find(|relation| relation.factions == pair)
        .is_some_and(|relation| relation.state == DiplomaticState::War)
}

pub(super) fn resolve_boundary(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<Vec<PersonId>, RuleError> {
    age_trainees(campaign, data);
    if campaign.completed_rounds == 0 || !campaign.completed_rounds.is_multiple_of(4) {
        return Ok(Vec::new());
    }
    let year = campaign.completed_rounds / 4;
    let households = campaign.households.keys().copied().collect::<Vec<_>>();
    let mut created = Vec::new();
    for id in households {
        let Some(household) = campaign.households.get(&id).cloned() else {
            continue;
        };
        if !eligible_for_birth(campaign, data, &household)
            || household.last_attempted_year == Some(year)
            || household.last_child_round.is_some_and(|previous| {
                campaign.completed_rounds.saturating_sub(previous)
                    < data.households.birth_gap_seasons
            })
            || dependent_count(campaign, data, id)
                >= data.households.maximum_dependent_children as usize
        {
            continue;
        }
        campaign
            .households
            .get_mut(&id)
            .expect("eligible household")
            .last_attempted_year = Some(year);
        let attempt = campaign.rng.people.below(1000);
        if attempt >= data.households.birth_chance_permille as usize {
            continue;
        }
        let child = add_birth(campaign, data, &household)?;
        campaign
            .households
            .get_mut(&id)
            .expect("birth household")
            .last_child_round = Some(campaign.completed_rounds);
        created.push(child);
    }
    Ok(created)
}

fn eligible_for_birth(
    campaign: &StrategicCampaign,
    data: &GameData,
    household: &crate::state::relationships::Household,
) -> bool {
    if !household.raising_children || !matches!(household.status, HouseholdStatus::Active) {
        return false;
    }
    let [first, second] = household.partners;
    let Some(first) = campaign.people.get(&first) else {
        return false;
    };
    let Some(second) = campaign.people.get(&second) else {
        return false;
    };
    [first, second].iter().all(|person| {
        person.is_alive()
            && !person.career.retired
            && (data.households.child_minimum_age_years..=data.households.child_maximum_age_years)
                .contains(&person.age_years(campaign.completed_rounds))
            && person_site(campaign, person.id) == Some(household.home)
    }) && safe_owned_site(campaign, household.faction, household.home).is_ok()
}

fn dependent_count(campaign: &StrategicCampaign, data: &GameData, household: HouseholdId) -> usize {
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

fn add_birth(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    household: &crate::state::relationships::Household,
) -> Result<PersonId, RuleError> {
    let id = campaign.next_ids.person;
    campaign.next_ids.person = PersonId(id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "person identifiers",
    })?);
    let given = campaign
        .rng
        .people
        .below(data.human_names.given_names.len());
    let family = campaign
        .rng
        .people
        .below(data.human_names.family_names.len());
    let appearance = crate::engine::portraits::allocate_for_person(campaign, &data.portraits, id)
        .map_err(RuleError::InvalidState)?;
    let person = Person::new_recruit(
        id,
        household.faction,
        format!(
            "{} {}",
            data.human_names.given_names[given], data.human_names.family_names[family]
        ),
        i64::from(campaign.completed_rounds),
        campaign.completed_rounds,
        PersonAssignment::Dependent {
            site: household.home,
        },
        appearance,
    );
    campaign.people.insert(id, person);
    campaign.families.insert(
        id,
        PersonFamily {
            origin: FamilyOrigin::Birth,
            origin_site: household.home,
            household: Some(household.id),
            links: BTreeMap::from([
                (household.partners[0], FamilyLink::BiologicalParent),
                (household.partners[1], FamilyLink::BiologicalParent),
            ]),
        },
    );
    Ok(id)
}

fn age_trainees(campaign: &mut StrategicCampaign, data: &GameData) {
    let aged = campaign
        .people
        .values()
        .filter_map(|person| {
            (matches!(person.assignment, PersonAssignment::Trainee { .. })
                && person.age_years(campaign.completed_rounds)
                    >= data.households.service_minimum_age_years)
                .then_some(person.id)
        })
        .collect::<Vec<_>>();
    for id in aged {
        let person = campaign.people.get_mut(&id).expect("trainee");
        if let PersonAssignment::Trainee { site } = person.assignment {
            person.assignment = PersonAssignment::Dependent { site };
        }
    }
}

pub(super) fn reconcile_homes(before: &StrategicCampaign, after: &mut StrategicCampaign) {
    let captured = before
        .world
        .sites
        .iter()
        .filter(|old| {
            old.controller.is_some()
                && after
                    .world
                    .site(old.id)
                    .is_some_and(|new| new.controller != old.controller)
        })
        .map(|site| (site.id, site.controller.expect("filtered owner")))
        .collect::<Vec<_>>();
    for (site, old_owner) in captured {
        reconcile_home_loss(after, site, old_owner, HouseholdEndReason::SiteCaptured);
    }
    let abandoned = before
        .world
        .sites
        .iter()
        .filter(|old| old.controller.is_some() && old.habitation != Habitation::Unsettled)
        .filter_map(|old| {
            let new = after.world.site(old.id)?;
            (new.controller == old.controller && new.habitation == Habitation::Unsettled)
                .then_some((old.id, old.controller.expect("filtered owner")))
        })
        .collect::<Vec<_>>();
    for (site, owner) in abandoned {
        reconcile_home_loss(after, site, owner, HouseholdEndReason::SiteAbandoned);
    }
}

fn reconcile_home_loss(
    campaign: &mut StrategicCampaign,
    site: SiteId,
    faction: FactionId,
    reason: HouseholdEndReason,
) {
    for household in campaign.households.values_mut().filter(|household| {
        household.home == site
            && household.faction == faction
            && matches!(household.status, HouseholdStatus::Active)
    }) {
        household.status = HouseholdStatus::Ended {
            completed_rounds: campaign.completed_rounds,
            reason,
        };
        household.raising_children = false;
    }
    for person in campaign.people.values_mut().filter(|person| {
        matches!(person.assignment, PersonAssignment::Trainee { site: assigned } if assigned == site)
            && person.faction == faction
    }) {
        person.assignment = PersonAssignment::Dependent { site };
    }
}
