//! Read-only reconstruction of a battle display from its saved opening and event prefix.

use super::simulation::{BattleEvent, BattlePosition, BattleResolution, BattleSide, BattleUnitId};
use crate::{data::economy::TroopKind, state::military::ArmyId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattlePresentationGroup {
    pub id: BattleUnitId,
    pub army: ArmyId,
    pub faction: crate::data::world::FactionId,
    pub side: BattleSide,
    pub kind: Option<TroopKind>,
    pub opening_slot: u8,
    pub slot: Option<u8>,
    pub headcount: u32,
    pub capacity: u32,
    pub morale: u32,
    pub is_routed: bool,
    pub guard_expires_round: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattlePresentationFrame {
    pub completed_events: usize,
    pub round: u32,
    pub active_event: Option<BattleEvent>,
    pub groups: Vec<BattlePresentationGroup>,
}

/// Projects completed deltas without changing the saved result or consulting live game data.
pub fn battle_presentation_at(
    resolution: &BattleResolution,
    completed_events: usize,
) -> BattlePresentationFrame {
    let completed_events = completed_events.min(resolution.events.len());
    let mut frame = BattlePresentationFrame {
        completed_events,
        round: 1,
        active_event: resolution.events.get(completed_events).cloned(),
        groups: resolution
            .opening
            .armies
            .iter()
            .flat_map(|army| {
                army.slots
                    .iter()
                    .enumerate()
                    .filter_map(move |(slot, unit)| {
                        let unit = unit.as_ref()?;
                        Some(BattlePresentationGroup {
                            id: unit.id,
                            army: army.id,
                            faction: army.faction,
                            side: army.side,
                            kind: unit.kind,
                            opening_slot: slot as u8,
                            slot: Some(slot as u8),
                            headcount: unit.headcount,
                            capacity: unit.capacity,
                            morale: resolution.opening_morale,
                            is_routed: false,
                            guard_expires_round: None,
                        })
                    })
            })
            .collect(),
    };

    for event in resolution.events.iter().take(completed_events) {
        apply_event(&mut frame, event);
    }
    frame
        .groups
        .sort_by_key(|group| (group.side, group.army, group.opening_slot, group.id));
    frame
}

fn apply_event(frame: &mut BattlePresentationFrame, event: &BattleEvent) {
    match event {
        BattleEvent::RoundStarted { round } => {
            frame.round = *round;
            for group in &mut frame.groups {
                if group
                    .guard_expires_round
                    .is_some_and(|expires| expires < *round)
                {
                    group.guard_expires_round = None;
                }
            }
        }
        BattleEvent::Damage {
            target, remaining, ..
        } => {
            if let Some(group) = group_mut(frame, *target) {
                group.headcount = *remaining;
                if *remaining == 0 {
                    group.slot = None;
                }
            }
        }
        BattleEvent::MoraleChanged { unit, after, .. } => {
            if let Some(group) = group_mut(frame, *unit) {
                group.morale = *after;
            }
        }
        BattleEvent::PositionChanged { unit, to, .. } => {
            if let Some(group) = group_mut(frame, *unit) {
                group.army = to.army;
                group.slot = Some(to.slot);
            }
        }
        BattleEvent::Routed {
            unit,
            position,
            survivors,
        } => apply_rout(frame, *unit, *position, *survivors),
        BattleEvent::GuardRaised {
            unit,
            expires_round,
        } => {
            if let Some(group) = group_mut(frame, *unit) {
                group.guard_expires_round = Some(*expires_round);
            }
        }
        BattleEvent::Activation { .. }
        | BattleEvent::Reaction { .. }
        | BattleEvent::BattleEnded { .. } => {}
    }
}

fn apply_rout(
    frame: &mut BattlePresentationFrame,
    id: BattleUnitId,
    position: BattlePosition,
    survivors: u32,
) {
    if let Some(group) = group_mut(frame, id) {
        group.army = position.army;
        group.slot = None;
        group.headcount = survivors;
        group.is_routed = true;
    }
}

fn group_mut(
    frame: &mut BattlePresentationFrame,
    id: BattleUnitId,
) -> Option<&mut BattlePresentationGroup> {
    frame.groups.iter_mut().find(|group| group.id == id)
}
