//! Compact recruitment choices stay beside the selected site or army.

use super::*;
use crate::ui::components::{body, button};
use kestrum::{data::economy::TroopKind, engine::RecruitOption};
use macroquad_toolkit::ui::{prepare_font_text, truncate_text_to_width_ex};

const X: f32 = 16.0;
const W: f32 = 326.0;
const PAGE_SIZE: usize = crate::ui::MAP_CONTEXT_PAGE_SIZE;

pub(super) fn prepare_text(ctx: &Context<'_>) {
    let mut strings = vec![ctx.army.status.clone()];
    strings.extend(ctx.army.options.iter().flat_map(|option| {
        [
            option.blocked.clone().unwrap_or_default(),
            ctx.text(crate::ui::army::troop_key(option.kind)),
        ]
    }));
    if let Some(font) = ctx.body_font() {
        let samples: Vec<_> = strings
            .iter()
            .flat_map(|value| {
                [
                    (16, value.as_str()),
                    (18, value.as_str()),
                    (20, value.as_str()),
                ]
            })
            .collect();
        prepare_font_text(font, &samples);
    }
}

pub(super) fn draw(
    ctx: &Context<'_>,
    rect: Rect,
    site: kestrum::data::world::SiteId,
    fallback_name: &str,
) -> Option<UiAction> {
    let ArmyMode::Recruit { army, kind } = ctx.army.mode else {
        return None;
    };
    let campaign = ctx.campaign_view?;
    let name = campaign
        .world
        .site(site)
        .map(|location| location.name.as_str())
        .unwrap_or(fallback_name);
    shell(ctx, rect, name, "recruit");
    let can_act = campaign.player_turn && !campaign.observer_mode;
    if let Some(action) = target_control(ctx, rect, site, army, can_act) {
        return Some(action);
    }
    if !ctx.army.status.is_empty() {
        compact_lines(
            ctx,
            &ctx.army.status,
            vec2(rect.x + X, rect.y + 140.0),
            CompactStyle {
                width: W,
                max_lines: 2,
                color: CREAM,
                size: 16.0,
            },
        );
    }

    if let Some(action) = draw_options(ctx, rect, kind) {
        return Some(action);
    }
    let pages = ctx.army.options.len().div_ceil(PAGE_SIZE).max(1);
    if let Some(action) = recruit_paging(ctx, rect, pages, can_act) {
        return Some(action);
    }
    recruit_footer(ctx, rect, kind, can_act)
}

fn target_control(
    ctx: &Context<'_>,
    rect: Rect,
    site: kestrum::data::world::SiteId,
    army: Option<kestrum::state::military::ArmyId>,
    can_act: bool,
) -> Option<UiAction> {
    let campaign = ctx.campaign_view?;
    let targets: Vec<_> = campaign
        .armies
        .iter()
        .filter(|candidate| candidate.faction == campaign.player && candidate.site == site)
        .collect();
    let next_target = match army {
        None => targets.first().map(|candidate| candidate.id),
        Some(current) => targets
            .iter()
            .position(|candidate| candidate.id == current)
            .and_then(|index| targets.get(index + 1).map(|candidate| candidate.id)),
    };
    let new_army = ctx.text("new_army");
    let target_name = army
        .and_then(|id| targets.iter().find(|candidate| candidate.id == id))
        .map(|candidate| candidate.name.as_str())
        .unwrap_or(new_army.as_str());
    let target_label = truncate_text_to_width_ex(
        &format!("{}: {target_name}  ›", ctx.text("armies")),
        W - 24.0,
        ctx.font(),
        21.0,
    );
    button(
        ctx,
        Rect::new(rect.x + X, rect.y + 82.0, W, 42.0),
        &target_label,
        can_act && !targets.is_empty(),
        false,
    )
    .then_some(UiAction::BeginRecruit(next_target))
}

fn draw_options(ctx: &Context<'_>, rect: Rect, kind: Option<TroopKind>) -> Option<UiAction> {
    let pages = ctx.army.options.len().div_ceil(PAGE_SIZE).max(1);
    let page = ctx.army.recruit_page.min(pages - 1);
    for (index, option) in ctx
        .army
        .options
        .iter()
        .skip(page * PAGE_SIZE)
        .take(PAGE_SIZE)
        .enumerate()
    {
        let card = Rect::new(rect.x + X, rect.y + 164.0 + index as f32 * 112.0, W, 108.0);
        draw_option(ctx, card, option, kind == Some(option.kind));
        if tapped(ctx, card) {
            return Some(UiAction::SelectRecruit(option.kind));
        }
    }
    None
}

fn recruit_paging(ctx: &Context<'_>, rect: Rect, pages: usize, can_act: bool) -> Option<UiAction> {
    let page = ctx.army.recruit_page.min(pages - 1);
    if pages <= 1 {
        return None;
    }
    let y = rect.y + 504.0;
    if button(
        ctx,
        Rect::new(rect.x + X, y, 100.0, 42.0),
        &ctx.text("previous"),
        can_act && page > 0,
        false,
    ) {
        return Some(UiAction::ContextRecruitPage(-1));
    }
    body(
        ctx,
        &format!("{} / {}", page + 1, pages),
        vec2(rect.x + X + 138.0, y + 27.0),
        17.0,
        MUTED,
    );
    button(
        ctx,
        Rect::new(rect.x + X + 226.0, y, 100.0, 42.0),
        &ctx.text("next"),
        can_act && page + 1 < pages,
        false,
    )
    .then_some(UiAction::ContextRecruitPage(1))
}

fn recruit_footer(
    ctx: &Context<'_>,
    rect: Rect,
    kind: Option<TroopKind>,
    can_act: bool,
) -> Option<UiAction> {
    if button(
        ctx,
        Rect::new(rect.x + X, rect.y + 552.0, 146.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::CancelArmyAction);
    }
    let selected = kind.and_then(|kind| ctx.army.options.iter().find(|option| option.kind == kind));
    if button(
        ctx,
        Rect::new(rect.x + X + 160.0, rect.y + 552.0, 166.0, 48.0),
        &ctx.text("recruit"),
        can_act && selected.is_some_and(|option| option.blocked.is_none()),
        true,
    ) {
        return Some(UiAction::ConfirmRecruit);
    }
    None
}

fn draw_option(ctx: &Context<'_>, rect: Rect, option: &RecruitOption, selected: bool) {
    card_frame(rect, selected);
    let name = truncate_text_to_width_ex(
        &ctx.text(crate::ui::army::troop_key(option.kind)),
        rect.w - 16.0,
        ctx.body_font(),
        20.0,
    );
    body(ctx, &name, vec2(rect.x + 8.0, rect.y + 22.0), 20.0, CREAM);
    body(
        ctx,
        &resources(ctx, option.cost),
        vec2(rect.x + 8.0, rect.y + 45.0),
        18.0,
        BRASS,
    );
    let unit = if option.kind == TroopKind::SiegeEngines {
        ctx.text("engine_teams")
    } else {
        ctx.text("troops")
    };
    body(
        ctx,
        &format!(
            "{} {unit} · {} {} {}",
            option.capacity,
            option.upkeep_gold,
            ctx.text("gold"),
            ctx.text("upkeep")
        ),
        vec2(rect.x + 8.0, rect.y + 66.0),
        16.0,
        MUTED,
    );
    let reason = option
        .blocked
        .clone()
        .unwrap_or_else(|| ctx.text("recruit_ready"));
    compact_lines(
        ctx,
        &reason,
        vec2(rect.x + 8.0, rect.y + 84.0),
        CompactStyle {
            width: rect.w - 16.0,
            max_lines: 2,
            color: if option.blocked.is_some() {
                BRASS
            } else {
                MUTED
            },
            size: 18.0,
        },
    );
}
