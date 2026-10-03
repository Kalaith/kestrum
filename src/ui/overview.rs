//! Compact owned accounts and map-linked known conditions.

use super::{components::*, Context, UiAction};
use kestrum::engine::{AttentionKind, AttentionTarget};
use macroquad::prelude::*;
use macroquad_toolkit::{colors::with_alpha, ui::truncate_text_to_width_ex};

const ACCOUNTS: Rect = Rect::new(24.0, 936.0, 1100.0, 60.0);
const ATTENTION: Rect = Rect::new(1572.0, 936.0, 324.0, 60.0);
const ROWS: usize = 3;

#[derive(Debug, Clone, Default)]
pub struct OverviewView {
    pub expanded: bool,
    pub page: usize,
}

pub fn attention_bounds(view: &OverviewView, selected: bool, count: usize) -> Rect {
    let height = if view.expanded && !selected && count > 0 {
        ATTENTION.h + count.min(ROWS) as f32 * 52.0 + if count > ROWS { 48.0 } else { 0.0 }
    } else {
        ATTENTION.h
    };
    Rect::new(
        ATTENTION.x,
        ATTENTION.bottom() - height,
        ATTENTION.w,
        height,
    )
}

pub fn controls_contain(point: Vec2, view: &OverviewView, selected: bool, count: usize) -> bool {
    ACCOUNTS.contains(point) || attention_bounds(view, selected, count).contains(point)
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    accounts(ctx);
    attention(ctx)
}

fn accounts(ctx: &Context<'_>) {
    let Some(view) = ctx.campaign_view else {
        return;
    };
    let Some(faction) = view
        .factions
        .iter()
        .find(|faction| faction.id == view.observer)
    else {
        return;
    };
    let Some(resources) = faction.resources else {
        return;
    };
    draw_rectangle(
        ACCOUNTS.x,
        ACCOUNTS.y,
        ACCOUNTS.w,
        ACCOUNTS.h,
        with_alpha(INK, 0.91),
    );
    let mut balances = format!(
        "{} {}    {} {}    {} {}",
        ctx.text("gold"),
        resources.gold,
        ctx.text("wood"),
        resources.wood,
        ctx.text("stone"),
        resources.stone
    );
    if let Some(receipt) = faction
        .last_economy
        .as_ref()
        .filter(|receipt| receipt.shortfall > 0)
    {
        balances.push_str(&format!(
            "    {} {}G",
            ctx.text("map_shortfall"),
            receipt.shortfall
        ));
    }
    body(ctx, &balances, vec2(38.0, ACCOUNTS.y + 25.0), 20.0, CREAM);
    let receipt = faction.last_economy.as_ref().map_or_else(
        || ctx.text("map_no_receipt"),
        |receipt| {
            let recovery = faction
                .last_recovery
                .as_ref()
                .filter(|recovery| recovery.completed_rounds == receipt.completed_rounds)
                .map_or(0, |recovery| recovery.gold_spent);
            format!(
                "{} {}: {} +{}G +{}W +{}S  |  {} {}/{}G  |  {} {}G",
                ctx.text("map_last_season"),
                receipt.completed_rounds,
                ctx.text("map_income"),
                receipt.income.gold,
                receipt.income.wood,
                receipt.income.stone,
                ctx.text("map_upkeep_paid_due"),
                receipt.upkeep_paid,
                receipt.upkeep_due,
                ctx.text("map_recovery_cost"),
                recovery,
            )
        },
    );
    let label = truncate_text_to_width_ex(&receipt, ACCOUNTS.w - 28.0, ctx.body_font(), 17.0);
    body(
        ctx,
        &label,
        vec2(38.0, ACCOUNTS.y + 49.0),
        17.0,
        if faction.deficit == Some(true) {
            BRASS
        } else {
            CREAM
        },
    );
}

fn attention(ctx: &Context<'_>) -> Option<UiAction> {
    let overview = ctx.overview?;
    let selected =
        ctx.navigation.selection().is_some() || ctx.movement.stage == super::MoveStage::Map;
    let expanded = ctx.overview_ui.expanded && !selected && !overview.attention.is_empty();
    let bounds = attention_bounds(ctx.overview_ui, selected, overview.attention.len());
    draw_rectangle(
        bounds.x,
        bounds.y,
        bounds.w,
        bounds.h,
        with_alpha(INK, 0.93),
    );
    let title = format!(
        "{} {}  {}",
        ctx.text("map_attention"),
        overview.attention.len(),
        if expanded { "−" } else { "+" }
    );
    let active = ctx.state.overlay == kestrum::state::Overlay::None;
    if button(
        ctx,
        ATTENTION,
        &title,
        active && !overview.attention.is_empty(),
        false,
    ) {
        return Some(UiAction::ToggleAttention);
    }
    if !expanded {
        return None;
    }
    let pages = overview.attention.len().div_ceil(ROWS);
    let page = ctx.overview_ui.page.min(pages.saturating_sub(1));
    for (row, entry) in overview
        .attention
        .iter()
        .skip(page * ROWS)
        .take(ROWS)
        .enumerate()
    {
        let rect = Rect::new(ATTENTION.x, bounds.y + row as f32 * 52.0, ATTENTION.w, 52.0);
        if let Some(action) = attention_row(ctx, rect, entry) {
            return Some(action);
        }
    }
    attention_pager(ctx, page, pages)
}

fn attention_row(
    ctx: &Context<'_>,
    rect: Rect,
    entry: &kestrum::engine::MapAttention,
) -> Option<UiAction> {
    let view = ctx.campaign_view?;
    let key = match entry.kind {
        AttentionKind::Siege => "map_attention_siege",
        AttentionKind::HostileContact => "map_attention_contact",
        AttentionKind::LocalThreat => "map_attention_threat",
        AttentionKind::RuinRisk => "map_attention_ruin",
        AttentionKind::DeclineRisk => "map_attention_decline",
        AttentionKind::Contested => "map_attention_contested",
        AttentionKind::Occupied => "map_attention_occupied",
        AttentionKind::Unsupplied => "map_attention_unsupplied",
    };
    let name = match entry.target {
        AttentionTarget::Site(id) => view.world.site(id).map(|site| site.name.as_str()),
        AttentionTarget::Army(id) => view
            .armies
            .iter()
            .find(|army| army.id == id)
            .map(|army| army.name.as_str()),
    }
    .unwrap_or_default();
    let tapped = button(
        ctx,
        rect,
        "",
        ctx.state.overlay == kestrum::state::Overlay::None,
        false,
    );
    body(
        ctx,
        &ctx.text(key),
        vec2(rect.x + 12.0, rect.y + 20.0),
        17.0,
        BRASS,
    );
    let label = truncate_text_to_width_ex(name, rect.w - 24.0, ctx.body_font(), 18.0);
    body(ctx, &label, vec2(rect.x + 12.0, rect.y + 43.0), 18.0, CREAM);
    tapped.then_some(UiAction::FocusAttention(entry.target))
}

fn attention_pager(ctx: &Context<'_>, page: usize, pages: usize) -> Option<UiAction> {
    let active = ctx.state.overlay == kestrum::state::Overlay::None;
    if pages > 1 {
        let y = ATTENTION.y - 48.0;
        if button(
            ctx,
            Rect::new(ATTENTION.x, y, 82.0, 48.0),
            &ctx.text("map_previous"),
            active && page > 0,
            false,
        ) {
            return Some(UiAction::AttentionPage(-1));
        }
        body(
            ctx,
            &format!("{}/{}", page + 1, pages),
            vec2(ATTENTION.x + 142.0, y + 30.0),
            18.0,
            CREAM,
        );
        if button(
            ctx,
            Rect::new(ATTENTION.right() - 82.0, y, 82.0, 48.0),
            &ctx.text("map_next"),
            active && page + 1 < pages,
            false,
        ) {
            return Some(UiAction::AttentionPage(1));
        }
    }
    None
}
