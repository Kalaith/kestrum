//! Motion, projectile, impact and short captions for the active receipt event.

use super::super::projection;
use super::group_label;
use crate::ui::{components, Context};
use kestrum::{
    data::{battle_tactics::TacticAction, economy::TroopKind},
    state::battle::{
        playback::{BattlePresentationFrame, BattlePresentationGroup},
        simulation::{BattleEvent, BattleResolution, BattleUnitId},
        BattleOutcome,
    },
};
use macroquad::prelude::*;

pub(super) fn draw(
    ctx: &Context<'_>,
    frame: &BattlePresentationFrame,
    resolution: &BattleResolution,
    progress: f32,
) {
    let Some(event) = &frame.active_event else {
        if frame.completed_events == resolution.events.len() {
            draw_result(ctx, resolution.outcome);
        }
        return;
    };
    if matches!(event, BattleEvent::RoundStarted { .. }) {
        return;
    }
    let caption = caption(ctx, event, resolution);
    components::centered(ctx, &caption, vec2(640.0, 213.0), 21.0, components::CREAM);
    match event {
        BattleEvent::Activation {
            actor,
            target: Some(target),
            action: TacticAction::Volley,
            ..
        } => draw_projectile(frame, resolution, *actor, *target, progress),
        BattleEvent::Damage { source, target, .. } => {
            let from = find_center(frame, resolution, *source);
            let to = find_center(frame, resolution, *target);
            draw_circle(
                to.x,
                to.y - 7.0,
                28.0 * (1.0 - progress),
                Color::new(0.96, 0.78, 0.41, 0.40),
            );
            draw_line(
                from.x,
                from.y - 17.0,
                to.x,
                to.y - 17.0,
                2.0,
                Color::new(0.90, 0.80, 0.62, 0.35),
            );
        }
        BattleEvent::Activation {
            actor,
            target: Some(target),
            action: TacticAction::Charge | TacticAction::Breakthrough,
            ..
        } => draw_charge_trail(frame, resolution, *actor, *target, progress),
        BattleEvent::Reaction { actor, against, .. } => {
            let brace = find_center(frame, resolution, *actor);
            let cavalry = find_center(frame, resolution, *against);
            draw_line(
                brace.x,
                brace.y - 48.0,
                brace.x,
                brace.y + 44.0,
                4.0,
                components::BRASS,
            );
            draw_line(
                cavalry.x,
                cavalry.y - 32.0,
                cavalry.x,
                cavalry.y + 32.0,
                2.0,
                Color::new(0.96, 0.84, 0.57, 0.72),
            );
        }
        BattleEvent::Routed { unit, .. } => {
            let center = find_center(frame, resolution, *unit);
            draw_line(
                center.x - 54.0,
                center.y - 4.0,
                center.x - 18.0,
                center.y - 4.0,
                3.0,
                Color::new(0.94, 0.83, 0.64, 0.82),
            );
        }
        _ => {}
    }
}

pub(super) fn animated_center(
    group: &BattlePresentationGroup,
    frame: &BattlePresentationFrame,
    resolution: &BattleResolution,
    base: Vec2,
    progress: f32,
) -> Vec2 {
    let pair = match frame.active_event.as_ref() {
        Some(BattleEvent::Activation {
            actor,
            target: Some(target),
            action: TacticAction::Charge | TacticAction::Breakthrough,
            ..
        }) => Some((*actor, *target, 0.32 * progress.clamp(0.0, 1.0))),
        Some(BattleEvent::Reaction { against, actor, .. })
            if follows_charge(resolution, frame.completed_events, *against, *actor) =>
        {
            Some((*against, *actor, 0.38))
        }
        Some(BattleEvent::Damage { source, target, .. })
            if follows_charge(resolution, frame.completed_events, *source, *target) =>
        {
            Some((*source, *target, 0.42))
        }
        _ => None,
    };
    let Some((actor, target, progress)) = pair else {
        return base;
    };
    if group.id != actor {
        return base;
    }
    let destination = frame
        .groups
        .iter()
        .find(|candidate| candidate.id == target)
        .map_or(base, |candidate| {
            projection::group_center(candidate, resolution)
        });
    base.lerp(destination, progress.clamp(0.0, 1.0))
}

fn follows_charge(
    resolution: &BattleResolution,
    completed_events: usize,
    actor: BattleUnitId,
    target: BattleUnitId,
) -> bool {
    resolution
        .events
        .iter()
        .take(completed_events)
        .rev()
        .take(4)
        .any(|event| {
            matches!(
                event,
                BattleEvent::Activation {
                    actor: active,
                    target: Some(active_target),
                    action: TacticAction::Charge | TacticAction::Breakthrough,
                    ..
                } if *active == actor && *active_target == target
            )
        })
}

pub(super) fn caption(
    ctx: &Context<'_>,
    event: &BattleEvent,
    resolution: &BattleResolution,
) -> String {
    match event {
        BattleEvent::RoundStarted { round } => format!("{} {round}", ctx.text("round")),
        BattleEvent::Activation { actor, action, .. } => {
            let kind = unit_kind(resolution, *actor).map_or_else(
                || ctx.text("battle_play_group"),
                |kind| group_label(ctx, Some(kind)),
            );
            let label = match action {
                TacticAction::Attack => "battle_play_attack",
                TacticAction::Volley => "battle_play_volley",
                TacticAction::Charge => "battle_play_charge",
                TacticAction::Breakthrough => "battle_play_breakthrough",
                TacticAction::Guard => "battle_play_guard",
                TacticAction::Brace => "battle_play_brace",
                TacticAction::Wait => "battle_play_wait",
                TacticAction::Advance => "battle_play_advance",
            };
            format!("{kind} · {}", ctx.text(label))
        }
        BattleEvent::Reaction { actor, .. } => {
            let kind = unit_kind(resolution, *actor).map_or_else(
                || ctx.text("battle_play_group"),
                |kind| group_label(ctx, Some(kind)),
            );
            format!("{kind} · {}", ctx.text("battle_play_brace"))
        }
        BattleEvent::Damage { .. } => ctx.text("battle_play_impact"),
        BattleEvent::MoraleChanged { .. } => ctx.text("battle_play_morale_break"),
        BattleEvent::GuardRaised { .. } => ctx.text("battle_play_guard"),
        BattleEvent::PositionChanged { .. } => ctx.text("battle_play_advance"),
        BattleEvent::Routed { .. } => ctx.text("battle_play_routed"),
        BattleEvent::BattleEnded { outcome, .. } => outcome_label(ctx, *outcome),
    }
}

fn draw_charge_trail(
    frame: &BattlePresentationFrame,
    resolution: &BattleResolution,
    actor: BattleUnitId,
    target: BattleUnitId,
    progress: f32,
) {
    let from = find_center(frame, resolution, actor);
    let to = find_center(frame, resolution, target);
    let end = from.lerp(to, 0.18 + progress * 0.42);
    let color = Color::new(0.91, 0.79, 0.54, 0.75);
    draw_line(from.x, from.y + 20.0, end.x, end.y + 20.0, 3.2, color);
    for sign in [-1.0, 1.0] {
        draw_line(
            end.x,
            end.y + 20.0,
            end.x - sign * 11.0,
            end.y + 14.0,
            2.0,
            color,
        );
    }
}

fn draw_projectile(
    frame: &BattlePresentationFrame,
    resolution: &BattleResolution,
    actor: BattleUnitId,
    target: BattleUnitId,
    progress: f32,
) {
    let from = find_center(frame, resolution, actor) + vec2(0.0, -20.0);
    let to = find_center(frame, resolution, target) + vec2(0.0, -20.0);
    let at = from.lerp(to, progress);
    draw_line(
        from.x,
        from.y,
        at.x,
        at.y,
        2.0,
        Color::new(0.93, 0.83, 0.62, 0.85),
    );
    let direction = (to - from).normalize_or_zero();
    let normal = vec2(-direction.y, direction.x);
    draw_triangle(
        at,
        at - direction * 7.0 - normal * 3.0,
        at - direction * 7.0 + normal * 3.0,
        components::CREAM,
    );
}

fn find_center(
    frame: &BattlePresentationFrame,
    resolution: &BattleResolution,
    id: BattleUnitId,
) -> Vec2 {
    frame
        .groups
        .iter()
        .find(|group| group.id == id)
        .map_or(vec2(640.0, 410.0), |group| {
            projection::group_center(group, resolution)
        })
}

fn draw_result(ctx: &Context<'_>, outcome: BattleOutcome) {
    let label = outcome_label(ctx, outcome);
    let width = measure_text(&label, ctx.font(), 22, 1.0).width + 42.0;
    let rect = Rect::new(640.0 - width / 2.0, 179.0, width, 45.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::new(0.09, 0.14, 0.13, 0.84),
    );
    components::centered(ctx, &label, vec2(640.0, 208.0), 22.0, components::CREAM);
}

fn outcome_label(ctx: &Context<'_>, outcome: BattleOutcome) -> String {
    match outcome {
        BattleOutcome::AttackerVictory => ctx.text("battle_play_victory"),
        BattleOutcome::DefenderVictory => ctx.text("battle_play_defender_victory"),
        BattleOutcome::Stalemate => ctx.text("battle_play_stalemate"),
        BattleOutcome::MutualDestruction => ctx.text("battle_play_mutual"),
    }
}

fn unit_kind(resolution: &BattleResolution, id: BattleUnitId) -> Option<TroopKind> {
    resolution
        .opening
        .armies
        .iter()
        .flat_map(|army| army.slots.iter().flatten())
        .find(|unit| unit.id == id)
        .and_then(|unit| unit.kind)
}
