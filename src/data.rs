//! Toolkit-loaded content assembled and validated without a graphics context.

pub mod economy;
pub mod rules;
mod setup_validation;
mod validation;
pub mod world;
mod world_validation;

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct GeographyLabel {
    pub name: String,
    pub position: [f32; 2],
    pub size: f32,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub presentation: PresentationData,
    pub economy: economy::Economy,
    pub rules: rules::CampaignRules,
    pub scenario: world::Scenario,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let data = Self {
            presentation: macroquad_toolkit::include_json!("../assets/data/game_config.json")?,
            economy: macroquad_toolkit::include_json!("../assets/data/economy.json")?,
            rules: macroquad_toolkit::include_json!("../assets/data/campaign_rules.json")?,
            scenario: macroquad_toolkit::include_json!("../assets/data/scenarios/rosemarch.json")?,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.presentation.validate()?;
        self.economy.validate()?;
        self.rules.validate()?;
        self.scenario.validate(&self.rules, &self.economy)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PresentationData {
    pub game_id: String,
    pub title: String,
    pub subtitle: String,
    pub edition: String,
    pub map_path: String,
    pub font_path: String,
    pub body_font_path: String,
    pub start_year: u32,
    pub seasons: Vec<String>,
    pub geography: Vec<GeographyLabel>,
    pub text: BTreeMap<String, String>,
}

const REQUIRED_TEXT: &[&str] = &[
    "new_game",
    "continue",
    "no_save",
    "settings",
    "help",
    "credits",
    "quit",
    "menu",
    "resume",
    "save",
    "load",
    "main_menu",
    "back",
    "close",
    "end_turn",
    "world_map",
    "turn",
    "year",
    "phase",
    "zoom_in",
    "zoom_out",
    "reset_view",
    "labels",
    "contrast",
    "on",
    "off",
    "standard",
    "high",
    "fullscreen",
    "help_title",
    "help_army",
    "help_recruit",
    "help_slots",
    "help_economy",
    "help_disband",
    "help_leadership",
    "recruit_success",
    "disband_success",
    "armies",
    "own_armies",
    "recruit",
    "disband",
    "gold",
    "wood",
    "stone",
    "troop_warriors",
    "troop_spearmen",
    "troop_archers",
    "troop_riders",
    "troop_medics",
    "troop_siege_engines",
    "empty_slot",
    "no_armies",
    "new_army_help",
    "new_army",
    "army_upkeep",
    "leadership",
    "commander",
    "no_commander",
    "upkeep_deficit",
    "attached_people",
    "recruit_exhausted",
    "select_formation",
    "engine_teams",
    "troops",
    "upkeep",
    "recruit_ready",
    "confirm_recruit",
    "disband_warning",
    "disband_people",
    "disband_last",
    "confirm_disband",
    "last_income",
    "upkeep_paid",
    "upkeep_shortfall",
    "help_pan",
    "help_zoom",
    "help_turn",
    "help_menu",
    "help_scope",
    "help_select",
    "help_region",
    "help_navigation",
    "enter_region",
    "select_place",
    "local_control",
    "political_claim",
    "unclaimed",
    "uncontrolled",
    "contested",
    "secure_control",
    "site_supplied",
    "site_unsupplied",
    "supplied_gate",
    "region_sites",
    "anchor_requirements",
    "anchors_help",
    "entrances",
    "connections",
    "gate",
    "anchor",
    "no_connections",
    "credits_title",
    "credits_body",
    "new_title",
    "new_warning",
    "cancel",
    "save_success",
    "load_success",
    "save_failed",
    "load_failed",
    "settings_failed",
    "round",
    "npc_phase",
    "npc_paused",
    "npc_prototype",
    "pause_npcs",
    "resume_npcs",
    "step_npc",
    "load_legacy",
    "legacy_phase",
    "save_player_only",
    "legacy_read_only",
    "save_catalogue",
    "new_save",
    "no_catalogue_saves",
    "previous",
    "next",
    "overwrite",
    "delete",
    "retry_storage",
    "overwrite_warning",
    "save_name_help",
    "confirm_overwrite",
    "create_save",
    "delete_warning",
    "confirm_delete",
    "round_unsaved",
    "round_unsaved_help",
    "retry_save",
    "continue_unsaved",
    "import_campaign",
    "storage_ready",
    "storage_waiting",
    "storage_busy",
    "save_during_orders",
    "manual_save",
    "round_save",
    "imported_save",
    "saved_name",
    "founding_save",
];

impl PresentationData {
    pub fn validate(&self) -> Result<(), String> {
        if self.game_id != "kestrum" || self.start_year == 0 || self.seasons.len() != 4 {
            return Err(
                "Kestrum requires its own save identity, a positive year, and four seasons".into(),
            );
        }
        for key in REQUIRED_TEXT {
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
