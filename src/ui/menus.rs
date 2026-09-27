//! Title, pause, and deliberately on-demand secondary screens.

use super::{components::*, Context, UiAction};
use kestrum::{
    navigation::{HEIGHT, WIDTH},
    state::Overlay,
};
use macroquad::prelude::*;

pub fn title(ctx: &Context<'_>) -> Option<UiAction> {
    for column in 0..1280 {
        let fraction = column as f32 / WIDTH;
        let alpha = (0.96 - fraction * 0.85).clamp(0.20, 0.96);
        draw_rectangle(
            column as f32,
            0.0,
            1.0,
            HEIGHT,
            Color::new(0.035, 0.08, 0.08, alpha),
        );
    }
    emblem(vec2(322.0, 150.0), 40.0);
    centered(ctx, &ctx.data.title, vec2(322.0, 264.0), 76.0, CREAM);
    horizontal_rule(vec2(322.0, 291.0), 207.0);
    let subtitle_width = measure_text(&ctx.data.subtitle, ctx.body_font(), 23, 1.0).width;
    body(
        ctx,
        &ctx.data.subtitle,
        vec2(322.0 - subtitle_width * 0.5, 325.0),
        23.0,
        CREAM,
    );
    let active = ctx.state.overlay == Overlay::None;
    let can_continue = ctx.save_exists || ctx.legacy_save_exists || ctx.state.campaign.is_some();
    let entries = [
        ("continue", UiAction::Continue, can_continue),
        ("new_game", UiAction::NewGame, true),
        ("settings", UiAction::Open(Overlay::Settings), true),
        ("help", UiAction::Open(Overlay::Help), true),
        ("credits", UiAction::Open(Overlay::Credits), true),
    ];
    for (index, (key, action, enabled)) in entries.into_iter().enumerate() {
        let rect = Rect::new(178.0, 353.0 + index as f32 * 51.0, 288.0, 48.0);
        let primary = if can_continue {
            key == "continue"
        } else {
            key == "new_game"
        };
        if button(ctx, rect, &ctx.text(key), active && enabled, primary) {
            return Some(action);
        }
        if key == "continue" && !can_continue {
            body(
                ctx,
                &ctx.text("no_save"),
                vec2(479.0, rect.y + 30.0),
                16.0,
                MUTED,
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if button(
        ctx,
        Rect::new(178.0, 608.0, 288.0, 48.0),
        &ctx.text("quit"),
        active,
        false,
    ) {
        return Some(UiAction::Quit);
    }
    if ctx.legacy_save_exists
        && button(
            ctx,
            Rect::new(850.0, 608.0, 350.0, 48.0),
            &ctx.text("load_legacy"),
            active,
            false,
        )
    {
        return Some(UiAction::LoadLegacy);
    }
    if button(
        ctx,
        Rect::new(850.0, 556.0, 350.0, 48.0),
        &ctx.text("save_catalogue"),
        active,
        false,
    ) {
        return Some(UiAction::Load);
    }
    if ctx.import_save_exists
        && button(
            ctx,
            Rect::new(850.0, 504.0, 350.0, 48.0),
            &ctx.text("import_campaign"),
            active,
            false,
        )
    {
        return Some(UiAction::ImportCampaign);
    }
    text(ctx, &ctx.data.edition, vec2(38.0, 693.0), 15.0, BRASS);
    body(ctx, "WebHatchery", vec2(1138.0, 693.0), 18.0, CREAM);
    None
}

pub fn overlay(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, Color::new(0.02, 0.05, 0.05, 0.60));
    let panel = Rect::new(378.0, 104.0, 524.0, 512.0);
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::new(0.06, 0.11, 0.11, 0.98),
    );
    draw_line(panel.x, panel.y, panel.right(), panel.y, 1.0, BRASS);
    draw_line(
        panel.x,
        panel.bottom(),
        panel.right(),
        panel.bottom(),
        1.0,
        BRASS,
    );
    let title_key = match ctx.state.overlay {
        Overlay::Menu => "menu",
        Overlay::Settings => "settings",
        Overlay::Help => "help_title",
        Overlay::Credits => "credits",
        Overlay::ConfirmNew => "new_title",
        Overlay::None
        | Overlay::Saves
        | Overlay::SaveRecovery
        | Overlay::Armies
        | Overlay::MoveGroup
        | Overlay::MoveReview
        | Overlay::Battle
        | Overlay::History
        | Overlay::Settlement => return None,
    };
    centered(ctx, &ctx.text(title_key), vec2(640.0, 159.0), 28.0, CREAM);
    horizontal_rule(vec2(640.0, 184.0), 206.0);
    let action = match ctx.state.overlay {
        Overlay::Menu => pause(ctx),
        Overlay::Settings => settings(ctx),
        Overlay::Help => help(ctx),
        Overlay::Credits => {
            credits(ctx);
            None
        }
        Overlay::ConfirmNew => confirm(ctx),
        Overlay::None
        | Overlay::Saves
        | Overlay::SaveRecovery
        | Overlay::Armies
        | Overlay::MoveGroup
        | Overlay::MoveReview
        | Overlay::Battle
        | Overlay::History
        | Overlay::Settlement => None,
    };
    if action.is_some() {
        return action;
    }
    let key = if ctx.state.overlay == Overlay::ConfirmNew {
        "cancel"
    } else {
        "back"
    };
    if button(
        ctx,
        Rect::new(522.0, 548.0, 236.0, 48.0),
        &ctx.text(key),
        true,
        false,
    ) {
        return Some(UiAction::Back);
    }
    None
}

fn pause(ctx: &Context<'_>) -> Option<UiAction> {
    for (index, (key, action)) in [
        ("save", UiAction::Save),
        ("load", UiAction::Load),
        ("records", UiAction::OpenRecords),
        ("settings", UiAction::Open(Overlay::Settings)),
        ("help", UiAction::Open(Overlay::Help)),
        ("main_menu", UiAction::MainMenu),
    ]
    .into_iter()
    .enumerate()
    {
        let enabled = match key {
            "load" => true,
            "save" => ctx.campaign_view.is_some_and(|view| view.player_turn),
            _ => true,
        };
        let label = if key == "save" && !enabled {
            ctx.text(if ctx.campaign_view.is_some() {
                "save_player_only"
            } else {
                "legacy_phase"
            })
        } else {
            ctx.text(key)
        };
        if button(
            ctx,
            Rect::new(482.0, 209.0 + index as f32 * 52.0, 316.0, 48.0),
            &label,
            enabled,
            index == 0,
        ) {
            return Some(action);
        }
    }
    None
}

fn settings(ctx: &Context<'_>) -> Option<UiAction> {
    for (index, (key, value, action)) in [
        (
            "labels",
            if ctx.preferences.hide_labels {
                "off"
            } else {
                "on"
            },
            UiAction::ToggleLabels,
        ),
        (
            "contrast",
            if ctx.preferences.high_contrast {
                "high"
            } else {
                "standard"
            },
            UiAction::ToggleContrast,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let y = 232.0 + index as f32 * 95.0;
        body(ctx, &ctx.text(key), vec2(415.0, y + 30.0), 23.0, CREAM);
        if button(
            ctx,
            Rect::new(724.0, y, 144.0, 48.0),
            &ctx.text(value),
            true,
            true,
        ) {
            return Some(action);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if button(
        ctx,
        Rect::new(478.0, 445.0, 324.0, 48.0),
        &ctx.text("fullscreen"),
        true,
        false,
    ) {
        return Some(UiAction::Fullscreen);
    }
    None
}

fn help(ctx: &Context<'_>) -> Option<UiAction> {
    let legacy = ctx.state.campaign.is_some() && ctx.campaign_view.is_none();
    let keys: &[&str] = if legacy {
        &["legacy_read_only", "help_pan", "help_zoom", "help_menu"]
    } else if ctx.help_page == 2 {
        &["help_move", "help_route", "help_spent"]
    } else if ctx.help_page == 3 {
        &["help_transfer", "help_recovery", "help_transfer_phase"]
    } else if ctx.help_page == 4 {
        &["help_battle", "help_battle_reports", "help_wounds"]
    } else if ctx.help_page == 6 {
        &[
            "help_construction",
            "help_builder",
            "help_construction_refund",
        ]
    } else if ctx.help_page == 5 {
        &["help_service", "help_history", "help_knowledge"]
    } else if ctx.help_page == 1 {
        &[
            "help_army",
            "help_recruit",
            "help_slots",
            "help_economy",
            "help_disband",
            "help_leadership",
        ]
    } else {
        &[
            "help_select",
            "help_region",
            "help_navigation",
            "help_turn",
            "help_menu",
            "help_scope",
        ]
    };
    for (index, key) in keys.iter().enumerate() {
        paragraph(
            ctx,
            &ctx.text(key),
            vec2(
                414.0,
                223.0 + index as f32 * if ctx.help_page >= 2 { 90.0 } else { 52.0 },
            ),
            453.0,
        );
    }
    if !legacy {
        if button(
            ctx,
            Rect::new(390.0, 548.0, 124.0, 48.0),
            &ctx.text("previous"),
            ctx.help_page > 0,
            false,
        ) {
            return Some(UiAction::HelpPage(-1));
        }
        if button(
            ctx,
            Rect::new(766.0, 548.0, 124.0, 48.0),
            &ctx.text("next"),
            ctx.help_page < 6,
            false,
        ) {
            return Some(UiAction::HelpPage(1));
        }
    }
    None
}

fn credits(ctx: &Context<'_>) {
    centered(
        ctx,
        &ctx.text("credits_title"),
        vec2(640.0, 246.0),
        20.0,
        BRASS,
    );
    for (index, line) in ctx.text("credits_body").lines().enumerate() {
        let width = measure_text(line, ctx.body_font(), 20, 1.0).width;
        body(
            ctx,
            line,
            vec2(640.0 - width * 0.5, 307.0 + index as f32 * 44.0),
            20.0,
            CREAM,
        );
    }
}

fn confirm(ctx: &Context<'_>) -> Option<UiAction> {
    paragraph(ctx, &ctx.text("new_warning"), vec2(422.0, 265.0), 435.0);
    if button(
        ctx,
        Rect::new(486.0, 399.0, 308.0, 48.0),
        &ctx.text("new_game"),
        true,
        true,
    ) {
        return Some(UiAction::ConfirmNew);
    }
    None
}
