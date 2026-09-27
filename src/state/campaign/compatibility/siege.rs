//! One complete K10 addition; partial modern receipts must not become field battles.

use serde_json::{json, Value};

const REPORT_FIELDS: [&str; 4] = [
    "context",
    "wall_permille",
    "fort_damage_added",
    "road_damage",
];

pub(super) fn initialize(value: &mut Value) -> Result<(), serde_json::Error> {
    let mut presence = vec![
        value.get("sieges").is_some(),
        value.pointer("/next_ids/siege").is_some(),
        value.pointer("/world/fort_damage").is_some(),
    ];
    if let Some(reports) = value.get("battles").and_then(Value::as_object) {
        for report in reports.values() {
            presence.extend(REPORT_FIELDS.map(|field| report.get(field).is_some()));
            if let Some(exchanges) = report.get("exchanges").and_then(Value::as_array) {
                presence.extend(
                    exchanges
                        .iter()
                        .map(|exchange| exchange.get("wall_permille").is_some()),
                );
            }
        }
    }
    if presence.iter().all(|present| *present) {
        return Ok(());
    }
    if presence.iter().any(|present| *present) {
        return Err(<serde_json::Error as serde::de::Error>::custom(
            "campaign.sieges: incomplete siege state or battle context compatibility group",
        ));
    }
    let campaign = value.as_object_mut().ok_or_else(|| malformed("campaign"))?;
    campaign.insert("sieges".into(), json!({}));
    campaign
        .get_mut("next_ids")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| malformed("next_ids"))?
        .insert("siege".into(), json!(1));
    campaign
        .get_mut("world")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| malformed("world"))?
        .insert("fort_damage".into(), json!({}));
    if let Some(reports) = campaign.get_mut("battles").and_then(Value::as_object_mut) {
        for report in reports.values_mut() {
            initialize_report(report)?;
        }
    }
    Ok(())
}

fn initialize_report(report: &mut Value) -> Result<(), serde_json::Error> {
    let report = report.as_object_mut().ok_or_else(|| malformed("battle"))?;
    report.insert("context".into(), json!({"kind":"field"}));
    report.insert("wall_permille".into(), json!(1000));
    report.insert("fort_damage_added".into(), json!(0));
    report.insert("road_damage".into(), Value::Null);
    if let Some(exchanges) = report.get_mut("exchanges").and_then(Value::as_array_mut) {
        for exchange in exchanges {
            exchange
                .as_object_mut()
                .ok_or_else(|| malformed("exchange"))?
                .insert("wall_permille".into(), json!(1000));
        }
    }
    Ok(())
}

fn malformed(field: &str) -> serde_json::Error {
    <serde_json::Error as serde::de::Error>::custom(format!(
        "campaign.sieges: malformed legacy {field}"
    ))
}
