//! Additive v2 geography upgrade, shared by interim slots and catalogue payloads.

use super::StrategicCampaign;

impl StrategicCampaign {
    pub(crate) fn decode_compatible(
        mut value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        // K02/K03 had neither field and cannot contain a historical regional
        // claim. Initialize both together, then derive the first claim from saved
        // controllers/HQs. Partial or explicitly malformed new fields stay errors.
        let legacy = value
            .get_mut("world")
            .and_then(serde_json::Value::as_object_mut)
            .is_some_and(|world| {
                if world.contains_key("region_control") || world.contains_key("contested_sites") {
                    return false;
                }
                world.insert("region_control".into(), serde_json::json!({}));
                world.insert("contested_sites".into(), serde_json::json!([]));
                true
            });
        let mut campaign: Self = serde_json::from_value(value)?;
        if legacy {
            campaign.reconcile_region_control();
        }
        Ok(campaign)
    }
}
