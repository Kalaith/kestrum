//! Presentation-only playback controls over an already resolved battle.

use super::*;
use kestrum::{
    data::{
        battle_tactics::{
            TacticAction, TacticCondition, TacticRule, TacticTrigger, TargetFilter, TargetPriority,
            TroopTactics,
        },
        economy::TroopKind,
    },
    state::military::FormationId,
};

impl Game {
    pub(super) fn advance_battlefield(&mut self, delta_seconds: f32) {
        if let Some(resolution) = &self.battlefield {
            self.battlefield_view
                .advance(delta_seconds, resolution.events.len());
        }
    }

    pub(super) fn apply_battlefield_action(&mut self, action: ui::BattlefieldAction) {
        match action {
            ui::BattlefieldAction::StartPendingBattle => {
                self.apply_campaign_command(Command::StartPendingBattle);
                return;
            }
            ui::BattlefieldAction::EditTactics { formation, edit } => {
                self.edit_battle_tactics(formation, edit);
                return;
            }
            ui::BattlefieldAction::SetTacticTrigger(trigger) => {
                self.battlefield_view.tactic_trigger = trigger;
                return;
            }
            ui::BattlefieldAction::SwapFormationSlots {
                army,
                first,
                second,
            } => {
                self.apply_campaign_command(Command::SwapFormationSlots {
                    army,
                    first,
                    second,
                });
                return;
            }
            ui::BattlefieldAction::Close => {
                self.state.overlay = self.battle_return.take().unwrap_or(Overlay::None);
                self.battlefield = None;
                self.battlefield_view = ui::BattlefieldView::default();
                return;
            }
            _ => {}
        }
        let event_count = self
            .battlefield
            .as_ref()
            .map_or(0, |resolution| resolution.events.len());
        self.battlefield_view.apply(action, event_count);
    }

    pub(super) fn open_pending_battlefield(&mut self) {
        let resolution = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| campaign.pending_battle.as_ref())
            .and_then(|pending| pending.report.simulation.clone());
        let Some(resolution) = resolution else {
            return;
        };
        self.battlefield = Some(resolution);
        self.battlefield_view = ui::BattlefieldView {
            is_paused: true,
            ..Default::default()
        };
        self.state.overlay = Overlay::Battlefield;
        self.error = None;
    }

    pub(super) fn refresh_pending_battlefield(&mut self) {
        if self.state.overlay != Overlay::Battlefield {
            self.open_pending_battlefield();
            return;
        }
        let resolution = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| campaign.pending_battle.as_ref())
            .and_then(|pending| pending.report.simulation.clone());
        if let Some(resolution) = resolution {
            self.battlefield = Some(resolution);
            self.battlefield_view.event_cursor = 0;
            self.battlefield_view.event_elapsed = 0.0;
            self.battlefield_view.is_paused = true;
        }
    }

    pub(super) fn open_committed_battlefield(&mut self, id: kestrum::state::battle::BattleId) {
        let resolution = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| campaign.battles.get(&id))
            .and_then(|report| report.simulation.clone());
        let Some(resolution) = resolution else {
            return;
        };
        self.battlefield = Some(resolution);
        self.battlefield_view = ui::BattlefieldView::default();
        self.state.overlay = Overlay::Battlefield;
        self.error = None;
    }

    pub(super) fn sync_pending_battle(&mut self) {
        if self.state.overlay == Overlay::Battlefield {
            return;
        }
        let pending = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .is_some_and(|campaign| campaign.pending_battle.is_some());
        if pending {
            self.open_pending_battlefield();
        }
    }

    fn edit_battle_tactics(&mut self, formation: FormationId, edit: ui::TacticEdit) {
        let current = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| campaign.formations.get(&formation))
            .map(|entry| (entry.kind, entry.tactics.clone()));
        let Some((kind, current)) = current else {
            return;
        };
        let Some(defaults) = self.data.battle_tactics.defaults_for(kind) else {
            return;
        };
        let mut tactics = current.unwrap_or_else(|| defaults.clone());
        let trigger = match edit {
            ui::TacticEdit::Add(trigger)
            | ui::TacticEdit::Remove(trigger, _)
            | ui::TacticEdit::Move(trigger, _, _)
            | ui::TacticEdit::CycleAction(trigger, _)
            | ui::TacticEdit::CycleCondition(trigger, _)
            | ui::TacticEdit::CycleTarget(trigger, _) => trigger,
        };
        let rows = tactics_rows_mut(&mut tactics, trigger);
        match edit {
            ui::TacticEdit::Add(trigger) => {
                if rows.len() >= self.data.battle_tactics.max_tactic_rows as usize {
                    return;
                }
                let attack = trigger == TacticTrigger::Activation;
                let id = next_rule_id(rows, trigger);
                rows.push(TacticRule {
                    id,
                    trigger,
                    action: if attack {
                        TacticAction::Attack
                    } else {
                        TacticAction::Brace
                    },
                    condition: TacticCondition::Always,
                    target_filter: if attack {
                        TargetFilter::AnyEnemy
                    } else {
                        TargetFilter::None
                    },
                    target_priority: TargetPriority::OwnColumnFirst,
                });
            }
            ui::TacticEdit::Remove(_, index) => {
                if index < rows.len() {
                    rows.remove(index);
                }
            }
            ui::TacticEdit::Move(_, index, up) => {
                let target = if up {
                    index.checked_sub(1)
                } else {
                    Some(index + 1)
                };
                if let Some(target) = target.filter(|target| *target < rows.len()) {
                    rows.swap(index, target);
                }
            }
            ui::TacticEdit::CycleAction(_, index) => {
                if let Some(rule) = rows.get_mut(index) {
                    let actions = legal_actions(kind, trigger);
                    rule.action = next_value(&actions, rule.action);
                    rule.target_filter = if is_attack(rule.action) {
                        TargetFilter::AnyEnemy
                    } else {
                        TargetFilter::None
                    };
                }
            }
            ui::TacticEdit::CycleCondition(_, index) => {
                if let Some(rule) = rows.get_mut(index) {
                    rule.condition = next_value(&legal_conditions(trigger), rule.condition);
                }
            }
            ui::TacticEdit::CycleTarget(_, index) => {
                if let Some(rule) = rows.get_mut(index).filter(|rule| is_attack(rule.action)) {
                    let filters: Vec<_> = [
                        TargetFilter::AnyEnemy,
                        TargetFilter::EnemyFront,
                        TargetFilter::EnemyRear,
                        TargetFilter::EnemyCavalry,
                        TargetFilter::ExposedEnemyRear,
                    ]
                    .into_iter()
                    .filter(|filter| {
                        rule.action != TacticAction::Breakthrough
                            || *filter != TargetFilter::EnemyFront
                    })
                    .collect();
                    rule.target_filter = next_value(&filters, rule.target_filter);
                }
            }
        }
        self.apply_campaign_command(Command::SetFormationTactics { formation, tactics });
    }
}

fn tactics_rows_mut(tactics: &mut TroopTactics, trigger: TacticTrigger) -> &mut Vec<TacticRule> {
    match trigger {
        TacticTrigger::Activation => &mut tactics.activation,
        TacticTrigger::IncomingAttack => &mut tactics.reaction,
        TacticTrigger::EndOfRound => &mut tactics.activation,
    }
}

fn legal_actions(kind: TroopKind, trigger: TacticTrigger) -> Vec<TacticAction> {
    if trigger == TacticTrigger::IncomingAttack {
        return vec![TacticAction::Brace, TacticAction::Guard];
    }
    let mut actions = vec![TacticAction::Attack];
    if matches!(kind, TroopKind::Archers | TroopKind::SiegeEngines) {
        actions.push(TacticAction::Volley);
    }
    if kind == TroopKind::Riders {
        actions.extend([TacticAction::Charge, TacticAction::Breakthrough]);
    }
    actions.extend([
        TacticAction::Guard,
        TacticAction::Advance,
        TacticAction::Wait,
    ]);
    actions
}

fn legal_conditions(trigger: TacticTrigger) -> Vec<TacticCondition> {
    if trigger == TacticTrigger::IncomingAttack {
        vec![
            TacticCondition::Always,
            TacticCondition::IncomingCavalryCharge,
        ]
    } else {
        vec![
            TacticCondition::Always,
            TacticCondition::EnemyRearExposed,
            TacticCondition::EnemyCavalryPresent,
            TacticCondition::FirstActivation,
            TacticCondition::SelfBelowHalf,
            TacticCondition::AllyInSameRowBelowHalf,
        ]
    }
}

fn next_value<T: Copy + PartialEq>(values: &[T], current: T) -> T {
    let next = values
        .iter()
        .position(|value| *value == current)
        .map_or(0, |index| (index + 1) % values.len());
    values[next]
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

fn next_rule_id(rows: &[TacticRule], trigger: TacticTrigger) -> String {
    let prefix = match trigger {
        TacticTrigger::Activation => "activation",
        TacticTrigger::IncomingAttack => "reaction",
        TacticTrigger::EndOfRound => "end",
    };
    let mut index = rows.len() + 1;
    while rows
        .iter()
        .any(|rule| rule.id == format!("custom_{prefix}_{index}"))
    {
        index += 1;
    }
    format!("custom_{prefix}_{index}")
}
