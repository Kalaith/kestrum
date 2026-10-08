//! Compact formation service and hero counts for accepted Observer boundaries.

use super::person_progression_audit;
use kestrum::{
    data::{world::FactionId, GameData},
    state::{
        evidence::{EvidenceKind, SeasonService},
        military::FormationId,
        people::PersonAssignment,
        relationships::FamilyOrigin,
        StrategicCampaign,
    },
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

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
    let selected_sources = campaign
        .factions
        .keys()
        .map(|faction| (*faction, selected_service_source(campaign, *faction)))
        .collect::<BTreeMap<_, _>>();
    let opportunities = campaign
        .formations
        .values()
        .filter_map(|formation| {
            let service = formation.service.recent.last()?;
            (service.completed_rounds.saturating_add(1) == campaign.completed_rounds)
                .then_some((formation, service))
        })
        .map(|(formation, service)| {
            let selected = selected_sources
                .get(&formation.faction)
                .copied()
                .flatten()
                == Some(formation.id);
            let independent = campaign.is_independent(formation.faction);
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
            } else if !independent {
                "faction_not_independent"
            } else if !selected {
                "not_selected_as_highest_service"
            } else if service.xp == 0 {
                "selected_source_had_no_xp"
            } else {
                "chance_not_met"
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
                "eligible": formation.headcount > 0 && service.xp > 0 && independent,
                "selected_source": selected,
                "slot_occupied_before_attempt": slot_was_occupied,
                "current_named_members": members,
                "emerged_person": emerged.map(|person| person_progression_audit(campaign, person.id)),
                "no_emergence_reason": reason
            })
        })
        .collect::<Vec<_>>();
    let selected = selected_sources
        .into_iter()
        .map(|(faction, formation)| {
            let Some(formation) = formation else {
                return (faction, Value::Null);
            };
            let service = campaign.formations[&formation]
                .service
                .recent
                .last()
                .expect("selected source has current service");
            let chance = if service.xp > 0 && campaign.is_independent(faction) {
                Some(json!({
                    "adult_roster_count": emergence_roster_count(campaign, faction),
                    "chance_permille": emergence_chance(campaign, data, faction, formation, service)
                }))
            } else {
                None
            };
            (faction, json!({ "formation": formation, "chance": chance }))
        })
        .collect::<BTreeMap<_, _>>();
    json!({
        "round": campaign.completed_rounds,
        "selected_sources": selected,
        "formation_opportunities": opportunities
    })
}

fn selected_service_source(
    campaign: &StrategicCampaign,
    faction: FactionId,
) -> Option<FormationId> {
    campaign
        .formations
        .values()
        .filter(|formation| formation.faction == faction && formation.headcount > 0)
        .filter_map(|formation| {
            let service = formation.service.recent.last()?;
            (service.completed_rounds.saturating_add(1) == campaign.completed_rounds)
                .then_some((formation, service))
        })
        .max_by_key(|(formation, service)| {
            (
                service.xp,
                formation.service.tier,
                std::cmp::Reverse(formation.id),
            )
        })
        .map(|(formation, _)| formation.id)
}

fn formation_members(campaign: &StrategicCampaign, formation: FormationId) -> Vec<Value> {
    campaign
        .people
        .values()
        .filter(|person| person.assignment == (PersonAssignment::Formation { formation }))
        .map(|person| person_progression_audit(campaign, person.id))
        .collect()
}

fn emergence_chance(
    campaign: &StrategicCampaign,
    data: &GameData,
    faction: FactionId,
    formation: FormationId,
    service: &SeasonService,
) -> u64 {
    let adults = emergence_roster_count(campaign, faction);
    let rules = &data.progression.emergence;
    let curve = u64::from(rules.roster_base).saturating_mul(u64::from(rules.roster_factor))
        / (u64::from(rules.roster_square_factor)
            .saturating_mul(adults.saturating_mul(adults))
            .saturating_add(u64::from(rules.roster_factor))
            .max(1));
    let tier_multiplier = match campaign.formations[&formation].service.tier {
        kestrum::state::evidence::Veterancy::Ordinary => 1000,
        kestrum::state::evidence::Veterancy::Seasoned => rules.seasoned_multiplier_permille,
        kestrum::state::evidence::Veterancy::Veteran => rules.veteran_multiplier_permille,
    } as u64;
    let exceptional = service
        .encounters
        .iter()
        .any(|encounter| encounter.tags.contains(&EvidenceKind::SurvivedOutnumbered))
        && service.encounters.iter().any(|encounter| {
            encounter.tags.contains(&EvidenceKind::CapturedAnchor)
                || encounter.tags.contains(&EvidenceKind::DefendedAnchor)
        });
    let adjusted = curve.saturating_mul(tier_multiplier) / 1000;
    let adjusted = if exceptional {
        adjusted.saturating_mul(u64::from(rules.exceptional_multiplier_permille)) / 1000
    } else {
        adjusted
    };
    adjusted.min(u64::from(rules.maximum_chance_permille))
}

fn emergence_roster_count(campaign: &StrategicCampaign, faction: FactionId) -> u64 {
    let adults = campaign
        .people
        .values()
        .filter(|person| {
            person.faction == faction
                && person.is_alive()
                && !person.career.retired
                && person.age_years(campaign.completed_rounds) >= 18
        })
        .count() as u64;
    let boundary_emergences = campaign
        .people
        .values()
        .filter(|person| {
            person.faction == faction
                && person.is_alive()
                && !person.career.retired
                && person.age_years(campaign.completed_rounds) >= 18
                && person
                    .career
                    .emergence
                    .as_ref()
                    .is_some_and(|record| record.completed_rounds == campaign.completed_rounds)
        })
        .count() as u64;
    adults.saturating_sub(boundary_emergences)
}
