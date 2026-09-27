//! K11 initializes current conditions without inventing earlier growth or threats.

use serde_json::{json, Value};

pub(super) fn initialize(value: &mut Value) -> Result<(), serde_json::Error> {
    let mut presence = vec![
        value.get("threats").is_some(),
        value.pointer("/next_ids/threat").is_some(),
        value.pointer("/world/development").is_some(),
    ];
    if let Some(factions) = value.get("factions").and_then(Value::as_object) {
        presence.extend(
            factions
                .values()
                .map(|faction| faction.get("last_hq_relocation").is_some()),
        );
    }
    if let Some(reports) = value.get("battles").and_then(Value::as_object) {
        for report in reports.values() {
            presence.push(report.pointer("/defender/kind").is_some());
            if let Some(exchanges) = report.get("exchanges").and_then(Value::as_array) {
                presence.extend(
                    exchanges
                        .iter()
                        .map(|exchange| exchange.get("threat_losses").is_some()),
                );
            }
        }
    }
    if presence.iter().all(|present| *present) {
        return Ok(());
    }
    if presence.iter().any(|present| *present) {
        return Err(malformed(
            "incomplete development, threat or battle compatibility group",
        ));
    }
    let campaign = value.as_object_mut().ok_or_else(|| malformed("campaign"))?;
    campaign.insert("threats".into(), json!({}));
    campaign
        .get_mut("next_ids")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| malformed("next_ids"))?
        .insert("threat".into(), json!(1));
    let world = campaign
        .get_mut("world")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| malformed("world"))?;
    let mut development = serde_json::Map::new();
    for site in world
        .get_mut("sites")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| malformed("sites"))?
    {
        let id = site
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(|| malformed("site id"))?;
        development.insert(
            id.to_string(),
            serde_json::to_value(crate::state::development::SiteDevelopment::default())?,
        );
        // This fixed authored tag describes the site, not a retroactive ruined state.
        if id == 13 && site.get("key") == Some(&json!("ruined_hold")) {
            let tags = site
                .get_mut("tags")
                .and_then(Value::as_array_mut)
                .ok_or_else(|| malformed("site tags"))?;
            if !tags.contains(&json!("ruins")) {
                tags.push(json!("ruins"));
            }
        }
    }
    world.insert("development".into(), Value::Object(development));
    for faction in campaign
        .get_mut("factions")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| malformed("factions"))?
        .values_mut()
    {
        faction
            .as_object_mut()
            .ok_or_else(|| malformed("faction"))?
            .insert("last_hq_relocation".into(), Value::Null);
    }
    if let Some(reports) = campaign.get_mut("battles").and_then(Value::as_object_mut) {
        for report in reports.values_mut() {
            initialize_report(report)?;
        }
    }
    Ok(())
}

fn initialize_report(report: &mut Value) -> Result<(), serde_json::Error> {
    let report = report.as_object_mut().ok_or_else(|| malformed("battle"))?;
    let defender = report
        .remove("defender")
        .ok_or_else(|| malformed("battle defender"))?;
    if !defender.is_object() {
        return Err(malformed("battle defender"));
    }
    report.insert("defender".into(), json!({"kind":"faction","side":defender}));
    for exchange in report
        .get_mut("exchanges")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| malformed("exchanges"))?
    {
        exchange
            .as_object_mut()
            .ok_or_else(|| malformed("exchange"))?
            .insert("threat_losses".into(), json!(0));
    }
    Ok(())
}

fn malformed(field: &str) -> serde_json::Error {
    <serde_json::Error as serde::de::Error>::custom(format!("campaign.development: {field}"))
}
