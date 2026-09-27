//! Earlier campaigns receive current population baselines and no invented paid work.

use super::StrategicCampaign;
use serde_json::Value;

pub(super) fn initialize(value: &mut Value) -> bool {
    if value.get("construction").is_some()
        || value.pointer("/next_ids/order").is_some()
        || value.pointer("/world/population").is_some()
        || value.pointer("/world/focus").is_some()
    {
        return false;
    }
    let Some(campaign) = value.as_object_mut() else {
        return false;
    };
    campaign.insert("construction".into(), serde_json::json!({}));
    if let Some(ids) = value.get_mut("next_ids").and_then(Value::as_object_mut) {
        ids.insert("order".into(), Value::from(1));
    }
    if let Some(world) = value.get_mut("world").and_then(Value::as_object_mut) {
        world.insert("population".into(), serde_json::json!({}));
        world.insert("focus".into(), serde_json::json!({}));
    }
    true
}

pub(super) fn restore(campaign: &mut StrategicCampaign) -> Result<(), serde_json::Error> {
    let rules: crate::data::construction::ConstructionRules =
        macroquad_toolkit::include_json!("../../../../assets/data/construction_rules.json")
            .map_err(<serde_json::Error as serde::de::Error>::custom)?;
    rules
        .validate()
        .map_err(<serde_json::Error as serde::de::Error>::custom)?;
    campaign.initialize_population(&rules);
    Ok(())
}
