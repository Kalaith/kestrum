//! Paged choices keep every construction and builder reachable by touch.

use super::*;

pub(super) fn build(ctx: &Context<'_>) -> Option<UiAction> {
    let road = ctx.settlement.mode == SettlementMode::Roads;
    let choices: Vec<_> = ctx
        .settlement
        .options
        .iter()
        .filter(|choice| matches!(choice.target, ConstructionTarget::Route(_)) == road)
        .collect();
    if choices.is_empty() {
        lines(
            ctx,
            &ctx.text("construction_no_routes"),
            vec2(112.0, 250.0),
            1056.0,
            2,
            MUTED,
        );
    }
    for (index, choice) in choices
        .iter()
        .skip(ctx.settlement.page * 4)
        .take(4)
        .enumerate()
    {
        let y = 230.0 + index as f32 * 90.0;
        let name = if road {
            format!(
                "{} · {}",
                kind_name(ctx, choice.option.kind),
                labels::target_name(ctx, choice.target)
            )
        } else {
            kind_name(ctx, choice.option.kind)
        };
        body(
            ctx,
            &truncate_text_to_width_ex(&name, 850.0, ctx.body_font(), 20.0),
            vec2(112.0, y),
            20.0,
            CREAM,
        );
        let cost = format!(
            "{} · {} {}",
            resources(ctx, choice.option.cost),
            choice.option.steps,
            ctx.text("construction_steps")
        );
        lines(ctx, &cost, vec2(112.0, y + 24.0), 850.0, 1, BRASS);
        let reason = choice
            .option
            .blocked
            .clone()
            .unwrap_or_else(|| ctx.text(labels::effect_key(choice.option.kind)));
        lines(ctx, &reason, vec2(112.0, y + 47.0), 850.0, 1, MUTED);
        if control(
            ctx,
            Rect::new(984.0, y - 17.0, 184.0, 48.0),
            "construction_review",
            true,
            false,
        ) {
            return Some(UiAction::SelectConstruction(
                choice.target,
                choice.option.kind,
            ));
        }
    }
    page_controls(ctx, choices.len())
}

pub(super) fn builders(ctx: &Context<'_>) -> Option<UiAction> {
    lines(
        ctx,
        &ctx.text("construction_builder_help"),
        vec2(112.0, 226.0),
        1056.0,
        2,
        MUTED,
    );
    if ctx.settlement.builders.is_empty() {
        lines(
            ctx,
            &ctx.text("construction_no_builders"),
            vec2(112.0, 302.0),
            1056.0,
            2,
            BRASS,
        );
    }
    for (index, choice) in ctx
        .settlement
        .builders
        .iter()
        .skip(ctx.settlement.page * 4)
        .take(4)
        .enumerate()
    {
        let y = 292.0 + index as f32 * 75.0;
        body(
            ctx,
            &truncate_text_to_width_ex(&choice.name, 850.0, ctx.body_font(), 20.0),
            vec2(112.0, y),
            20.0,
            CREAM,
        );
        let detail = choice.blocked.as_ref().unwrap_or(&choice.location);
        lines(ctx, detail, vec2(112.0, y + 25.0), 850.0, 2, MUTED);
        if control(
            ctx,
            Rect::new(984.0, y - 18.0, 184.0, 48.0),
            "construction_assign",
            choice.blocked.is_none(),
            Some(choice.id) == ctx.settlement.builder,
        ) {
            return Some(UiAction::SelectBuilder(choice.id));
        }
    }
    page_controls(ctx, ctx.settlement.builders.len())
}

pub(super) fn focus(ctx: &Context<'_>) -> Option<UiAction> {
    for (index, choice) in ctx.settlement.focuses.iter().enumerate() {
        let x = 112.0 + (index % 2) as f32 * 540.0;
        let y = 222.0 + (index / 2) as f32 * 100.0;
        if control(
            ctx,
            Rect::new(x, y, 516.0, 48.0),
            focus_key(choice.focus),
            true,
            Some(choice.focus) == ctx.settlement.focus,
        ) {
            return Some(UiAction::SelectFocus(choice.focus));
        }
        if let Some(reason) = &choice.blocked {
            lines(ctx, reason, vec2(x + 8.0, y + 71.0), 500.0, 1, MUTED);
        }
    }
    let cost = ctx.economy.orders[&kestrum::data::economy::OrderKind::ChangeFocus].cost;
    lines(
        ctx,
        &format!(
            "{}: {}",
            ctx.text("construction_prepaid"),
            resources(ctx, cost)
        ),
        vec2(112.0, 520.0),
        1056.0,
        1,
        BRASS,
    );
    lines(
        ctx,
        &ctx.text(match ctx.settlement.focus.unwrap_or(Focus::Growth) {
            Focus::Growth => "focus_effect_growth",
            Focus::Fortification => "focus_effect_fortification",
            Focus::TroopTraining => "focus_effect_training",
            Focus::Gold => "focus_effect_gold",
            Focus::Wood => "focus_effect_wood",
            Focus::Stone => "focus_effect_stone",
        }),
        vec2(112.0, 548.0),
        1056.0,
        2,
        MUTED,
    );
    let selected = ctx.settlement.focus.and_then(|focus| {
        ctx.settlement
            .focuses
            .iter()
            .find(|choice| choice.focus == focus)
    });
    if control(
        ctx,
        Rect::new(916.0, 626.0, 252.0, 48.0),
        "focus_confirm",
        selected.is_some_and(|choice| choice.blocked.is_none()),
        true,
    ) {
        return Some(UiAction::ConfirmFocus);
    }
    None
}
