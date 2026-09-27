//! A complete earlier schema gains empty service, never invented past rewards.

use crate::state::{
    evidence::{EvidenceLedger, FormationService},
    StrategicCampaign,
};
use serde_json::Value;

pub(super) fn initialize(value: &mut Value) -> Result<bool, serde_json::Error> {
    if value.get("history").is_some()
        || value.get("knowledge").is_some()
        || value.pointer("/next_ids/history").is_some()
        || contains_member_field(value, "formations", "service")
        || contains_member_field(value, "people", "evidence")
    {
        return Ok(false);
    }
    if let Some(entries) = value.get_mut("formations").and_then(Value::as_object_mut) {
        for member in entries.values_mut() {
            if let Some(member) = member.as_object_mut() {
                member.insert(
                    "service".into(),
                    serde_json::to_value(FormationService::default())?,
                );
            }
        }
    }
    if let Some(entries) = value.get_mut("people").and_then(Value::as_object_mut) {
        for member in entries.values_mut() {
            if let Some(member) = member.as_object_mut() {
                member.insert(
                    "evidence".into(),
                    serde_json::to_value(EvidenceLedger::default())?,
                );
            }
        }
    }
    if let Some(campaign) = value.as_object_mut() {
        campaign.insert(
            "history".into(),
            serde_json::to_value(crate::state::history::CampaignHistory::default())?,
        );
        campaign.insert(
            "knowledge".into(),
            serde_json::to_value(crate::state::knowledge::CampaignKnowledge::default())?,
        );
    }
    if let Some(ids) = value.get_mut("next_ids").and_then(Value::as_object_mut) {
        ids.insert("history".into(), Value::from(1));
    }
    Ok(true)
}

fn contains_member_field(value: &Value, collection: &str, field: &str) -> bool {
    value
        .get(collection)
        .and_then(Value::as_object)
        .is_some_and(|entries| entries.values().any(|member| member.get(field).is_some()))
}

pub(super) fn restore(campaign: &mut StrategicCampaign) -> Result<(), serde_json::Error> {
    crate::engine::history::restore_battle_history(campaign);
    crate::engine::knowledge::restore_battle_knowledge(campaign);
    let rules: crate::data::progression::HistoryRules =
        macroquad_toolkit::include_json!("../../../../assets/data/history_rules.json")
            .map_err(<serde_json::Error as serde::de::Error>::custom)?;
    rules
        .validate()
        .map_err(<serde_json::Error as serde::de::Error>::custom)?;
    crate::engine::history::prune_with_rules(campaign, &rules);
    crate::engine::knowledge::prune_observations(
        &mut campaign.knowledge,
        campaign.completed_rounds,
        &rules,
    );
    Ok(())
}
