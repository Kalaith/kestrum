//! Small portrait frames and neutral silhouettes for unavailable art.

use crate::game::portraits::{PortraitCache, PortraitDraw, PortraitFallback};
use kestrum::data::portraits::AppearanceDescriptor;
use macroquad::prelude::{draw_ellipse, draw_rectangle, draw_rectangle_lines, Color, Rect};

const FRAME: Color = Color::new(0.07, 0.12, 0.12, 0.96);
const BORDER: Color = Color::new(0.52, 0.46, 0.30, 0.85);
const SILHOUETTE: Color = Color::new(0.42, 0.47, 0.43, 0.94);

pub fn draw(
    cache: &PortraitCache,
    appearance: Option<&AppearanceDescriptor>,
    age_years: Option<u32>,
    child_threshold: u32,
    bounds: Rect,
) -> PortraitDraw {
    draw_result(
        cache.draw(appearance, age_years, child_threshold, bounds),
        bounds,
    )
}

pub fn draw_adult(
    cache: &PortraitCache,
    appearance: &AppearanceDescriptor,
    bounds: Rect,
) -> PortraitDraw {
    draw_result(cache.draw_adult(appearance, bounds), bounds)
}

fn draw_result(result: PortraitDraw, bounds: Rect) -> PortraitDraw {
    if let PortraitDraw::Fallback {
        kind,
        asset_drawn: false,
    } = result
    {
        neutral_silhouette(kind, bounds);
    }
    result
}

fn neutral_silhouette(kind: PortraitFallback, bounds: Rect) {
    if bounds.w <= 0.0 || bounds.h <= 0.0 {
        return;
    }
    draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, FRAME);
    draw_rectangle_lines(
        bounds.x,
        bounds.y,
        bounds.w,
        bounds.h,
        (bounds.w.min(bounds.h) / 32.0).clamp(1.0, 2.0),
        BORDER,
    );
    let scale = bounds.w.min(bounds.h);
    let (head_scale, head_y) = if kind == PortraitFallback::Child {
        (0.31, 0.35)
    } else {
        (0.27, 0.32)
    };
    let head_w = scale * head_scale;
    let head_h = scale * head_scale * 1.18;
    draw_ellipse(
        bounds.x + bounds.w * 0.5,
        bounds.y + bounds.h * head_y,
        head_w,
        head_h,
        0.0,
        SILHOUETTE,
    );
    draw_ellipse(
        bounds.x + bounds.w * 0.5,
        bounds.y + bounds.h * 0.80,
        bounds.w * 0.44,
        bounds.h * 0.13,
        0.0,
        SILHOUETTE,
    );
}
