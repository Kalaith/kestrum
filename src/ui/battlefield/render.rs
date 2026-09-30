//! Full-scene battlefield, contextual inspection and quiet playback controls.

mod events;

use super::{projection, terrain, troop_sprites, BattlefieldAction};
use crate::ui::{components, Context, UiAction};
use kestrum::{
    data::economy::TroopKind,
    state::battle::{
        playback::{battle_presentation_at, BattlePresentationFrame, BattlePresentationGroup},
        simulation::{BattleEvent, BattleResolution, BattleSide},
    },
};
use macroquad::prelude::*;

const CONTROL_Y: f32 = 658.0;

pub(super) fn prepare_text(ctx: &Context<'_>) {
    let Some(resolution) = ctx.battle_resolution else {
        return;
    };
    let frame = battle_presentation_at(resolution, ctx.battlefield.event_cursor);
    let mut body = vec![
        (16, ctx.text("battle_play_front")),
        (16, ctx.text("battle_play_rear")),
        (18, ctx.text("battle_play_morale")),
        (18, ctx.text("battle_play_guard")),
        (20, ctx.text("battle_play_pause")),
        (20, ctx.text("battle_play_resume")),
        (20, ctx.text("battle_play_step")),
        (20, ctx.text("battle_play_skip")),
        (20, ctx.text("battle_play_complete")),
        (18, ctx.text("battle_play_inspect")),
        (20, ctx.text("battle_play_attack")),
        (20, ctx.text("battle_play_volley")),
        (20, ctx.text("battle_play_charge")),
        (20, ctx.text("battle_play_breakthrough")),
        (20, ctx.text("battle_play_brace")),
        (20, ctx.text("battle_play_wait")),
        (20, ctx.text("battle_play_advance")),
        (16, ctx.text("battle_play_gap")),
        (18, ctx.text("battle_play_routed")),
        (18, ctx.text("battle_play_victory")),
        (18, ctx.text("battle_play_defender_victory")),
        (18, ctx.text("battle_play_stalemate")),
        (18, ctx.text("battle_play_mutual")),
        (18, ctx.text("close")),
        (16, String::from("1×")),
        (16, String::from("2×")),
        (16, String::from("3×")),
    ];
    for group in &frame.groups {
        body.push((16, group_label(ctx, group.kind)));
        body.push((16, format!("{} / {}", group.headcount, group.capacity)));
        body.push((16, format!("{}%", group.morale)));
    }
    for army in &resolution.opening.armies {
        body.push((18, army.name.clone()));
    }
    if let Some(event) = &frame.active_event {
        body.push((20, events::caption(ctx, event, resolution)));
    }
    if let Some(selected) = ctx.battlefield.selected {
        if let Some(group) = frame.groups.iter().find(|group| group.id == selected) {
            body.push((18, selected_description(ctx, group)));
        }
    }
    let prepared: Vec<_> = body
        .iter()
        .map(|(size, text)| (*size, text.as_str()))
        .collect();
    if let Some(font) = ctx.body_font() {
        macroquad_toolkit::ui::prepare_font_text(font, &prepared);
    }
    if let Some(font) = ctx.font() {
        let titles: Vec<_> = resolution
            .opening
            .armies
            .iter()
            .map(|army| (22, army.name.as_str()))
            .collect();
        macroquad_toolkit::ui::prepare_font_text(font, &titles);
    }
}

pub(super) fn draw(ctx: &Context<'_>) -> Option<UiAction> {
    let resolution = ctx.battle_resolution?;
    terrain::draw();
    let frame = battle_presentation_at(resolution, ctx.battlefield.event_cursor);
    draw_header(ctx, resolution, &frame);
    draw_lane_guides(ctx);

    let mut action = None;
    for group in &frame.groups {
        let base = projection::group_center(group, resolution);
        let center =
            events::animated_center(group, &frame, resolution, base, ctx.battlefield.progress());
        if group.headcount == 0 && !group.is_routed {
            draw_broken_slot(ctx, group, center, &frame);
        } else if group.is_routed {
            draw_routed_group(ctx, group, center);
        } else {
            let braced = matches!(
                frame.active_event.as_ref(),
                Some(BattleEvent::Reaction { actor, .. }) if *actor == group.id
            );
            troop_sprites::draw_group(troop_sprites::GroupSprite {
                center,
                kind: group.kind,
                headcount: group.headcount,
                capacity: group.capacity,
                side: group.side,
                facing: if group.side == BattleSide::Attacker {
                    1.0
                } else {
                    -1.0
                },
                is_braced: braced,
                is_routed: false,
            });
            draw_unit_label(ctx, group, base);
            if ctx.battlefield.selected == Some(group.id) {
                draw_selection(base);
            }
            if ctx.pointer.released_on(projection::selection_bounds(base))
                && ctx
                    .origin
                    .is_some_and(|origin| projection::selection_bounds(base).contains(origin))
            {
                action = Some(UiAction::Battlefield(BattlefieldAction::Select(Some(
                    group.id,
                ))));
            }
        }
    }

    events::draw(ctx, &frame, resolution, ctx.battlefield.progress());
    let inspector_action = draw_selection_inspector(ctx, &frame);
    draw_controls(ctx).or(inspector_action).or(action)
}

fn draw_header(ctx: &Context<'_>, resolution: &BattleResolution, frame: &BattlePresentationFrame) {
    draw_rectangle(0.0, 0.0, 1280.0, 80.0, Color::new(0.05, 0.10, 0.11, 0.77));
    let attackers: Vec<_> = resolution
        .opening
        .armies
        .iter()
        .filter(|army| army.side == BattleSide::Attacker)
        .collect();
    let defenders: Vec<_> = resolution
        .opening
        .armies
        .iter()
        .filter(|army| army.side == BattleSide::Defender)
        .collect();
    let left_name = attackers
        .first()
        .map_or_else(|| ctx.text("battle_attacker"), |army| army.name.clone());
    let right_name = defenders
        .first()
        .map_or_else(|| ctx.text("battle_defender"), |army| army.name.clone());
    draw_pennant(34.0, 49.0, 1.0, BattleSide::Attacker);
    draw_pennant(1246.0, 49.0, -1.0, BattleSide::Defender);
    components::text(ctx, &left_name, vec2(62.0, 36.0), 22.0, components::CREAM);
    components::body(
        ctx,
        &side_summary(ctx, &frame.groups, BattleSide::Attacker),
        vec2(62.0, 59.0),
        16.0,
        components::MUTED,
    );
    let right_width = measure_text(&right_name, ctx.font(), 22, 1.0).width;
    components::text(
        ctx,
        &right_name,
        vec2(1218.0 - right_width, 36.0),
        22.0,
        components::CREAM,
    );
    let summary = side_summary(ctx, &frame.groups, BattleSide::Defender);
    let summary_width = measure_text(&summary, ctx.body_font(), 16, 1.0).width;
    components::body(
        ctx,
        &summary,
        vec2(1218.0 - summary_width, 59.0),
        16.0,
        components::MUTED,
    );
    draw_round_plaque(ctx, frame.round);
}

fn side_summary(ctx: &Context<'_>, groups: &[BattlePresentationGroup], side: BattleSide) -> String {
    let side_groups: Vec<_> = groups.iter().filter(|group| group.side == side).collect();
    let troops: u32 = side_groups.iter().map(|group| group.headcount).sum();
    let morale = if side_groups.is_empty() {
        0
    } else {
        side_groups.iter().map(|group| group.morale).sum::<u32>() / side_groups.len() as u32
    };
    format!("{troops} · {} {}%", ctx.text("battle_play_morale"), morale)
}

fn draw_round_plaque(ctx: &Context<'_>, round: u32) {
    draw_rectangle(574.0, 16.0, 132.0, 48.0, Color::new(0.14, 0.19, 0.17, 0.90));
    draw_rectangle_lines(574.0, 16.0, 132.0, 48.0, 1.0, components::BRASS);
    let label = format!("{} {round}", ctx.text("round"));
    components::centered(ctx, &label, vec2(640.0, 47.0), 20.0, components::CREAM);
}

fn draw_pennant(x: f32, y: f32, facing: f32, side: BattleSide) {
    let cloth = side_color(side, 1.0);
    draw_line(
        x,
        y - 17.0,
        x,
        y + 19.0,
        2.0,
        Color::new(0.28, 0.21, 0.15, 1.0),
    );
    draw_triangle(
        vec2(x, y - 16.0),
        vec2(x + facing * 22.0, y - 10.0),
        vec2(x, y + 5.0),
        cloth,
    );
    draw_circle(x + facing * 7.0, y - 5.0, 2.3, components::BRASS);
}

fn draw_lane_guides(ctx: &Context<'_>) {
    for y in [356.0, 468.0, 580.0] {
        draw_line(112.0, y, 1168.0, y, 1.0, Color::new(0.92, 0.81, 0.59, 0.10));
    }
    for x in [412.0, 868.0] {
        draw_line(x, 252.0, x, 598.0, 1.5, Color::new(0.94, 0.85, 0.67, 0.28));
        for y in (266..600).step_by(15) {
            draw_circle(x, y as f32, 1.8, Color::new(0.94, 0.85, 0.67, 0.32));
        }
    }
    for (label, x) in [
        (ctx.text("battle_play_rear"), 315.0),
        (ctx.text("battle_play_front"), 496.0),
        (ctx.text("battle_play_front"), 784.0),
        (ctx.text("battle_play_rear"), 965.0),
    ] {
        components::centered(ctx, &label, vec2(x, 264.0), 16.0, components::CREAM);
    }
}

fn draw_unit_label(ctx: &Context<'_>, group: &BattlePresentationGroup, center: Vec2) {
    let rect = Rect::new(center.x - 49.0, center.y + 33.0, 98.0, 48.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.05, 0.09, 0.09, 0.76),
    );
    let kind = group_label(ctx, group.kind);
    components::centered(
        ctx,
        &kind,
        vec2(center.x, center.y + 51.0),
        15.0,
        components::CREAM,
    );
    let count = format!("{} / {}", group.headcount, group.capacity);
    components::centered(
        ctx,
        &count,
        vec2(center.x, center.y + 70.0),
        15.0,
        components::CREAM,
    );
    draw_morale_bar(group, center);
    if group.guard_expires_round.is_some() {
        draw_circle(center.x + 40.0, center.y + 39.0, 4.0, components::BRASS);
    }
}

fn draw_morale_bar(group: &BattlePresentationGroup, center: Vec2) {
    let left = center.x - 42.0;
    let top = center.y + 76.0;
    draw_rectangle(left, top, 84.0, 3.0, Color::new(0.0, 0.0, 0.0, 0.48));
    let color = if group.morale <= 35 {
        Color::new(0.78, 0.32, 0.25, 1.0)
    } else {
        Color::new(0.83, 0.70, 0.39, 1.0)
    };
    draw_rectangle(
        left,
        top,
        84.0 * group.morale.min(100) as f32 / 100.0,
        3.0,
        color,
    );
}

fn draw_broken_slot(
    ctx: &Context<'_>,
    group: &BattlePresentationGroup,
    center: Vec2,
    frame: &BattlePresentationFrame,
) {
    let focus = matches!(
        frame.active_event.as_ref(),
        Some(BattleEvent::Damage { target, remaining: 0, .. }) if *target == group.id
    );
    let tint = if focus {
        Color::new(0.95, 0.81, 0.48, 0.95)
    } else {
        Color::new(0.61, 0.55, 0.43, 0.60)
    };
    draw_circle(
        center.x,
        center.y + 2.0,
        19.0,
        Color::new(0.15, 0.16, 0.14, 0.31),
    );
    draw_line(
        center.x - 13.0,
        center.y - 9.0,
        center.x + 11.0,
        center.y + 12.0,
        3.0,
        tint,
    );
    draw_line(
        center.x + 12.0,
        center.y - 8.0,
        center.x - 11.0,
        center.y + 11.0,
        3.0,
        tint,
    );
    let label = ctx.text("battle_play_gap");
    components::centered(
        ctx,
        &label,
        center + vec2(0.0, 34.0),
        16.0,
        components::CREAM,
    );
}

fn draw_routed_group(ctx: &Context<'_>, group: &BattlePresentationGroup, center: Vec2) {
    let outer_x = if group.side == BattleSide::Attacker {
        112.0
    } else {
        1168.0
    };
    let retreat_center = vec2(outer_x, center.y);
    troop_sprites::draw_group(troop_sprites::GroupSprite {
        center: retreat_center,
        kind: group.kind,
        headcount: group.headcount,
        capacity: group.capacity,
        side: group.side,
        facing: if group.side == BattleSide::Attacker {
            -1.0
        } else {
            1.0
        },
        is_braced: false,
        is_routed: true,
    });
    let sign = if group.side == BattleSide::Attacker {
        -1.0
    } else {
        1.0
    };
    for index in 0..3 {
        let x = retreat_center.x + sign * (30.0 + index as f32 * 12.0);
        draw_line(
            x,
            center.y + 2.0,
            x + sign * 8.0,
            center.y - 3.0,
            1.8,
            components::CREAM,
        );
    }
    components::centered(
        ctx,
        &ctx.text("battle_play_routed"),
        retreat_center + vec2(0.0, 52.0),
        16.0,
        components::CREAM,
    );
}

fn draw_selection(center: Vec2) {
    let bounds = projection::selection_bounds(center);
    draw_rectangle_lines(
        bounds.x,
        bounds.y,
        bounds.w,
        bounds.h,
        2.0,
        components::BRASS,
    );
}

fn draw_selection_inspector(
    ctx: &Context<'_>,
    frame: &BattlePresentationFrame,
) -> Option<UiAction> {
    let id = ctx.battlefield.selected?;
    let group = frame.groups.iter().find(|group| group.id == id)?;
    let rect = Rect::new(470.0, 91.0, 340.0, 56.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.05, 0.10, 0.10, 0.87),
    );
    draw_line(
        rect.x,
        rect.y,
        rect.x,
        rect.y + rect.h,
        3.0,
        components::BRASS,
    );
    let heading = format!(
        "{} · {} / {}",
        group_label(ctx, group.kind),
        group.headcount,
        group.capacity
    );
    components::text(
        ctx,
        &heading,
        vec2(rect.x + 14.0, rect.y + 24.0),
        18.0,
        components::CREAM,
    );
    let details = selected_description(ctx, group);
    components::body(
        ctx,
        &details,
        vec2(rect.x + 14.0, rect.y + 45.0),
        16.0,
        components::MUTED,
    );
    components::button(
        ctx,
        Rect::new(rect.x + rect.w - 78.0, rect.y + 14.0, 66.0, 30.0),
        &ctx.text("close"),
        true,
        false,
    )
    .then_some(UiAction::Battlefield(BattlefieldAction::Select(None)))
}

fn selected_description(ctx: &Context<'_>, group: &BattlePresentationGroup) -> String {
    let status = if group.is_routed {
        ctx.text("battle_play_routed")
    } else if group.guard_expires_round.is_some() {
        ctx.text("battle_play_guard")
    } else {
        ctx.text("battle_play_morale")
    };
    format!("{status} · {}%", group.morale)
}

fn draw_controls(ctx: &Context<'_>) -> Option<UiAction> {
    draw_rectangle(
        0.0,
        CONTROL_Y - 8.0,
        1280.0,
        70.0,
        Color::new(0.04, 0.08, 0.09, 0.85),
    );
    let resolution = ctx.battle_resolution.expect("draw requires a resolution");
    let finished = ctx.battlefield.event_cursor >= resolution.events.len();
    let pause_label = if finished {
        ctx.text("battle_play_complete")
    } else if ctx.battlefield.is_paused {
        ctx.text("battle_play_resume")
    } else {
        ctx.text("battle_play_pause")
    };
    let buttons = [
        (
            Rect::new(335.0, 667.0, 125.0, 40.0),
            pause_label,
            !finished,
            0,
        ),
        (
            Rect::new(469.0, 667.0, 102.0, 40.0),
            ctx.text("battle_play_step"),
            !finished,
            1,
        ),
    ];
    let mut action = None;
    components::body(
        ctx,
        &ctx.text("battle_play_inspect"),
        vec2(66.0, 690.0),
        16.0,
        components::MUTED,
    );
    for (rect, label, enabled, id) in buttons {
        if components::button(ctx, rect, &label, enabled, id == 0) {
            action = Some(if id == 0 {
                UiAction::Battlefield(BattlefieldAction::TogglePause)
            } else {
                UiAction::Battlefield(BattlefieldAction::Step)
            });
        }
    }
    for (index, speed) in [1_u8, 2, 3].into_iter().enumerate() {
        let rect = Rect::new(590.0 + index as f32 * 54.0, 667.0, 48.0, 40.0);
        if components::button(
            ctx,
            rect,
            &format!("{speed}×"),
            true,
            ctx.battlefield.speed == speed,
        ) {
            action = Some(UiAction::Battlefield(BattlefieldAction::SetSpeed(speed)));
        }
    }
    let skip_label = ctx.text("battle_play_skip");
    if components::button(
        ctx,
        Rect::new(766.0, 667.0, 165.0, 40.0),
        &skip_label,
        !finished,
        false,
    ) {
        action = Some(UiAction::Battlefield(BattlefieldAction::SkipToResult));
    }
    action
}

fn group_label(ctx: &Context<'_>, kind: Option<TroopKind>) -> String {
    let key = match kind {
        Some(TroopKind::Warriors) => "troop_warriors",
        Some(TroopKind::Spearmen) => "troop_spearmen",
        Some(TroopKind::Archers) => "troop_archers",
        Some(TroopKind::Riders) => "troop_riders",
        Some(TroopKind::Medics) => "troop_medics",
        Some(TroopKind::SiegeEngines) => "troop_siege_engines",
        None => "battle_play_group",
    };
    ctx.text(key)
}

fn side_color(side: BattleSide, alpha: f32) -> Color {
    match side {
        BattleSide::Attacker => Color::new(0.18, 0.35, 0.53, alpha),
        BattleSide::Defender => Color::new(0.57, 0.20, 0.17, alpha),
    }
}
