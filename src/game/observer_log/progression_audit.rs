//! Compact formation service and hero counts for accepted Observer boundaries.

use super::person_progression_audit;
use kestrum::{
    data::GameData,
    state::{
        military::FormationId, people::PersonAssignment, relationships::FamilyOrigin,
        StrategicCampaign,
    },
};
use serde_json::{json, Value};

pub(super) fn faction_progression_counts(campaign: &StrategicCampaign) -> Value {
    let factions = campaign
        .factions
        .values()
        .map(|faction| {
            let people = campaign
                .people
                .values()
                .filter(|person| person.faction == faction.id)
                .collect::<Vec<_>>();
            let living_attached = people
                .iter()
                .filter(|person| {
                    person.is_alive()
                        && !person.career.retired
                        && matches!(person.assignment, PersonAssignment::Formation { .. })
                })
                .copied()
                .collect::<Vec<_>>();
            let armies = campaign
                .armies
                .values()
                .filter(|army| army.faction == faction.id)
                .collect::<Vec<_>>();
            let formations = campaign
                .formations
                .values()
                .filter(|formation| formation.faction == faction.id && formation.headcount > 0)
                .collect::<Vec<_>>();
            let unstaffed_formations = formations
                .iter()
                .filter(|formation| {
                    !living_attached.iter().any(|person| {
                        person.assignment
                            == (PersonAssignment::Formation {
                                formation: formation.id,
                            })
                    })
                })
                .count();
            let leaderless_armies = armies
                .iter()
                .filter(|army| {
                    army.commander
                        .and_then(|id| campaign.people.get(&id))
                        .is_none_or(|person| {
                            !person.is_alive()
                                || person.career.retired
                                || !matches!(person.assignment, PersonAssignment::Formation { formation }
                                    if army.formation_ids().any(|id| id == formation))
                        })
                })
                .count();
            let attached_apprentices = living_attached
                .iter()
                .filter(|person| {
                    person.career.emergence.is_some() && person.career.recognition.is_none()
                })
                .count();
            let attached_heroes = living_attached
                .iter()
                .filter(|person| person.career.recognition.is_some())
                .count();
            let apprentice_armies = armies
                .iter()
                .filter(|army| {
                    living_attached.iter().any(|person| {
                        person.career.emergence.is_some()
                            && person.career.recognition.is_none()
                            && matches!(person.assignment, PersonAssignment::Formation { formation }
                                if army.formation_ids().any(|id| id == formation))
                    })
                })
                .count();
            let hero_armies = armies
                .iter()
                .filter(|army| {
                    living_attached.iter().any(|person| {
                        person.career.recognition.is_some()
                            && matches!(person.assignment, PersonAssignment::Formation { formation }
                                if army.formation_ids().any(|id| id == formation))
                    })
                })
                .count();
            json!({
                "faction": faction.id,
                "people_total": people.len(),
                "emerged_total": people.iter().filter(|person| person.career.emergence.is_some()).count(),
                "invited_apprentices_total": campaign.families.iter().filter(|(id, family)| {
                    family.origin == FamilyOrigin::LocalApprentice
                        && campaign.people.get(id).is_some_and(|person| person.faction == faction.id)
                }).count(),
                "births_total": campaign.families.iter().filter(|(id, family)| {
                    family.origin == FamilyOrigin::Birth
                        && campaign.people.get(id).is_some_and(|person| person.faction == faction.id)
                }).count(),
                "living_attached_apprentices": attached_apprentices,
                "living_attached_heroes": attached_heroes,
                "apprentice_armies": apprentice_armies,
                "hero_armies": hero_armies,
                "armies_total": armies.len(),
                "retirees_total": people.iter().filter(|person| person.career.retired).count(),
                "deaths_total": people.iter().filter(|person| !person.is_alive()).count(),
                "unstaffed_formations": unstaffed_formations,
                "leaderless_armies": leaderless_armies
            })
        })
        .collect::<Vec<_>>();
    json!({ "round": campaign.completed_rounds, "factions": factions })
}

pub(super) fn formation_service_audit(campaign: &StrategicCampaign, data: &GameData) -> Value {
    let threshold = data.progression.emergence.vacant_slot_engagements;
    let opportunities = campaign
        .formations
        .values()
        .filter_map(|formation| {
            let service = formation.service.recent.last()?;
            (service.completed_rounds.saturating_add(1) == campaign.completed_rounds)
                .then_some((formation, service))
        })
        .map(|(formation, service)| {
            let emerged = campaign.people.values().find(|person| {
                person.career.emergence.as_ref().is_some_and(|record| {
                    record.completed_rounds == campaign.completed_rounds
                        && record.source_formation == formation.id
                })
            });
            let members = formation_members(campaign, formation.id);
            let slot_was_occupied = campaign.people.values().any(|person| {
                person.assignment
                    == (PersonAssignment::Formation {
                        formation: formation.id,
                    })
                    && !person.career.emergence.as_ref().is_some_and(|record| {
                        record.completed_rounds == campaign.completed_rounds
                            && record.source_formation == formation.id
                    })
            });
            let reason = if emerged.is_some() {
                "emerged"
            } else if formation.headcount == 0 {
                "no_living_troops"
            } else if slot_was_occupied {
                "named_slot_staffed"
            } else if formation.service.vacancy_service_progress >= threshold {
                "threshold_reached_without_current_meaningful_service"
            } else {
                "collecting_vacant_slot_service"
            };
            json!({
                "faction": formation.faction,
                "formation": formation.id,
                "troop": formation.kind,
                "headcount": formation.headcount,
                "capacity": formation.capacity,
                "season": service.completed_rounds,
                "service_xp": service.xp,
                "tier": formation.service.tier,
                "qualifying_encounters": service.encounters.iter().filter(|entry| entry.meaningful).count(),
                "vacant_slot_service_progress": formation.service.vacancy_service_progress,
                "vacant_slot_service_required": threshold,
                "threshold_reached": emerged.is_some() || formation.service.vacancy_service_progress >= threshold,
                "eligible": formation.headcount > 0 && !slot_was_occupied,
                "named_slot_occupied_after_boundary": slot_was_occupied,
                "current_named_members": members,
                "emerged_person": emerged.map(|person| person_progression_audit(campaign, person.id)),
                "threshold_outcome": reason
            })
        })
        .collect::<Vec<_>>();
    json!({
        "round": campaign.completed_rounds,
        "vacant_slot_service_required": threshold,
        "formation_opportunities": opportunities
    })
}

fn formation_members(campaign: &StrategicCampaign, formation: FormationId) -> Vec<Value> {
    campaign
        .people
        .values()
        .filter(|person| person.assignment == (PersonAssignment::Formation { formation }))
        .map(|person| person_progression_audit(campaign, person.id))
        .collect()
}
