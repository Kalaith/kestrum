//! Explicit family, youth-entry and legacy-designation actions.

mod helpers;
use helpers::{
    active_household_for, dependent_children, dependent_children_for_guardian, eligible_for_army,
    has_active_partner, owned_household, owned_person, require_local,
};

use super::family::{safe_owned_site, shared_seasons};
use crate::{
    data::{
        world::{Facility, FactionId, SiteId},
        GameData,
    },
    engine::{person_site, Command, RuleError},
    state::{
        campaign::StrategicCampaign,
        people::{Person, PersonAssignment, PersonId, PersonStatus},
        relationships::{
            FamilyLink, FamilyOrigin, Household, HouseholdEndReason, HouseholdId, HouseholdStatus,
            LegacyCategory, PersonFamily, SuccessorDesignation, SuccessorLink,
        },
    },
};
use std::collections::BTreeMap;

pub(super) fn validate(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    match command {
        Command::FormHousehold {
            first,
            second,
            site,
        } => validate_partnership(campaign, data, owner, *first, *second, *site),
        Command::EndHousehold { household } => {
            let entry = owned_household(campaign, owner, *household)?;
            if matches!(entry.status, HouseholdStatus::Ended { .. }) {
                Err(reason("This household has already ended."))
            } else {
                Ok(())
            }
        }
        Command::SetHouseholdChildraising { household, enabled } => {
            let entry = owned_household(campaign, owner, *household)?;
            if matches!(entry.status, HouseholdStatus::Ended { .. })
                || entry.raising_children == *enabled
            {
                return Err(reason("Choose an active household and a changed setting."));
            }
            if *enabled {
                validate_birth_opportunity(campaign, data, entry)?;
            }
            Ok(())
        }
        Command::AdoptWard { guardian, site } => {
            validate_adoption(campaign, data, owner, *guardian, *site)
        }
        Command::AssignTrainee { person, site } => {
            let child = owned_person(campaign, owner, *person)?;
            if child.assignment != (PersonAssignment::Dependent { site: *site })
                || child.status != PersonStatus::Fit
                || child.class != crate::data::world::PersonClass::Recruit
                || child.career.retired
                || !(data.households.trainee_minimum_age_years
                    ..data.households.service_minimum_age_years)
                    .contains(&child.age_years(campaign.completed_rounds))
            {
                return Err(reason(
                    "Choose a dependent aged thirteen to sixteen for local training.",
                ));
            }
            require_local(campaign, owner, *person, *site, true)
        }
        Command::EnterService { person, formation } => {
            validate_service(campaign, data, owner, *person, *formation)
        }
        Command::InviteApprentice { site } => validate_apprentice(campaign, data, owner, *site),
        Command::DesignateSuccessor {
            predecessor,
            successor,
            category,
            link,
        } => validate_designation(
            campaign,
            data,
            owner,
            *predecessor,
            *successor,
            *category,
            *link,
        ),
        _ => Ok(()),
    }
}

pub(super) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    command: &Command,
) -> Result<Vec<PersonId>, RuleError> {
    match command {
        Command::FormHousehold {
            first,
            second,
            site,
        } => {
            form_household(campaign, data, owner, *first, *second, *site)?;
            Ok(Vec::new())
        }
        Command::EndHousehold { household } => {
            end_household(campaign, owner, *household)?;
            Ok(Vec::new())
        }
        Command::SetHouseholdChildraising { household, enabled } => {
            campaign
                .households
                .get_mut(household)
                .expect("validated household")
                .raising_children = *enabled;
            Ok(Vec::new())
        }
        Command::AdoptWard { guardian, site } => {
            Ok(vec![adopt_ward(campaign, data, owner, *guardian, *site)?])
        }
        Command::AssignTrainee { person, site } => {
            campaign
                .people
                .get_mut(person)
                .expect("validated youth")
                .assignment = PersonAssignment::Trainee { site: *site };
            Ok(Vec::new())
        }
        Command::EnterService { person, formation } => {
            enter_service(campaign, *person, *formation);
            Ok(Vec::new())
        }
        Command::InviteApprentice { site } => {
            Ok(vec![invite_apprentice(campaign, data, owner, *site)?])
        }
        Command::DesignateSuccessor {
            predecessor,
            successor,
            category,
            link,
        } => {
            designate(campaign, *predecessor, *successor, *category, *link);
            Ok(Vec::new())
        }
        _ => Err(RuleError::InvalidState("Unexpected K15 command.".into())),
    }
}

fn validate_partnership(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    first_id: PersonId,
    second_id: PersonId,
    site: SiteId,
) -> Result<(), RuleError> {
    if first_id == second_id {
        return Err(reason("Choose two distinct adults."));
    }
    let first = owned_person(campaign, owner, first_id)?;
    let second = owned_person(campaign, owner, second_id)?;
    let minimum_age = data.households.partnership_minimum_age_years;
    if [first, second].iter().any(|person| {
        !person.is_alive()
            || person.career.retired
            || person.age_years(campaign.completed_rounds) < minimum_age
            || has_active_partner(campaign, person.id)
            || person_site(campaign, person.id) != Some(site)
    }) {
        return Err(reason(
            "Both adults must be living, unpartnered and present at this settlement.",
        ));
    }
    if campaign.known_close_family_relation(first_id, second_id) {
        return Err(reason(
            "Known parent, child or sibling relationships cannot form a household.",
        ));
    }
    if shared_seasons(first, second) < data.households.partnership_shared_seasons {
        return Err(reason(
            "Partnership needs four recorded shared-service seasons.",
        ));
    }
    safe_owned_site(campaign, owner, site)
}

fn validate_birth_opportunity(
    campaign: &StrategicCampaign,
    data: &GameData,
    household: &Household,
) -> Result<(), RuleError> {
    let [first_id, second_id] = household.partners;
    let first = &campaign.people[&first_id];
    let second = &campaign.people[&second_id];
    let ages = [
        first.age_years(campaign.completed_rounds),
        second.age_years(campaign.completed_rounds),
    ];
    if household.home != person_site(campaign, first_id).unwrap_or(SiteId(0))
        || household.home != person_site(campaign, second_id).unwrap_or(SiteId(0))
        || [first, second]
            .iter()
            .any(|person| !person.is_alive() || person.career.retired)
        || ages.iter().any(|age| {
            !(data.households.child_minimum_age_years..=data.households.child_maximum_age_years)
                .contains(age)
        })
    {
        return Err(reason(
            "Both living partners aged twenty to forty must be present at their safe settlement.",
        ));
    }
    safe_owned_site(campaign, household.faction, household.home)
}

fn validate_adoption(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    guardian_id: PersonId,
    site: SiteId,
) -> Result<(), RuleError> {
    let guardian = owned_person(campaign, owner, guardian_id)?;
    if !guardian.is_alive()
        || guardian.career.retired
        || guardian.age_years(campaign.completed_rounds)
            < data.households.partnership_minimum_age_years
        || person_site(campaign, guardian_id) != Some(site)
        || campaign.world.population.get(&site).copied().unwrap_or(0) == 0
    {
        return Err(reason(
            "A living adult guardian needs a local population unit at this site.",
        ));
    }
    safe_owned_site(campaign, owner, site)?;
    let count = active_household_for(campaign, guardian_id).map_or_else(
        || dependent_children_for_guardian(campaign, data, guardian_id),
        |household| dependent_children(campaign, data, household.id),
    );
    if count >= data.households.maximum_dependent_children as usize {
        return Err(reason(
            "This guardian household already has two dependent children.",
        ));
    }
    Ok(())
}

fn validate_service(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    person_id: PersonId,
    formation: Option<crate::state::military::FormationId>,
) -> Result<(), RuleError> {
    let person = owned_person(campaign, owner, person_id)?;
    let (PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site }) =
        person.assignment
    else {
        return Err(reason(
            "Only a dependent or trainee can enter service this way.",
        ));
    };
    if person.status != PersonStatus::Fit
        || person.career.retired
        || person.class != crate::data::world::PersonClass::Recruit
        || person.age_years(campaign.completed_rounds) < data.households.service_minimum_age_years
    {
        return Err(reason(
            "Field service begins at seventeen for a fit Recruit.",
        ));
    }
    safe_owned_site(campaign, owner, site)?;
    if let Some(formation_id) = formation {
        let formation_entry =
            campaign
                .formations
                .get(&formation_id)
                .ok_or(RuleError::UnknownFormation {
                    formation: formation_id,
                })?;
        if formation_entry.faction != owner {
            return Err(RuleError::FormationNotOwned {
                formation: formation_id,
            });
        }
        let army = campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation_id))
            .ok_or_else(|| RuleError::InvalidState("Formation has no army.".into()))?;
        if army.site != site {
            return Err(RuleError::NotColocated);
        }
    }
    Ok(())
}

fn validate_apprentice(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
) -> Result<(), RuleError> {
    let location = campaign
        .world
        .site(site)
        .ok_or(RuleError::UnknownSite { site })?;
    if location.controller != Some(owner)
        || location.habitation < data.households.apprentice_minimum_habitation
        || campaign.supply_path(owner, site).is_none()
        || campaign.sieges.contains_key(&site)
        || campaign.world.structural_damage(site) >= data.economy.facility_failure_damage
        || campaign.world.population.get(&site).copied().unwrap_or(0) == 0
    {
        return Err(reason(
            "A local apprentice needs an owned, supplied Village with residents.",
        ));
    }
    if campaign.factions[&owner].deficit {
        return Err(RuleError::Deficit { faction: owner });
    }
    let year = campaign.completed_rounds / 4;
    if campaign.apprentice_last_invited_year.get(&owner) == Some(&year) {
        return Err(reason(
            "This faction can invite only one apprentice per year.",
        ));
    }
    if campaign.people.values().any(|person| {
        person.faction == owner
            && person.is_alive()
            && person.age_years(campaign.completed_rounds)
                < data.households.service_minimum_age_years
    }) {
        return Err(reason(
            "A faction with dependents cannot invite a separate apprentice.",
        ));
    }
    if campaign.factions[&owner].resources.gold < data.households.apprentice_cost_gold {
        return Err(RuleError::InsufficientResources {
            required: crate::data::economy::Resources {
                gold: data.households.apprentice_cost_gold,
                wood: 0,
                stone: 0,
            },
            available: campaign.factions[&owner].resources,
        });
    }
    Ok(())
}

fn validate_designation(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    predecessor_id: PersonId,
    successor_id: PersonId,
    category: LegacyCategory,
    link: SuccessorLink,
) -> Result<(), RuleError> {
    if predecessor_id == successor_id {
        return Err(reason("A person cannot designate themself."));
    }
    let predecessor = owned_person(campaign, owner, predecessor_id)?;
    let successor = owned_person(campaign, owner, successor_id)?;
    if !successor.is_alive() {
        return Err(reason("Choose a living successor."));
    }
    let site = person_site(campaign, predecessor_id)
        .ok_or_else(|| reason("A predecessor needs a current physical site."))?;
    if !link_exists(campaign, predecessor_id, successor_id, link, site) {
        return Err(reason(
            "That blood, pupil, temple or local political link is not established.",
        ));
    }
    match category {
        LegacyCategory::Command => {
            let army = campaign
                .armies
                .values()
                .find(|army| army.commander == Some(predecessor_id))
                .ok_or_else(|| reason("Only a current commander can name a command successor."))?;
            if !eligible_for_army(campaign, data, successor, army.id) {
                return Err(reason(
                    "A command successor must already be a fit adult in the same army.",
                ));
            }
        }
        LegacyCategory::Household => {
            let same_family = campaign.shares_household(predecessor_id, successor_id);
            if !same_family
                || person_site(campaign, successor_id) != Some(site)
                || !matches!(link, SuccessorLink::Blood | SuccessorLink::Adopted)
            {
                return Err(reason(
                    "A household successor must be a local blood relative or adopted ward.",
                ));
            }
        }
        LegacyCategory::Institution => {
            if person_site(campaign, successor_id) != Some(site)
                || !institution_link(campaign, predecessor, successor, link, site)
            {
                return Err(reason("Institutional succession needs a qualified local pupil, governor or temple disciple."));
            }
        }
        LegacyCategory::Item => {
            if person_site(campaign, successor_id) != Some(site) {
                return Err(reason(
                    "An item successor must be at the current holder's site.",
                ));
            }
        }
    }
    if campaign
        .successors
        .get(&predecessor_id)
        .is_some_and(|items| items.contains_key(&category))
    {
        return Err(reason(
            "Choose at most one successor for each legacy category.",
        ));
    }
    Ok(())
}

fn link_exists(
    campaign: &StrategicCampaign,
    predecessor: PersonId,
    successor: PersonId,
    link: SuccessorLink,
    site: SiteId,
) -> bool {
    match link {
        SuccessorLink::Blood => campaign.known_blood_relation(predecessor, successor),
        SuccessorLink::Adopted => campaign.adopted_relation(predecessor, successor),
        SuccessorLink::Martial => campaign.is_pupil(predecessor, successor),
        SuccessorLink::Religious => {
            campaign.is_pupil(predecessor, successor)
                && campaign.world.site(site).is_some_and(|entry| {
                    entry.controller == Some(campaign.people[&predecessor].faction)
                        && entry.facilities.contains(&Facility::Temple)
                })
                && person_site(campaign, successor) == Some(site)
        }
        SuccessorLink::Political => {
            let succession =
                shared_seasons(&campaign.people[&predecessor], &campaign.people[&successor]);
            person_site(campaign, successor) == Some(site)
                && succession >= 4
                && (campaign.people[&predecessor].career.site_role.is_some()
                    || campaign.people[&successor].career.site_role.is_some())
        }
    }
}

fn institution_link(
    campaign: &StrategicCampaign,
    predecessor: &Person,
    successor: &Person,
    link: SuccessorLink,
    site: SiteId,
) -> bool {
    if !safe_owned_site(campaign, predecessor.faction, site).is_ok()
        || person_site(campaign, successor.id) != Some(site)
    {
        return false;
    }
    match link {
        SuccessorLink::Religious => {
            campaign
                .world
                .site(site)
                .is_some_and(|entry| entry.facilities.contains(&Facility::Temple))
                && campaign.is_pupil(predecessor.id, successor.id)
        }
        SuccessorLink::Political => {
            predecessor.career.site_role.is_some() || successor.career.site_role.is_some()
        }
        SuccessorLink::Martial => {
            campaign
                .world
                .site(site)
                .is_some_and(|entry| entry.facilities.contains(&Facility::TrainingGround))
                && campaign.is_pupil(predecessor.id, successor.id)
        }
        SuccessorLink::Blood | SuccessorLink::Adopted => false,
    }
}

fn form_household(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    first: PersonId,
    second: PersonId,
    site: SiteId,
) -> Result<(), RuleError> {
    validate_partnership(campaign, data, owner, first, second, site)?;
    let id = campaign.next_ids.household;
    campaign.next_ids.household = crate::state::relationships::HouseholdId(
        id.0.checked_add(1).ok_or(RuleError::Overflow {
            field: "household identifiers",
        })?,
    );
    let mut partners = [first, second];
    partners.sort();
    campaign.households.insert(
        id,
        Household {
            id,
            faction: owner,
            partners,
            home: site,
            formed_round: campaign.completed_rounds,
            status: HouseholdStatus::Active,
            raising_children: false,
            last_attempted_year: None,
            last_child_round: None,
        },
    );
    Ok(())
}

fn end_household(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    id: HouseholdId,
) -> Result<(), RuleError> {
    let household = owned_household(campaign, owner, id)?;
    if matches!(household.status, HouseholdStatus::Ended { .. }) {
        return Err(reason("This household has already ended."));
    }
    let household = campaign
        .households
        .get_mut(&id)
        .expect("validated household");
    household.status = HouseholdStatus::Ended {
        completed_rounds: campaign.completed_rounds,
        reason: HouseholdEndReason::Chosen,
    };
    household.raising_children = false;
    Ok(())
}

fn adopt_ward(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    guardian: PersonId,
    site: SiteId,
) -> Result<PersonId, RuleError> {
    validate_adoption(campaign, data, owner, guardian, site)?;
    let id = new_person_id(campaign)?;
    let household = active_household_for(campaign, guardian).map(|entry| entry.id);
    let name = generated_name(campaign, data);
    let population = campaign
        .world
        .population
        .get_mut(&site)
        .expect("validated population");
    *population = population
        .checked_sub(1)
        .ok_or_else(|| reason("This site has no population unit to adopt."))?;
    let birth_round =
        i64::from(campaign.completed_rounds) - i64::from(data.households.ward_age_years) * 4;
    let service_start_round = campaign.completed_rounds;
    insert_family_person(
        campaign,
        FamilyPersonRecord {
            id,
            faction: owner,
            name,
            birth_round,
            service_start_round,
            assignment: PersonAssignment::Dependent { site },
            family: PersonFamily {
                origin: FamilyOrigin::AdoptedWard,
                origin_site: site,
                household,
                links: BTreeMap::from([(guardian, FamilyLink::AdoptiveGuardian)]),
            },
        },
    );
    Ok(id)
}

fn enter_service(
    campaign: &mut StrategicCampaign,
    id: PersonId,
    formation: Option<crate::state::military::FormationId>,
) {
    let person = campaign
        .people
        .get_mut(&id)
        .expect("validated service entrant");
    let site = match person.assignment {
        PersonAssignment::Dependent { site } | PersonAssignment::Trainee { site } => site,
        _ => unreachable!("validated youth assignment"),
    };
    person.service_start_round = campaign.completed_rounds;
    person.assignment = formation.map_or(PersonAssignment::Site { site }, |formation| {
        PersonAssignment::Formation { formation }
    });
}

fn invite_apprentice(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: FactionId,
    site: SiteId,
) -> Result<PersonId, RuleError> {
    validate_apprentice(campaign, data, owner, site)?;
    let id = new_person_id(campaign)?;
    let cost = data.households.apprentice_cost_gold;
    campaign
        .factions
        .get_mut(&owner)
        .expect("validated faction")
        .resources
        .gold -= cost;
    *campaign
        .world
        .population
        .get_mut(&site)
        .expect("validated population") -= 1;
    campaign
        .apprentice_last_invited_year
        .insert(owner, campaign.completed_rounds / 4);
    let birth_round = i64::from(campaign.completed_rounds) - 17 * 4;
    let name = generated_name(campaign, data);
    let service_start_round = campaign.completed_rounds;
    insert_family_person(
        campaign,
        FamilyPersonRecord {
            id,
            faction: owner,
            name,
            birth_round,
            service_start_round,
            assignment: PersonAssignment::Site { site },
            family: PersonFamily {
                origin: FamilyOrigin::LocalApprentice,
                origin_site: site,
                household: None,
                links: BTreeMap::new(),
            },
        },
    );
    Ok(id)
}

fn designate(
    campaign: &mut StrategicCampaign,
    predecessor: PersonId,
    successor: PersonId,
    category: LegacyCategory,
    link: SuccessorLink,
) {
    let site = person_site(campaign, predecessor);
    let army = (category == LegacyCategory::Command)
        .then(|| {
            campaign
                .armies
                .values()
                .find(|army| army.commander == Some(predecessor))
                .map(|army| army.id)
        })
        .flatten();
    campaign.successors.entry(predecessor).or_default().insert(
        category,
        SuccessorDesignation {
            predecessor,
            successor,
            category,
            link,
            designated_round: campaign.completed_rounds,
            shared_seasons: campaign.people[&predecessor]
                .career
                .relationships
                .get(&successor)
                .map_or(0, |relationship| relationship.shared_service_seasons),
            political_role_witnessed: link == SuccessorLink::Political
                && (campaign.people[&predecessor].career.site_role.is_some()
                    || campaign.people[&successor].career.site_role.is_some()),
            link_witnessed: matches!(link, SuccessorLink::Martial | SuccessorLink::Religious),
            army,
            site,
        },
    );
}

fn generated_name(campaign: &mut StrategicCampaign, data: &GameData) -> String {
    let given = campaign
        .rng
        .people
        .below(data.human_names.given_names.len());
    let family = campaign
        .rng
        .people
        .below(data.human_names.family_names.len());
    format!(
        "{} {}",
        data.human_names.given_names[given], data.human_names.family_names[family]
    )
}

fn new_person_id(campaign: &mut StrategicCampaign) -> Result<PersonId, RuleError> {
    let id = campaign.next_ids.person;
    campaign.next_ids.person = PersonId(id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "person identifiers",
    })?);
    Ok(id)
}

struct FamilyPersonRecord {
    id: PersonId,
    faction: FactionId,
    name: String,
    birth_round: i64,
    service_start_round: u32,
    assignment: PersonAssignment,
    family: PersonFamily,
}

fn insert_family_person(campaign: &mut StrategicCampaign, record: FamilyPersonRecord) {
    let person = Person::new_recruit(
        record.id,
        record.faction,
        record.name,
        record.birth_round,
        record.service_start_round,
        record.assignment,
    );
    campaign.people.insert(record.id, person);
    campaign.families.insert(record.id, record.family);
}

fn reason(message: &str) -> RuleError {
    RuleError::Progression(message.into())
}
