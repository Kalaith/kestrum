//! K14 saves gain empty family state without consuming RNG or inventing people.

use serde::de::Error as _;
use serde_json::Value;

pub(super) fn initialize(value: &mut Value) -> Result<(), serde_json::Error> {
    let Some(campaign) = value.as_object_mut() else {
        return Err(serde_json::Error::custom("campaign must be an object"));
    };
    let fields = [
        "households",
        "families",
        "successors",
        "apprentice_last_invited_year",
    ];
    let present = fields
        .iter()
        .filter(|field| campaign.contains_key(**field))
        .count();
    let household_id = campaign
        .get("next_ids")
        .and_then(Value::as_object)
        .is_some_and(|ids| ids.contains_key("household"));
    if present == 0 && !household_id {
        campaign.insert("households".into(), Value::Object(Default::default()));
        campaign.insert("families".into(), Value::Object(Default::default()));
        campaign.insert("successors".into(), Value::Object(Default::default()));
        campaign.insert(
            "apprentice_last_invited_year".into(),
            Value::Object(Default::default()),
        );
        campaign
            .get_mut("next_ids")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| serde_json::Error::custom("campaign.next_ids is missing"))?
            .insert("household".into(), Value::from(1));
        return Ok(());
    }
    if present != fields.len() || !household_id {
        return Err(serde_json::Error::custom(
            "campaign: incomplete K15 household state",
        ));
    }
    Ok(())
}
