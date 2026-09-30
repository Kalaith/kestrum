//! Bounded leader and support abilities used by the pure battle resolver.

use super::Runtime;
use crate::{
    data::battle_tactics::{
        BattleCapability, TacticAction, TacticCondition, TargetFilter, TargetPriority,
    },
    state::battle::simulation::{
        BattleEvent, BattleResolutionReason, BattleUnitId, SkippedTactic, TacticSkipReason,
    },
};

impl Runtime {
    pub(super) fn resolve_opening_actions(
        &mut self,
    ) -> Option<(
        u32,
        crate::state::battle::BattleOutcome,
        BattleResolutionReason,
    )> {
        for id in self.initiative_order(1) {
            if !self.can_activate(id) {
                continue;
            }
            let capabilities = self.units[&id].input.capabilities.clone();
            for capability in capabilities {
                let action = match capability {
                    BattleCapability::CavalryExploitOpening => self
                        .select_target(
                            id,
                            TacticAction::Breakthrough,
                            TargetFilter::ExposedEnemyRear,
                            TargetPriority::LowestStrength,
                        )
                        .map(|target| {
                            (
                                TacticAction::Breakthrough,
                                target,
                                self.rules.cavalry_opening_damage_permille,
                            )
                        }),
                    BattleCapability::SiegeBombardment => self
                        .select_target(
                            id,
                            TacticAction::Volley,
                            TargetFilter::AnyEnemy,
                            TargetPriority::LowestResistance,
                        )
                        .map(|target| {
                            (
                                TacticAction::Volley,
                                target,
                                self.rules.opening_bombardment_damage_permille,
                            )
                        }),
                    _ => None,
                };
                let Some((attack, target, modifier)) = action else {
                    continue;
                };
                self.events.push(BattleEvent::OpeningAction {
                    actor: id,
                    ability: capability,
                    target,
                });
                self.resolve_attack_with_modifier(id, target, attack, modifier);
                if let Some(result) = self.annihilation_result() {
                    return Some(result);
                }
            }
        }
        None
    }

    pub(super) fn activate(&mut self, id: BattleUnitId) {
        let Some(position) = self.units.get(&id).and_then(|unit| unit.position) else {
            return;
        };
        let rules = self.units[&id].input.activation_tactics.clone();
        let mut skipped = Vec::new();
        let mut chosen = None;
        for rule in rules {
            if !self.condition_holds(id, rule.condition) {
                skipped.push(SkippedTactic {
                    rule_id: rule.id,
                    reason: TacticSkipReason::ConditionFalse,
                });
                continue;
            }
            if !self.action_available(id, rule.action) {
                skipped.push(SkippedTactic {
                    rule_id: rule.id,
                    reason: TacticSkipReason::ActionUnavailable,
                });
                continue;
            }
            let needs_target = is_attack(rule.action) || rule.action == TacticAction::Stabilize;
            let target = if needs_target {
                self.select_target(id, rule.action, rule.target_filter, rule.target_priority)
            } else {
                None
            };
            if needs_target && target.is_none() {
                skipped.push(SkippedTactic {
                    rule_id: rule.id,
                    reason: TacticSkipReason::NoLegalTarget,
                });
                continue;
            }
            chosen = Some((rule.id, rule.action, target));
            break;
        }
        let (rule_id, action, target) = chosen.unwrap_or_else(|| {
            let target = self.select_target(
                id,
                TacticAction::Attack,
                TargetFilter::AnyEnemy,
                TargetPriority::OwnColumnFirst,
            );
            (
                String::new(),
                target.map_or(TacticAction::Wait, |_| TacticAction::Attack),
                target,
            )
        });
        let rule_id = (!rule_id.is_empty()).then_some(rule_id);
        self.events.push(BattleEvent::Activation {
            actor: id,
            action,
            rule_id,
            target,
            skipped,
        });
        self.units
            .get_mut(&id)
            .expect("validated actor")
            .activation_count += 1;

        match action {
            TacticAction::Attack
            | TacticAction::Volley
            | TacticAction::Charge
            | TacticAction::Breakthrough => {
                if let Some(target) = target {
                    self.resolve_attack(id, target, action);
                }
            }
            TacticAction::Guard => self.raise_guard(id, self.round),
            TacticAction::Advance => self.advance(id, position),
            TacticAction::Rally => {
                self.units
                    .get_mut(&id)
                    .expect("rallying unit")
                    .used_abilities
                    .insert(BattleCapability::OfficerRally);
                self.events.push(BattleEvent::AbilityUsed {
                    actor: id,
                    ability: BattleCapability::OfficerRally,
                    target: None,
                });
                self.restore_morale(id, self.rules.rally_morale_restore);
            }
            TacticAction::HoldTheLine => {
                self.units
                    .get_mut(&id)
                    .expect("holding unit")
                    .used_abilities
                    .insert(BattleCapability::InfantryHoldTheLine);
                self.events.push(BattleEvent::AbilityUsed {
                    actor: id,
                    ability: BattleCapability::InfantryHoldTheLine,
                    target: None,
                });
                self.advance(id, position);
                self.raise_guard(id, self.round);
            }
            TacticAction::Stabilize => {
                if let Some(target) = target {
                    self.events.push(BattleEvent::AbilityUsed {
                        actor: id,
                        ability: BattleCapability::MedicStabilization,
                        target: Some(target),
                    });
                    self.restore_morale(target, self.rules.medic_morale_restore);
                }
            }
            TacticAction::Brace | TacticAction::Wait => {}
        }
    }

    fn action_available(&self, id: BattleUnitId, action: TacticAction) -> bool {
        let unit = &self.units[&id];
        match action {
            TacticAction::Brace => false,
            TacticAction::Volley => matches!(
                unit.input.kind,
                Some(
                    crate::data::economy::TroopKind::Archers
                        | crate::data::economy::TroopKind::SiegeEngines
                )
            ),
            TacticAction::Charge | TacticAction::Breakthrough => {
                unit.input.kind == Some(crate::data::economy::TroopKind::Riders)
            }
            TacticAction::Advance => unit.position.is_some_and(|position| {
                let slot = usize::from(position.slot);
                slot >= 3 && self.armies[&position.army].slots[slot - 3].is_none()
            }),
            TacticAction::Rally => {
                unit.input
                    .capabilities
                    .contains(&BattleCapability::OfficerRally)
                    && !unit
                        .used_abilities
                        .contains(&BattleCapability::OfficerRally)
                    && unit.morale < self.rules.initial_morale
            }
            TacticAction::HoldTheLine => {
                let vacant_front = unit.position.is_some_and(|position| {
                    let slot = usize::from(position.slot);
                    slot >= 3 && self.armies[&position.army].slots[slot - 3].is_none()
                });
                vacant_front
                    && unit
                        .input
                        .capabilities
                        .contains(&BattleCapability::InfantryHoldTheLine)
                    && !unit
                        .used_abilities
                        .contains(&BattleCapability::InfantryHoldTheLine)
            }
            TacticAction::Stabilize => {
                unit.input
                    .capabilities
                    .contains(&BattleCapability::MedicStabilization)
                    && self
                        .select_target(
                            id,
                            TacticAction::Stabilize,
                            TargetFilter::AllyLowestMorale,
                            TargetPriority::LowestMorale,
                        )
                        .is_some()
            }
            TacticAction::Attack | TacticAction::Guard | TacticAction::Wait => true,
        }
    }

    fn condition_holds(&self, id: BattleUnitId, condition: TacticCondition) -> bool {
        let unit = &self.units[&id];
        match condition {
            TacticCondition::Always => true,
            TacticCondition::EnemyRearExposed => self.units.values().any(|enemy| {
                enemy.position.is_some_and(|position| {
                    self.armies[&position.army].side
                        == self.armies[&unit.opening.army].side.opposing()
                        && usize::from(position.slot) >= 3
                        && self.is_exposed(position)
                })
            }),
            TacticCondition::EnemyCavalryPresent => self.units.values().any(|enemy| {
                enemy.input.kind == Some(crate::data::economy::TroopKind::Riders)
                    && enemy.position.is_some_and(|position| {
                        self.armies[&position.army].side
                            == self.armies[&unit.opening.army].side.opposing()
                    })
            }),
            TacticCondition::FirstActivation => unit.activation_count == 0,
            TacticCondition::SelfBelowHalf => {
                unit.headcount.saturating_mul(2) < unit.input.capacity
            }
            TacticCondition::SelfBelowHalfMorale => {
                unit.morale.saturating_mul(2) < self.rules.initial_morale
            }
            TacticCondition::AllyInSameRowBelowHalf => {
                let Some(position) = unit.position else {
                    return false;
                };
                let army = &self.armies[&position.army];
                let rear = position.slot >= 3;
                army.slots.iter().enumerate().any(|(slot, ally)| {
                    ally.is_some_and(|ally| {
                        ally != id
                            && (slot >= 3) == rear
                            && self.units[&ally].headcount.saturating_mul(2)
                                < self.units[&ally].input.capacity
                    })
                })
            }
            TacticCondition::AllyBelowHalfMorale => unit.position.is_some_and(|position| {
                self.armies[&position.army]
                    .slots
                    .iter()
                    .flatten()
                    .any(|ally| {
                        *ally != id
                            && self.units[ally].morale.saturating_mul(2) < self.rules.initial_morale
                    })
            }),
            TacticCondition::IncomingCavalryCharge => false,
        }
    }

    fn resolve_attack(&mut self, actor: BattleUnitId, target: BattleUnitId, action: TacticAction) {
        let multiplier = match action {
            TacticAction::Charge => self.rules.charge_damage_permille,
            TacticAction::Breakthrough => self.rules.breakthrough_damage_permille,
            _ => 1000,
        };
        self.resolve_attack_with_modifier(actor, target, action, multiplier);
    }

    fn resolve_attack_with_modifier(
        &mut self,
        actor: BattleUnitId,
        target: BattleUnitId,
        action: TacticAction,
        mut multiplier: u32,
    ) {
        if !self.can_damage(target) {
            return;
        }
        if action == TacticAction::Charge {
            if let Some(brace_factor) = self.resolve_reaction(target, actor) {
                multiplier = multiplier.saturating_mul(brace_factor) / 1000;
            }
        }
        if !self.can_damage(target) {
            return;
        }
        if self.units[&target]
            .guard_expires_round
            .is_some_and(|expires| expires >= self.round)
        {
            multiplier = multiplier.saturating_mul(self.rules.guard_damage_permille) / 1000;
        }
        if matches!(
            action,
            TacticAction::Attack | TacticAction::Charge | TacticAction::Breakthrough
        ) && self.units[&target].input.kind
            == Some(crate::data::economy::TroopKind::SiegeEngines)
        {
            multiplier =
                multiplier.saturating_mul(self.rules.siege_engine_close_damage_permille) / 1000;
        }
        let damage = self.damage_amount(actor, target, multiplier);
        self.apply_damage(actor, target, damage);
    }

    pub(super) fn resolve_routs(&mut self) {
        let ids: Vec<_> = self.units.keys().copied().collect();
        for id in ids {
            let Some(position) = self.units[&id].position else {
                continue;
            };
            if self.units[&id].headcount == 0 || self.units[&id].morale > self.rules.rout_morale {
                continue;
            }
            let side = self.armies[&position.army].side;
            let survivors = self.units[&id].headcount;
            self.armies
                .get_mut(&position.army)
                .expect("routing army")
                .slots[usize::from(position.slot)] = None;
            let unit = self.units.get_mut(&id).expect("routing unit");
            unit.position = None;
            unit.routed = true;
            self.events.push(BattleEvent::Routed {
                unit: id,
                position,
                survivors,
            });
            let allies: Vec<_> = self
                .units
                .iter()
                .filter_map(|(ally_id, ally)| {
                    let ally_position = ally.position?;
                    (self.armies[&ally_position.army].side == side && ally.headcount > 0)
                        .then_some(*ally_id)
                })
                .collect();
            for ally in allies {
                self.change_morale(ally, self.rules.ally_rout_morale_loss);
            }
        }
    }

    fn restore_morale(&mut self, id: BattleUnitId, amount: u32) {
        let unit = self.units.get_mut(&id).expect("morale target");
        let before = unit.morale;
        unit.morale = unit
            .morale
            .saturating_add(amount)
            .min(self.rules.initial_morale);
        let after = unit.morale;
        if before != after {
            self.events.push(BattleEvent::MoraleChanged {
                unit: id,
                before,
                after,
            });
        }
    }
}

fn is_attack(action: TacticAction) -> bool {
    matches!(
        action,
        TacticAction::Attack
            | TacticAction::Volley
            | TacticAction::Charge
            | TacticAction::Breakthrough
    )
}
