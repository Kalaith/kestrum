//! Additive v2 upgrades, shared by interim slots and catalogue payloads.

use super::StrategicCampaign;
mod construction;
mod development;
mod evidence;
mod households;
mod progression;
mod siege;

impl StrategicCampaign {
    pub(crate) fn decode_compatible(
        mut value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        initialize_earlier_military(&mut value);
        initialize_earlier_battles(&mut value);
        let earlier_evidence = evidence::initialize(&mut value)?;
        let earlier_construction = construction::initialize(&mut value);
        siege::initialize(&mut value)?;
        development::initialize(&mut value)?;
        progression::initialize(&mut value)?;
        households::initialize(&mut value)?;
        let earlier_diplomacy = value.get("diplomacy").is_none() && value.get("ai").is_none();
        if earlier_diplomacy {
            if let Some(fields) = value.as_object_mut() {
                fields.insert(
                    "diplomacy".into(),
                    serde_json::to_value(super::super::diplomacy::CampaignDiplomacy::default())?,
                );
                fields.insert(
                    "ai".into(),
                    serde_json::to_value(super::super::ai::AiState::default())?,
                );
            }
        }
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
        if earlier_diplomacy {
            campaign.initialize_diplomacy();
        }
        if legacy {
            campaign.reconcile_region_control();
        }
        if earlier_evidence {
            evidence::restore(&mut campaign)?;
        }
        if earlier_construction {
            construction::restore(&mut campaign)?;
        }
        Ok(campaign)
    }
}

fn initialize_earlier_battles(value: &mut serde_json::Value) {
    if value.get("battles").is_some()
        || value.pointer("/next_ids/battle").is_some()
        || value.pointer("/world/occupation").is_some()
    {
        return;
    }
    if let Some(campaign) = value.as_object_mut() {
        campaign.insert("battles".into(), serde_json::json!({}));
    }
    if let Some(ids) = value
        .get_mut("next_ids")
        .and_then(serde_json::Value::as_object_mut)
    {
        ids.insert("battle".into(), serde_json::json!(1));
    }
    if let Some(world) = value
        .get_mut("world")
        .and_then(serde_json::Value::as_object_mut)
    {
        world.insert("occupation".into(), serde_json::json!({}));
        // Explicit migration of the shipped pre-combat Rosemarch bridge. Never
        // repair malformed modern topology, and never infer combat from names.
        if let Some(sites) = world
            .get_mut("sites")
            .and_then(serde_json::Value::as_array_mut)
        {
            for site in sites {
                if site.get("id") == Some(&serde_json::json!(8))
                    && site.get("key") == Some(&serde_json::json!("bridge"))
                    && site.get("geography") == Some(&serde_json::json!("river"))
                    && site.get("tags") == Some(&serde_json::json!([]))
                {
                    site["tags"] = serde_json::json!(["bridge"]);
                }
            }
        }
    }
}

fn initialize_earlier_military(value: &mut serde_json::Value) {
    let military_absent = ["armies", "formations", "people"]
        .iter()
        .all(|key| value.get(*key).is_none());
    let counters_absent = value
        .get("next_ids")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|ids| {
            ["army", "formation", "person"]
                .iter()
                .all(|key| !ids.contains_key(*key))
        });
    if !military_absent || !counters_absent {
        return;
    }
    // Earlier campaigns never simulated troops or people. Granting founders at a
    // later date would invent military history, so continuation starts with none.
    if let Some(campaign) = value.as_object_mut() {
        for key in ["armies", "formations", "people"] {
            campaign.insert(key.into(), serde_json::json!({}));
        }
    }
    if let Some(ids) = value
        .get_mut("next_ids")
        .and_then(serde_json::Value::as_object_mut)
    {
        for key in ["army", "formation", "person"] {
            ids.insert(key.into(), serde_json::json!(1));
        }
    }
    if let Some(world) = value
        .get_mut("world")
        .and_then(serde_json::Value::as_object_mut)
    {
        world
            .entry("site_damage")
            .or_insert_with(|| serde_json::json!({}));
    }
}
