//! Paginated co-located group selection with no stack limit.

use super::*;

pub(super) fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let armies: Vec<_> = campaign
        .armies
        .iter()
        .filter(|army| Some(army.site) == ctx.movement.site)
        .collect();
    let page_count = armies.len().div_ceil(MOVE_GROUP_PAGE_SIZE).max(1);
    let page = ctx.movement.page.min(page_count - 1);
    lines(
        ctx,
        &ctx.text("move_group_help"),
        vec2(112.0, 124.0),
        1040.0,
        2,
        MUTED,
    );
    for (index, army) in armies
        .iter()
        .skip(page * MOVE_GROUP_PAGE_SIZE)
        .take(MOVE_GROUP_PAGE_SIZE)
        .enumerate()
    {
        let rect = Rect::new(112.0, 169.0 + index as f32 * 59.0, 1056.0, 52.0);
        let selected = ctx.movement.armies.contains(&army.id);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if selected {
                Color::new(0.21, 0.29, 0.25, 1.0)
            } else {
                Color::new(0.10, 0.17, 0.16, 1.0)
            },
        );
        draw_rectangle_lines(rect.x + 14.0, rect.y + 15.0, 22.0, 22.0, 2.0, BRASS);
        if selected {
            draw_rectangle(rect.x + 19.0, rect.y + 20.0, 12.0, 12.0, CREAM);
        }
        let name = truncate_text_to_width_ex(&army.name, 742.0, ctx.body_font(), 21.0);
        body(ctx, &name, vec2(rect.x + 52.0, rect.y + 33.0), 21.0, CREAM);
        let remaining = ctx.movement.remaining.get(&army.id).copied().unwrap_or(0);
        body(
            ctx,
            &format!("{}: {remaining}", ctx.text("movement_left")),
            vec2(934.0, rect.y + 32.0),
            18.0,
            MUTED,
        );
        if tapped(ctx, rect) {
            return Some(UiAction::ToggleMoveArmy(army.id));
        }
    }
    if button(
        ctx,
        Rect::new(112.0, 533.0, 160.0, 48.0),
        &ctx.text("previous"),
        page > 0,
        false,
    ) {
        return Some(UiAction::MoveGroupPage(-1));
    }
    centered(
        ctx,
        &format!("{} / {page_count}", page + 1),
        vec2(640.0, 564.0),
        20.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(1008.0, 533.0, 160.0, 48.0),
        &ctx.text("next"),
        page + 1 < page_count,
        false,
    ) {
        return Some(UiAction::MoveGroupPage(1));
    }
    if !ctx.movement.status.is_empty() {
        lines(
            ctx,
            &ctx.movement.status,
            vec2(112.0, 608.0),
            1040.0,
            1,
            BRASS,
        );
    }
    if button(
        ctx,
        Rect::new(112.0, 626.0, 166.0, 48.0),
        &ctx.text("cancel"),
        true,
        false,
    ) {
        return Some(UiAction::CancelMove);
    }
    let label = format!(
        "{} ({})",
        ctx.text("choose_destination"),
        ctx.movement.armies.len()
    );
    if button(
        ctx,
        Rect::new(788.0, 626.0, 380.0, 48.0),
        &label,
        !ctx.movement.armies.is_empty() && campaign.player_turn,
        true,
    ) {
        return Some(UiAction::ChooseMoveDestination);
    }
    None
}
