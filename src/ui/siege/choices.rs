//! Paged own-side groups, concrete exits, and confirmation consequences.

use super::*;

pub(super) fn forces(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.siege.view.as_ref()?;
    let campaign = ctx.campaign_view?;
    for (index, id) in view
        .own_armies
        .iter()
        .skip(ctx.siege.page * SIEGE_PAGE_SIZE)
        .take(SIEGE_PAGE_SIZE)
        .enumerate()
    {
        let Some(army) = campaign.armies.iter().find(|army| army.id == *id) else {
            continue;
        };
        let y = 239.0 + index as f32 * 58.0;
        let name = truncate_text_to_width_ex(&army.name, 808.0, ctx.body_font(), 20.0);
        body(ctx, &name, vec2(112.0, y), 20.0, CREAM);
        let remaining = ctx.siege.remaining.get(id).copied().unwrap_or(0);
        let supply = ctx.text(if view.supplied_armies.contains(id) {
            "siege_supplied"
        } else {
            "siege_cutoff"
        });
        lines(
            ctx,
            &format!("{supply} · {}: {remaining}", ctx.text("movement_left")),
            vec2(112.0, y + 24.0),
            808.0,
            1,
            MUTED,
        );
        let selected = ctx.siege.selected.contains(id);
        if control(
            ctx,
            Rect::new(966.0, y - 18.0, 202.0, 48.0),
            if selected { "selected" } else { "select" },
            true,
            selected,
        ) {
            return Some(UiAction::ToggleSiegeArmy(*id));
        }
    }
    if let Some(action) = pages(ctx, view.own_armies.len(), 532.0) {
        return Some(action);
    }
    control(
        ctx,
        Rect::new(860.0, 626.0, 308.0, 48.0),
        "siege_orders",
        !ctx.siege.selected.is_empty(),
        true,
    )
    .then_some(UiAction::SiegeMode(SiegeMode::Orders))
}

pub(super) fn orders(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.siege.view.as_ref()?;
    for (index, option) in view.actions.iter().enumerate() {
        let y = 228.0 + index as f32 * 110.0;
        if control(
            ctx,
            Rect::new(112.0, y, 252.0, 48.0),
            action_key(option.action),
            true,
            false,
        ) {
            return Some(UiAction::ChooseSiegeAction(option.action));
        }
        lines(
            ctx,
            &ctx.text(consequence_key(option.action)),
            vec2(396.0, y + 18.0),
            772.0,
            2,
            CREAM,
        );
        if let Some(reason) = &option.blocked {
            lines(ctx, reason, vec2(396.0, y + 68.0), 772.0, 2, BRASS);
        }
    }
    None
}

pub(super) fn exits(ctx: &Context<'_>) -> Option<UiAction> {
    for (index, exit) in ctx
        .siege
        .exits
        .iter()
        .skip(ctx.siege.page * SIEGE_PAGE_SIZE)
        .take(SIEGE_PAGE_SIZE)
        .enumerate()
    {
        let y = 239.0 + index as f32 * 58.0;
        body(
            ctx,
            &truncate_text_to_width_ex(&exit.name, 808.0, ctx.body_font(), 20.0),
            vec2(112.0, y),
            20.0,
            CREAM,
        );
        if let Some(reason) = &exit.blocked {
            lines(ctx, reason, vec2(112.0, y + 24.0), 808.0, 1, BRASS);
        }
        if control(
            ctx,
            Rect::new(966.0, y - 18.0, 202.0, 48.0),
            "siege_choose_exit",
            exit.blocked.is_none(),
            false,
        ) {
            return Some(UiAction::SiegeDestination(exit.site));
        }
    }
    if ctx.siege.exits.is_empty() {
        lines(
            ctx,
            &ctx.text("siege_no_exit"),
            vec2(112.0, 242.0),
            1056.0,
            4,
            BRASS,
        );
    }
    pages(ctx, ctx.siege.exits.len(), 532.0)
}

pub(super) fn review(ctx: &Context<'_>) -> Option<UiAction> {
    let action = ctx.siege.action?;
    body(
        ctx,
        &ctx.text(action_key(action)),
        vec2(112.0, 242.0),
        20.0,
        CREAM,
    );
    lines(
        ctx,
        &ctx.text(consequence_key(action)),
        vec2(112.0, 279.0),
        1056.0,
        3,
        CREAM,
    );
    let count = ctx.siege.selected.len();
    lines(
        ctx,
        &format!("{}: {count}", ctx.text("siege_selected_forces")),
        vec2(112.0, 364.0),
        1056.0,
        1,
        MUTED,
    );
    if let Some(destination) = ctx.siege.destination {
        lines(
            ctx,
            &format!(
                "{}: {}",
                ctx.text("siege_exit"),
                site_name(ctx, Some(destination))
            ),
            vec2(112.0, 400.0),
            1056.0,
            2,
            CREAM,
        );
    }
    if matches!(
        action,
        SiegeAction::Assault | SiegeAction::Sortie | SiegeAction::Escape
    ) {
        lines(
            ctx,
            &ctx.text(if action == SiegeAction::Assault {
                "siege_assault_risk"
            } else {
                "siege_battle_risk"
            }),
            vec2(112.0, 466.0),
            1056.0,
            3,
            MUTED,
        );
    }
    if let Some(reason) = &ctx.siege.blocked {
        lines(ctx, reason, vec2(112.0, 548.0), 1056.0, 2, BRASS);
    }
    control(
        ctx,
        Rect::new(860.0, 626.0, 308.0, 48.0),
        "siege_confirm",
        ctx.siege.blocked.is_none(),
        true,
    )
    .then_some(UiAction::ConfirmSiege)
}
