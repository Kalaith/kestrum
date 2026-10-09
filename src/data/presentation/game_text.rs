//! Authored player-facing copy for Kestrum's title, calendar, atlas and UI.

use super::{
    campaign_text::CAMPAIGN_TEXT, observer_text::OBSERVER_TEXT, overview_text::OVERVIEW_TEXT,
    progression_text::PROGRESSION_TEXT, required_text::REQUIRED_TEXT, tutorial_text::TUTORIAL_TEXT,
    GeographyLabel,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameTextData {
    pub title: String,
    pub subtitle: String,
    pub edition: String,
    pub seasons: Vec<String>,
    pub geography: BTreeMap<String, String>,
    pub text: BTreeMap<String, String>,
}

impl GameTextData {
    pub fn validate(&self, geography_layout: &[GeographyLabel]) -> Result<(), String> {
        for (field, value) in [
            ("title", &self.title),
            ("subtitle", &self.subtitle),
            ("edition", &self.edition),
        ] {
            if value.trim().is_empty() {
                return Err(format!("game_text.json: {field} cannot be empty"));
            }
        }
        if self.seasons.len() != 4 || self.seasons.iter().any(|season| season.trim().is_empty()) {
            return Err("game_text.json: Kestrum requires four non-empty season names".into());
        }

        let layout_ids: BTreeSet<_> = geography_layout
            .iter()
            .map(|label| label.id.as_str())
            .collect();
        let text_ids: BTreeSet<_> = self.geography.keys().map(String::as_str).collect();
        if layout_ids != text_ids {
            return Err("game_text.json: geography labels must match game_config.json IDs".into());
        }
        if let Some((id, _)) = self
            .geography
            .iter()
            .find(|(_, label)| label.trim().is_empty())
        {
            return Err(format!(
                "game_text.json: geography label {id} cannot be empty"
            ));
        }

        for key in REQUIRED_TEXT
            .iter()
            .chain(PROGRESSION_TEXT)
            .chain(TUTORIAL_TEXT)
            .chain(OVERVIEW_TEXT)
            .chain(OBSERVER_TEXT)
            .chain(CAMPAIGN_TEXT)
        {
            if self
                .text
                .get(*key)
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(format!(
                    "game_text.json: missing Kestrum interface text: {key}"
                ));
            }
        }
        Ok(())
    }

    pub fn text<'a>(&'a self, key: &'a str) -> &'a str {
        self.text.get(key).map(String::as_str).unwrap_or(key)
    }
}
