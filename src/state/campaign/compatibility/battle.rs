//! Initialize selected battle leaders for saves from before that field existed.

use super::super::StrategicCampaign;
use crate::state::{military::FormationId, people::PersonAssignment};
use crate::{data::battle_tactics::leader_capabilities, state::people::PersonStatus};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn missing_leader_fields(value: &Value) -> BTreeSet<FormationId> {
    value
        .get("formations")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|formations| formations.iter())
        .filter_map(|(key, formation)| {
            if formation.get("battle_leader").is_some() {
                return None;
            }
            key.parse::<u32>()
                .ok()
                .filter(|id| *id > 0)
                .map(FormationId)
        })
        .collect()
}

pub(super) fn initialize_earlier_leaders(
    campaign: &mut StrategicCampaign,
    missing_fields: &BTreeSet<FormationId>,
) {
    for formation_id in missing_fields {
        let Some(formation) = campaign.formations.get(formation_id) else {
            continue;
        };
        let ordinary = leader_capabilities(formation.kind, None);
        let commander = campaign
            .armies
            .values()
            .find(|army| {
                army.commander.is_some() && army.formation_ids().any(|id| id == *formation_id)
            })
            .and_then(|army| army.commander);
        let Some(commander) = commander else {
            continue;
        };
        let Some(person) = campaign.people.get(&commander) else {
            continue;
        };
        let grants = leader_capabilities(formation.kind, Some(person.class));
        let valid = person.id == commander
            && person.faction == formation.faction
            && person.assignment
                == (PersonAssignment::Formation {
                    formation: *formation_id,
                })
            && person.status == PersonStatus::Fit
            && !person.career.retired
            && person.age_years(campaign.completed_rounds) >= 17
            && grants.len() > ordinary.len();
        if valid {
            campaign
                .formations
                .get_mut(formation_id)
                .expect("formation checked above")
                .battle_leader = Some(commander);
        }
    }
}
