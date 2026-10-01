//! Full-bleed terrain and sparse campaign controls anchored to its edges.

use super::{components::*, selection, world, Context, UiAction};
use kestrum::{
    navigation::{MapNavigation, MapScope, MapView, HEIGHT, WIDTH},
    state::{world::CampaignWorld, Overlay, Screen},
};
use macroquad::prelude::*;

const MENU: Rect = Rect::new(1148.0, 20.0, 108.0, 48.0);
const ZOOM_OUT: Rect = Rect::new(24.0, 646.0, 48.0, 48.0);
const ZOOM_IN: Rect = Rect::new(78.0, 646.0, 48.0, 48.0);
const RECENTER: Rect = Rect::new(138.0, 646.0, 144.0, 48.0);
const END_TURN: Rect = Rect::new(1072.0, 646.0, 184.0, 48.0);
const STEP_NPC: Rect = Rect::new(954.0, 646.0, 108.0, 48.0);
const KINGDOM_DECISION: Rect = Rect::new(928.0, 646.0, 328.0, 48.0);

pub fn map_controls_contain(
    point: Vec2,
    navigation: &MapNavigation,
    campaign_world: Option<&CampaignWorld>,
    view: &MapView,
    movement: &super::MoveView,
) -> bool {
    [
        MENU,
        ZOOM_OUT,
        ZOOM_IN,
        RECENTER,
        END_TURN,
        STEP_NPC,
        KINGDOM_DECISION,
    ]
    .iter()
    .any(|rect| rect.contains(point))
        || (matches!(navigation.scope(), MapScope::Region(_)) && world::WORLD_MAP.contains(point))
        || campaign_world
            .and_then(|world| {
                if movement.stage == super::MoveStage::Map {
                    Some(super::movement_panel_bounds(
                        movement, navigation, world, view,
                    ))
                } else {
                    selection::bounds(navigation, world, view)
                }
            })
            .is_some_and(|rect| rect.contains(point))
}

pub fn draw_landscape(ctx: &Context<'_>) {
    if let Some(texture) = ctx.assets.get_texture("atlas") {
        let corner = ctx.view.project(Vec2::ZERO);
        draw_texture_ex(
            texture,
            corner.x,
            corner.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(WIDTH, HEIGHT) * ctx.view.camera.zoom()),
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
            draw_rectangle(0.0, row as f32, WIDTH, 1.0, shade);
            draw_rectangle(0.0, HEIGHT - row as f32 - 1.0, WIDTH, 1.0, shade);
        }
        world::draw(ctx, exploration.as_ref());
    }
}

fn geography(ctx: &Context<'_>) {
    for label in &ctx.data.geography {
        let at = ctx
            .view
            .project(vec2(label.position[0] * WIDTH, label.position[1] * HEIGHT));
        if !(100.0..620.0).contains(&at.y) {
            continue;
        }
        let width = measure_text(&label.name, ctx.font(), label.size as u16, 1.0).width;
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
            centered(ctx, &label.name, at + offset, label.size, CREAM);
        }
        centered(ctx, &label.name, at, label.size, INK);
    }
}

pub fn hud(ctx: &Context<'_>) -> Option<UiAction> {
    let active = ctx.state.overlay == Overlay::None;
    if ctx.navigation.scope() == MapScope::World {
        emblem(vec2(44.0, 43.0), 19.0);
        text(ctx, &ctx.text("world_map"), vec2(80.0, 40.0), 24.0, CREAM);
        body(ctx, &ctx.data.title, vec2(81.0, 60.0), 16.0, BRASS);
    }
    if let Some(campaign) = &ctx.state.campaign {
        let season = &ctx.data.seasons[campaign.season_index()];
        centered(
            ctx,
            &format!(
                "{season}  /  {} {}",
                ctx.text("year"),
                campaign.year(ctx.data.start_year)
            ),
            vec2(640.0, 36.0),
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
        let width = measure_text(&status, ctx.body_font(), 18, 1.0).width;
        body(ctx, &status, vec2(640.0 - width * 0.5, 59.0), 18.0, CREAM);
    }
    if ctx.navigation.selection().is_none() {
        compass(ctx);
    }
    // A narrow dark wash preserves contrast without reserving a panel for the map.
    draw_rectangle(
        18.0,
        640.0,
        270.0,
        60.0,
        Color::new(INK.r, INK.g, INK.b, 0.72),
    );
    for (rect, key, intent) in [
        (MENU, "menu", UiAction::Open(Overlay::Menu)),
        (ZOOM_OUT, "zoom_out", UiAction::Zoom(1.0 / 1.25)),
        (ZOOM_IN, "zoom_in", UiAction::Zoom(1.25)),
        (RECENTER, "reset_view", UiAction::Recenter),
    ] {
        if button(ctx, rect, &ctx.text(key), active, key == "end_turn") {
            return Some(intent);
        }
    }
    let moving = ctx.movement.stage == super::MoveStage::Map;
    let phase_action = if moving {
        None
    } else {
        phase_controls(ctx, active)
    };
    let navigation_action = world::navigation(ctx);
    let selection_action = if moving {
        super::draw_move_map_overlay(ctx)
    } else {
        selection::draw(ctx)
    };
    selection_action.or(navigation_action).or(phase_action)
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
            300.0,
            640.0,
            640.0,
            60.0,
            macroquad_toolkit::colors::with_alpha(INK, 0.88),
        );
        let phase = if view.npc_paused {
            "npc_paused"
        } else {
            "npc_phase"
        };
        body(ctx, &ctx.text(phase), vec2(318.0, 663.0), 18.0, CREAM);
        body(
            ctx,
            &ctx.text("npc_prototype"),
            vec2(318.0, 686.0),
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
    let center = vec2(1212.0, 563.0);
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
