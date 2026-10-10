//! Compact map actions reuse the previews and command intents from the full sheets.

mod recruit;
mod settlement;

use super::Context;
use crate::ui::{
    components::{BRASS, CREAM, INK, MUTED},
    ArmyMode, SettlementMode, UiAction,
};
use kestrum::{
    data::{economy::Resources, world::SiteId},
    navigation::{MapScope, MapSelection},
    state::Overlay,
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::wrap_text_ex;

pub(super) fn active(ctx: &Context<'_>) -> bool {
    if ctx.state.overlay != Overlay::None {
        return false;
    }
    let recruitment = matches!(ctx.army.mode, ArmyMode::Recruit { .. })
        && ctx
            .army
            .site
            .is_some_and(|site| selection_matches(ctx, site));
    let construction = matches!(
        ctx.settlement.mode,
        SettlementMode::Build
            | SettlementMode::Roads
            | SettlementMode::Review
            | SettlementMode::Builders
            | SettlementMode::Focus
    ) && ctx
        .settlement
        .site
        .is_some_and(|site| selection_matches(ctx, site));
    recruitment || construction
}

pub(super) fn selection_matches(ctx: &Context<'_>, site: SiteId) -> bool {
    let Some(view) = ctx.campaign_view else {
        return false;
    };
    let Some(location) = view.world.site(site) else {
        return false;
    };
    match (ctx.navigation.scope(), ctx.navigation.selection()) {
        (MapScope::World, Some(MapSelection::Marker(marker))) => marker == location.marker,
        (MapScope::Region(_), Some(MapSelection::Site(selected))) => selected == site,
        _ => false,
    }
}

pub(super) fn draw(ctx: &Context<'_>, rect: Rect, fallback_name: &str) -> Option<UiAction> {
    if matches!(ctx.army.mode, ArmyMode::Recruit { .. })
        && ctx
            .army
            .site
            .is_some_and(|site| selection_matches(ctx, site))
    {
        recruit::draw(ctx, rect, ctx.army.site?, fallback_name)
    } else {
        settlement::draw(ctx, rect, ctx.settlement.site?, fallback_name)
    }
}

pub(super) fn prepare_text(ctx: &Context<'_>) {
    if !active(ctx) {
        return;
    }
    recruit::prepare_text(ctx);
    settlement::prepare_text(ctx);
}

#[derive(Clone, Copy)]
pub(super) struct CompactStyle {
    pub width: f32,
    pub max_lines: usize,
    pub color: Color,
    pub size: f32,
}

pub(super) fn card_frame(rect: Rect, selected: bool) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if selected {
            Color::new(0.20, 0.28, 0.24, 1.0)
        } else {
            Color::new(0.10, 0.17, 0.16, 1.0)
        },
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if selected { 2.0 } else { 1.0 },
        if selected { BRASS } else { MUTED },
    );
}

pub(super) fn compact_lines(ctx: &Context<'_>, value: &str, at: Vec2, style: CompactStyle) {
    for (index, line) in wrap_text_ex(value, style.width, ctx.body_font(), style.size)
        .iter()
        .take(style.max_lines)
        .enumerate()
    {
        crate::ui::components::body(
            ctx,
            line,
            vec2(at.x, at.y + index as f32 * (style.size + 2.0)),
            style.size,
            style.color,
        );
    }
}

pub(super) fn resources(ctx: &Context<'_>, value: Resources) -> String {
    format!(
        "{} {} · {} {} · {} {}",
        value.gold,
        ctx.text("gold"),
        value.wood,
        ctx.text("wood"),
        value.stone,
        ctx.text("stone")
    )
}

pub(super) fn tapped(ctx: &Context<'_>, rect: Rect) -> bool {
    ctx.pointer.released_on(rect) && ctx.origin.is_some_and(|origin| rect.contains(origin))
}

pub(super) fn shell(ctx: &Context<'_>, rect: Rect, name: &str, mode: &str) {
    use macroquad_toolkit::ui::truncate_text_to_width_ex;
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, INK);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, BRASS);
    let title = truncate_text_to_width_ex(name, 326.0, ctx.font(), 22.0);
    crate::ui::components::text(ctx, &title, vec2(rect.x + 16.0, rect.y + 34.0), 22.0, CREAM);
    crate::ui::components::body(
        ctx,
        &ctx.text(mode),
        vec2(rect.x + 16.0, rect.y + 62.0),
        16.0,
        BRASS,
    );
    draw_line(
        rect.x + 16.0,
        rect.y + 71.0,
        rect.right() - 16.0,
        rect.y + 71.0,
        1.0,
        BRASS,
    );
}
