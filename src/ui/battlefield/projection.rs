//! Logical battlefield positions and lane mapping shared by drawing and picking.

use kestrum::state::battle::{
    playback::BattlePresentationGroup,
    simulation::{BattleResolution, BattleSide},
};
use macroquad::prelude::*;

pub(super) fn group_center(group: &BattlePresentationGroup, resolution: &BattleResolution) -> Vec2 {
    let army = group.army;
    let side_armies: Vec<_> = resolution
        .opening
        .armies
        .iter()
        .filter(|candidate| candidate.side == group.side)
        .map(|candidate| candidate.id)
        .collect();
    let army_index = side_armies
        .iter()
        .position(|candidate| *candidate == army)
        .unwrap_or(0);
    let slot = group.slot.unwrap_or(group.opening_slot);
    let lane = usize::from(slot % 3);
    let rear = slot >= 3;
    let lane_y = [302.0, 414.0, 526.0][lane];
    let army_spread = army_index as f32 * 116.0;
    let x = match (group.side, rear) {
        (BattleSide::Attacker, false) => 500.0 - army_spread,
        (BattleSide::Attacker, true) => 328.0 - army_spread,
        (BattleSide::Defender, false) => 780.0 + army_spread,
        (BattleSide::Defender, true) => 952.0 + army_spread,
    };
    vec2(x, lane_y)
}

pub(super) fn selection_bounds(center: Vec2) -> Rect {
    Rect::new(center.x - 56.0, center.y - 46.0, 112.0, 104.0)
}
