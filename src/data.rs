//! Toolkit-loaded content assembled and validated without a graphics context.

pub mod ai;
pub mod battle_tactics;
pub mod combat;
pub mod construction;
pub mod development;
pub mod diplomacy;
pub mod economy;
pub mod generation;
pub mod households;
pub mod lifecycle;
pub mod notifications;
pub mod portraits;
pub mod progression;
pub mod rules;
mod setup_validation;
pub mod siege;
pub mod threats;
mod validation;
pub mod world;
mod world_layout_validation;
mod world_validation;

mod presentation;
pub use presentation::{GameTextData, GeographyLabel, MapCameraSettings, PresentationData};
mod production_topology;

#[derive(Debug, Clone)]
pub struct GameData {
    pub notifications: notifications::NotificationRules,
    pub portraits: portraits::PortraitCatalog,
    pub ai: ai::AiRules,
    pub battle_tactics: battle_tactics::BattleTacticsRules,
    pub diplomacy: diplomacy::DiplomacyRules,
    pub development: development::DevelopmentRules,
    pub lifecycle: lifecycle::LifecycleRules,
    pub households: households::HouseholdRules,
    pub threats: threats::ThreatRules,
    pub siege: siege::SiegeRules,
    pub construction: construction::ConstructionRules,
    pub progression: progression::ProgressionRules,
    pub history: progression::HistoryRules,
    pub human_names: progression::HumanNamePool,
    pub troops: combat::Troops,
    pub combat: combat::CombatRules,
    pub presentation: PresentationData,
    pub game_text: GameTextData,
    pub economy: economy::Economy,
    pub rules: rules::CampaignRules,
    /// Small authored warfare fixture retained for tests and captures.
    pub scenario: world::Scenario,
    pub production_layout: world::WorldLayout,
}

impl GameData {
    pub fn load() -> Result<Self, String> {
        let mut data = Self {
            notifications: macroquad_toolkit::include_json!("../assets/data/notifications.json")?,
            portraits: macroquad_toolkit::include_json!("../assets/data/portrait_catalog.json")?,
            ai: macroquad_toolkit::include_json!("../assets/data/ai.json")?,
            battle_tactics: macroquad_toolkit::include_json!("../assets/data/battle_tactics.json")?,
            diplomacy: macroquad_toolkit::include_json!("../assets/data/diplomacy.json")?,
            development: macroquad_toolkit::include_json!("../assets/data/development.json")?,
            lifecycle: macroquad_toolkit::include_json!("../assets/data/lifecycle.json")?,
            households: macroquad_toolkit::include_json!("../assets/data/household_rules.json")?,
            threats: macroquad_toolkit::include_json!("../assets/data/threats.json")?,
            siege: macroquad_toolkit::include_json!("../assets/data/siege_rules.json")?,
            construction: macroquad_toolkit::include_json!(
                "../assets/data/construction_rules.json"
            )?,
            progression: macroquad_toolkit::include_json!("../assets/data/progression.json")?,
            history: macroquad_toolkit::include_json!("../assets/data/history_rules.json")?,
            human_names: macroquad_toolkit::include_json!("../assets/data/human_names.json")?,
            troops: macroquad_toolkit::include_json!("../assets/data/troops.json")?,
            combat: macroquad_toolkit::include_json!("../assets/data/combat_rules.json")?,
            presentation: macroquad_toolkit::include_json!("../assets/data/game_config.json")?,
            game_text: macroquad_toolkit::include_json!("../assets/data/game_text.json")?,
            economy: macroquad_toolkit::include_json!("../assets/data/economy.json")?,
            rules: macroquad_toolkit::include_json!("../assets/data/campaign_rules.json")?,
            scenario: macroquad_toolkit::include_json!("../assets/data/scenarios/rosemarch.json")?,
            production_layout: macroquad_toolkit::include_json!(
                "../assets/data/world_layout.json"
            )?,
        };
        data.presentation.map =
            macroquad_toolkit::include_json!("../assets/data/map_presentation.json")?;
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.notifications.validate()?;
        self.portraits.validate()?;
        self.ai.validate()?;
        self.battle_tactics.validate()?;
        self.diplomacy.validate()?;
        self.development.validate()?;
        self.lifecycle.validate()?;
        self.households.validate()?;
        self.threats.validate(&self.scenario)?;
        self.siege.validate()?;
        self.construction.validate()?;
        self.progression.validate()?;
        self.history.validate()?;
        self.human_names.validate()?;
        if !self
            .progression
            .recognition
            .required_facts
            .iter()
            .all(|fact| self.human_names.epithets.contains_key(fact))
        {
            return Err("human_names.json: missing an epithet for supported recognition".into());
        }
        self.troops.validate()?;
        self.combat.validate()?;
        self.presentation.validate()?;
        self.game_text.validate(&self.presentation.geography)?;
        self.economy.validate()?;
        self.rules.validate()?;
        self.scenario.validate(&self.rules, &self.economy)?;
        self.production_layout.validate(&self.rules, &self.economy)
    }
}
