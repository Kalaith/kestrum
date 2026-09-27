//! Relationships remember only direct joint service and repeated mutual combat.

use crate::{
    data::GameData,
    engine::RuleError,
    state::{
        battle::BattleSideReport,
        campaign::{DomainFact, DomainFactKind},
        people::{PersonId, PersonRelationship},
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn record(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    facts: &[DomainFact],
) -> Result<(), RuleError> {
    for fact in facts {
        match &fact.kind {
            DomainFactKind::ArmiesMoved {
                faction,
                movement: Some(receipt),
                ..
            } => {
                let group = receipt
                    .people
                    .iter()
                    .filter_map(|id| {
                        campaign
                            .people
                            .get(id)
                            .filter(|person| person.is_alive() && person.faction == *faction)
                    })
                    .map(|person| person.id)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>();
                for (index, first) in group.iter().enumerate() {
                    for second in group.iter().skip(index + 1) {
                        update_shared(campaign, *first, *second, fact.completed_rounds);
                    }
                }
            }
            DomainFactKind::BattleResolved { battle, .. } => {
                let report = campaign
                    .battles
                    .get(battle)
                    .ok_or_else(|| {
                        RuleError::InvalidState(
                            "Relationship receipt references a missing battle.".into(),
                        )
                    })?
                    .clone();
                let attacker = participants(campaign, &report.attacker);
                let Some(defender) = report.defender.faction_side() else {
                    continue;
                };
                let defender = participants(campaign, defender);
                for group in [&attacker, &defender] {
                    for (index, first) in group.iter().enumerate() {
                        for second in group.iter().skip(index + 1) {
                            update_shared(campaign, *first, *second, report.completed_rounds);
                        }
                    }
                }
                for first in &attacker {
                    for second in &defender {
                        update_mutual(campaign, *first, *second, report.completed_rounds);
                    }
                }
            }
            _ => continue,
        }
    }
    prune(campaign, data.progression.relationships_per_person);
    Ok(())
}

fn participants(campaign: &StrategicCampaign, side: &BattleSideReport) -> Vec<PersonId> {
    side.armies
        .iter()
        .flat_map(|army| &army.people)
        .filter_map(|snapshot| {
            campaign
                .people
                .get(&snapshot.id)
                .filter(|person| person.is_alive() && person.faction == side.faction)
                .map(|person| person.id)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn relation(
    campaign: &mut StrategicCampaign,
    first: PersonId,
    second: PersonId,
) -> &mut PersonRelationship {
    campaign
        .people
        .get_mut(&first)
        .expect("participant")
        .career
        .relationships
        .entry(second)
        .or_insert(PersonRelationship {
            shared_service_seasons: 0,
            last_shared_service_round: None,
            mutual_combat_rounds: 0,
            last_mutual_combat_round: None,
        })
}

fn update_shared(campaign: &mut StrategicCampaign, first: PersonId, second: PersonId, round: u32) {
    for (owner, other) in [(first, second), (second, first)] {
        let relationship = relation(campaign, owner, other);
        if relationship.last_shared_service_round != Some(round) {
            relationship.shared_service_seasons =
                relationship.shared_service_seasons.saturating_add(1);
            relationship.last_shared_service_round = Some(round);
        }
    }
}

fn update_mutual(campaign: &mut StrategicCampaign, first: PersonId, second: PersonId, round: u32) {
    for (owner, other) in [(first, second), (second, first)] {
        let relationship = relation(campaign, owner, other);
        relationship.mutual_combat_rounds = relationship.mutual_combat_rounds.saturating_add(1);
        relationship.last_mutual_combat_round = Some(round);
    }
}

fn prune(campaign: &mut StrategicCampaign, maximum: usize) {
    let mut keep = BTreeMap::<PersonId, BTreeSet<PersonId>>::new();
    for (id, person) in &campaign.people {
        if !person.is_alive() {
            continue;
        }
        let mut entries = person
            .career
            .relationships
            .iter()
            .filter(|(other, _)| {
                campaign
                    .people
                    .get(other)
                    .is_some_and(|person| person.is_alive())
            })
            .map(|(other, relation)| {
                (
                    *other,
                    relation
                        .shared_service_seasons
                        .saturating_add(relation.mutual_combat_rounds),
                    relation
                        .last_mutual_combat_round
                        .unwrap_or(0)
                        .max(relation.last_shared_service_round.unwrap_or(0)),
                )
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|(other, score, last)| {
            (std::cmp::Reverse(*score), std::cmp::Reverse(*last), *other)
        });
        keep.insert(
            *id,
            entries
                .into_iter()
                .take(maximum)
                .map(|(other, _, _)| other)
                .collect(),
        );
    }
    // A relationship stays symmetric; if either person's bounded set excludes it, both drop it.
    let pairs = campaign
        .people
        .iter()
        .flat_map(|(id, person)| {
            person
                .career
                .relationships
                .keys()
                .filter_map(move |other| (*id < *other).then_some((*id, *other)))
        })
        .collect::<Vec<_>>();
    for (first, second) in pairs {
        if !keep.get(&first).is_some_and(|ids| ids.contains(&second))
            || !keep.get(&second).is_some_and(|ids| ids.contains(&first))
        {
            if let Some(person) = campaign.people.get_mut(&first) {
                person.career.relationships.remove(&second);
            }
            if let Some(person) = campaign.people.get_mut(&second) {
                person.career.relationships.remove(&first);
            }
        }
    }
}
