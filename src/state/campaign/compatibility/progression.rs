//! Earlier K12 people receive one deterministic disposition when first decoded.

use crate::state::people::{Disposition, PersonCareer, Tendency};
use serde::de::Error as _;
use serde_json::Value;

pub(super) fn initialize(value: &mut Value) -> Result<(), serde_json::Error> {
    let ids = {
        let Some(people) = value.get("people").and_then(Value::as_object) else {
            return Ok(());
        };
        if people.is_empty() {
            return Ok(());
        }
        let present = people
            .values()
            .filter(|person| person.get("career").is_some())
            .count();
        if present == people.len() {
            return Ok(());
        }
        if present != 0 {
            return Err(serde_json::Error::custom(
                "campaign.people: mixed K12/K13 career fields",
            ));
        }
        people
            .keys()
            .map(|id| {
                id.parse::<u32>()
                    .map(|number| (number, id.clone()))
                    .map_err(serde_json::Error::custom)
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let rng = value.pointer_mut("/rng/people").ok_or_else(|| {
        serde_json::Error::custom("campaign.rng.people: required for K13 disposition migration")
    })?;
    let mut rng: macroquad_toolkit::rng::SeededRng = serde_json::from_value(rng.clone())?;
    let mut ids = ids;
    ids.sort_by_key(|(id, _)| *id);
    for (_, id) in ids {
        let disposition = Disposition {
            courage: tendency(&mut rng),
            care: tendency(&mut rng),
            curiosity: tendency(&mut rng),
        };
        let career = PersonCareer {
            disposition,
            ..PersonCareer::default()
        };
        value
            .pointer_mut(&format!("/people/{id}"))
            .and_then(Value::as_object_mut)
            .expect("validated person record")
            .insert("career".into(), serde_json::to_value(career)?);
    }
    *value.pointer_mut("/rng/people").expect("rng checked") = serde_json::to_value(rng)?;
    Ok(())
}

fn tendency(rng: &mut macroquad_toolkit::rng::SeededRng) -> Tendency {
    match rng.below(3) {
        0 => Tendency::Negative,
        1 => Tendency::Neutral,
        _ => Tendency::Positive,
    }
}
