//! Toolkit-loaded content assembled and validated without a graphics context.

pub mod combat;
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
    pub troops: combat::Troops,
    pub combat: combat::CombatRules,
    pub presentation: PresentationData,
    pub economy: economy::Economy,
    pub rules: rules::CampaignRules,
    pub scenario: world::Scenario,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let data = Self {
            troops: macroquad_toolkit::include_json!("../assets/data/troops.json")?,
            combat: macroquad_toolkit::include_json!("../assets/data/combat_rules.json")?,
            presentation: macroquad_toolkit::include_json!("../assets/data/game_config.json")?,
            economy: macroquad_toolkit::include_json!("../assets/data/economy.json")?,
            rules: macroquad_toolkit::include_json!("../assets/data/campaign_rules.json")?,
            scenario: macroquad_toolkit::include_json!("../assets/data/scenarios/rosemarch.json")?,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.troops.validate()?;
        self.combat.validate()?;
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
    "help_move",
    "help_route",
    "help_transfer",
    "help_spent",
    "help_recovery",
    "help_transfer_phase",
    "move_group",
    "move_group_help",
    "move_armies",
    "move_army",
    "selected_armies",
    "move_enter_region",
    "route_cost",
    "cost",
    "movement_left",
    "movement_spent",
    "army_movement_left",
    "move_stops_at",
    "move_uncertain_contact",
    "move_route_clear",
    "move_supplied_after",
    "move_unsupplied_after",
    "move_pick_destination",
    "route_review",
    "confirm_move",
    "cancel_move",
    "choose_destination",
    "back_to_map",
    "army_orders",
    "army_people",
    "composition",
    "recovery",
    "next_recovery",
    "last_recovery",
    "recovery_forecast",
    "transfer_formation",
    "transfer",
    "transfer_preserves_movement",
    "transfer_paused_help",
    "people_transfer_help",
    "no_local_people",
    "assigned_site",
    "choose_army",
    "transfer_choose_army",
    "split_new_army",
    "confirm_transfer",
    "formation_slots",
    "empty_formation_slot",
    "person_needs_formation",
    "person_fit",
    "person_wounded",
    "wound_steps_remaining",
    "battle_reports",
    "help_battle",
    "help_battle_reports",
    "help_wounds",
    "battle_recorded",
    "no_battle_reports",
    "older_report",
    "newer_report",
    "battle_outcome",
    "battle_forces",
    "battle_factors",
    "battle_attacker",
    "battle_defender",
    "battle_victory",
    "battle_stalemate",
    "battle_mutual_destruction",
    "battle_annihilation",
    "battle_rout",
    "battle_exchange_limit",
    "battle_exchanges",
    "battle_exhausted",
    "battle_previous_control",
    "battle_structural_damage",
    "battle_occupation",
    "battle_holds_site",
    "battle_withdrew",
    "battle_army_destroyed",
    "battle_counted_elements",
    "battle_slot",
    "battle_combat_losses",
    "battle_encirclement_losses",
    "battle_destroyed",
    "class_officer",
    "person_dead",
    "battle_person_died_wipe",
    "battle_person_no_refuge",
    "battle_wound_wipe",
    "battle_wound_command",
    "battle_assumed_command",
    "battle_previous_commander",
    "battle_person_event",
    "no_battle_people",
    "battle_simultaneous",
    "battle_simultaneous_detail",
    "battle_terrain",
    "battle_defender_resistance",
    "battle_initial_leadership",
    "battle_leadership_change",
    "battle_counter_attack",
    "battle_exchange",
    "battle_attacker_losses",
    "battle_defender_losses",
    "battle_recorded_place",
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
    "recruit_exhausted",
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
