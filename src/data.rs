//! Toolkit-loaded content assembled and validated without a graphics context.

pub mod ai;
pub mod combat;
pub mod construction;
pub mod development;
pub mod diplomacy;
pub mod economy;
pub mod progression;
pub mod rules;
mod setup_validation;
pub mod siege;
pub mod threats;
mod validation;
pub mod world;
mod world_validation;

mod presentation;
pub use presentation::{GeographyLabel, PresentationData};

#[derive(Debug, Clone)]
pub struct GameData {
    pub ai: ai::AiRules,
    pub diplomacy: diplomacy::DiplomacyRules,
    pub development: development::DevelopmentRules,
    pub threats: threats::ThreatRules,
    pub siege: siege::SiegeRules,
    pub construction: construction::ConstructionRules,
    pub progression: progression::ProgressionRules,
    pub history: progression::HistoryRules,
    pub human_names: progression::HumanNamePool,
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
            ai: macroquad_toolkit::include_json!("../assets/data/ai.json")?,
            diplomacy: macroquad_toolkit::include_json!("../assets/data/diplomacy.json")?,
            development: macroquad_toolkit::include_json!("../assets/data/development.json")?,
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
            economy: macroquad_toolkit::include_json!("../assets/data/economy.json")?,
            rules: macroquad_toolkit::include_json!("../assets/data/campaign_rules.json")?,
            scenario: macroquad_toolkit::include_json!("../assets/data/scenarios/rosemarch.json")?,
        };
        data.validate()?;
        Ok(data)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.ai.validate()?;
        self.diplomacy.validate()?;
        self.development.validate()?;
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
        self.economy.validate()?;
        self.rules.validate()?;
        self.scenario.validate(&self.rules, &self.economy)
    }
}
