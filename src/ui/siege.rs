//! Siege decisions use own participants and observer-safe rule previews.

mod choices;

use super::{components::*, Context, UiAction};
use kestrum::{
    data::world::SiteId,
    engine::{SiegeRole, SiegeView},
    state::{military::ArmyId, siege::SiegeAction},
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};
use std::collections::BTreeMap;

pub const SIEGE_PAGE_SIZE: usize = 5;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SiegeMode {
    #[default]
    Forces,
    Orders,
    Exits,
    Review,
}

#[derive(Debug)]
pub struct SiegeExit {
    pub site: SiteId,
    pub name: String,
    pub blocked: Option<String>,
}

#[derive(Debug, Default)]
pub struct SiegePanel {
    pub site: Option<SiteId>,
    pub mode: SiegeMode,
    pub selected: Vec<ArmyId>,
    pub action: Option<SiegeAction>,
    pub destination: Option<SiteId>,
    pub page: usize,
    pub view: Option<SiegeView>,
    pub remaining: BTreeMap<ArmyId, u32>,
    pub exits: Vec<SiegeExit>,
    pub blocked: Option<String>,
    pub status: String,
}

pub fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(80.0, 38.0, 1120.0, 650.0, INK);
    heading(ctx);
    let action = if ctx.siege.view.is_some() {
        match ctx.siege.mode {
            SiegeMode::Forces => choices::forces(ctx),
            SiegeMode::Orders => choices::orders(ctx),
            SiegeMode::Exits => choices::exits(ctx),
            SiegeMode::Review => choices::review(ctx),
        }
    } else {
        lines(
            ctx,
            &ctx.text("siege_ended"),
            vec2(112.0, 235.0),
            1056.0,
            3,
            CREAM,
        );
        None
    };
    lines(ctx, &ctx.siege.status, vec2(112.0, 594.0), 1056.0, 1, BRASS);
    action.or_else(|| {
        control(
            ctx,
            Rect::new(112.0, 626.0, 166.0, 48.0),
            "back",
            true,
            false,
        )
        .then_some(UiAction::SiegeBack)
    })
}

fn heading(ctx: &Context<'_>) {
    let name = site_name(ctx, ctx.siege.site);
    let title = format!("{} / {name}", ctx.text("siege"));
    text(
        ctx,
        &truncate_text_to_width_ex(&title, 1056.0, ctx.font(), 28.0),
        vec2(112.0, 83.0),
        28.0,
        CREAM,
    );
    if let Some(view) = &ctx.siege.view {
        let role = ctx.text(match view.role {
            SiegeRole::Defender => "siege_defending",
            SiegeRole::Besieger => "siege_besieging",
            SiegeRole::Observer => "observer_siege_observing",
        });
        let summary = format!(
            "{role} · {}: {} · {}: {}% · {}: {:.2}×",
            ctx.text("siege_elapsed"),
            view.elapsed_steps,
            ctx.text("siege_fort_damage"),
            view.fort_damage,
            ctx.text("siege_walls"),
            view.wall_permille as f32 / 1000.0
        );
        lines(ctx, &summary, vec2(112.0, 123.0), 1056.0, 2, BRASS);
        if view.role != SiegeRole::Observer {
            lines(
                ctx,
                &ctx.text("siege_enemy_unknown"),
                vec2(112.0, 176.0),
                1056.0,
                2,
                MUTED,
            );
        }
    }
}

fn site_name(ctx: &Context<'_>, id: Option<SiteId>) -> String {
    ctx.campaign_view
        .and_then(|view| view.world.site(id?))
        .map(|site| site.name.clone())
        .unwrap_or_default()
}

fn control(ctx: &Context<'_>, rect: Rect, key: &str, enabled: bool, primary: bool) -> bool {
    button(ctx, rect, &ctx.text(key), enabled, primary)
}

fn lines(ctx: &Context<'_>, value: &str, at: Vec2, width: f32, count: usize, color: Color) {
    for (index, line) in wrap_text_ex(value, width, ctx.body_font(), 18.0)
        .iter()
        .take(count)
        .enumerate()
    {
        body(ctx, line, at + vec2(0.0, index as f32 * 23.0), 18.0, color);
    }
}

pub fn action_key(action: SiegeAction) -> &'static str {
    match action {
        SiegeAction::Maintain => "siege_maintain",
        SiegeAction::Assault => "siege_assault",
        SiegeAction::Withdraw => "siege_withdraw",
        SiegeAction::Sortie => "siege_sortie",
        SiegeAction::Escape => "siege_escape",
    }
}

fn consequence_key(action: SiegeAction) -> &'static str {
    match action {
        SiegeAction::Maintain => "siege_maintain_help",
        SiegeAction::Assault => "siege_assault_help",
        SiegeAction::Withdraw => "siege_withdraw_help",
        SiegeAction::Sortie => "siege_sortie_help",
        SiegeAction::Escape => "siege_escape_help",
    }
}

fn pages(ctx: &Context<'_>, count: usize, y: f32) -> Option<UiAction> {
    let total = count.div_ceil(SIEGE_PAGE_SIZE).max(1);
    let page = ctx.siege.page.min(total - 1);
    centered(
        ctx,
        &format!("{} / {total}", page + 1),
        vec2(640.0, y + 31.0),
        20.0,
        CREAM,
    );
    for (x, delta, key, enabled) in [
        (112.0, -1, "previous", page > 0),
        (1008.0, 1, "next", page + 1 < total),
    ] {
        if control(ctx, Rect::new(x, y, 160.0, 48.0), key, enabled, false) {
            return Some(UiAction::SiegePage(delta));
        }
    }
    None
}

pub fn prepare_text(ctx: &Context<'_>) {
    let mut strings = vec![
        ctx.siege.status.clone(),
        site_name(ctx, ctx.siege.site),
        site_name(ctx, ctx.siege.destination),
    ];
    strings.extend(ctx.siege.blocked.iter().cloned());
    if let Some(view) = &ctx.siege.view {
        strings.extend(
            view.actions
                .iter()
                .filter_map(|option| option.blocked.clone()),
        );
        if let Some(campaign) = ctx.campaign_view {
            strings.extend(
                view.own_armies
                    .iter()
                    .skip(ctx.siege.page * SIEGE_PAGE_SIZE)
                    .take(SIEGE_PAGE_SIZE)
                    .filter_map(|id| {
                        campaign
                            .armies
                            .iter()
                            .find(|army| army.id == *id)
                            .map(|army| army.name.clone())
                    }),
            );
        }
    }
    for exit in ctx
        .siege
        .exits
        .iter()
        .skip(ctx.siege.page * SIEGE_PAGE_SIZE)
        .take(SIEGE_PAGE_SIZE)
    {
        strings.push(exit.name.clone());
        strings.extend(exit.blocked.iter().cloned());
    }
    if let Some(font) = ctx.body_font() {
        let samples: Vec<_> = strings
            .iter()
            .flat_map(|s| [(18, s.as_str()), (20, s.as_str())])
            .collect();
        macroquad_toolkit::ui::prepare_font_text(font, &samples);
    }
}
