//! Presentation controls for a committed or captured formation battle.

mod projection;
mod render;
mod terrain;
mod troop_sprites;

use kestrum::state::battle::simulation::BattleUnitId;

const EVENT_SECONDS: f32 = 0.42;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattlefieldAction {
    StartPendingBattle,
    Close,
    TogglePause,
    Step,
    SetSpeed(u8),
    SkipToResult,
    Select(Option<BattleUnitId>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BattlefieldView {
    pub event_cursor: usize,
    pub event_elapsed: f32,
    pub is_paused: bool,
    pub speed: u8,
    pub selected: Option<BattleUnitId>,
}

impl Default for BattlefieldView {
    fn default() -> Self {
        Self {
            event_cursor: 0,
            event_elapsed: 0.0,
            is_paused: false,
            speed: 1,
            selected: None,
        }
    }
}

pub fn draw(ctx: &super::Context<'_>) -> Option<super::UiAction> {
    render::draw(ctx)
}

pub fn prepare_text(ctx: &super::Context<'_>) {
    render::prepare_text(ctx);
}

impl BattlefieldView {
    pub fn advance(&mut self, delta_seconds: f32, event_count: usize) {
        if self.is_paused || self.event_cursor >= event_count {
            return;
        }
        self.event_elapsed += delta_seconds.max(0.0) * f32::from(self.speed.clamp(1, 3));
        while self.event_elapsed >= EVENT_SECONDS && self.event_cursor < event_count {
            self.event_elapsed -= EVENT_SECONDS;
            self.event_cursor += 1;
        }
        if self.event_cursor >= event_count {
            self.event_elapsed = 0.0;
            self.is_paused = true;
        }
    }

    pub fn apply(&mut self, action: BattlefieldAction, event_count: usize) {
        match action {
            BattlefieldAction::StartPendingBattle | BattlefieldAction::Close => {}
            BattlefieldAction::TogglePause => self.is_paused = !self.is_paused,
            BattlefieldAction::Step => {
                self.is_paused = true;
                self.event_elapsed = 0.0;
                self.event_cursor = self.event_cursor.saturating_add(1).min(event_count);
            }
            BattlefieldAction::SetSpeed(speed) => self.speed = speed.clamp(1, 3),
            BattlefieldAction::SkipToResult => {
                self.event_cursor = event_count;
                self.event_elapsed = 0.0;
                self.is_paused = true;
            }
            BattlefieldAction::Select(selected) => {
                self.selected = if self.selected == selected {
                    None
                } else {
                    selected
                };
            }
        }
    }

    pub fn progress(self) -> f32 {
        (self.event_elapsed / EVENT_SECONDS).clamp(0.0, 1.0)
    }
}
