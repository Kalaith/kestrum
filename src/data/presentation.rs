//! Presentation labels and required player-facing text.

use serde::Deserialize;
use std::collections::BTreeSet;
mod campaign_text;
mod game_text;
mod life_text;
mod map_presentation;
mod observer_text;
mod overview_text;
mod progression_text;
mod required_text;
mod tutorial_text;
pub use game_text::GameTextData;
pub use map_presentation::{MapCameraSettings, MapPresentation};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeographyLabel {
    pub id: String,
    pub position: [f32; 2],
    pub size: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationData {
    #[serde(skip)]
    pub map: MapPresentation,
    pub game_id: String,
    pub map_path: String,
    pub font_path: String,
    pub body_font_path: String,
    pub start_year: u32,
    pub npc_action_delay_seconds: f32,
    pub geography: Vec<GeographyLabel>,
}

impl PresentationData {
    pub fn validate(&self) -> Result<(), String> {
        self.map.validate()?;
        if !self.npc_action_delay_seconds.is_finite()
            || !(0.0..=1.0).contains(&self.npc_action_delay_seconds)
        {
            return Err(
                "game_config.json: NPC presentation delay must be between zero and one second."
                    .into(),
            );
        }
        if self.game_id != "kestrum" || self.start_year == 0 {
            return Err("game_config.json: invalid save identity or start year".into());
        }
        let mut ids = BTreeSet::new();
        for label in &self.geography {
            if label.id.trim().is_empty()
                || !ids.insert(label.id.as_str())
                || !label
                    .position
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
                || !label.size.is_finite()
                || !(12.0..=36.0).contains(&label.size)
            {
                return Err(format!(
                    "game_config.json: invalid geography label: {}",
                    label.id
                ));
            }
        }
        Ok(())
    }
}
