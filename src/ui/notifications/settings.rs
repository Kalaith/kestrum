//! Campaign and title-menu delivery controls share the same preference choices.

use super::*;
use crate::ui::components::{body, button, centered, BRASS, CREAM, MUTED};
use kestrum::state::notifications::{NotificationDelivery, NotificationKind};
use macroquad::prelude::*;
use macroquad_toolkit::ui::truncate_text_to_width_ex;

const CARD_KINDS_PER_PAGE: usize = 3;
const GLOBAL_KINDS_PER_PAGE: usize = 2;

pub(super) fn draw_card(ctx: &Context<'_>) -> Option<NotificationAction> {
    super::detail::draw_surface();
    if let Some(action) = super::detail::draw_header(ctx) {
        return Some(action);
    }
    if let Some(action) = draw_categories(ctx) {
        return Some(action);
    }
    if let Some(action) = draw_card_bulk_actions(ctx) {
        return Some(action);
    }
    let kinds = visible_kinds(ctx.notifications.settings_category);
    let pages = kinds.len().div_ceil(CARD_KINDS_PER_PAGE).max(1);
    let page = ctx.notifications.settings_page.min(pages - 1);
    for (offset, kind) in kinds
        .iter()
        .skip(page * CARD_KINDS_PER_PAGE)
        .take(CARD_KINDS_PER_PAGE)
        .enumerate()
    {
        let row_y = CARD.y + 260.0 + offset as f32 * 74.0;
        if let Some(action) = draw_kind_row(ctx, *kind, row_y, CARD.x + 14.0, CARD.w - 28.0) {
            return Some(action);
        }
    }
    draw_pager(
        ctx,
        page,
        pages,
        Rect::new(CARD.x + 14.0, CARD.y + 492.0, CARD.w - 28.0, 48.0),
    )
}

pub(super) fn draw_global(ctx: &Context<'_>) -> Option<NotificationAction> {
    let panel = Rect::new(378.0, 104.0, 524.0, 512.0);
    centered(
        ctx,
        rules_term(ctx, "ui_notification_settings", "Notification delivery"),
        vec2(panel.x + panel.w * 0.5, panel.y + 111.0),
        19.0,
        BRASS,
    );
    if let Some(action) = draw_global_category(ctx, panel) {
        return Some(action);
    }
    let kinds = visible_kinds(ctx.notifications.settings_category);
    let pages = kinds.len().div_ceil(GLOBAL_KINDS_PER_PAGE).max(1);
    let page = ctx.notifications.settings_page.min(pages - 1);
    for (offset, kind) in kinds
        .iter()
        .skip(page * GLOBAL_KINDS_PER_PAGE)
        .take(GLOBAL_KINDS_PER_PAGE)
        .enumerate()
    {
        let y = panel.y + 178.0 + offset as f32 * 68.0;
        if let Some(action) = draw_kind_row(ctx, *kind, y, panel.x + 18.0, panel.w - 36.0) {
            return Some(action);
        }
    }
    if let Some(action) = draw_pager(
        ctx,
        page,
        pages,
        Rect::new(panel.x + 18.0, panel.y + 315.0, panel.w - 36.0, 44.0),
    ) {
        return Some(action);
    }
    if let Some(action) = draw_bulk_actions(
        ctx,
        Rect::new(panel.x + 18.0, panel.y + 372.0, panel.w - 36.0, 46.0),
    ) {
        return Some(action);
    }
    button(
        ctx,
        Rect::new(panel.x + 144.0, panel.bottom() - 66.0, 236.0, 48.0),
        rules_term(ctx, "ui_back", "Back"),
        true,
        false,
    )
    .then_some(NotificationAction::CloseGlobalSettings)
}

fn draw_categories(ctx: &Context<'_>) -> Option<NotificationAction> {
    let width = (CARD.w - 34.0) / 4.0;
    for (index, category) in categories().into_iter().enumerate() {
        let row = index / 4;
        let col = index % 4;
        let rect = Rect::new(
            CARD.x + 12.0 + col as f32 * (width + 2.0),
            CARD.y + 108.0 + row as f32 * 46.0,
            width,
            44.0,
        );
        let label = truncate_text_to_width_ex(
            category_short_name(ctx, category),
            rect.w - 8.0,
            ctx.font(),
            18.0,
        );
        let selected = ctx.notifications.settings_category == category;
        let clicked = button(ctx, rect, "", true, selected);
        centered(
            ctx,
            &label,
            vec2(rect.x + rect.w * 0.5, rect.y + 29.0),
            18.0,
            CREAM,
        );
        if clicked {
            return Some(NotificationAction::ToggleSettingsCategory(category));
        }
    }
    None
}

fn draw_card_bulk_actions(ctx: &Context<'_>) -> Option<NotificationAction> {
    draw_bulk_actions(
        ctx,
        Rect::new(CARD.x + 14.0, CARD.y + 205.0, CARD.w - 28.0, 46.0),
    )
}

fn draw_bulk_actions(ctx: &Context<'_>, row: Rect) -> Option<NotificationAction> {
    let gap = 10.0;
    let width = (row.w - gap) * 0.5;
    if button(
        ctx,
        Rect::new(row.x, row.y, width, row.h),
        rules_term(ctx, "ui_all_off", "All off"),
        true,
        false,
    ) {
        return Some(NotificationAction::AllOff);
    }
    button(
        ctx,
        Rect::new(row.x + width + gap, row.y, width, row.h),
        rules_term(ctx, "ui_restore_defaults", "Restore defaults"),
        true,
        false,
    )
    .then_some(NotificationAction::RestoreDefaults)
}

fn draw_global_category(ctx: &Context<'_>, panel: Rect) -> Option<NotificationAction> {
    let options = categories();
    let current = options
        .iter()
        .position(|category| *category == ctx.notifications.settings_category)
        .unwrap_or(0);
    let y = panel.y + 126.0;
    if button(
        ctx,
        Rect::new(panel.x + 18.0, y, 104.0, 48.0),
        rules_term(ctx, "ui_previous", "Previous"),
        current > 0,
        false,
    ) {
        return Some(NotificationAction::ToggleSettingsCategory(
            options[current - 1],
        ));
    }
    let label = category_name(ctx, options[current]);
    centered(
        ctx,
        label,
        vec2(panel.x + panel.w * 0.5, y + 30.0),
        17.0,
        CREAM,
    );
    button(
        ctx,
        Rect::new(panel.right() - 122.0, y, 104.0, 48.0),
        rules_term(ctx, "ui_next", "Next"),
        current + 1 < options.len(),
        false,
    )
    .then(|| NotificationAction::ToggleSettingsCategory(options[current + 1]))
}

fn draw_kind_row(
    ctx: &Context<'_>,
    kind: NotificationKind,
    y: f32,
    x: f32,
    width: f32,
) -> Option<NotificationAction> {
    let label = ctx
        .notification_rules
        .kind(kind)
        .map(|rule| rule.label.as_str())
        .unwrap_or("Notification type");
    let name = truncate_text_to_width_ex(label, width - 274.0, ctx.body_font(), 15.0);
    body(ctx, &name, vec2(x, y + 20.0), 15.0, CREAM);
    let start = x + width - 262.0;
    let labels = [
        (
            NotificationDelivery::TopBarAndHistory,
            "ui_top_bar_and_history",
            "Top + history",
        ),
        (
            NotificationDelivery::HistoryOnly,
            "ui_history_only",
            "History",
        ),
        (NotificationDelivery::Off, "ui_off", "Off"),
    ];
    let selected = ctx
        .preferences
        .notifications
        .choice(kind, ctx.notification_rules);
    for (index, (delivery, key, fallback)) in labels.into_iter().enumerate() {
        let rect = Rect::new(start + index as f32 * 88.0, y, 84.0, 44.0);
        if compact_delivery_button(
            ctx,
            rect,
            rules_term(ctx, key, fallback),
            selected == delivery,
        ) {
            return Some(NotificationAction::SetDelivery(kind, delivery));
        }
    }
    None
}

fn compact_delivery_button(ctx: &Context<'_>, rect: Rect, label: &str, selected: bool) -> bool {
    let hovered = ctx.pointer.hovering_over(rect);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if selected {
            Color::new(0.22, 0.29, 0.25, 0.98)
        } else if hovered {
            Color::new(0.16, 0.22, 0.21, 0.94)
        } else {
            Color::new(0.05, 0.10, 0.10, 0.52)
        },
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.0,
        if selected { BRASS } else { MUTED },
    );
    const LABEL_SIZE: u16 = 14;
    let lines = delivery_label_lines(ctx, label, rect.w - 8.0, LABEL_SIZE);
    let line_height = LABEL_SIZE as f32 + 2.0;
    let total_height = lines.len() as f32 * line_height;
    let first_baseline = rect.y + (rect.h - total_height) * 0.5 + LABEL_SIZE as f32;
    for (index, line) in lines.iter().enumerate() {
        let width = measure_text(line, ctx.body_font(), LABEL_SIZE, 1.0).width;
        body(
            ctx,
            line,
            vec2(
                rect.x + (rect.w - width) * 0.5,
                first_baseline + index as f32 * line_height,
            ),
            LABEL_SIZE as f32,
            CREAM,
        );
    }
    ctx.pointer.released_on(rect) && ctx.origin.is_some_and(|origin| rect.contains(origin))
}

fn delivery_label_lines(ctx: &Context<'_>, label: &str, max_width: f32, size: u16) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in label.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty()
            && measure_text(&candidate, ctx.body_font(), size, 1.0).width > max_width
        {
            lines.push(std::mem::take(&mut line));
        }
        let word = if measure_text(word, ctx.body_font(), size, 1.0).width > max_width {
            truncate_text_to_width_ex(word, max_width, ctx.body_font(), size as f32)
        } else {
            word.to_owned()
        };
        if line.is_empty() {
            line = word;
        } else {
            line.push(' ');
            line.push_str(&word);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

fn draw_pager(
    ctx: &Context<'_>,
    page: usize,
    pages: usize,
    bounds: Rect,
) -> Option<NotificationAction> {
    if pages <= 1 {
        return None;
    }
    let button_w = 116.0;
    if button(
        ctx,
        Rect::new(bounds.x, bounds.y, button_w, bounds.h),
        rules_term(ctx, "ui_previous", "Previous"),
        page > 0,
        false,
    ) {
        return Some(NotificationAction::Page(-1));
    }
    centered(
        ctx,
        &format!("{}/{}", page + 1, pages),
        vec2(bounds.x + bounds.w * 0.5, bounds.y + bounds.h * 0.62),
        15.0,
        MUTED,
    );
    button(
        ctx,
        Rect::new(bounds.right() - button_w, bounds.y, button_w, bounds.h),
        rules_term(ctx, "ui_next", "Next"),
        page + 1 < pages,
        false,
    )
    .then_some(NotificationAction::Page(1))
}

fn visible_kinds(selected: NotificationSettingsCategory) -> Vec<NotificationKind> {
    let wanted = category_for(selected);
    NotificationKind::ALL
        .into_iter()
        .filter(|kind| wanted.is_none_or(|category| kind.category() == category))
        .collect()
}

fn category_name<'a>(ctx: &'a Context<'_>, category: NotificationSettingsCategory) -> &'a str {
    if let Some(category) = category_for(category) {
        ctx.notification_rules
            .categories
            .get(&category)
            .map(String::as_str)
            .unwrap_or("Events")
    } else {
        rules_term(ctx, "ui_all_notification_types", "All types")
    }
}

pub(super) fn category_short_name<'a>(
    ctx: &'a Context<'_>,
    category: NotificationSettingsCategory,
) -> &'a str {
    let (key, fallback) = match category {
        NotificationSettingsCategory::All => ("ui_category_all_short", "All"),
        NotificationSettingsCategory::People => ("ui_category_people_short", "People"),
        NotificationSettingsCategory::Places => ("ui_category_places_short", "Places"),
        NotificationSettingsCategory::Security => ("ui_category_security_short", "Security"),
        NotificationSettingsCategory::Orders => ("ui_category_orders_short", "Orders"),
        NotificationSettingsCategory::Military => ("ui_category_military_short", "Military"),
        NotificationSettingsCategory::EconomyDiplomacy => {
            ("ui_category_economy_diplomacy_short", "Economy")
        }
        NotificationSettingsCategory::Remembrance => ("ui_category_remembrance_short", "Legacy"),
    };
    rules_term(ctx, key, fallback)
}
