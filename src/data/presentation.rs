//! Presentation labels and required player-facing text.

use serde::Deserialize;
use std::collections::BTreeMap;
mod life_text;
mod map_presentation;
mod overview_text;
mod progression_text;
mod required_text;
mod tutorial_text;
pub use map_presentation::MapPresentation;
use progression_text::PROGRESSION_TEXT;
use required_text::REQUIRED_TEXT;

#[derive(Debug, Clone, Deserialize)]
pub struct GeographyLabel {
    pub name: String,
    pub position: [f32; 2],
    pub size: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresentationData {
    #[serde(skip)]
    pub map: MapPresentation,
    pub game_id: String,
    pub title: String,
    pub subtitle: String,
    pub edition: String,
    pub map_path: String,
    pub font_path: String,
    pub body_font_path: String,
    pub start_year: u32,
    pub npc_action_delay_seconds: f32,
    pub seasons: Vec<String>,
    pub geography: Vec<GeographyLabel>,
    pub text: BTreeMap<String, String>,
}

impl PresentationData {
    pub fn validate(&self) -> Result<(), String> {
        self.map.validate()?;
        if !self.npc_action_delay_seconds.is_finite()
            || !(0.0..=1.0).contains(&self.npc_action_delay_seconds)
        {
            return Err("NPC presentation delay must be between zero and one second.".into());
        }
        if self.game_id != "kestrum" || self.start_year == 0 || self.seasons.len() != 4 {
            return Err(
                "Kestrum requires its own save identity, a positive year, and four seasons".into(),
            );
        }
        for key in REQUIRED_TEXT
            .iter()
            .chain(PROGRESSION_TEXT)
            .chain(tutorial_text::TUTORIAL_TEXT)
            .chain(overview_text::OVERVIEW_TEXT)
        {
            if self
                .text
                .get(*key)
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(format!("Missing Kestrum interface text: {key}"));
            }
        }
        for label in &self.geography {
            if label.name.trim().is_empty()
                || !label
                    .position
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
                || !label.size.is_finite()
                || !(12.0..=36.0).contains(&label.size)
            {
                return Err(format!("Invalid geography label: {}", label.name));
            }
        }
        Ok(())
    }

    pub fn text<'a>(&'a self, key: &'a str) -> &'a str {
        self.text.get(key).map(String::as_str).unwrap_or(key)
    }
}
