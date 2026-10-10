//! Compact construction and focus choices retain the selected-place map context.

use super::*;
use crate::ui::{
    components::{body, button},
    settlement::{focus_key, kind_name, BuildChoice, BuilderChoice},
};
use kestrum::{
    data::{economy::OrderKind, world::SiteId},
    state::construction::{ConstructionKind, ConstructionTarget, Focus},
};
use macroquad_toolkit::ui::{prepare_font_text, truncate_text_to_width_ex};

const X: f32 = 16.0;
const W: f32 = 326.0;
const PAGE_SIZE: usize = crate::ui::MAP_CONTEXT_PAGE_SIZE;

pub(super) fn prepare_text(ctx: &Context<'_>) {
    let mut strings = Vec::new();
    strings.push(ctx.settlement.status.as_str());
    strings.extend(ctx.settlement.options.iter().flat_map(|choice| {
        [
            choice.option.blocked.as_deref().unwrap_or(""),
            effect_key(choice.option.kind),
        ]
    }));
    strings.extend(ctx.settlement.focuses.iter().flat_map(|choice| {
        [
            choice.blocked.as_deref().unwrap_or(""),
            focus_effect_key(choice.focus),
        ]
    }));
    strings.extend(ctx.settlement.builders.iter().map(|choice| {
        choice
            .blocked
            .as_deref()
            .unwrap_or(choice.location.as_str())
    }));
    if let (Some(site), Some(campaign)) = (ctx.settlement.site, ctx.campaign_view) {
        if let Some(location) = campaign.world.site(site) {
            strings.push(location.name.as_str());
        }
    }
    if let Some(font) = ctx.body_font() {
        let samples: Vec<_> = strings
            .into_iter()
            .flat_map(|value| [(16, value), (18, value), (20, value)])
            .collect();
        prepare_font_text(font, &samples);
    }
}

pub(super) fn draw(
    ctx: &Context<'_>,
    rect: Rect,
    site: SiteId,
    fallback_name: &str,
) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let name = campaign
        .world
        .site(site)
        .map(|location| location.name.as_str())
        .unwrap_or(fallback_name);
    let mode = ctx.settlement.mode;
    let mode_key = match mode {
        SettlementMode::Focus => "settlement_focus",
        SettlementMode::Review => "construction_review",
        SettlementMode::Builders => "construction_choose_builder",
        SettlementMode::Roads => "settlement_roads",
        _ => "settlement_build",
    };
    shell(ctx, rect, name, mode_key);
    if let Some(action) = tabs(ctx, rect, site, mode) {
        return Some(action);
    }
    match mode {
        SettlementMode::Build | SettlementMode::Roads => draw_construction(ctx, rect, mode),
        SettlementMode::Review => draw_review(ctx, rect),
        SettlementMode::Builders => draw_builders(ctx, rect),
        SettlementMode::Focus => draw_focus(ctx, rect),
        _ => Some(UiAction::SettlementBack),
    }
}

fn tabs(ctx: &Context<'_>, rect: Rect, site: SiteId, mode: SettlementMode) -> Option<UiAction> {
    let active = ctx
        .campaign_view
        .is_some_and(|campaign| campaign.player_turn);
    for (index, (next, key)) in [
        (SettlementMode::Build, "settlement_build"),
        (SettlementMode::Roads, "settlement_roads"),
        (SettlementMode::Focus, "settlement_focus"),
    ]
    .into_iter()
    .enumerate()
    {
        if button(
            ctx,
            Rect::new(
                rect.x + X + index as f32 * 110.0,
                rect.y + 82.0,
                104.0,
                42.0,
            ),
            &ctx.text(key),
            active,
            mode == next,
        ) {
            return Some(UiAction::ContextSettlement(site, next));
        }
    }
    None
}

fn draw_construction(ctx: &Context<'_>, rect: Rect, mode: SettlementMode) -> Option<UiAction> {
    let routes = mode == SettlementMode::Roads;
    let choices: Vec<_> = ctx
        .settlement
        .options
        .iter()
        .filter(|choice| matches!(choice.target, ConstructionTarget::Route(_)) == routes)
        .collect();
    let pages = choices.len().div_ceil(PAGE_SIZE).max(1);
    let page = ctx.settlement.page.min(pages - 1);
    let start = page * PAGE_SIZE;
    for (index, choice) in choices.iter().skip(start).take(PAGE_SIZE).enumerate() {
        let card = Rect::new(rect.x + X, rect.y + 138.0 + index as f32 * 120.0, W, 116.0);
        draw_build_choice(ctx, card, choice);
        if tapped(ctx, card) {
            return Some(UiAction::SelectConstruction(
                choice.target,
                choice.option.kind,
            ));
        }
    }
    if choices.is_empty() {
        compact_lines(
            ctx,
            &ctx.text(if routes {
                "construction_no_routes"
            } else {
                "construction_no_orders"
            }),
            vec2(rect.x + X, rect.y + 170.0),
            CompactStyle {
                width: W,
                max_lines: 3,
                color: MUTED,
                size: 18.0,
            },
        );
    }
    page_footer(ctx, rect, page, pages)
}

fn draw_build_choice(ctx: &Context<'_>, rect: Rect, choice: &BuildChoice) {
    card_frame(rect, false);
    let title = if matches!(choice.target, ConstructionTarget::Route(_)) {
        format!(
            "{} · {}",
            kind_name(ctx, choice.option.kind),
            target_name(ctx, choice.target)
        )
    } else {
        kind_name(ctx, choice.option.kind)
    };
    body(
        ctx,
        &truncate_text_to_width_ex(&title, W - 16.0, ctx.body_font(), 20.0),
        vec2(rect.x + 8.0, rect.y + 22.0),
        20.0,
        CREAM,
    );
    body(
        ctx,
        &resources(ctx, choice.option.cost),
        vec2(rect.x + 8.0, rect.y + 44.0),
        18.0,
        BRASS,
    );
    body(
        ctx,
        &format!("{} {}", choice.option.steps, ctx.text("construction_steps")),
        vec2(rect.x + 8.0, rect.y + 65.0),
        17.0,
        MUTED,
    );
    let detail = choice
        .option
        .blocked
        .as_deref()
        .map(str::to_owned)
        .unwrap_or_else(|| ctx.text(effect_key(choice.option.kind)));
    compact_lines(
        ctx,
        &detail,
        vec2(rect.x + 8.0, rect.y + 86.0),
        CompactStyle {
            width: W - 16.0,
            max_lines: 2,
            color: if choice.option.blocked.is_some() {
                BRASS
            } else {
                MUTED
            },
            size: 17.0,
        },
    );
}

fn draw_review(ctx: &Context<'_>, rect: Rect) -> Option<UiAction> {
    let Some((target, kind)) = ctx.settlement.choice else {
        return Some(UiAction::SettlementBack);
    };
    let Some(choice) = ctx
        .settlement
        .options
        .iter()
        .find(|choice| choice.target == target && choice.option.kind == kind)
    else {
        return Some(UiAction::SettlementBack);
    };
    let blocked = ctx
        .settlement
        .blocked
        .as_ref()
        .or(choice.option.blocked.as_ref());
    let reason = blocked
        .cloned()
        .unwrap_or_else(|| ctx.text(effect_key(kind)));
    draw_review_summary(ctx, rect, target, kind, choice, &reason, blocked.is_some());
    let can_act = ctx
        .campaign_view
        .is_some_and(|campaign| campaign.player_turn);
    draw_review_controls(
        ctx,
        rect,
        can_act && ctx.settlement.builder.is_some() && blocked.is_none(),
    )
}

fn draw_review_summary(
    ctx: &Context<'_>,
    rect: Rect,
    target: ConstructionTarget,
    kind: ConstructionKind,
    choice: &BuildChoice,
    reason: &str,
    blocked: bool,
) {
    body(
        ctx,
        &kind_name(ctx, kind),
        vec2(rect.x + X, rect.y + 150.0),
        20.0,
        CREAM,
    );
    compact_lines(
        ctx,
        &target_name(ctx, target),
        vec2(rect.x + X, rect.y + 181.0),
        CompactStyle {
            width: W,
            max_lines: 2,
            color: MUTED,
            size: 18.0,
        },
    );
    body(
        ctx,
        &ctx.text("construction_prepaid"),
        vec2(rect.x + X, rect.y + 225.0),
        18.0,
        BRASS,
    );
    body(
        ctx,
        &resources(ctx, choice.option.cost),
        vec2(rect.x + X, rect.y + 248.0),
        18.0,
        BRASS,
    );
    body(
        ctx,
        &format!("{} {}", choice.option.steps, ctx.text("construction_steps")),
        vec2(rect.x + X, rect.y + 276.0),
        18.0,
        MUTED,
    );
    let builder_name = ctx
        .settlement
        .builder
        .and_then(|id| ctx.campaign_view?.armies.iter().find(|army| army.id == id))
        .map(|army| army.name.clone())
        .unwrap_or_else(|| ctx.text("construction_no_builder"));
    body(
        ctx,
        &truncate_text_to_width_ex(
            &format!("{}: {builder_name}", ctx.text("construction_builder")),
            W,
            ctx.body_font(),
            18.0,
        ),
        vec2(rect.x + X, rect.y + 308.0),
        18.0,
        CREAM,
    );
    compact_lines(
        ctx,
        reason,
        vec2(rect.x + X, rect.y + 340.0),
        CompactStyle {
            width: W,
            max_lines: 6,
            color: if blocked { BRASS } else { MUTED },
            size: 18.0,
        },
    );
}

fn draw_review_controls(ctx: &Context<'_>, rect: Rect, can_confirm: bool) -> Option<UiAction> {
    let action_y = rect.y + 476.0;
    if button(
        ctx,
        Rect::new(rect.x + X, action_y, 100.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::SettlementBack);
    }
    if button(
        ctx,
        Rect::new(rect.x + X + 108.0, action_y, 218.0, 48.0),
        &ctx.text("construction_choose_builder"),
        true,
        false,
    ) {
        return Some(UiAction::ChooseBuilder);
    }
    if button(
        ctx,
        Rect::new(rect.x + X, rect.y + 552.0, W, 48.0),
        &ctx.text("construction_confirm"),
        can_confirm,
        true,
    ) {
        return Some(UiAction::ConfirmConstruction);
    }
    None
}

fn draw_builders(ctx: &Context<'_>, rect: Rect) -> Option<UiAction> {
    let pages = ctx.settlement.builders.len().div_ceil(PAGE_SIZE).max(1);
    let page = ctx.settlement.page.min(pages - 1);
    for (index, choice) in ctx
        .settlement
        .builders
        .iter()
        .skip(page * PAGE_SIZE)
        .take(PAGE_SIZE)
        .enumerate()
    {
        let card = Rect::new(rect.x + X, rect.y + 138.0 + index as f32 * 120.0, W, 116.0);
        draw_builder(ctx, card, choice);
        if tapped(ctx, card) && choice.blocked.is_none() {
            return Some(UiAction::SelectBuilder(choice.id));
        }
    }
    if ctx.settlement.builders.is_empty() {
        compact_lines(
            ctx,
            &ctx.text("construction_no_builders"),
            vec2(rect.x + X, rect.y + 170.0),
            CompactStyle {
                width: W,
                max_lines: 4,
                color: BRASS,
                size: 18.0,
            },
        );
    }
    page_footer(ctx, rect, page, pages)
}

fn draw_builder(ctx: &Context<'_>, rect: Rect, choice: &BuilderChoice) {
    card_frame(rect, Some(choice.id) == ctx.settlement.builder);
    body(
        ctx,
        &truncate_text_to_width_ex(&choice.name, W - 16.0, ctx.body_font(), 20.0),
        vec2(rect.x + 8.0, rect.y + 23.0),
        20.0,
        CREAM,
    );
    compact_lines(
        ctx,
        choice.blocked.as_deref().unwrap_or(&choice.location),
        vec2(rect.x + 8.0, rect.y + 53.0),
        CompactStyle {
            width: W - 16.0,
            max_lines: 3,
            color: if choice.blocked.is_some() {
                BRASS
            } else {
                MUTED
            },
            size: 18.0,
        },
    );
}

fn draw_focus(ctx: &Context<'_>, rect: Rect) -> Option<UiAction> {
    for (index, choice) in ctx.settlement.focuses.iter().enumerate() {
        let column = index % 2;
        let row = index / 2;
        let card = Rect::new(
            rect.x + X + column as f32 * 166.0,
            rect.y + 138.0 + row as f32 * 68.0,
            160.0,
            60.0,
        );
        card_frame(card, Some(choice.focus) == ctx.settlement.focus);
        body(
            ctx,
            &truncate_text_to_width_ex(
                &ctx.text(focus_key(choice.focus)),
                card.w - 14.0,
                ctx.body_font(),
                18.0,
            ),
            vec2(card.x + 7.0, card.y + 36.0),
            18.0,
            CREAM,
        );
        if tapped(ctx, card) {
            return Some(UiAction::SelectFocus(choice.focus));
        }
    }
    let cost = ctx.economy.orders[&OrderKind::ChangeFocus].cost;
    body(
        ctx,
        &ctx.text("construction_prepaid"),
        vec2(rect.x + X, rect.y + 348.0),
        18.0,
        BRASS,
    );
    body(
        ctx,
        &resources(ctx, cost),
        vec2(rect.x + X, rect.y + 372.0),
        18.0,
        BRASS,
    );
    let selected = ctx
        .settlement
        .focuses
        .iter()
        .find(|choice| Some(choice.focus) == ctx.settlement.focus);
    if let Some(choice) = selected {
        let detail = choice
            .blocked
            .clone()
            .unwrap_or_else(|| ctx.text(focus_effect_key(choice.focus)));
        compact_lines(
            ctx,
            &detail,
            vec2(rect.x + X, rect.y + 402.0),
            CompactStyle {
                width: W,
                max_lines: 6,
                color: if choice.blocked.is_some() {
                    BRASS
                } else {
                    MUTED
                },
                size: 18.0,
            },
        );
    }
    let can_act = ctx
        .campaign_view
        .is_some_and(|campaign| campaign.player_turn);
    if button(
        ctx,
        Rect::new(rect.x + X, rect.y + 552.0, 146.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::SettlementBack);
    }
    if button(
        ctx,
        Rect::new(rect.x + X + 160.0, rect.y + 552.0, 166.0, 48.0),
        &ctx.text("focus_confirm"),
        can_act && selected.is_some_and(|choice| choice.blocked.is_none()),
        true,
    ) {
        return Some(UiAction::ConfirmFocus);
    }
    None
}

fn page_footer(ctx: &Context<'_>, rect: Rect, page: usize, pages: usize) -> Option<UiAction> {
    let can_act = ctx
        .campaign_view
        .is_some_and(|campaign| campaign.player_turn);
    let y = rect.y + 552.0;
    if button(
        ctx,
        Rect::new(rect.x + X, y, 100.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::SettlementBack);
    }
    if pages > 1 {
        if button(
            ctx,
            Rect::new(rect.x + X + 109.0, y, 100.0, 48.0),
            &ctx.text("previous"),
            can_act && page > 0,
            false,
        ) {
            return Some(UiAction::SettlementPage(-1));
        }
        if button(
            ctx,
            Rect::new(rect.x + X + 218.0, y, 108.0, 48.0),
            &ctx.text("next"),
            can_act && page + 1 < pages,
            false,
        ) {
            return Some(UiAction::SettlementPage(1));
        }
    }
    None
}

fn target_name(ctx: &Context<'_>, target: ConstructionTarget) -> String {
    let Some(world) = ctx.campaign_view.map(|campaign| &campaign.world) else {
        return String::new();
    };
    let name = |site| {
        world
            .site(site)
            .map(|site| site.name.as_str())
            .unwrap_or("")
    };
    match target {
        ConstructionTarget::Site(site) => name(site).to_owned(),
        ConstructionTarget::Route(route) => world
            .route(route)
            .map(|route| format!("{} — {}", name(route.from), name(route.to)))
            .unwrap_or_default(),
    }
}

fn effect_key(kind: ConstructionKind) -> &'static str {
    match kind {
        ConstructionKind::Outpost => "construction_outpost_effect",
        ConstructionKind::Road => "construction_road_effect",
        ConstructionKind::RoadRepair => "construction_repair_effect",
        ConstructionKind::Fort => "construction_fort_effect",
        ConstructionKind::Facility(facility) => match facility {
            kestrum::data::world::Facility::TrainingGround => "facility_training_effect",
            kestrum::data::world::Facility::Stable => "facility_stable_effect",
            kestrum::data::world::Facility::Infirmary => "facility_infirmary_effect",
            kestrum::data::world::Facility::Workshop => "facility_workshop_effect",
            kestrum::data::world::Facility::Temple => "facility_temple_effect",
        },
    }
}

fn focus_effect_key(focus: Focus) -> &'static str {
    match focus {
        Focus::Growth => "focus_effect_growth",
        Focus::Fortification => "focus_effect_fortification",
        Focus::TroopTraining => "focus_effect_training",
        Focus::Gold => "focus_effect_gold",
        Focus::Wood => "focus_effect_wood",
        Focus::Stone => "focus_effect_stone",
    }
}
