//! Sparse local households, age-gated service entry, and qualified succession.

mod commands;
mod family;
mod names;
mod options;
pub(crate) use names::fresh_person_name;
pub use options::{household_option, HouseholdAction, HouseholdOption, HouseholdSelection};

use crate::{
    data::GameData,
    engine::{Command, RuleError},
    state::{
        campaign::{FactionStatus, StrategicCampaign},
        people::PersonId,
        relationships::{HouseholdEndReason, HouseholdStatus, LegacyCategory, SuccessionNotice},
    },
};

pub(super) fn validate(
    campaign: &StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    command: &Command,
) -> Result<(), RuleError> {
    commands::validate(campaign, data, owner, command)
}

pub(super) fn execute(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    owner: crate::data::world::FactionId,
    command: &Command,
) -> Result<Vec<PersonId>, RuleError> {
    commands::execute(campaign, data, owner, command)
}

pub(super) fn resolve_boundary(
    campaign: &mut StrategicCampaign,
    data: &GameData,
) -> Result<Vec<PersonId>, RuleError> {
    family::resolve_boundary(campaign, data)
}

pub(super) fn reconcile(before: &StrategicCampaign, candidate: &mut StrategicCampaign) {
    family::reconcile_homes(before, candidate);
    for (id, household) in &mut candidate.households {
        let defeated = candidate
            .factions
            .get(&household.faction)
            .is_some_and(|faction| faction.status == FactionStatus::Eliminated);
        if defeated && matches!(household.status, HouseholdStatus::Active) {
            household.status = HouseholdStatus::Ended {
                completed_rounds: candidate.completed_rounds,
                reason: HouseholdEndReason::FactionDefeated,
            };
            household.raising_children = false;
        }
        let _ = id;
    }
    let designations = std::mem::take(&mut candidate.successors);
    candidate.successors = designations
        .into_iter()
        .filter_map(|(predecessor, categories)| {
            let faction = candidate.people.get(&predecessor).map(|person| person.faction)?;
            if candidate
                .factions
                .get(&faction)
                .is_none_or(|entry| entry.status == FactionStatus::Eliminated)
            {
                return None;
            }
            let retained = categories
                .into_iter()
                .filter(|(category, designation)| {
                    let successor = candidate.people.get(&designation.successor);
                    let valid = successor.is_some_and(|person| {
                        person.faction == faction && person.is_alive()
                    });
                    valid && (*category != LegacyCategory::Command || {
                        let Some(army_id) = designation.army else { return false };
                        let Some(army) = candidate.armies.get(&army_id) else { return false };
                        army.faction == faction
                            && successor.is_some_and(|person| {
                                !person.career.retired
                                    && person.is_fit_for_field(candidate.completed_rounds, 17)
                                    && matches!(person.assignment,
                                        crate::state::people::PersonAssignment::Formation { formation }
                                            if army.formation_ids().any(|id| id == formation))
                            })
                    })
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            (!retained.is_empty()).then_some((predecessor, retained))
        })
        .collect();
}

pub(super) fn notices(
    before: &StrategicCampaign,
    after: &StrategicCampaign,
) -> Vec<SuccessionNotice> {
    before
        .armies
        .values()
        .filter_map(|army| {
            let predecessor = army.commander?;
            let old = before.people.get(&predecessor)?;
            let current = after.people.get(&predecessor)?;
            if !old.is_alive()
                || old.career.retired
                || current.is_alive() && !current.career.retired
            {
                return None;
            }
            after.armies.get(&army.id).map(|updated| SuccessionNotice {
                predecessor,
                successor: updated.commander,
                army: army.id,
            })
        })
        .collect()
}

pub(crate) fn release_household_after_death(campaign: &mut StrategicCampaign, person: PersonId) {
    for household in campaign.households.values_mut().filter(|household| {
        household.partners.contains(&person) && matches!(household.status, HouseholdStatus::Active)
    }) {
        household.status = HouseholdStatus::Ended {
            completed_rounds: campaign.completed_rounds,
            reason: HouseholdEndReason::PartnerDied,
        };
        household.raising_children = false;
    }
}

pub(crate) fn resolve_person_departure(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    person: PersonId,
) {
    let Some(entry) = campaign.people.get(&person) else {
        return;
    };
    let faction = entry.faction;
    if campaign
        .factions
        .get(&faction)
        .is_none_or(|entry| entry.status == FactionStatus::Eliminated)
    {
        return;
    }
    let armies = campaign
        .armies
        .values()
        .filter(|army| army.commander == Some(person))
        .map(|army| army.id)
        .collect::<Vec<_>>();
    for army in &armies {
        campaign
            .armies
            .get_mut(army)
            .expect("commander army")
            .commander = None;
    }
    for army_id in armies {
        let designated = campaign
            .successors
            .get(&person)
            .and_then(|categories| categories.get(&LegacyCategory::Command))
            .filter(|designation| designation.army == Some(army_id))
            .map(|designation| designation.successor)
            .filter(|successor| eligible_commander(campaign, data, army_id, *successor));
        let successor = designated.or_else(|| fallback_commander(campaign, data, person, army_id));
        campaign
            .armies
            .get_mut(&army_id)
            .expect("commander army")
            .commander = successor;
    }
    if let Some(categories) = campaign.successors.get_mut(&person) {
        categories.remove(&LegacyCategory::Command);
        if categories.is_empty() {
            campaign.successors.remove(&person);
        }
    }
}

fn fallback_commander(
    campaign: &StrategicCampaign,
    data: &GameData,
    predecessor: PersonId,
    army: crate::state::military::ArmyId,
) -> Option<PersonId> {
    let eligible = campaign
        .armies
        .get(&army)?
        .formation_ids()
        .flat_map(|formation| {
            campaign.people.values().filter(move |person| {
                person.assignment
                    == (crate::state::people::PersonAssignment::Formation { formation })
                    && eligible_commander(campaign, data, army, person.id)
            })
        })
        .map(|person| person.id)
        .collect::<std::collections::BTreeSet<_>>();
    eligible
        .iter()
        .filter(|id| campaign.is_pupil(predecessor, **id))
        .copied()
        .max_by_key(|id| {
            (
                campaign.people[id].age_years(campaign.completed_rounds),
                std::cmp::Reverse(*id),
            )
        })
        .or_else(|| {
            eligible
                .into_iter()
                .min_by_key(|id| (std::cmp::Reverse(command_evidence(campaign, *id)), *id))
        })
}

fn command_evidence(campaign: &StrategicCampaign, person: PersonId) -> u32 {
    use crate::state::evidence::EvidenceKind;
    let entry = &campaign.people[&person];
    entry
        .evidence
        .counts
        .get(&EvidenceKind::AssumedCommand)
        .copied()
        .unwrap_or(0)
        .saturating_add(
            entry
                .evidence
                .counts
                .get(&EvidenceKind::CommandedVictory)
                .copied()
                .unwrap_or(0),
        )
        .saturating_add(
            entry
                .career
                .discipline_service_seasons
                .get(&crate::data::progression::TrainingDiscipline::Command)
                .copied()
                .unwrap_or(0),
        )
}

fn eligible_commander(
    campaign: &StrategicCampaign,
    data: &GameData,
    army: crate::state::military::ArmyId,
    person: PersonId,
) -> bool {
    let Some(entry) = campaign.people.get(&person) else {
        return false;
    };
    campaign.armies.get(&army).is_some_and(|army| {
        entry.faction == army.faction
            && !entry.career.retired
            && entry.is_fit_for_field(
                campaign.completed_rounds,
                data.rules.leadership.field_min_age_years,
            )
            && matches!(entry.assignment, crate::state::people::PersonAssignment::Formation { formation }
                if army.formation_ids().any(|id| id == formation))
    })
}
