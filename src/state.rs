//! Game ownership, explicit legacy compatibility, and guarded campaign actions.

pub mod ai;
pub mod battle;
pub mod campaign;
pub mod construction;
pub mod development;
pub mod diplomacy;
pub mod evidence;
pub mod history;
pub mod knowledge;
pub mod legacy;
pub mod mentorship;
pub mod military;
pub mod people;
pub mod persistence;
pub mod relationships;
pub mod siege;
pub mod threat;
pub mod tutorial;
mod validation;
pub mod world;

pub use campaign::{
    CampaignId, CampaignPhase, FactionStatus, StrategicCampaign, STRATEGIC_VERSION,
};

use crate::{
    data::GameData,
    engine::{self, ActionOutcome, Actor, Command, RuleError},
};
use serde::{de::Error, Deserialize, Deserializer, Serialize};

pub const SAVE_SLOT: &str = "kestrum_campaign_v1";
pub const STRATEGIC_SAVE_SLOT: &str = "kestrum_strategic_v2";
pub const SAVE_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    Campaign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    None,
    Menu,
    Settings,
    Help,
    Credits,
    ConfirmNew,
    Setup,
    Saves,
    SaveRecovery,
    Armies,
    MoveGroup,
    MoveReview,
    Battle,
    Battlefield,
    History,
    Settlement,
    Siege,
    Threat,
    Kingdom,
    CampaignEnd,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ShellCampaign {
    pub version: u32,
    pub turn: u32,
}

impl Default for ShellCampaign {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            turn: 1,
        }
    }
}

impl ShellCampaign {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != SAVE_VERSION || self.turn == 0 {
            return Err(
                "This empty-atlas campaign has an unsupported version or invalid turn".into(),
            );
        }
        Ok(())
    }
}

/// Flat JSON deliberately retains the original v1 shape and explicit schema version.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Campaign {
    Shell(ShellCampaign),
    Strategic(Box<StrategicCampaign>),
}

impl<'de> Deserialize<'de> for Campaign {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Serde's untagged buffer loses the JSON map-key deserializer needed by
        // numeric FactionId keys. Decode the selected schema through JSON itself,
        // which also preserves every u64 RNG bit on native and WASM targets.
        let value = serde_json::Value::deserialize(deserializer)?;
        match value.get("version").and_then(serde_json::Value::as_u64) {
            Some(version) if version == u64::from(SAVE_VERSION) => serde_json::from_value(value)
                .map(Self::Shell)
                .map_err(D::Error::custom),
            Some(version) if version == u64::from(STRATEGIC_VERSION) => {
                StrategicCampaign::decode_compatible(value)
                    .map(|campaign| Self::Strategic(Box::new(campaign)))
                    .map_err(D::Error::custom)
            }
            Some(version) => Err(D::Error::custom(format!(
                "Unsupported campaign schema version {version}"
            ))),
            None => Err(D::Error::custom(
                "Campaign schema version must be an unsigned integer",
            )),
        }
    }
}

impl Campaign {
    pub fn validate(&self, data: &GameData) -> Result<(), String> {
        match self {
            Self::Shell(shell) => shell.validate(),
            Self::Strategic(campaign) => campaign.validate(data),
        }
    }

    pub fn strategic(&self) -> Option<&StrategicCampaign> {
        match self {
            Self::Strategic(campaign) => Some(campaign),
            Self::Shell(_) => None,
        }
    }

    pub fn season_index(&self) -> usize {
        match self {
            Self::Shell(shell) => (shell.turn.saturating_sub(1) % 4) as usize,
            Self::Strategic(campaign) => campaign.season_index(),
        }
    }

    pub fn year(&self, start: u32) -> u32 {
        match self {
            Self::Shell(shell) => start.saturating_add(shell.turn.saturating_sub(1) / 4),
            Self::Strategic(campaign) => campaign.year(start),
        }
    }

    pub fn display_turn(&self) -> u32 {
        match self {
            Self::Shell(shell) => shell.turn,
            Self::Strategic(campaign) => campaign.completed_rounds.saturating_add(1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Preferences {
    pub hide_labels: bool,
    pub high_contrast: bool,
}

#[derive(Debug)]
pub struct GameState {
    pub screen: Screen,
    pub overlay: Overlay,
    pub campaign: Option<Campaign>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            screen: Screen::Title,
            overlay: Overlay::None,
            campaign: None,
        }
    }
}

impl GameState {
    pub fn new_game(&mut self, data: &GameData) -> Result<(), String> {
        let campaign = StrategicCampaign::new(data)?;
        self.load_campaign(Campaign::Strategic(Box::new(campaign)), data)
    }

    pub fn load_campaign(&mut self, campaign: Campaign, data: &GameData) -> Result<(), String> {
        campaign.validate(data)?;
        self.campaign = Some(campaign);
        self.screen = Screen::Campaign;
        self.overlay = Overlay::None;
        Ok(())
    }

    pub fn end_turn(&mut self, data: &GameData) -> Result<ActionOutcome, RuleError> {
        self.command(data, Command::EndTurn)
    }

    pub fn command(
        &mut self,
        data: &GameData,
        command: Command,
    ) -> Result<ActionOutcome, RuleError> {
        let military_order = self.overlay == Overlay::Armies
            && matches!(
                &command,
                Command::Recruit { .. }
                    | Command::Disband { .. }
                    | Command::TransferFormation { .. }
                    | Command::TransferPerson { .. }
                    | Command::SplitArmy { .. }
                    | Command::SetCommander { .. }
                    | Command::TrainPerson { .. }
                    | Command::PracticeRiding { .. }
                    | Command::CancelPersonCourse { .. }
                    | Command::RecoverPersonAtSite { .. }
                    | Command::RetirePerson { .. }
                    | Command::AppointGovernor { .. }
                    | Command::StartMentorship { .. }
                    | Command::EndMentorship { .. }
                    | Command::FormHousehold { .. }
                    | Command::EndHousehold { .. }
                    | Command::SetHouseholdChildraising { .. }
                    | Command::AdoptWard { .. }
                    | Command::AssignTrainee { .. }
                    | Command::EnterService { .. }
                    | Command::InviteApprentice { .. }
                    | Command::DesignateSuccessor { .. }
                    | Command::SpecializeFormation { .. }
                    | Command::CancelFormationCourse { .. }
            );
        let military_order = military_order
            || (self.overlay == Overlay::MoveReview && matches!(&command, Command::Move(_)));
        let military_order = military_order
            || (self.overlay == Overlay::Siege && matches!(&command, Command::Siege(_)))
            || (self.overlay == Overlay::Threat && matches!(&command, Command::ClearThreat { .. }));
        let military_order = military_order
            || (self.overlay == Overlay::Settlement
                && matches!(
                    &command,
                    Command::StartConstruction { .. }
                        | Command::CancelConstruction { .. }
                        | Command::ReassignBuilder { .. }
                        | Command::SetFocus { .. }
                        | Command::RenameSite { .. }
                        | Command::Resettle { .. }
                        | Command::MoveCapital { .. }
                        | Command::RelocateHeadquarters { .. }
                ));
        let military_order = military_order
            || (self.overlay == Overlay::Kingdom
                && matches!(
                    &command,
                    Command::DeclareWar { .. }
                        | Command::OfferPeace { .. }
                        | Command::RespondPeace { .. }
                        | Command::ResolveDefeat { .. }
                ));
        let military_order = military_order
            || (self.overlay == Overlay::History
                && matches!(&command, Command::TransferLegacyItem { .. }));
        let military_order = military_order
            || (self.overlay == Overlay::Battlefield
                && matches!(
                    &command,
                    Command::SetFormationTactics { .. } | Command::SwapFormationSlots { .. }
                ));
        if self.screen != Screen::Campaign || (self.overlay != Overlay::None && !military_order) {
            return Err(RuleError::PlayObstructed);
        }
        let lesson = match &command {
            Command::Move(_) => Some(tutorial::TutorialStep::Movement),
            Command::EndTurn => Some(tutorial::TutorialStep::FirstTurn),
            _ => None,
        };
        let campaign = self.strategic_campaign()?;
        let outcome = engine::apply(campaign, data, Actor::Player, command)?;
        if let Some(step) = lesson.filter(|step| {
            *step != tutorial::TutorialStep::Movement
                || outcome
                    .movement
                    .as_ref()
                    .is_some_and(|movement| movement.path.len() > 1)
                || outcome.battle.is_some()
        }) {
            campaign.tutorial.record(step);
        }
        Ok(outcome)
    }

    pub fn advance_npc(&mut self, data: &GameData) -> Result<ActionOutcome, RuleError> {
        let campaign = self.playing_campaign()?;
        engine::advance_npc(campaign, data)
    }

    fn playing_campaign(&mut self) -> Result<&mut StrategicCampaign, RuleError> {
        if self.screen != Screen::Campaign || self.overlay != Overlay::None {
            return Err(RuleError::PlayObstructed);
        }
        self.strategic_campaign()
    }

    fn strategic_campaign(&mut self) -> Result<&mut StrategicCampaign, RuleError> {
        match &mut self.campaign {
            Some(Campaign::Strategic(campaign)) => Ok(campaign),
            Some(Campaign::Shell(_)) => Err(RuleError::LegacyReadOnly),
            None => Err(RuleError::NoCampaign),
        }
    }

    pub fn back(&mut self) {
        self.overlay = match self.overlay {
            Overlay::None if self.screen == Screen::Campaign => Overlay::Menu,
            Overlay::Settings | Overlay::Help | Overlay::Saves
                if self.screen == Screen::Campaign =>
            {
                Overlay::Menu
            }
            _ => Overlay::None,
        };
    }

    pub fn main_menu(&mut self) {
        self.screen = Screen::Title;
        self.overlay = Overlay::None;
    }
}
