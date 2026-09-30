//! Presentation controls for a committed or captured formation battle.

mod projection;
mod render;
mod tactics;
mod terrain;
mod troop_sprites;

use kestrum::{
    data::battle_tactics::{BattleDoctrine, TacticTrigger},
    state::{
        battle::simulation::BattleUnitId,
        military::{ArmyId, FormationId},
        people::PersonId,
    },
};

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
    EditTactics {
        formation: FormationId,
        edit: TacticEdit,
    },
    SetBattleLeader {
        formation: FormationId,
        leader: Option<PersonId>,
    },
    SetBattleDoctrine {
        army: ArmyId,
        doctrine: BattleDoctrine,
    },
    SaveBattleTemplate {
        army: ArmyId,
    },
    ApplyBattleTemplate {
        army: ArmyId,
        index: u8,
    },
    SetTemplateIndex(u8),
    SetTacticTrigger(TacticTrigger),
    SwapFormationSlots {
        army: ArmyId,
        first: u8,
        second: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TacticEdit {
    Add(TacticTrigger),
    Remove(TacticTrigger, usize),
    Move(TacticTrigger, usize, bool),
    CycleAction(TacticTrigger, usize),
    CycleCondition(TacticTrigger, usize),
    CycleTarget(TacticTrigger, usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BattlefieldView {
    pub event_cursor: usize,
    pub event_elapsed: f32,
    pub is_paused: bool,
    pub speed: u8,
    pub selected: Option<BattleUnitId>,
    pub tactic_trigger: TacticTrigger,
    pub template_index: u8,
}

impl Default for BattlefieldView {
    fn default() -> Self {
        Self {
            event_cursor: 0,
            event_elapsed: 0.0,
            is_paused: false,
            speed: 1,
            selected: None,
            tactic_trigger: TacticTrigger::Activation,
            template_index: 0,
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
            BattlefieldAction::SetTacticTrigger(trigger) => self.tactic_trigger = trigger,
            BattlefieldAction::SetTemplateIndex(index) => self.template_index = index,
            BattlefieldAction::EditTactics { .. }
            | BattlefieldAction::SwapFormationSlots { .. }
            | BattlefieldAction::SetBattleLeader { .. }
            | BattlefieldAction::SetBattleDoctrine { .. }
            | BattlefieldAction::SaveBattleTemplate { .. }
            | BattlefieldAction::ApplyBattleTemplate { .. } => {}
        }
    }

    pub fn progress(self) -> f32 {
        (self.event_elapsed / EVENT_SECONDS).clamp(0.0, 1.0)
    }
}
