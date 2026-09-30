//! Ordered activations, legal reach, bounded reactions and deterministic outcomes.

mod abilities;
mod validation;

use crate::{
    data::{
        battle_tactics::{
            BattleCapability, BattleTacticsRules, TacticAction, TacticCondition, TacticTrigger,
            TargetFilter, TargetPriority,
        },
        economy::TroopKind,
    },
    state::{
        battle::{simulation::*, BattleOutcome},
        military::ArmyId,
    },
};
use std::collections::{BTreeMap, BTreeSet};
use validation::canonical_input;

const RESOLVER_VERSION: u32 = 2;
const SLOTS_PER_ARMY: usize = 6;

#[derive(Debug, Clone)]
struct RuntimeArmy {
    side: BattleSide,
    slots: [Option<BattleUnitId>; SLOTS_PER_ARMY],
}

#[derive(Debug, Clone)]
struct RuntimeUnit {
    input: BattleUnitInput,
    opening: BattlePosition,
    position: Option<BattlePosition>,
    headcount: u32,
    morale: u32,
    routed: bool,
    guard_expires_round: Option<u32>,
    reaction_used: bool,
    activation_count: u32,
    used_abilities: BTreeSet<BattleCapability>,
}

struct Runtime {
    opening: FormationBattleInput,
    rules: BattleTacticsRules,
    armies: BTreeMap<ArmyId, RuntimeArmy>,
    units: BTreeMap<BattleUnitId, RuntimeUnit>,
    events: Vec<BattleEvent>,
    round: u32,
}

pub fn resolve_battle(
    input: &FormationBattleInput,
    rules: &BattleTacticsRules,
) -> Result<BattleResolution, String> {
    rules.validate()?;
    let opening = canonical_input(input, rules)?;
    let mut runtime = Runtime::new(opening.clone(), rules.clone());
    let mut ending = None;

    for round in 1..=rules.max_rounds {
        runtime.begin_round(round);
        if round == 1 {
            ending = runtime.resolve_opening_actions();
        }
        let mut activated = BTreeSet::new();
        for id in runtime.initiative_order(round) {
            if ending.is_some() {
                break;
            }
            if !runtime.can_activate(id) || !activated.insert(id) {
                continue;
            }
            runtime.activate(id);
            if let Some(result) = runtime.annihilation_result() {
                ending = Some(result);
                break;
            }
        }
        if ending.is_none() {
            runtime.resolve_routs();
            ending = runtime.rout_result();
        }
        if ending.is_some() {
            break;
        }
    }

    let (completed_rounds, outcome, reason) = ending.unwrap_or_else(|| {
        (
            rules.max_rounds,
            runtime.compare_remaining_power(),
            BattleResolutionReason::RoundLimit,
        )
    });
    runtime
        .events
        .push(BattleEvent::BattleEnded { outcome, reason });
    Ok(BattleResolution {
        resolver_version: RESOLVER_VERSION,
        rules_revision: rules.rules_revision.clone(),
        opening,
        opening_morale: rules.initial_morale,
        completed_rounds,
        outcome,
        reason,
        units: runtime.final_units(),
        events: runtime.events,
    })
}

impl Runtime {
    fn new(opening: FormationBattleInput, rules: BattleTacticsRules) -> Self {
        let mut armies = BTreeMap::new();
        let mut units = BTreeMap::new();
        for army in &opening.armies {
            let mut slots = [None; SLOTS_PER_ARMY];
            for (index, unit) in army.slots.iter().enumerate() {
                if let Some(unit) = unit {
                    let position = BattlePosition {
                        army: army.id,
                        slot: index as u8,
                    };
                    slots[index] = Some(unit.id);
                    units.insert(
                        unit.id,
                        RuntimeUnit {
                            input: unit.clone(),
                            opening: position,
                            position: Some(position),
                            headcount: unit.headcount,
                            morale: rules.initial_morale,
                            routed: false,
                            guard_expires_round: None,
                            reaction_used: false,
                            activation_count: 0,
                            used_abilities: BTreeSet::new(),
                        },
                    );
                }
            }
            armies.insert(
                army.id,
                RuntimeArmy {
                    side: army.side,
                    slots,
                },
            );
        }
        Self {
            opening,
            rules,
            armies,
            units,
            events: Vec::new(),
            round: 0,
        }
    }

    fn begin_round(&mut self, round: u32) {
        self.round = round;
        self.events.push(BattleEvent::RoundStarted { round });
        for unit in self.units.values_mut() {
            unit.reaction_used = false;
            if unit
                .guard_expires_round
                .is_some_and(|expires| expires < round)
            {
                unit.guard_expires_round = None;
            }
        }
    }

    fn initiative_order(&self, round: u32) -> Vec<BattleUnitId> {
        let mut order: Vec<_> = self
            .units
            .iter()
            .filter_map(|(id, unit)| unit.position.map(|position| (*id, position)))
            .collect();
        order.sort_by(|(left_id, left_position), (right_id, right_position)| {
            let left = &self.units[left_id];
            let right = &self.units[right_id];
            right
                .input
                .initiative
                .cmp(&left.input.initiative)
                .then_with(|| {
                    let left_side = self.armies[&left_position.army].side;
                    let right_side = self.armies[&right_position.army].side;
                    side_priority(left_side, round).cmp(&side_priority(right_side, round))
                })
                .then_with(|| left_position.army.cmp(&right_position.army))
                .then_with(|| left_position.slot.cmp(&right_position.slot))
                .then_with(|| left_id.cmp(right_id))
        });
        order.into_iter().map(|(id, _)| id).collect()
    }

    fn can_activate(&self, id: BattleUnitId) -> bool {
        self.units
            .get(&id)
            .is_some_and(|unit| unit.position.is_some() && unit.headcount > 0 && !unit.routed)
    }

    fn select_target(
        &self,
        actor: BattleUnitId,
        action: TacticAction,
        filter: TargetFilter,
        priority: TargetPriority,
    ) -> Option<BattleUnitId> {
        let actor_unit = &self.units[&actor];
        let origin = actor_unit.position?;
        let actor_army = &self.armies[&origin.army];
        let column = origin.slot % 3;
        let mut targets: Vec<_> = self
            .units
            .iter()
            .filter_map(|(id, unit)| {
                let position = unit.position?;
                let army = &self.armies[&position.army];
                let eligible_side = if action == TacticAction::Stabilize {
                    position.army == origin.army
                        && *id != actor
                        && unit.morale < self.rules.initial_morale
                } else {
                    army.side == actor_army.side.opposing()
                };
                (unit.headcount > 0
                    && !unit.routed
                    && eligible_side
                    && self.matches_filter(actor, *id, position, filter)
                    && self.in_reach(action, position))
                .then_some((*id, position))
            })
            .collect();
        targets.sort_by(|(left_id, left_position), (right_id, right_position)| {
            let left = &self.units[left_id];
            let right = &self.units[right_id];
            let preference = match priority {
                TargetPriority::OwnColumnFirst => {
                    let left_same = u8::from(left_position.slot % 3 != column);
                    let right_same = u8::from(right_position.slot % 3 != column);
                    left_same.cmp(&right_same)
                }
                TargetPriority::LowestStrength => strength_ratio(left).cmp(&strength_ratio(right)),
                TargetPriority::HighestStrength => strength_ratio(right).cmp(&strength_ratio(left)),
                TargetPriority::LowestResistance => {
                    left.input.resistance.cmp(&right.input.resistance)
                }
                TargetPriority::LowestMorale => left.morale.cmp(&right.morale),
            };
            preference
                .then_with(|| left_position.army.cmp(&right_position.army))
                .then_with(|| left_position.slot.cmp(&right_position.slot))
                .then_with(|| left_id.cmp(right_id))
        });
        targets.first().map(|(id, _)| *id)
    }

    fn matches_filter(
        &self,
        actor: BattleUnitId,
        id: BattleUnitId,
        position: BattlePosition,
        filter: TargetFilter,
    ) -> bool {
        match filter {
            TargetFilter::None => false,
            TargetFilter::AnyEnemy => true,
            TargetFilter::EnemyFront => position.slot < 3,
            TargetFilter::EnemyRear => position.slot >= 3,
            TargetFilter::EnemyCavalry => self.units[&id].input.kind == Some(TroopKind::Riders),
            TargetFilter::ExposedEnemyRear => position.slot >= 3 && self.is_exposed(position),
            TargetFilter::AllyLowestMorale => {
                self.units[&actor]
                    .position
                    .is_some_and(|actor_position| actor_position.army == position.army)
                    && actor != id
                    && self.units[&id].morale < self.rules.initial_morale
            }
        }
    }

    fn in_reach(&self, action: TacticAction, target: BattlePosition) -> bool {
        match action {
            TacticAction::Attack => target.slot < 3 || !self.has_front(target.army),
            TacticAction::Volley => true,
            TacticAction::Charge => target.slot < 3,
            TacticAction::Breakthrough => target.slot >= 3 && self.is_exposed(target),
            TacticAction::Stabilize => true,
            _ => false,
        }
    }

    fn has_front(&self, army: ArmyId) -> bool {
        self.armies[&army].slots[..3]
            .iter()
            .any(|id| id.is_some_and(|id| self.units[&id].headcount > 0 && !self.units[&id].routed))
    }

    fn is_exposed(&self, position: BattlePosition) -> bool {
        let slot = usize::from(position.slot);
        slot >= 3 && self.armies[&position.army].slots[slot - 3].is_none()
    }

    fn resolve_reaction(&mut self, defender: BattleUnitId, attacker: BattleUnitId) -> Option<u32> {
        let defender_state = &self.units[&defender];
        if defender_state.reaction_used || defender_state.headcount == 0 {
            return None;
        }
        let rules = defender_state.input.reaction_tactics.clone();
        let incoming_cavalry = self.units[&attacker].input.kind == Some(TroopKind::Riders);
        let chosen = rules.into_iter().find(|rule| {
            rule.trigger == TacticTrigger::IncomingAttack
                && (rule.condition == TacticCondition::Always
                    || (rule.condition == TacticCondition::IncomingCavalryCharge
                        && incoming_cavalry))
        })?;
        self.units
            .get_mut(&defender)
            .expect("reaction defender")
            .reaction_used = true;
        match chosen.action {
            TacticAction::Brace => {
                let retaliation =
                    self.damage_amount(defender, attacker, self.rules.brace_retaliation_permille);
                self.events.push(BattleEvent::Reaction {
                    actor: defender,
                    against: attacker,
                    rule_id: chosen.id,
                    damage_reduction_permille: self.rules.brace_damage_permille,
                    retaliation,
                });
                self.apply_damage(defender, attacker, retaliation);
                Some(self.rules.brace_damage_permille)
            }
            TacticAction::Guard => {
                self.events.push(BattleEvent::Reaction {
                    actor: defender,
                    against: attacker,
                    rule_id: chosen.id,
                    damage_reduction_permille: self.rules.guard_damage_permille,
                    retaliation: 0,
                });
                self.raise_guard(defender, self.round);
                Some(self.rules.guard_damage_permille)
            }
            _ => None,
        }
    }

    fn damage_amount(&self, source: BattleUnitId, target: BattleUnitId, modifier: u32) -> u32 {
        let attacker = &self.units[&source];
        let defender = &self.units[&target];
        let numerator = u128::from(attacker.headcount)
            * u128::from(attacker.input.attack)
            * u128::from(modifier);
        let target_side = self.armies[&defender.position.expect("active target").army].side;
        let terrain = if target_side == BattleSide::Defender {
            self.opening.terrain_permille
        } else {
            1000
        };
        let denominator = u128::from(self.rules.casualty_divisor)
            * u128::from(defender.input.resistance)
            * u128::from(terrain);
        (numerator / denominator)
            .max(1)
            .min(u128::from(defender.headcount)) as u32
    }

    fn apply_damage(&mut self, source: BattleUnitId, target: BattleUnitId, amount: u32) {
        if amount == 0 || !self.can_damage(target) {
            return;
        }
        let state = self.units.get_mut(&target).expect("damage target");
        let before = state.headcount;
        state.headcount -= amount.min(before);
        let remaining = state.headcount;
        let morale_before = state.morale;
        state.morale = state
            .morale
            .saturating_sub(amount.saturating_mul(self.rules.morale_loss_per_casualty));
        let morale_after = state.morale;
        let position = state.position;
        self.events.push(BattleEvent::Damage {
            source,
            target,
            amount: before - remaining,
            remaining,
        });
        if morale_after != morale_before {
            self.events.push(BattleEvent::MoraleChanged {
                unit: target,
                before: morale_before,
                after: morale_after,
            });
        }
        if remaining == 0 {
            if let Some(position) = position {
                self.armies
                    .get_mut(&position.army)
                    .expect("target army")
                    .slots[usize::from(position.slot)] = None;
            }
            self.units.get_mut(&target).expect("target unit").position = None;
        }
    }

    fn can_damage(&self, id: BattleUnitId) -> bool {
        self.units
            .get(&id)
            .is_some_and(|unit| unit.position.is_some() && unit.headcount > 0)
    }

    fn raise_guard(&mut self, id: BattleUnitId, expires_round: u32) {
        self.units
            .get_mut(&id)
            .expect("guarding unit")
            .guard_expires_round = Some(expires_round);
        self.events.push(BattleEvent::GuardRaised {
            unit: id,
            expires_round,
        });
    }

    fn advance(&mut self, id: BattleUnitId, from: BattlePosition) {
        let rear_slot = usize::from(from.slot);
        if rear_slot < 3 || self.armies[&from.army].slots[rear_slot - 3].is_some() {
            return;
        }
        self.armies.get_mut(&from.army).expect("own army").slots[rear_slot] = None;
        self.armies.get_mut(&from.army).expect("own army").slots[rear_slot - 3] = Some(id);
        let to = BattlePosition {
            army: from.army,
            slot: from.slot - 3,
        };
        self.units.get_mut(&id).expect("advancing unit").position = Some(to);
        self.events
            .push(BattleEvent::PositionChanged { unit: id, from, to });
    }

    fn change_morale(&mut self, id: BattleUnitId, amount: u32) {
        let unit = self.units.get_mut(&id).expect("allied unit");
        let before = unit.morale;
        unit.morale = unit.morale.saturating_sub(amount);
        if unit.morale != before {
            self.events.push(BattleEvent::MoraleChanged {
                unit: id,
                before,
                after: unit.morale,
            });
        }
    }

    fn annihilation_result(&self) -> Option<(u32, BattleOutcome, BattleResolutionReason)> {
        let attacker_dead = self
            .side_units(BattleSide::Attacker)
            .all(|unit| unit.headcount == 0);
        let defender_dead = self
            .side_units(BattleSide::Defender)
            .all(|unit| unit.headcount == 0);
        if !attacker_dead && !defender_dead {
            return None;
        }
        let outcome = match (attacker_dead, defender_dead) {
            (true, true) => BattleOutcome::MutualDestruction,
            (true, false) => BattleOutcome::DefenderVictory,
            (false, true) => BattleOutcome::AttackerVictory,
            (false, false) => unreachable!(),
        };
        Some((self.round, outcome, BattleResolutionReason::Annihilation))
    }

    fn rout_result(&self) -> Option<(u32, BattleOutcome, BattleResolutionReason)> {
        let attacker_active = self
            .side_units(BattleSide::Attacker)
            .any(|unit| unit.position.is_some());
        let defender_active = self
            .side_units(BattleSide::Defender)
            .any(|unit| unit.position.is_some());
        if attacker_active && defender_active {
            return None;
        }
        let outcome = match (attacker_active, defender_active) {
            (true, false) => BattleOutcome::AttackerVictory,
            (false, true) => BattleOutcome::DefenderVictory,
            (false, false) => BattleOutcome::DefenderVictory,
            (true, true) => unreachable!(),
        };
        Some((self.round, outcome, BattleResolutionReason::Rout))
    }

    fn compare_remaining_power(&self) -> BattleOutcome {
        let start_attacker = self.starting_power(BattleSide::Attacker);
        let start_defender = self.starting_power(BattleSide::Defender);
        let remaining_attacker = self.remaining_power(BattleSide::Attacker);
        let remaining_defender = self.remaining_power(BattleSide::Defender);
        let left = remaining_attacker * start_defender * 1000;
        let right = remaining_defender * start_attacker * 1000;
        let margin =
            start_attacker * start_defender * u128::from(self.rules.victory_margin_permille);
        if left > right && left - right >= margin {
            BattleOutcome::AttackerVictory
        } else if right > left && right - left >= margin {
            BattleOutcome::DefenderVictory
        } else {
            BattleOutcome::Stalemate
        }
    }

    fn starting_power(&self, side: BattleSide) -> u128 {
        self.opening
            .armies
            .iter()
            .filter(|army| army.side == side)
            .flat_map(|army| army.slots.iter().flatten())
            .map(|unit| u128::from(unit.headcount) * u128::from(unit.attack))
            .sum()
    }

    fn remaining_power(&self, side: BattleSide) -> u128 {
        self.side_units(side)
            .map(|unit| u128::from(unit.headcount) * u128::from(unit.input.attack))
            .sum()
    }

    fn side_units(&self, side: BattleSide) -> impl Iterator<Item = &RuntimeUnit> {
        self.units
            .values()
            .filter(move |unit| self.armies[&unit.opening.army].side == side)
    }

    fn final_units(&self) -> Vec<BattleFinalUnit> {
        let mut final_units: Vec<_> = self
            .units
            .iter()
            .map(|(id, unit)| BattleFinalUnit {
                id: *id,
                army: unit.opening.army,
                opening_slot: unit.opening.slot,
                slot: unit.position.map(|position| position.slot),
                headcount: unit.headcount,
                morale: unit.morale,
                is_routed: unit.routed,
            })
            .collect();
        final_units.sort_by_key(|unit| (unit.army, unit.opening_slot, unit.id));
        final_units
    }
}

fn side_priority(side: BattleSide, round: u32) -> u8 {
    let first = if round % 2 == 1 {
        BattleSide::Attacker
    } else {
        BattleSide::Defender
    };
    u8::from(side != first)
}

fn strength_ratio(unit: &RuntimeUnit) -> u64 {
    u64::from(unit.headcount) * 1_000_000 / u64::from(unit.input.capacity)
}
