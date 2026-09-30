//! Deterministic rival preparation from observed troop roles and deployment only.

use crate::{data::battle_tactics::BattleDoctrine, data::economy::TroopKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservedTroopPosition {
    /// Ordinal within the observed battle, not a persistent campaign identity.
    pub army_index: u8,
    pub kind: TroopKind,
    pub slot: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RivalSlotMove {
    pub army_index: u8,
    pub from: u8,
    pub to: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RivalBattlePlan {
    pub doctrine: BattleDoctrine,
    pub slot_moves: Vec<RivalSlotMove>,
}

/// Pick a legal preset and fill a useful line from a faction's own troops and
/// the enemy roles/slots visible in this encounter. Tactics and hidden campaign
/// state are deliberately absent from the input contract.
pub fn plan_rival_battle(
    own: &[ObservedTroopPosition],
    known_enemy: &[ObservedTroopPosition],
) -> RivalBattlePlan {
    let exposed_rear = known_enemy.iter().any(|rear| {
        rear.slot >= 3
            && !known_enemy
                .iter()
                .any(|front| front.army_index == rear.army_index && front.slot == rear.slot - 3)
    });
    let enemy_cavalry = known_enemy
        .iter()
        .any(|formation| formation.kind == TroopKind::Riders);
    let has_support = own
        .iter()
        .any(|formation| matches!(formation.kind, TroopKind::Archers | TroopKind::SiegeEngines));
    let doctrine = if exposed_rear {
        BattleDoctrine::Breakthrough
    } else if enemy_cavalry {
        BattleDoctrine::DefensiveLine
    } else if has_support {
        BattleDoctrine::RangedSupport
    } else {
        BattleDoctrine::Breakthrough
    };

    let mut by_army = BTreeMap::<u8, Vec<ObservedTroopPosition>>::new();
    for formation in own.iter().copied().filter(|formation| formation.slot < 6) {
        by_army
            .entry(formation.army_index)
            .or_default()
            .push(formation);
    }
    let mut slot_moves = Vec::new();
    for (army_index, mut formations) in by_army {
        formations.sort_by_key(|formation| (role_order(doctrine, formation.kind), formation.slot));
        let mut occupied = BTreeSet::new();
        for formation in formations {
            let preferred = if front_line(formation.kind) {
                [0, 1, 2, 3, 4, 5]
            } else {
                [3, 4, 5, 0, 1, 2]
            };
            let target = preferred
                .into_iter()
                .find(|slot| !occupied.contains(slot))
                .expect("a six-slot army has room for its reported units");
            occupied.insert(target);
            if formation.slot != target {
                slot_moves.push(RivalSlotMove {
                    army_index,
                    from: formation.slot,
                    to: target,
                });
            }
        }
    }
    RivalBattlePlan {
        doctrine,
        slot_moves,
    }
}

fn role_order(doctrine: BattleDoctrine, kind: TroopKind) -> u8 {
    use BattleDoctrine::*;
    use TroopKind::*;
    match (doctrine, kind) {
        (DefensiveLine, Spearmen) => 0,
        (DefensiveLine, Warriors) => 1,
        (DefensiveLine, Riders) => 2,
        (RangedSupport, Warriors) => 0,
        (RangedSupport, Spearmen) => 1,
        (RangedSupport, Riders) => 2,
        (Breakthrough, Riders) => 0,
        (Breakthrough, Warriors) => 1,
        (Breakthrough, Spearmen) => 2,
        (_, SiegeEngines) => 3,
        (_, Archers) => 4,
        (_, Medics) => 5,
    }
}

fn front_line(kind: TroopKind) -> bool {
    matches!(
        kind,
        TroopKind::Warriors | TroopKind::Spearmen | TroopKind::Riders
    )
}
