//! Explicit read-only shell and earlier strategic-slot compatibility.

use crate::{data::GameData, state::Campaign};
use macroquad_toolkit::persistence::decode_slot_with_migration;

pub fn load_legacy(raw: &str, data: &GameData) -> Result<Campaign, String> {
    let candidate: Campaign = decode_slot_with_migration(raw, "2", |version, value| {
        if !matches!(version.as_deref(), Some("1" | "1.0" | "1.0.0")) {
            return Err("This earlier save uses an unsupported storage version.".into());
        }
        let payload = value
            .get("data")
            .ok_or("The earlier save is missing its campaign payload.")?;
        serde_json::from_value(payload.clone())
            .map_err(|error| format!("The earlier campaign is invalid: {error}"))
    })?;
    candidate.validate(data)?;
    Ok(candidate)
}
