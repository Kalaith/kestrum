//! Pre-discount saves paid the original full prices; retain that historical receipt.

use serde::de::Error as _;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct LegacyPrices {
    classes: BTreeMap<String, i64>,
    formations: BTreeMap<String, i64>,
}

pub(super) fn initialize(value: &mut Value) -> Result<(), serde_json::Error> {
    let prices: LegacyPrices =
        macroquad_toolkit::include_json!("../../../../assets/data/legacy_course_prices.json")
            .map_err(serde_json::Error::custom)?;
    for (table, path, costs) in [
        ("people", "/career/course", &prices.classes),
        ("formations", "/service/course", &prices.formations),
    ] {
        let Some(entries) = value.get_mut(table).and_then(Value::as_object_mut) else {
            continue;
        };
        for entry in entries.values_mut() {
            let Some(course) = entry.pointer_mut(path).and_then(Value::as_object_mut) else {
                continue;
            };
            if course.contains_key("paid_gold")
                || course.get("kind").and_then(Value::as_str) == Some("riding_practice")
            {
                continue;
            }
            let price = course
                .get("target")
                .and_then(Value::as_str)
                .and_then(|target| costs.get(target))
                .ok_or_else(|| serde_json::Error::custom("legacy course has an unknown target"))?;
            course.insert("paid_gold".into(), Value::from(*price));
        }
    }
    Ok(())
}
