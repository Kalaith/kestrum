//! Restrained typography, heraldry, and toolkit plaque controls.

use super::Context;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_plaque, PlaquePalette, PlaqueState, PlaqueStyle, SurfaceStyle};

pub const INK: Color = Color::new(0.07, 0.12, 0.12, 1.0);
pub const CREAM: Color = Color::new(0.95, 0.90, 0.76, 1.0);
pub const BRASS: Color = Color::new(0.78, 0.66, 0.40, 1.0);
pub const MUTED: Color = Color::new(0.65, 0.68, 0.62, 1.0);

pub fn text(ctx: &Context<'_>, label: &str, at: Vec2, size: f32, color: Color) {
    draw_text_ex(
        label,
        at.x,
        at.y,
        TextParams {
            font: ctx.font(),
            font_size: size as u16,
            color,
            ..Default::default()
        },
    );
}

pub fn centered(ctx: &Context<'_>, label: &str, at: Vec2, size: f32, color: Color) {
    let width = measure_text(label, ctx.font(), size as u16, 1.0).width;
    text(ctx, label, at - vec2(width * 0.5, 0.0), size, color);
}

pub fn body(ctx: &Context<'_>, label: &str, at: Vec2, size: f32, color: Color) {
    draw_text_ex(
        label,
        at.x,
        at.y,
        TextParams {
            font: ctx.body_font(),
            font_size: size as u16,
            color,
            ..Default::default()
        },
    );
}

pub fn paragraph(ctx: &Context<'_>, label: &str, at: Vec2, max_width: f32) {
    let mut row = String::new();
    let mut y = at.y;
    for word in label.split_whitespace() {
        let candidate = if row.is_empty() {
            word.to_owned()
        } else {
            format!("{row} {word}")
        };
        if measure_text(&candidate, ctx.body_font(), 19, 1.0).width > max_width && !row.is_empty() {
            body(ctx, &row, vec2(at.x, y), 19.0, CREAM);
            y += 28.0;
            row = word.to_owned();
        } else {
            row = candidate;
        }
    }
    body(ctx, &row, vec2(at.x, y), 19.0, CREAM);
}

pub fn button(ctx: &Context<'_>, rect: Rect, label: &str, enabled: bool, primary: bool) -> bool {
    button_font(ctx, rect, label, enabled, primary, ctx.font())
}

pub fn input_key(ctx: &Context<'_>, rect: Rect, label: &str) -> bool {
    button_font(ctx, rect, label, true, true, ctx.body_font())
}

fn button_font(
    ctx: &Context<'_>,
    rect: Rect,
    label: &str,
    enabled: bool,
    primary: bool,
    font: Option<&Font>,
) -> bool {
    let hovered = enabled && ctx.pointer.hovering_over(rect);
    let pressed = enabled && ctx.pointer.pressing(rect);
    let palette = PlaquePalette {
        normal: if primary {
            Color::new(0.17, 0.24, 0.22, 0.96)
        } else {
            Color::new(0.05, 0.10, 0.10, 0.0)
        },
        hovered: Color::new(0.32, 0.37, 0.28, 0.85),
        pressed: Color::new(0.37, 0.40, 0.27, 0.96),
        disabled: Color::new(0.0, 0.0, 0.0, 0.0),
        border: BRASS,
        text: CREAM,
    };
    let style = PlaqueStyle {
        shadow: None,
        frame: SurfaceStyle::new(BLANK),
        inset: 0.0,
        face_border_width: if primary { 1.0 } else { 0.0 },
        face_inner_border: None,
        top_highlight: None,
        corner_marks: None,
        ..Default::default()
    };
    draw_plaque(
        rect,
        &style,
        &palette,
        PlaqueState {
            enabled,
            hovered,
            pressed,
            selected: false,
        },
    );
    let width = measure_text(label, font, 21, 1.0).width;
    draw_text_ex(
        label,
        rect.x + (rect.w - width) * 0.5,
        rect.y + rect.h * 0.5 + 7.0,
        TextParams {
            font,
            font_size: 21,
            color: if enabled { CREAM } else { MUTED },
            ..Default::default()
        },
    );
    enabled
        && ctx.pointer.released_on(rect)
        && ctx.origin.is_some_and(|origin| rect.contains(origin))
}

pub fn emblem(center: Vec2, radius: f32) {
    let top = center + vec2(0.0, -radius);
    let left = center + vec2(-radius * 0.65, -radius * 0.45);
    let right = center + vec2(radius * 0.65, -radius * 0.45);
    let tip = center + vec2(0.0, radius);
    draw_triangle(top, left, right, BRASS);
    draw_triangle(left, right, tip, BRASS);
    draw_triangle(
        center + vec2(0.0, -radius * 0.65),
        center + vec2(-radius * 0.42, -radius * 0.26),
        center + vec2(0.0, radius * 0.63),
        INK,
    );
    draw_line(
        center.x,
        center.y - radius * 0.45,
        center.x,
        center.y + radius * 0.45,
        2.0,
        CREAM,
    );
    for sign in [-1.0, 1.0] {
        for step in 0..4 {
            let y = center.y + radius * (0.45 - step as f32 * 0.30);
            draw_line(
                center.x + sign * radius * 0.95,
                y,
                center.x + sign * radius * 1.2,
                y - radius * 0.24,
                2.0,
                BRASS,
            );
        }
    }
}

pub fn horizontal_rule(center: Vec2, half_width: f32) {
    draw_line(
        center.x - half_width,
        center.y,
        center.x - 10.0,
        center.y,
        1.0,
        BRASS,
    );
    draw_line(
        center.x + 10.0,
        center.y,
        center.x + half_width,
        center.y,
        1.0,
        BRASS,
    );
    draw_poly(center.x, center.y, 4, 4.0, 0.0, BRASS);
}
