//! Every physical route edge remains inspectable, including regional entrances.

use super::*;

pub(super) fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let preview = ctx.movement.preview.as_ref()?;
    body(
        ctx,
        &cost_summary(ctx, preview),
        vec2(112.0, 123.0),
        20.0,
        CREAM,
    );
    lines(
        ctx,
        &route_consequence(ctx, preview),
        vec2(112.0, 158.0),
        1040.0,
        2,
        BRASS,
    );
    let page_count = preview.steps.len().div_ceil(ROUTE_PAGE_SIZE).max(1);
    let page = ctx.movement.route_page.min(page_count - 1);
    for (index, step) in preview
        .steps
        .iter()
        .enumerate()
        .skip(page * ROUTE_PAGE_SIZE)
        .take(ROUTE_PAGE_SIZE)
    {
        let y = 212.0 + (index % ROUTE_PAGE_SIZE) as f32 * 50.0;
        let from = campaign
            .world
            .site(step.from)
            .map(|site| site.name.as_str())
            .unwrap_or_default();
        let to = campaign
            .world
            .site(step.to)
            .map(|site| site.name.as_str())
            .unwrap_or_default();
        let from = truncate_text_to_width_ex(from, 432.0, ctx.body_font(), 18.0);
        let to = truncate_text_to_width_ex(to, 432.0, ctx.body_font(), 18.0);
        let color = if index < preview.reachable_steps {
            CREAM
        } else {
            MUTED
        };
        body(ctx, &format!("{}", index + 1), vec2(112.0, y), 18.0, MUTED);
        body(ctx, &from, vec2(150.0, y), 18.0, color);
        body(ctx, "→", vec2(584.0, y), 18.0, color);
        body(ctx, &to, vec2(618.0, y), 18.0, color);
        body(
            ctx,
            &format!("{}: {}", ctx.text("cost"), step.cost),
            vec2(1060.0, y),
            18.0,
            color,
        );
        draw_line(
            112.0,
            y + 17.0,
            1168.0,
            y + 17.0,
            1.0,
            Color::new(0.21, 0.29, 0.25, 1.0),
        );
    }
    review_controls(ctx, preview, page, page_count)
}

fn review_controls(
    ctx: &Context<'_>,
    preview: &MovementPreview,
    page: usize,
    page_count: usize,
) -> Option<UiAction> {
    if button(
        ctx,
        Rect::new(112.0, 526.0, 160.0, 48.0),
        &ctx.text("previous"),
        page > 0,
        false,
    ) {
        return Some(UiAction::MoveRoutePage(-1));
    }
    centered(
        ctx,
        &format!("{} / {page_count}", page + 1),
        vec2(640.0, 557.0),
        20.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(1008.0, 526.0, 160.0, 48.0),
        &ctx.text("next"),
        page + 1 < page_count,
        false,
    ) {
        return Some(UiAction::MoveRoutePage(1));
    }
    let note = if ctx.movement.status.is_empty() {
        ctx.text(if preview.supplied_after {
            "move_supplied_after"
        } else {
            "move_unsupplied_after"
        })
    } else {
        ctx.movement.status.clone()
    };
    lines(ctx, &note, vec2(112.0, 591.0), 1040.0, 2, MUTED);
    if button(
        ctx,
        Rect::new(112.0, 626.0, 220.0, 48.0),
        &ctx.text("back_to_map"),
        true,
        false,
    ) {
        return Some(UiAction::ChooseMoveDestination);
    }
    if button(
        ctx,
        Rect::new(860.0, 626.0, 308.0, 48.0),
        &ctx.text("confirm_move"),
        preview.can_confirm(),
        true,
    ) {
        return Some(UiAction::ConfirmMove);
    }
    None
}
