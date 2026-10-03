//! The shallow, stable-position notification targets below the campaign header.

use super::*;
use crate::ui::components::{body, button, centered, BRASS, CREAM, INK, MUTED};
use kestrum::state::notifications::{NotificationCategory, NotificationPriority};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::truncate_text_to_width_ex;

pub(super) fn draw(
    ctx: &Context<'_>,
    projection: Option<&NotificationProjection>,
) -> Option<NotificationAction> {
    let (groups, overflow) = groups_for_rail(ctx.notifications, projection);
    let has_more = overflow > 0;
    let width = rail_width(groups.len(), has_more);
    draw_rectangle(
        RAIL_X - 8.0,
        RAIL_Y,
        width + 16.0,
        RAIL_HEIGHT,
        with_alpha(INK, 0.72),
    );

    let mut x = RAIL_X;
    for group in groups.iter().take(RAIL_MAX_GROUPS) {
        let rect = Rect::new(x, RAIL_Y + 4.0, RAIL_BUTTON, RAIL_BUTTON);
        if button(ctx, rect, "", true, false) {
            let anchor = group.receipt_ids.first().copied()?;
            return Some(if group.receipt_ids.len() > 1 {
                NotificationAction::OpenGroup(anchor)
            } else {
                NotificationAction::Open(anchor)
            });
        }
        draw_group_symbol(rect, group.category, group.kind, group.priority);
        if group.receipt_ids.len() > 1 {
            body(
                ctx,
                &group.receipt_ids.len().to_string(),
                vec2(rect.right() - 17.0, rect.y + 16.0),
                13.0,
                CREAM,
            );
        }
        if group.unread_count > 0 {
            draw_circle(
                rect.right() - 8.0,
                rect.y + 8.0,
                4.5,
                priority_color(group.priority),
            );
            draw_circle_lines(
                rect.right() - 8.0,
                rect.y + 8.0,
                4.5,
                1.0,
                Color::new(0.04, 0.07, 0.07, 1.0),
            );
        }
        x += RAIL_BUTTON + RAIL_GAP;
    }

    if overflow > 0 {
        let rect = Rect::new(x, RAIL_Y + 4.0, MORE_WIDTH, RAIL_BUTTON);
        let label = format!("{} ({overflow})", rules_term(ctx, "ui_more", "More"));
        if button(ctx, rect, &label, true, false) {
            return Some(NotificationAction::SetTab(NotificationTab::Recent));
        }
        x += MORE_WIDTH + RAIL_GAP;
    }

    let rect = Rect::new(x, RAIL_Y + 4.0, NOTIFICATIONS_WIDTH, RAIL_BUTTON);
    let unread_count = projection.map_or(0, |projection| projection.top_bar_unread_count);
    let label = rules_term(ctx, "ui_notifications", "Notifications");
    if button(ctx, rect, "", true, ctx.notifications.is_open) {
        return Some(NotificationAction::Toggle);
    }
    let label_space = rect.w - if unread_count > 0 { 48.0 } else { 0.0 };
    let label = truncate_text_to_width_ex(label, label_space - 16.0, ctx.font(), 21.0);
    centered(
        ctx,
        &label,
        vec2(rect.x + label_space * 0.5, rect.y + 41.0),
        21.0,
        CREAM,
    );
    if unread_count > 0 {
        let count = unread_count.min(99);
        let unread_priority = projection
            .and_then(|projection| {
                projection
                    .rail
                    .iter()
                    .filter(|group| group.unread_count > 0)
                    .max_by_key(|group| priority_rank(group.priority))
                    .map(|group| group.priority)
            })
            .unwrap_or(NotificationPriority::Information);
        let badge = format!("{count}");
        let width = measure_text(&badge, ctx.body_font(), 15, 1.0).width;
        let badge_rect = Rect::new(
            rect.right() - width - 18.0,
            rect.y + 5.0,
            width + 12.0,
            22.0,
        );
        draw_rectangle(
            badge_rect.x,
            badge_rect.y,
            badge_rect.w,
            badge_rect.h,
            priority_color(unread_priority),
        );
        body(
            ctx,
            &badge,
            vec2(badge_rect.x + 6.0, badge_rect.y + 16.0),
            15.0,
            INK,
        );
    }
    None
}

fn priority_rank(priority: NotificationPriority) -> u8 {
    match priority {
        NotificationPriority::Urgent => 4,
        NotificationPriority::Warning => 3,
        NotificationPriority::Information => 2,
        NotificationPriority::History => 1,
    }
}

fn draw_group_symbol(
    rect: Rect,
    category: NotificationCategory,
    kind: NotificationKind,
    priority: NotificationPriority,
) {
    let color = priority_color(priority);
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    match kind {
        NotificationKind::ConstructionCompleted
        | NotificationKind::ConstructionExpected
        | NotificationKind::ConstructionBlocked
        | NotificationKind::ConstructionResumed
        | NotificationKind::ConstructionLost => draw_work_icon(center, color),
        _ => match category {
            NotificationCategory::People => draw_person_icon(center, color),
            NotificationCategory::Places => draw_place_icon(center, color),
            NotificationCategory::Security | NotificationCategory::Military => {
                draw_shield_icon(center, color)
            }
            NotificationCategory::Orders => draw_route_icon(center, color),
            NotificationCategory::EconomyDiplomacy => draw_coin_icon(center, color),
            NotificationCategory::Remembrance => draw_star_icon(center, color),
        },
    }
}

fn draw_person_icon(center: Vec2, color: Color) {
    draw_circle(center.x, center.y - 7.0, 7.0, color);
    draw_ellipse(center.x, center.y + 12.0, 16.0, 9.0, 0.0, color);
}

fn draw_place_icon(center: Vec2, color: Color) {
    draw_rectangle(center.x - 14.0, center.y - 2.0, 28.0, 22.0, color);
    draw_triangle(
        center + vec2(-18.0, -2.0),
        center + vec2(0.0, -20.0),
        center + vec2(18.0, -2.0),
        color,
    );
    draw_rectangle(center.x - 3.0, center.y + 7.0, 6.0, 13.0, INK);
}

fn draw_work_icon(center: Vec2, color: Color) {
    draw_rectangle(center.x - 13.0, center.y - 12.0, 26.0, 25.0, color);
    draw_rectangle(center.x - 5.0, center.y - 6.0, 10.0, 14.0, INK);
    draw_line(
        center.x - 17.0,
        center.y + 17.0,
        center.x + 17.0,
        center.y + 17.0,
        2.0,
        color,
    );
}

fn draw_shield_icon(center: Vec2, color: Color) {
    draw_poly(center.x, center.y, 5, 19.0, 36.0, color);
    draw_line(center.x, center.y - 8.0, center.x, center.y + 7.0, 3.0, INK);
    draw_circle(center.x, center.y + 13.0, 1.8, INK);
}

fn draw_route_icon(center: Vec2, color: Color) {
    draw_line(
        center.x - 15.0,
        center.y + 12.0,
        center.x - 3.0,
        center.y - 11.0,
        3.0,
        color,
    );
    draw_line(
        center.x - 3.0,
        center.y - 11.0,
        center.x + 11.0,
        center.y + 4.0,
        3.0,
        color,
    );
    draw_triangle(
        center + vec2(7.0, 4.0),
        center + vec2(17.0, 4.0),
        center + vec2(12.0, 13.0),
        color,
    );
    draw_circle(center.x - 15.0, center.y + 12.0, 4.0, CREAM);
}

fn draw_coin_icon(center: Vec2, color: Color) {
    draw_circle(center.x, center.y, 16.0, color);
    draw_circle_lines(center.x, center.y, 10.0, 2.0, INK);
    draw_line(center.x, center.y - 8.0, center.x, center.y + 8.0, 2.0, INK);
}

fn draw_star_icon(center: Vec2, color: Color) {
    draw_poly(center.x, center.y, 5, 17.0, 0.0, color);
    draw_poly(center.x, center.y, 5, 7.0, 36.0, INK);
}

fn priority_color(priority: NotificationPriority) -> Color {
    match priority {
        NotificationPriority::Urgent => Color::new(0.91, 0.39, 0.28, 1.0),
        NotificationPriority::Warning => Color::new(0.91, 0.66, 0.30, 1.0),
        NotificationPriority::Information => BRASS,
        NotificationPriority::History => MUTED,
    }
}
