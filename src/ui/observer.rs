//! Read-only controls and views for watching every kingdom's AI campaign.

use super::{components::*, selection, world, Context, UiAction};
use kestrum::{
    data::world::FactionId,
    navigation::{MapScope, HEIGHT, WIDTH},
    state::{FactionStatus, Overlay},
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{truncate_text_to_width_ex, wrap_text_ex};

const MENU: Rect = Rect::new(WIDTH - 132.0, 24.0, 108.0, 48.0);
const ZOOM_OUT: Rect = Rect::new(24.0, HEIGHT - 72.0, 48.0, 48.0);
const ZOOM_IN: Rect = Rect::new(80.0, HEIGHT - 72.0, 48.0, 48.0);
const OVERVIEW: Rect = Rect::new(144.0, HEIGHT - 72.0, 148.0, 48.0);
const RECENTER: Rect = Rect::new(304.0, HEIGHT - 72.0, 148.0, 48.0);
const MAP_KEY: Rect = Rect::new(464.0, HEIGHT - 72.0, 132.0, 48.0);
const KINGDOMS: Rect = Rect::new(WIDTH - 712.0, HEIGHT - 72.0, 148.0, 48.0);
const SPEED_ONE: Rect = Rect::new(WIDTH - 552.0, HEIGHT - 72.0, 56.0, 48.0);
const SPEED_TWO: Rect = Rect::new(WIDTH - 488.0, HEIGHT - 72.0, 56.0, 48.0);
const SPEED_FOUR: Rect = Rect::new(WIDTH - 424.0, HEIGHT - 72.0, 56.0, 48.0);
const STEP: Rect = Rect::new(WIDTH - 352.0, HEIGHT - 72.0, 96.0, 48.0);
const PLAY: Rect = Rect::new(WIDTH - 244.0, HEIGHT - 72.0, 220.0, 48.0);
const ROSTER: Rect = Rect::new(80.0, 38.0, 1120.0, 650.0);

/// The observer toolbar is the only permanent control strip on the map.
pub fn controls_contain(point: Vec2) -> bool {
    [
        MENU, ZOOM_OUT, ZOOM_IN, OVERVIEW, RECENTER, MAP_KEY, KINGDOMS, SPEED_ONE, SPEED_TWO,
        SPEED_FOUR, STEP, PLAY,
    ]
    .iter()
    .any(|rect| rect.contains(point))
}

pub fn draw_hud(ctx: &Context<'_>) -> Option<UiAction> {
    if let Some(action) = header(ctx) {
        return Some(action);
    }
    if ctx.navigation.scope() == MapScope::World {
        emblem(vec2(44.0, 43.0), 19.0);
        text(ctx, &ctx.text("world_map"), vec2(80.0, 40.0), 24.0, CREAM);
        body(ctx, &ctx.data.title, vec2(81.0, 60.0), 16.0, BRASS);
    }
    if ctx.navigation.selection().is_none() {
        compass(ctx);
    }
    draw_rectangle(
        18.0,
        HEIGHT - 78.0,
        816.0,
        60.0,
        Color::new(INK.r, INK.g, INK.b, 0.72),
    );
    draw_rectangle(
        KINGDOMS.x - 12.0,
        HEIGHT - 78.0,
        WIDTH - KINGDOMS.x + 12.0,
        60.0,
        Color::new(INK.r, INK.g, INK.b, 0.88),
    );
    if let Some(action) = map_buttons(ctx) {
        return Some(action);
    }
    if let Some(action) = playback_buttons(ctx) {
        return Some(action);
    }
    world::navigation(ctx).or_else(|| selection::draw(ctx))
}

fn header(ctx: &Context<'_>) -> Option<UiAction> {
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
    draw_rectangle(
        1584.0,
        18.0,
        188.0,
        56.0,
        Color::new(INK.r, INK.g, INK.b, 0.84),
    );
    text(
        ctx,
        &ctx.text("observer_marker"),
        vec2(1602.0, 42.0),
        18.0,
        BRASS,
    );
    if button(
        ctx,
        MENU,
        &ctx.text("menu"),
        ctx.state.overlay == Overlay::None,
        false,
    ) {
        return Some(UiAction::Open(Overlay::Menu));
    }
    if let Some(campaign) = ctx.state.campaign.as_ref() {
        let season = &ctx.data.seasons[campaign.season_index()];
        centered(
            ctx,
            &format!(
                "{season}  /  {} {}",
                ctx.text("year"),
                campaign.year(ctx.data.start_year)
            ),
            vec2(WIDTH * 0.5, 35.0),
            22.0,
            CREAM,
        );
        let status = observer_status(ctx);
        let status = truncate_text_to_width_ex(&status, 640.0, ctx.body_font(), 18.0);
        let width = measure_text(&status, ctx.body_font(), 18, 1.0).width;
        body(
            ctx,
            &status,
            vec2(WIDTH * 0.5 - width * 0.5, 62.0),
            18.0,
            CREAM,
        );
    }
    None
}

fn map_buttons(ctx: &Context<'_>) -> Option<UiAction> {
    let active = ctx.state.overlay == Overlay::None;
    for (rect, key, action, primary) in [
        (ZOOM_OUT, "zoom_out", UiAction::Zoom(1.0 / 1.25), false),
        (ZOOM_IN, "zoom_in", UiAction::Zoom(1.25), false),
        (
            OVERVIEW,
            if ctx.view.overview_active() {
                "map_return_view"
            } else {
                "map_overview"
            },
            UiAction::Overview,
            false,
        ),
        (RECENTER, "reset_view", UiAction::Recenter, false),
        (MAP_KEY, "map_key", UiAction::MapKey, false),
    ] {
        if button(ctx, rect, &ctx.text(key), active, primary) {
            return Some(action);
        }
    }
    if button(ctx, KINGDOMS, &ctx.text("observer_kingdoms"), active, false) {
        return Some(UiAction::OpenObserverKingdoms);
    }
    None
}

fn playback_buttons(ctx: &Context<'_>) -> Option<UiAction> {
    let finished = observer_finished(ctx);
    if finished {
        button(ctx, PLAY, &ctx.text("observer_stopped"), false, false);
        for (rect, speed) in [(SPEED_ONE, 1), (SPEED_TWO, 2), (SPEED_FOUR, 4)] {
            button(
                ctx,
                rect,
                &format!("{speed}×"),
                false,
                ctx.observer.speed == speed,
            );
        }
        button(ctx, STEP, &ctx.text("observer_step"), false, false);
        return None;
    }
    let active = ctx.state.overlay == Overlay::None;
    for (rect, speed) in [(SPEED_ONE, 1), (SPEED_TWO, 2), (SPEED_FOUR, 4)] {
        if button(
            ctx,
            rect,
            &format!("{speed}×"),
            active,
            ctx.observer.speed == speed,
        ) {
            return Some(UiAction::SetObserverSpeed(speed));
        }
    }
    if button(
        ctx,
        STEP,
        &ctx.text("observer_step"),
        active && ctx.observer.paused,
        false,
    ) {
        return Some(UiAction::StepObserver);
    }
    let (key, action) = if ctx.observer.paused {
        ("observer_resume", UiAction::ToggleObserverPaused)
    } else {
        ("observer_pause", UiAction::ToggleObserverPaused)
    };
    button(ctx, PLAY, &ctx.text(key), active, true).then_some(action)
}

fn observer_finished(ctx: &Context<'_>) -> bool {
    ctx.state
        .campaign
        .as_ref()
        .and_then(kestrum::state::Campaign::strategic)
        .is_some_and(|campaign| campaign.observer_finished())
}

fn observer_status(ctx: &Context<'_>) -> String {
    if let Some(winner) = ctx
        .campaign_view
        .filter(|_| observer_finished(ctx))
        .and_then(|view| {
            let mut survivors = view
                .factions
                .iter()
                .filter(|faction| faction.status == FactionStatus::Independent);
            let winner = survivors.next()?;
            survivors.next().is_none().then_some(winner.name.as_str())
        })
    {
        return format!("{}: {winner}", ctx.text("observer_winner"));
    }
    if observer_finished(ctx) {
        return ctx.text("observer_stopped");
    }
    let acting = ctx
        .campaign_view
        .map(|view| view.active_faction_name.as_str())
        .unwrap_or_default();
    format!("{}: {acting}", ctx.text("observer_active"))
}

pub fn draw_setup(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(64.0, 28.0, 1152.0, 664.0, INK);
    draw_rectangle_lines(64.0, 28.0, 1152.0, 664.0, 1.0, BRASS);
    text(
        ctx,
        &ctx.text("observer_setup_title"),
        vec2(112.0, 78.0),
        32.0,
        CREAM,
    );
    body(
        ctx,
        &ctx.text("observer_setup_intro"),
        vec2(112.0, 112.0),
        18.0,
        MUTED,
    );
    draw_line(112.0, 132.0, 1168.0, 132.0, 1.0, BRASS);
    if let Some(action) = setup_factions(ctx) {
        return Some(action);
    }
    if let Some(action) = setup_seed(ctx) {
        return Some(action);
    }
    setup_session_note(ctx);
    setup_explanation(ctx);
    if button(
        ctx,
        Rect::new(120.0, 612.0, 230.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::Back);
    }
    button(
        ctx,
        Rect::new(848.0, 612.0, 320.0, 48.0),
        &ctx.text("observer_start"),
        true,
        true,
    )
    .then_some(UiAction::StartObserver)
}

fn setup_factions(ctx: &Context<'_>) -> Option<UiAction> {
    body(
        ctx,
        &ctx.text("observer_factions"),
        vec2(120.0, 184.0),
        22.0,
        CREAM,
    );
    let mut help_y = 216.0;
    for line in wrap_text_ex(
        &ctx.text("observer_factions_help"),
        500.0,
        ctx.body_font(),
        18.0,
    ) {
        body(ctx, &line, vec2(120.0, help_y), 18.0, MUTED);
        help_y += 25.0;
    }
    if button(
        ctx,
        Rect::new(148.0, 266.0, 64.0, 56.0),
        "−",
        ctx.setup.factions > ctx.rules.min_factions,
        false,
    ) {
        return Some(UiAction::ChangeSetupFactionCount(-1));
    }
    centered(
        ctx,
        &ctx.setup.factions.to_string(),
        vec2(278.0, 304.0),
        36.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(344.0, 266.0, 64.0, 56.0),
        "+",
        ctx.setup.factions < ctx.rules.max_factions,
        false,
    ) {
        return Some(UiAction::ChangeSetupFactionCount(1));
    }
    None
}

fn setup_seed(ctx: &Context<'_>) -> Option<UiAction> {
    body(
        ctx,
        &ctx.text("setup_seed"),
        vec2(120.0, 388.0),
        20.0,
        CREAM,
    );
    body(
        ctx,
        &ctx.setup.seed.to_string(),
        vec2(120.0, 430.0),
        23.0,
        CREAM,
    );
    if button(
        ctx,
        Rect::new(352.0, 398.0, 232.0, 50.0),
        &ctx.text("setup_randomize_seed"),
        true,
        false,
    ) {
        return Some(UiAction::RandomizeSetupSeed);
    }
    None
}

fn setup_session_note(ctx: &Context<'_>) {
    let mut y = 480.0;
    for line in wrap_text_ex(
        &ctx.text("observer_session_note"),
        500.0,
        ctx.body_font(),
        18.0,
    ) {
        body(ctx, &line, vec2(120.0, y), 18.0, MUTED);
        y += 26.0;
    }
}

fn setup_explanation(ctx: &Context<'_>) {
    draw_rectangle(
        672.0,
        162.0,
        496.0,
        386.0,
        Color::new(0.10, 0.16, 0.16, 0.72),
    );
    draw_line(672.0, 162.0, 1168.0, 162.0, 1.0, BRASS);
    text(
        ctx,
        &ctx.text("observer_all_ai"),
        vec2(704.0, 204.0),
        24.0,
        CREAM,
    );
    let mut y = 248.0;
    for line in wrap_text_ex(
        &ctx.text("observer_full_visibility"),
        432.0,
        ctx.body_font(),
        19.0,
    ) {
        body(ctx, &line, vec2(704.0, y), 19.0, CREAM);
        y += 28.0;
    }
    y += 16.0;
    for line in wrap_text_ex(&ctx.text("observer_tutorial"), 432.0, ctx.body_font(), 18.0) {
        body(ctx, &line, vec2(704.0, y), 18.0, MUTED);
        y += 26.0;
    }
}

pub fn draw_kingdoms(ctx: &Context<'_>) -> Option<UiAction> {
    let view = ctx.campaign_view?;
    draw_rectangle(ROSTER.x, ROSTER.y, ROSTER.w, ROSTER.h, INK);
    draw_rectangle_lines(ROSTER.x, ROSTER.y, ROSTER.w, ROSTER.h, 1.0, BRASS);
    text(
        ctx,
        &ctx.text("observer_roster_title"),
        vec2(112.0, 82.0),
        30.0,
        CREAM,
    );
    body(
        ctx,
        &ctx.text("observer_roster_help"),
        vec2(112.0, 120.0),
        18.0,
        MUTED,
    );
    for (index, faction) in view.factions.iter().enumerate() {
        let rect = Rect::new(112.0, 150.0 + index as f32 * 58.0, 1056.0, 52.0);
        let active = faction.id == view.active_faction;
        let hover = ctx.pointer.hovering_over(rect);
        if active || hover {
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::new(0.17, 0.24, 0.22, 0.96),
            );
        }
        draw_line(
            rect.x,
            rect.bottom(),
            rect.right(),
            rect.bottom(),
            1.0,
            BRASS,
        );
        let emblem_color = world::faction_color(ctx, Some(faction.id));
        draw_circle(rect.x + 16.0, rect.y + 25.0, 6.0, emblem_color);
        let name = truncate_text_to_width_ex(&faction.name, 450.0, ctx.body_font(), 20.0);
        body(ctx, &name, vec2(rect.x + 32.0, rect.y + 22.0), 20.0, CREAM);
        let status = truncate_text_to_width_ex(
            &faction_status(ctx, faction.id),
            520.0,
            ctx.body_font(),
            16.0,
        );
        body(
            ctx,
            &status,
            vec2(rect.x + 16.0, rect.y + 44.0),
            16.0,
            if active { BRASS } else { MUTED },
        );
        let territory = view
            .world
            .sites
            .iter()
            .filter(|site| site.controller == Some(faction.id))
            .count();
        let details = if active {
            format!(
                "{}: {territory}    {}",
                ctx.text("observer_sites"),
                ctx.text("observer_active")
            )
        } else {
            format!("{}: {territory}", ctx.text("observer_sites"))
        };
        body(
            ctx,
            &details,
            vec2(rect.x + 760.0, rect.y + 32.0),
            17.0,
            CREAM,
        );
        if ctx.state.overlay != Overlay::None
            && ctx.pointer.released_on(rect)
            && ctx.origin.is_some_and(|origin| rect.contains(origin))
        {
            return Some(UiAction::FocusObserverFaction(faction.id));
        }
    }
    if button(
        ctx,
        Rect::new(112.0, 626.0, 180.0, 48.0),
        &ctx.text("back"),
        true,
        false,
    ) {
        return Some(UiAction::Back);
    }
    None
}

fn faction_status(ctx: &Context<'_>, id: FactionId) -> String {
    let Some(faction) = ctx
        .campaign_view
        .and_then(|view| view.factions.iter().find(|faction| faction.id == id))
    else {
        return String::new();
    };
    match faction.status {
        FactionStatus::Independent => ctx.text("observer_independent"),
        FactionStatus::Eliminated => ctx.text("kingdom_eliminated"),
        FactionStatus::Vassal { sovereign } => {
            let name = ctx
                .campaign_view
                .and_then(|view| view.factions.iter().find(|faction| faction.id == sovereign))
                .map_or_else(
                    || ctx.text("map_unknown_kingdom"),
                    |ruler| ruler.name.clone(),
                );
            format!("{}: {name}", ctx.text("kingdom_subordinate"))
        }
    }
}

pub fn observer_pause_menu(ctx: &Context<'_>) -> Option<UiAction> {
    if button(
        ctx,
        Rect::new(414.0, 209.0, 452.0, 48.0),
        &ctx.text("observer_kingdoms"),
        true,
        true,
    ) {
        return Some(UiAction::OpenObserverKingdoms);
    }
    for (index, (key, action)) in [
        ("settings", UiAction::Open(Overlay::Settings)),
        ("help", UiAction::Open(Overlay::Help)),
        ("main_menu", UiAction::MainMenu),
    ]
    .into_iter()
    .enumerate()
    {
        if button(
            ctx,
            Rect::new(414.0 + index as f32 * 152.0, 296.0, 140.0, 52.0),
            &ctx.text(key),
            true,
            false,
        ) {
            return Some(action);
        }
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
