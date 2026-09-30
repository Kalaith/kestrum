//! Compact preparation controls for the currently selected friendly formation.

use super::{BattlefieldAction, TacticEdit};
use crate::ui::{components, Context, UiAction};
use kestrum::{
    data::battle_tactics::{TacticAction, TacticCondition, TacticTrigger, TargetFilter},
    state::{battle::simulation::BattleUnitId, military::FormationId},
};
use macroquad::prelude::*;

pub(super) fn prepare_text(ctx: &Context<'_>) {
    let labels = [
        (16, ctx.text("battle_tactic_activation")),
        (16, ctx.text("battle_tactic_reaction")),
        (14, ctx.text("battle_tactic_fallback")),
        (14, ctx.text("battle_tactic_add")),
        (12, ctx.text("battle_tactic_always")),
        (12, ctx.text("battle_tactic_enemy_rear")),
        (12, ctx.text("battle_tactic_enemy_cavalry")),
        (12, ctx.text("battle_tactic_first_activation")),
        (12, ctx.text("battle_tactic_self_hurt")),
        (12, ctx.text("battle_tactic_ally_hurt")),
        (12, ctx.text("battle_tactic_cavalry_charge")),
        (12, ctx.text("battle_tactic_any_enemy")),
        (12, ctx.text("battle_tactic_front")),
        (12, ctx.text("battle_tactic_rear")),
        (12, ctx.text("battle_tactic_exposed_rear")),
        (12, ctx.text("battle_tactic_no_target")),
    ];
    let prepared: Vec<_> = labels
        .iter()
        .map(|(size, label)| (*size, label.as_str()))
        .collect();
    if let Some(font) = ctx.body_font() {
        macroquad_toolkit::ui::prepare_font_text(font, &prepared);
    }
}

pub(super) fn draw_pending_editor(ctx: &Context<'_>, id: BattleUnitId) -> Option<Option<UiAction>> {
    let BattleUnitId::Formation(formation_id) = id else {
        return None;
    };
    let campaign = ctx.state.campaign.as_ref()?.strategic()?;
    let formation = campaign.formations.get(&formation_id)?;
    if formation.faction != campaign.player {
        return None;
    }
    let army = campaign
        .armies
        .values()
        .find(|army| army.formation_ids().any(|id| id == formation_id))?;
    let pending = campaign.pending_battle.as_ref()?;
    if !pending
        .report
        .faction_sides()
        .filter(|side| side.faction == campaign.player)
        .flat_map(|side| &side.armies)
        .any(|entry| entry.id == army.id)
    {
        return None;
    }
    let rules = formation
        .tactics
        .as_ref()
        .or_else(|| ctx.battle_tactics.defaults_for(formation.kind))?;
    let (trigger, rows) = if ctx.battlefield.tactic_trigger == TacticTrigger::Activation {
        (TacticTrigger::Activation, &rules.activation)
    } else {
        (TacticTrigger::IncomingAttack, &rules.reaction)
    };
    let visible_rows = rows.len().min(5);
    let rect = Rect::new(310.0, 86.0, 660.0, 57.0 + visible_rows as f32 * 18.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.04, 0.08, 0.08, 0.94),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, components::BRASS);
    let slot = army
        .slots
        .iter()
        .position(|entry| *entry == Some(formation_id))?;
    components::text(
        ctx,
        &format!("{} · {}", troop_label(ctx, formation.kind), slot + 1),
        vec2(rect.x + 12.0, rect.y + 17.0),
        15.0,
        components::CREAM,
    );
    let mut action = None;
    for (left, delta) in [(true, -1_i32), (false, 1_i32)] {
        let x = if left { rect.x + 520.0 } else { rect.x + 556.0 };
        if components::button(
            ctx,
            Rect::new(x, rect.y + 1.0, 32.0, 22.0),
            if left { "<" } else { ">" },
            (0..6).contains(&(slot as i32 + delta)),
            false,
        ) {
            action = Some(UiAction::Battlefield(
                BattlefieldAction::SwapFormationSlots {
                    army: army.id,
                    first: slot as u8,
                    second: (slot as i32 + delta) as u8,
                },
            ));
        }
    }
    for (trigger_tab, x, label) in [
        (
            TacticTrigger::Activation,
            rect.x + 160.0,
            ctx.text("battle_tactic_activation"),
        ),
        (
            TacticTrigger::IncomingAttack,
            rect.x + 320.0,
            ctx.text("battle_tactic_reaction"),
        ),
    ] {
        if components::button(
            ctx,
            Rect::new(x, rect.y + 1.0, 150.0, 22.0),
            &label,
            true,
            trigger == trigger_tab,
        ) {
            action = Some(UiAction::Battlefield(BattlefieldAction::SetTacticTrigger(
                trigger_tab,
            )));
        }
    }
    if trigger == TacticTrigger::Activation {
        components::body(
            ctx,
            &ctx.text("battle_tactic_fallback"),
            vec2(rect.x + 12.0, rect.y + 30.0),
            12.0,
            components::MUTED,
        );
    }
    for (index, rule) in rows.iter().take(5).enumerate() {
        let y = rect.y + 37.0 + index as f32 * 18.0;
        let move_up = components::button(
            ctx,
            Rect::new(rect.x + 10.0, y, 22.0, 18.0),
            "U",
            index > 0,
            false,
        );
        let move_down = components::button(
            ctx,
            Rect::new(rect.x + 34.0, y, 22.0, 18.0),
            "D",
            index + 1 < rows.len(),
            false,
        );
        if move_up || move_down {
            action = Some(UiAction::Battlefield(BattlefieldAction::EditTactics {
                formation: formation_id,
                edit: TacticEdit::Move(trigger, index, move_up),
            }));
        }
        if components::button(
            ctx,
            Rect::new(rect.x + 60.0, y, 116.0, 18.0),
            &action_label(ctx, rule.action),
            true,
            false,
        ) {
            action = Some(edit_tactics(
                formation_id,
                trigger,
                index,
                EditField::Action,
            ));
        }
        if components::button(
            ctx,
            Rect::new(rect.x + 180.0, y, 122.0, 18.0),
            &condition_label(ctx, rule.condition),
            true,
            false,
        ) {
            action = Some(edit_tactics(
                formation_id,
                trigger,
                index,
                EditField::Condition,
            ));
        }
        if components::button(
            ctx,
            Rect::new(rect.x + 306.0, y, 122.0, 18.0),
            &target_label(ctx, rule.target_filter),
            is_attack(rule.action),
            false,
        ) {
            action = Some(edit_tactics(
                formation_id,
                trigger,
                index,
                EditField::Target,
            ));
        }
        if components::button(
            ctx,
            Rect::new(rect.x + 432.0, y, 24.0, 18.0),
            "X",
            true,
            false,
        ) {
            action = Some(UiAction::Battlefield(BattlefieldAction::EditTactics {
                formation: formation_id,
                edit: TacticEdit::Remove(trigger, index),
            }));
        }
        components::body(
            ctx,
            &(index + 1).to_string(),
            vec2(rect.x + 472.0, y + 13.0),
            12.0,
            components::MUTED,
        );
    }
    if components::button(
        ctx,
        Rect::new(rect.x + 10.0, rect.y + rect.h - 18.0, 110.0, 18.0),
        &ctx.text("battle_tactic_add"),
        rows.len() < ctx.battle_tactics.max_tactic_rows as usize,
        false,
    ) {
        action = Some(UiAction::Battlefield(BattlefieldAction::EditTactics {
            formation: formation_id,
            edit: TacticEdit::Add(trigger),
        }));
    }
    Some(action)
}

#[derive(Clone, Copy)]
enum EditField {
    Action,
    Condition,
    Target,
}

fn edit_tactics(
    formation: FormationId,
    trigger: TacticTrigger,
    index: usize,
    field: EditField,
) -> UiAction {
    let edit = match field {
        EditField::Action => TacticEdit::CycleAction(trigger, index),
        EditField::Condition => TacticEdit::CycleCondition(trigger, index),
        EditField::Target => TacticEdit::CycleTarget(trigger, index),
    };
    UiAction::Battlefield(BattlefieldAction::EditTactics { formation, edit })
}

fn action_label(ctx: &Context<'_>, action: TacticAction) -> String {
    match action {
        TacticAction::Attack => ctx.text("battle_play_attack"),
        TacticAction::Volley => ctx.text("battle_play_volley"),
        TacticAction::Charge => ctx.text("battle_play_charge"),
        TacticAction::Breakthrough => ctx.text("battle_play_breakthrough"),
        TacticAction::Guard => ctx.text("battle_play_guard"),
        TacticAction::Brace => ctx.text("battle_play_brace"),
        TacticAction::Wait => ctx.text("battle_play_wait"),
        TacticAction::Advance => ctx.text("battle_play_advance"),
    }
}

fn condition_label(ctx: &Context<'_>, condition: TacticCondition) -> String {
    let key = match condition {
        TacticCondition::Always => "battle_tactic_always",
        TacticCondition::EnemyRearExposed => "battle_tactic_enemy_rear",
        TacticCondition::EnemyCavalryPresent => "battle_tactic_enemy_cavalry",
        TacticCondition::FirstActivation => "battle_tactic_first_activation",
        TacticCondition::SelfBelowHalf => "battle_tactic_self_hurt",
        TacticCondition::AllyInSameRowBelowHalf => "battle_tactic_ally_hurt",
        TacticCondition::IncomingCavalryCharge => "battle_tactic_cavalry_charge",
    };
    ctx.text(key)
}

fn target_label(ctx: &Context<'_>, target: TargetFilter) -> String {
    let key = match target {
        TargetFilter::None => "battle_tactic_no_target",
        TargetFilter::AnyEnemy => "battle_tactic_any_enemy",
        TargetFilter::EnemyFront => "battle_tactic_front",
        TargetFilter::EnemyRear => "battle_tactic_rear",
        TargetFilter::EnemyCavalry => "battle_tactic_enemy_cavalry",
        TargetFilter::ExposedEnemyRear => "battle_tactic_exposed_rear",
    };
    ctx.text(key)
}

fn troop_label(ctx: &Context<'_>, kind: kestrum::data::economy::TroopKind) -> String {
    let key = match kind {
        kestrum::data::economy::TroopKind::Warriors => "troop_warriors",
        kestrum::data::economy::TroopKind::Spearmen => "troop_spearmen",
        kestrum::data::economy::TroopKind::Archers => "troop_archers",
        kestrum::data::economy::TroopKind::Riders => "troop_riders",
        kestrum::data::economy::TroopKind::Medics => "troop_medics",
        kestrum::data::economy::TroopKind::SiegeEngines => "troop_siege_engines",
    };
    ctx.text(key)
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
