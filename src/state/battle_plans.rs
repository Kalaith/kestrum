//! Campaign-local reusable plans, keyed by deployment slot rather than entity id.

use crate::data::{
    battle_tactics::{BattleDoctrine, TroopTactics},
    economy::TroopKind,
};
use serde::{Deserialize, Serialize};

pub const MAX_BATTLE_TEMPLATES: usize = 12;
pub const MAX_BATTLE_TEMPLATE_NAME_CHARS: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattlePlanTemplate {
    pub name: String,
    pub doctrine: Option<BattleDoctrine>,
    pub slots: Vec<BattlePlanSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattlePlanSlot {
    pub slot: u8,
    pub kind: TroopKind,
    pub tactics: TroopTactics,
    pub explicit_override: bool,
}
