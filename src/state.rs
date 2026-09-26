//! Explicit title/campaign transitions and the empty campaign's saved chronology.

use serde::{Deserialize, Serialize};

pub const SAVE_SLOT: &str = "kestrum_campaign_v1";
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Campaign {
    pub version: u32,
    pub turn: u32,
}

impl Default for Campaign {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            turn: 1,
        }
    }
}

impl Campaign {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != SAVE_VERSION || self.turn == 0 {
            return Err("This campaign has an unsupported version or invalid turn".into());
        }
        Ok(())
    }

    pub fn end_turn(&mut self) {
        self.turn = self.turn.saturating_add(1);
    }
    pub fn season_index(&self) -> usize {
        ((self.turn - 1) % 4) as usize
    }
    pub fn year(&self, start: u32) -> u32 {
        start.saturating_add((self.turn - 1) / 4)
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
    pub fn new_game(&mut self) {
        self.campaign = Some(Campaign::default());
        self.screen = Screen::Campaign;
        self.overlay = Overlay::None;
    }

    pub fn load_campaign(&mut self, campaign: Campaign) -> Result<(), String> {
        campaign.validate()?;
        self.campaign = Some(campaign);
        self.screen = Screen::Campaign;
        self.overlay = Overlay::None;
        Ok(())
    }

    pub fn end_turn(&mut self) -> bool {
        if self.screen != Screen::Campaign || self.overlay != Overlay::None {
            return false;
        }
        if let Some(campaign) = &mut self.campaign {
            campaign.end_turn();
            return true;
        }
        false
    }

    pub fn back(&mut self) {
        self.overlay = match self.overlay {
            Overlay::None if self.screen == Screen::Campaign => Overlay::Menu,
            Overlay::Settings | Overlay::Help if self.screen == Screen::Campaign => Overlay::Menu,
            _ => Overlay::None,
        };
    }

    pub fn main_menu(&mut self) {
        self.screen = Screen::Title;
        self.overlay = Overlay::None;
    }
}
