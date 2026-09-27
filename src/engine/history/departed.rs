//! Full dead-person records expire; witnessed identities remain in their own labels.

use crate::{
    data::progression::HistoryRules,
    state::{
        campaign::DomainFactKind,
        people::{PersonId, PersonStatus},
        StrategicCampaign,
    },
};
use std::collections::BTreeSet;

pub(super) fn prune(campaign: &mut StrategicCampaign, rules: &HistoryRules) {
    // Pending receipts still need their real participants during migration. At a
    // normal completed boundary these have already been consumed. Command links
    // remain strong; current valid death handling has already released them.
    let referenced = referenced_people(campaign);
    let now = campaign.completed_rounds;
    campaign.people.retain(|id, person| match person.status {
        PersonStatus::Dead {
            completed_rounds, ..
        } => {
            referenced.contains(id)
                || now.saturating_sub(completed_rounds) <= rules.departed_max_age_rounds
        }
        _ => true,
    });
    let mut departed: Vec<_> = campaign
        .people
        .values()
        .filter_map(|person| match person.status {
            PersonStatus::Dead {
                completed_rounds, ..
            } if !referenced.contains(&person.id) => Some((completed_rounds, person.id)),
            _ => None,
        })
        .collect();
    departed.sort();
    let excess = departed.len().saturating_sub(rules.departed_max_entries);
    for (_, id) in departed.into_iter().take(excess) {
        campaign.people.remove(&id);
    }
}

fn referenced_people(campaign: &StrategicCampaign) -> BTreeSet<PersonId> {
    let mut references: BTreeSet<_> = campaign
        .armies
        .values()
        .filter_map(|army| army.commander)
        .collect();
    for fact in &campaign.pending_facts {
        match &fact.kind {
            DomainFactKind::BattleResolved { battle, movement } => {
                if let Some(report) = campaign.battles.get(battle) {
                    for side in [&report.attacker, &report.defender] {
                        references.extend(
                            side.armies
                                .iter()
                                .flat_map(|army| army.people.iter())
                                .map(|person| person.id),
                        );
                    }
                }
                if let Some(movement) = movement {
                    references.extend(&movement.people);
                }
            }
            DomainFactKind::ArmiesMoved {
                movement: Some(receipt),
                ..
            } => references.extend(&receipt.people),
            DomainFactKind::PersonTransferred { person, .. } => {
                references.insert(*person);
            }
            _ => {}
        }
    }
    references
}
