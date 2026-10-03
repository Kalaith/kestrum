//! Title, pause, and deliberately on-demand secondary screens.

use super::{components::*, Context, UiAction};
use kestrum::{
    navigation::{HEIGHT, WIDTH},
    state::Overlay,
};
use macroquad::prelude::*;
mod help;
pub use help::{HELP_PAGE_COUNT, MAP_KEY_PAGE};

pub fn title(ctx: &Context<'_>) -> Option<UiAction> {
    for column in 0..WIDTH as i32 {
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
    emblem(vec2(480.0, 258.0), 48.0);
    centered(ctx, &ctx.game_text.title, vec2(480.0, 382.0), 88.0, CREAM);
    horizontal_rule(vec2(480.0, 414.0), 236.0);
    let subtitle_width = measure_text(&ctx.game_text.subtitle, ctx.body_font(), 23, 1.0).width;
    body(
        ctx,
        &ctx.game_text.subtitle,
        vec2(480.0 - subtitle_width * 0.5, 454.0),
        23.0,
        CREAM,
    );
    let active = ctx.state.overlay == Overlay::None;
    let can_continue = ctx.save_exists || ctx.legacy_save_exists || ctx.state.campaign.is_some();
    let entries = [
        ("continue", UiAction::Continue, can_continue),
        ("new_game", UiAction::NewGame, true),
        ("observer", UiAction::OpenObserverSetup, true),
        ("settings", UiAction::Open(Overlay::Settings), true),
        ("help", UiAction::Open(Overlay::Help), true),
        ("credits", UiAction::Open(Overlay::Credits), true),
    ];
    for (index, (key, action, enabled)) in entries.into_iter().enumerate() {
        let rect = Rect::new(318.0, 482.0 + index as f32 * 54.0, 324.0, 48.0);
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
                vec2(662.0, rect.y + 30.0),
                16.0,
                MUTED,
            );
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    if button(
        ctx,
        Rect::new(318.0, 820.0, 324.0, 52.0),
        &ctx.text("quit"),
        active,
        false,
    ) {
        return Some(UiAction::Quit);
    }
    if ctx.legacy_save_exists
        && button(
            ctx,
            Rect::new(1506.0, 944.0, 350.0, 52.0),
            &ctx.text("load_legacy"),
            active,
            false,
        )
    {
        return Some(UiAction::LoadLegacy);
    }
    if button(
        ctx,
        Rect::new(1506.0, 884.0, 350.0, 52.0),
        &ctx.text("save_catalogue"),
        active,
        false,
    ) {
        return Some(UiAction::Load);
    }
    if ctx.import_save_exists
        && button(
            ctx,
            Rect::new(1506.0, 824.0, 350.0, 52.0),
            &ctx.text("import_campaign"),
            active,
            false,
        )
    {
        return Some(UiAction::ImportCampaign);
    }
    text(ctx, &ctx.game_text.edition, vec2(48.0, 1040.0), 17.0, BRASS);
    body(ctx, "WebHatchery", vec2(1734.0, 1040.0), 18.0, CREAM);
    None
}

pub fn overlay(ctx: &Context<'_>) -> Option<UiAction> {
    let panel = if ctx.state.overlay == Overlay::Help {
        Rect::new(278.0, 104.0, 724.0, 512.0)
    } else {
        Rect::new(378.0, 104.0, 524.0, 512.0)
    };
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
        Overlay::Setup
        | Overlay::ObserverSetup
        | Overlay::ObserverKingdoms
        | Overlay::None
        | Overlay::Saves
        | Overlay::SaveRecovery
        | Overlay::Armies
        | Overlay::MoveGroup
        | Overlay::MoveReview
        | Overlay::Battle
        | Overlay::History
        | Overlay::Threat
        | Overlay::Kingdom
        | Overlay::CampaignEnd
        | Overlay::Siege
        | Overlay::Settlement
        | Overlay::Battlefield => return None,
    };
    let notification_settings =
        ctx.state.overlay == Overlay::Settings && ctx.notifications.global_settings_open;
    if !notification_settings {
        centered(ctx, &ctx.text(title_key), vec2(640.0, 159.0), 28.0, CREAM);
        horizontal_rule(vec2(640.0, 184.0), 206.0);
    }
    let action = match ctx.state.overlay {
        Overlay::Menu => {
            if ctx
                .campaign_view
                .is_some_and(|campaign| campaign.observer_mode)
            {
                super::observer::observer_pause_menu(ctx)
            } else {
                pause(ctx)
            }
        }
        Overlay::Settings => settings(ctx),
        Overlay::Help => help::draw(ctx),
        Overlay::Credits => {
            credits(ctx);
            None
        }
        Overlay::ConfirmNew => confirm(ctx),
        Overlay::Setup
        | Overlay::ObserverSetup
        | Overlay::ObserverKingdoms
        | Overlay::None
        | Overlay::Saves
        | Overlay::SaveRecovery
        | Overlay::Armies
        | Overlay::MoveGroup
        | Overlay::MoveReview
        | Overlay::Battle
        | Overlay::History
        | Overlay::Threat
        | Overlay::Kingdom
        | Overlay::CampaignEnd
        | Overlay::Siege
        | Overlay::Settlement
        | Overlay::Battlefield => None,
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
    if button(
        ctx,
        Rect::new(414.0, 209.0, 452.0, 48.0),
        &ctx.text("kingdom"),
        ctx.campaign_view.is_some(),
        true,
    ) {
        return Some(UiAction::OpenKingdom(None));
    }
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
            "save" => {
                ctx.campaign_view.is_some_and(|view| view.player_turn)
                    || ctx.kingdom.is_save_boundary()
            }
            _ => true,
        };
        let label = ctx.text(key);
        if key == "save" && !enabled {
            body(
                ctx,
                &ctx.text(if ctx.campaign_view.is_some() {
                    "save_player_only"
                } else {
                    "legacy_phase"
                }),
                vec2(414.0, 511.0),
                16.0,
                MUTED,
            );
        }
        if button(
            ctx,
            Rect::new(
                414.0 + (index % 2) as f32 * 238.0,
                296.0 + (index / 2) as f32 * 68.0,
                214.0,
                48.0,
            ),
            &label,
            enabled,
            false,
        ) {
            return Some(action);
        }
    }
    None
}

fn settings(ctx: &Context<'_>) -> Option<UiAction> {
    if ctx.notifications.global_settings_open {
        return super::notifications::draw_global_settings(ctx);
    }
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
    if button(
        ctx,
        Rect::new(478.0, 382.0, 324.0, 48.0),
        super::notifications::rules_term(ctx, "ui_notification_settings", "Notifications"),
        true,
        false,
    ) {
        return Some(UiAction::Notification(
            super::NotificationAction::OpenGlobalSettings,
        ));
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
