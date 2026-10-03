//! Full-bleed terrain and sparse campaign controls anchored to its edges.

use super::{components::*, selection, world, Context, UiAction};
use kestrum::{
    navigation::{MapNavigation, MapScaleBand, MapScope, MapView, HEIGHT, WIDTH},
    state::{world::CampaignWorld, Overlay, Screen},
};
use macroquad::prelude::*;

const MENU: Rect = Rect::new(WIDTH - 132.0, 24.0, 108.0, 48.0);
const ZOOM_OUT: Rect = Rect::new(24.0, HEIGHT - 72.0, 48.0, 48.0);
const ZOOM_IN: Rect = Rect::new(80.0, HEIGHT - 72.0, 48.0, 48.0);
const OVERVIEW: Rect = Rect::new(144.0, HEIGHT - 72.0, 148.0, 48.0);
const RECENTER: Rect = Rect::new(304.0, HEIGHT - 72.0, 148.0, 48.0);
const MAP_KEY: Rect = Rect::new(464.0, HEIGHT - 72.0, 132.0, 48.0);
const END_TURN: Rect = Rect::new(WIDTH - 208.0, HEIGHT - 72.0, 184.0, 48.0);
const STEP_NPC: Rect = Rect::new(WIDTH - 326.0, HEIGHT - 72.0, 108.0, 48.0);
const KINGDOM_DECISION: Rect = Rect::new(WIDTH - 352.0, HEIGHT - 72.0, 328.0, 48.0);

pub fn map_controls_contain(
    point: Vec2,
    navigation: &MapNavigation,
    campaign_world: Option<&CampaignWorld>,
    view: &MapView,
    movement: &super::MoveView,
    hide_inspector: bool,
) -> bool {
    [
        MENU,
        ZOOM_OUT,
        ZOOM_IN,
        OVERVIEW,
        RECENTER,
        MAP_KEY,
        END_TURN,
        STEP_NPC,
        KINGDOM_DECISION,
    ]
    .iter()
    .any(|rect| rect.contains(point))
        || (matches!(navigation.scope(), MapScope::Region(_)) && world::WORLD_MAP.contains(point))
        || (!hide_inspector
            && campaign_world
                .and_then(|world| {
                    if movement.stage == super::MoveStage::Map {
                        Some(super::movement_panel_bounds(
                            movement, navigation, world, view,
                        ))
                    } else {
                        selection::bounds(navigation, world, view)
                    }
                })
                .is_some_and(|rect| rect.contains(point)))
}

pub fn draw_landscape(ctx: &Context<'_>) {
    if let Some(texture) = ctx.assets.get_texture("atlas") {
        let campaign = ctx.state.screen == Screen::Campaign;
        let corner = if campaign {
            ctx.view.project(Vec2::ZERO)
        } else {
            Vec2::ZERO
        };
        let size = if campaign {
            ctx.view.extent() * ctx.view.camera.zoom()
        } else {
            vec2(WIDTH, HEIGHT)
        };
        draw_texture_ex(
            texture,
            corner.x,
            corner.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
    }
    if ctx.state.screen == Screen::Campaign {
        if matches!(ctx.navigation.scope(), MapScope::Region(_)) {
            draw_rectangle(
                0.0,
                0.0,
                WIDTH,
                HEIGHT,
                Color::new(INK.r, INK.g, INK.b, 0.62),
            );
        }
        if !ctx.preferences.hide_labels
            && ctx.campaign_view.is_some_and(|visible| {
                visible.world.markers.len() <= 24
                    && ctx
                        .state
                        .campaign
                        .as_ref()
                        .and_then(kestrum::state::Campaign::strategic)
                        .is_some_and(|campaign| {
                            visible.world.markers.len() == campaign.world.markers.len()
                        })
            })
        {
            geography(ctx);
        }
        let exploration = world::map_exploration(ctx);
        if let Some(area) = &exploration {
            world::draw_fog(ctx, area);
        }
        for row in 0..100 {
            let opacity = (1.0 - row as f32 / 100.0).powi(2) * 0.82;
            let shade = Color::new(INK.r, INK.g, INK.b, opacity);
            draw_rectangle(0.0, HEIGHT - row as f32 - 1.0, WIDTH, 1.0, shade);
        }
        world::draw(ctx, exploration.as_ref());
    }
}

fn geography(ctx: &Context<'_>) {
    for label in &ctx.data.geography {
        let Some(name) = ctx.game_text.geography.get(&label.id) else {
            continue;
        };
        let at = ctx.view.project_normalized(label.position);
        if !(100.0..HEIGHT - 160.0).contains(&at.y) {
            continue;
        }
        let width = measure_text(name, ctx.font(), label.size as u16, 1.0).width;
        if at.x - width * 0.5 < 18.0 || at.x + width * 0.5 > WIDTH - 18.0 {
            continue;
        }
        if ctx.preferences.high_contrast {
            draw_rectangle(
                at.x - width * 0.5 - 10.0,
                at.y - label.size - 4.0,
                width + 20.0,
                label.size + 14.0,
                Color::new(0.94, 0.89, 0.72, 0.92),
            );
        }
        for offset in [
            vec2(-1.0, 0.0),
            vec2(1.0, 0.0),
            vec2(0.0, -1.0),
            vec2(0.0, 1.0),
        ] {
            centered(ctx, name, at + offset, label.size, CREAM);
        }
        centered(ctx, name, at, label.size, INK);
    }
}

pub fn hud(ctx: &Context<'_>) -> Option<UiAction> {
    let active = ctx.state.overlay == Overlay::None;
    // Paint after the map layers so bright peaks and political ink cannot
    // wash out the header. The fade keeps the landscape continuous below it.
    for row in 0..140 {
        let opacity = (1.0 - (row as f32 / 140.0).powi(2)) * 0.88;
        draw_rectangle(
            0.0,
            row as f32,
            WIDTH,
            1.0,
            Color::new(INK.r, INK.g, INK.b, opacity),
        );
    }
    let overview_action = super::overview::draw(ctx);
    if ctx.navigation.scope() == MapScope::World {
        emblem(vec2(44.0, 43.0), 19.0);
        text(ctx, &ctx.text("world_map"), vec2(80.0, 40.0), 24.0, CREAM);
        body(ctx, &ctx.game_text.title, vec2(81.0, 60.0), 16.0, BRASS);
    }
    if let Some(campaign) = ctx
        .state
        .campaign
        .as_ref()
        .filter(|_| super::tutorial_bounds(ctx.state).is_none())
    {
        let season = &ctx.game_text.seasons[campaign.season_index()];
        centered(
            ctx,
            &format!(
                "{season}  /  {} {}",
                ctx.text("year"),
                campaign.year(ctx.data.start_year)
            ),
            vec2(WIDTH * 0.5, 36.0),
            21.0,
            CREAM,
        );
        let status = if let Some(view) = ctx.campaign_view {
            format!(
                "{} {}   ·   {}",
                ctx.text("round"),
                campaign.display_turn(),
                if view
                    .factions
                    .iter()
                    .any(|faction| faction.id == view.active_faction)
                {
                    view.active_faction_name.clone()
                } else {
                    ctx.text("map_unknown_kingdom")
                }
            )
        } else {
            ctx.text("legacy_phase")
        };
        let fitted =
            macroquad_toolkit::ui::truncate_text_to_width_ex(&status, 740.0, ctx.body_font(), 18.0);
        let width = measure_text(&fitted, ctx.body_font(), 18, 1.0).width;
        body(
            ctx,
            &fitted,
            vec2(WIDTH * 0.5 - width * 0.5, 59.0),
            18.0,
            CREAM,
        );
    }
    if ctx.navigation.selection().is_none() && !ctx.overview_ui.expanded {
        compass(ctx);
    }
    // A narrow dark wash preserves contrast without reserving a panel for the map.
    draw_rectangle(
        18.0,
        HEIGHT - 78.0,
        816.0,
        60.0,
        Color::new(INK.r, INK.g, INK.b, 0.72),
    );
    for (rect, key, intent) in [
        (MENU, "menu", UiAction::Open(Overlay::Menu)),
        (ZOOM_OUT, "zoom_out", UiAction::Zoom(1.0 / 1.25)),
        (ZOOM_IN, "zoom_in", UiAction::Zoom(1.25)),
        (
            OVERVIEW,
            if ctx.view.overview_active() {
                "map_return_view"
            } else {
                "map_overview"
            },
            UiAction::Overview,
        ),
        (RECENTER, "reset_view", UiAction::Recenter),
        (MAP_KEY, "map_key", UiAction::MapKey),
    ] {
        if button(ctx, rect, &ctx.text(key), active, key == "end_turn") {
            return Some(intent);
        }
    }
    let band = match ctx.view.band() {
        MapScaleBand::Overview => "map_band_overview",
        MapScaleBand::Campaign => "map_band_campaign",
        MapScaleBand::Detail => "map_band_detail",
    };
    body(
        ctx,
        &ctx.text(band),
        vec2(616.0, HEIGHT - 40.0),
        18.0,
        CREAM,
    );
    let moving = ctx.movement.stage == super::MoveStage::Map;
    let phase_action = phase_controls(ctx, active);
    let navigation_action = world::navigation(ctx);
    let selection_action = if super::is_open_for_map(ctx) {
        None
    } else if moving {
        super::draw_move_map_overlay(ctx)
    } else {
        selection::draw(ctx)
    };
    selection_action
        .or(navigation_action)
        .or(phase_action)
        .or(overview_action)
}

fn phase_controls(ctx: &Context<'_>, active: bool) -> Option<UiAction> {
    let view = ctx.campaign_view?;
    if ctx
        .kingdom
        .data
        .as_ref()
        .is_some_and(|view| view.ending.is_some())
    {
        return None;
    }
    if ctx
        .kingdom
        .data
        .as_ref()
        .is_some_and(|view| !view.incoming_offers.is_empty() || !view.pending_defeats.is_empty())
    {
        return button(
            ctx,
            KINGDOM_DECISION,
            &ctx.text("kingdom_decision"),
            active,
            true,
        )
        .then_some(UiAction::OpenKingdom(None));
    }
    let (key, action) = if view.player_turn {
        ("end_turn", UiAction::EndTurn)
    } else if view.npc_paused {
        ("resume_npcs", UiAction::PauseNpcs(false))
    } else {
        ("pause_npcs", UiAction::PauseNpcs(true))
    };
    if !view.player_turn {
        draw_rectangle(
            860.0,
            HEIGHT - 78.0,
            680.0,
            60.0,
            macroquad_toolkit::colors::with_alpha(INK, 0.88),
        );
        let phase = if view.npc_paused {
            "npc_paused"
        } else {
            "npc_phase"
        };
        body(
            ctx,
            &ctx.text(phase),
            vec2(878.0, HEIGHT - 55.0),
            18.0,
            CREAM,
        );
        body(
            ctx,
            &ctx.text("npc_prototype"),
            vec2(878.0, HEIGHT - 32.0),
            16.0,
            CREAM,
        );
        if button(
            ctx,
            STEP_NPC,
            &ctx.text("step_npc"),
            active && view.npc_paused,
            false,
        ) {
            return Some(UiAction::StepNpc);
        }
    }
    if button(ctx, END_TURN, &ctx.text(key), active, true) {
        return Some(action);
    }
    None
}

fn compass(ctx: &Context<'_>) {
    let center = vec2(WIDTH - 68.0, HEIGHT - 216.0);
    draw_circle_lines(center.x, center.y, 22.0, 1.0, CREAM);
    draw_triangle(
        center + vec2(0.0, -31.0),
        center + vec2(-6.0, 8.0),
        center + vec2(6.0, 8.0),
        INK,
    );
    draw_triangle(
        center + vec2(0.0, 31.0),
        center + vec2(-5.0, -8.0),
        center + vec2(5.0, -8.0),
        CREAM,
    );
    centered(ctx, "N", center + vec2(0.0, -39.0), 18.0, INK);
}
