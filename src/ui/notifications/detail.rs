//! Recent receipts preserve their dated copy while live actions remain guarded.

use super::*;
use crate::ui::components::{body, button, centered, text, BRASS, CREAM, MUTED};
use kestrum::{
    data::world::SiteId,
    engine::VisibleCampaign,
    state::{
        military::FormationId,
        notifications::{
            NotificationDetail, NotificationPriority, NotificationSubjectSnapshot,
            PersonNotificationSnapshot,
        },
        people::PersonAssignment,
    },
};
use macroquad::prelude::*;
use macroquad_toolkit::ui::truncate_text_to_width_ex;

const RECENT_PAGE_SIZE: usize = 5;
const DETAIL_PAGE_LINES: usize = 7;
const RECENT_ROW_HEIGHT: f32 = 60.0;

pub(super) fn draw_card(
    ctx: &Context<'_>,
    projection: &NotificationProjection,
) -> Option<NotificationAction> {
    draw_surface();
    if let Some(action) = draw_header(ctx) {
        return Some(action);
    }
    match ctx.notifications.selected.as_ref() {
        Some(selected) => draw_selected(ctx, selected),
        None => draw_recent(ctx, projection),
    }
}

pub(super) fn draw_surface() {
    draw_rectangle(
        CARD.x,
        CARD.y,
        CARD.w,
        CARD.h,
        Color::new(0.035, 0.075, 0.075, 0.97),
    );
    draw_rectangle_lines(CARD.x, CARD.y, CARD.w, CARD.h, 1.0, BRASS);
    draw_rectangle(CARD.x, CARD.y, CARD.w, 3.0, BRASS);
}

pub(super) fn draw_header(ctx: &Context<'_>) -> Option<NotificationAction> {
    text(
        ctx,
        rules_term(ctx, "ui_notifications", "Notifications"),
        vec2(CARD.x + 16.0, CARD.y + 29.0),
        22.0,
        CREAM,
    );
    let close = Rect::new(CARD.right() - 94.0, CARD.y + 6.0, 82.0, 50.0);
    if button(
        ctx,
        close,
        rules_term(ctx, "ui_close", "Close"),
        true,
        false,
    ) {
        return Some(NotificationAction::Close);
    }
    for (tab, key, label, x) in [
        (
            NotificationTab::Recent,
            "ui_recent",
            "Recent",
            CARD.x + 14.0,
        ),
        (
            NotificationTab::Settings,
            "ui_settings",
            "Settings",
            CARD.x + 132.0,
        ),
    ] {
        if button(
            ctx,
            Rect::new(x, CARD.y + 48.0, 108.0, 50.0),
            rules_term(ctx, key, label),
            true,
            ctx.notifications.tab == tab,
        ) {
            return Some(NotificationAction::SetTab(tab));
        }
    }
    None
}

fn draw_recent(
    ctx: &Context<'_>,
    projection: &NotificationProjection,
) -> Option<NotificationAction> {
    let pages = projection.recent.len().div_ceil(RECENT_PAGE_SIZE).max(1);
    let page = ctx.notifications.recent_page.min(pages - 1);
    let items = projection
        .recent
        .iter()
        .skip(page * RECENT_PAGE_SIZE)
        .take(RECENT_PAGE_SIZE);
    for (row, item) in items.enumerate() {
        let rect = Rect::new(
            CARD.x + 12.0,
            CARD.y + 108.0 + row as f32 * RECENT_ROW_HEIGHT,
            CARD.w - 24.0,
            RECENT_ROW_HEIGHT - 4.0,
        );
        if button(ctx, rect, "", true, false) {
            return Some(NotificationAction::Open(item.id));
        }
        let title = truncate_text_to_width_ex(&item.title, rect.w - 34.0, ctx.body_font(), 17.0);
        body(ctx, &title, vec2(rect.x + 13.0, rect.y + 22.0), 17.0, CREAM);
        let status_key = if item.is_read { "ui_read" } else { "ui_unread" };
        let status = rules_term(
            ctx,
            status_key,
            if item.is_read { "Read" } else { "Unread" },
        );
        let season = rules_term(ctx, "ui_season", "Season {count}")
            .replace("{count}", &item.completed_rounds.to_string());
        let subtitle = format!("{season} - {status}");
        body(
            ctx,
            &subtitle,
            vec2(rect.x + 13.0, rect.y + 44.0),
            14.0,
            priority_color(item.priority),
        );
        if !item.is_read {
            draw_circle(
                rect.right() - 13.0,
                rect.y + 15.0,
                4.0,
                priority_color(item.priority),
            );
        }
    }

    if projection.recent.is_empty() {
        text(
            ctx,
            rules_term(ctx, "ui_no_recent_events", "No recent events"),
            vec2(CARD.x + 20.0, CARD.y + 148.0),
            20.0,
            CREAM,
        );
    }
    if let Some(action) = draw_omitted(ctx, projection) {
        return Some(action);
    }
    if let Some(action) = draw_recent_pager(ctx, page, pages) {
        return Some(action);
    }
    draw_recent_footer(ctx, projection)
}

fn draw_omitted(
    ctx: &Context<'_>,
    projection: &NotificationProjection,
) -> Option<NotificationAction> {
    if projection.omitted_count == 0 {
        return None;
    }
    let omitted = format!(
        "{}: {} - {} {}",
        rules_term(ctx, "ui_events_omitted", "Older events omitted"),
        projection.omitted_count,
        projection.omitted_unread_count,
        rules_term(ctx, "ui_unread", "unread")
    );
    let label = truncate_text_to_width_ex(&omitted, 290.0, ctx.body_font(), 14.0);
    body(
        ctx,
        &label,
        vec2(CARD.x + 16.0, CARD.y + 430.0),
        14.0,
        MUTED,
    );
    let records = Rect::new(CARD.right() - 142.0, CARD.y + 406.0, 126.0, 46.0);
    button(
        ctx,
        records,
        rules_term(ctx, "ui_records", "Records"),
        true,
        false,
    )
    .then_some(NotificationAction::OpenRecords)
}

fn draw_recent_pager(ctx: &Context<'_>, page: usize, pages: usize) -> Option<NotificationAction> {
    if pages <= 1 {
        return None;
    }
    let y = CARD.y + 454.0;
    if button(
        ctx,
        Rect::new(CARD.x + 14.0, y, 116.0, 48.0),
        rules_term(ctx, "ui_previous", "Previous"),
        page > 0,
        false,
    ) {
        return Some(NotificationAction::Page(-1));
    }
    centered(
        ctx,
        &format!("{}/{}", page + 1, pages),
        vec2(CARD.x + CARD.w * 0.5, y + 30.0),
        16.0,
        MUTED,
    );
    button(
        ctx,
        Rect::new(CARD.right() - 130.0, y, 116.0, 48.0),
        rules_term(ctx, "ui_next", "Next"),
        page + 1 < pages,
        false,
    )
    .then_some(NotificationAction::Page(1))
}

fn draw_recent_footer(
    ctx: &Context<'_>,
    projection: &NotificationProjection,
) -> Option<NotificationAction> {
    let y = CARD.bottom() - 58.0;
    if button(
        ctx,
        Rect::new(CARD.x + 14.0, y, 204.0, 48.0),
        rules_term(ctx, "ui_mark_all_read", "Mark all read"),
        projection.unread_count > 0,
        false,
    ) {
        return Some(NotificationAction::MarkAllRead);
    }
    button(
        ctx,
        Rect::new(CARD.x + 228.0, y, 238.0, 48.0),
        rules_term(ctx, "ui_dismiss_read", "Dismiss read"),
        true,
        false,
    )
    .then_some(NotificationAction::DismissRead)
}

fn draw_selected(ctx: &Context<'_>, item: &RenderedNotification) -> Option<NotificationAction> {
    if let Some(person) = person_snapshot(item) {
        draw_person_portrait(ctx, person);
        draw_person_identity(ctx, person);
    } else {
        draw_kind_marker(ctx, item);
    }
    draw_receipt_heading(ctx, item);
    if let Some(action) = draw_detail_page(ctx, item) {
        return Some(action);
    }
    if let Some(action) = draw_group_navigation(ctx) {
        return Some(action);
    }
    draw_subject_actions(ctx, item)
}

fn draw_person_portrait(ctx: &Context<'_>, person: &PersonNotificationSnapshot) {
    let rect = Rect::new(CARD.x + 18.0, CARD.y + 111.0, 84.0, 84.0);
    crate::ui::portraits::draw(
        ctx.portraits,
        person.appearance.as_ref(),
        person.age_years,
        ctx.household_rules.service_minimum_age_years,
        rect,
    );
}

fn draw_person_identity(ctx: &Context<'_>, person: &PersonNotificationSnapshot) {
    let class_key = format!("class_{:?}", person.class).to_ascii_lowercase();
    let class = ctx
        .notification_rules
        .terms
        .get(&class_key)
        .map(String::as_str)
        .unwrap_or("Person");
    let identity = truncate_text_to_width_ex(
        &format!("{} - {class}", person.name),
        356.0,
        ctx.body_font(),
        18.0,
    );
    body(
        ctx,
        &identity,
        vec2(CARD.x + 116.0, CARD.y + 176.0),
        18.0,
        BRASS,
    );
    if let Some(age) = person.age_years {
        body(
            ctx,
            &rules_term(ctx, "ui_age_at_event", "Age at event: {count}")
                .replace("{count}", &age.to_string()),
            vec2(CARD.x + 116.0, CARD.y + 200.0),
            15.0,
            MUTED,
        );
    }
}

fn draw_kind_marker(ctx: &Context<'_>, item: &RenderedNotification) {
    let center = vec2(CARD.x + 54.0, CARD.y + 158.0);
    let color = priority_color(item.priority);
    draw_circle(center.x, center.y, 25.0, color);
    centered(
        ctx,
        category_mark(item.category),
        center + vec2(0.0, 7.0),
        20.0,
        CREAM,
    );
}

fn draw_receipt_heading(ctx: &Context<'_>, item: &RenderedNotification) {
    let x = CARD.x + 116.0;
    let title = truncate_text_to_width_ex(&item.title, CARD.w - 132.0, ctx.body_font(), 19.0);
    body(ctx, &title, vec2(x, CARD.y + 121.0), 19.0, CREAM);
    let season = rules_term(ctx, "ui_season", "Season {count}")
        .replace("{count}", &item.completed_rounds.to_string());
    let subtitle = format!("{season} - {}", priority_name(ctx, item.priority));
    body(
        ctx,
        &subtitle,
        vec2(x, CARD.y + 150.0),
        14.0,
        priority_color(item.priority),
    );
}

fn draw_detail_page(ctx: &Context<'_>, item: &RenderedNotification) -> Option<NotificationAction> {
    let mut lines = wrap_text(ctx, &item.body, CARD.w - 40.0, 17.0);
    if item.kind.is_forecast() {
        if let Some(current) = ctx
            .notification_projection
            .and_then(|projection| projection.recent.iter().find(|entry| entry.id == item.id))
        {
            let state = if current.is_active {
                rules_term(ctx, "ui_active", "Active")
            } else {
                rules_term(ctx, "ui_cleared", "Cleared")
            };
            lines.extend(wrap_text(
                ctx,
                &format!(
                    "{}: {state}",
                    rules_term(ctx, "ui_current_condition", "Current condition")
                ),
                CARD.w - 40.0,
                17.0,
            ));
        }
    }
    let (map_available, route_available, _) = subject_actions(ctx, item);
    if item.subject.is_some() && !map_available && !route_available {
        let unavailable = rules_term(
            ctx,
            "ui_unknown_subject",
            "This event's subject is no longer available",
        );
        lines.extend(wrap_text(ctx, unavailable, CARD.w - 40.0, 17.0));
    }
    let pages = lines.len().div_ceil(DETAIL_PAGE_LINES).max(1);
    let page = ctx.notifications.detail_page.min(pages - 1);
    let top = if person_snapshot(item).is_some() {
        CARD.y + 224.0
    } else {
        CARD.y + 202.0
    };
    for (index, line) in lines
        .iter()
        .skip(page * DETAIL_PAGE_LINES)
        .take(DETAIL_PAGE_LINES)
        .enumerate()
    {
        body(
            ctx,
            line,
            vec2(CARD.x + 18.0, top + index as f32 * 22.0),
            16.0,
            CREAM,
        );
    }
    if pages > 1 {
        let y = CARD.y + 385.0;
        if button(
            ctx,
            Rect::new(CARD.x + 14.0, y, 116.0, 46.0),
            rules_term(ctx, "ui_previous", "Previous"),
            page > 0,
            false,
        ) {
            return Some(NotificationAction::Page(-1));
        }
        centered(
            ctx,
            &format!("{}/{}", page + 1, pages),
            vec2(CARD.x + CARD.w * 0.5, y + 29.0),
            15.0,
            MUTED,
        );
        if button(
            ctx,
            Rect::new(CARD.right() - 130.0, y, 116.0, 46.0),
            rules_term(ctx, "ui_next", "Next"),
            page + 1 < pages,
            false,
        ) {
            return Some(NotificationAction::Page(1));
        }
    }
    None
}

fn draw_group_navigation(ctx: &Context<'_>) -> Option<NotificationAction> {
    let ids = &ctx.notifications.group_ids;
    if ids.len() < 2 {
        return None;
    }
    let page = ctx.notifications.group_page.min(ids.len() - 1);
    let y = CARD.y + 434.0;
    if button(
        ctx,
        Rect::new(CARD.x + 14.0, y, 116.0, 46.0),
        rules_term(ctx, "ui_previous", "Previous"),
        page > 0,
        false,
    ) {
        return Some(NotificationAction::SelectGroupItem(ids[page - 1]));
    }
    centered(
        ctx,
        &format!(
            "{} {}/{}",
            rules_term(ctx, "ui_event", "Event"),
            page + 1,
            ids.len()
        ),
        vec2(CARD.x + CARD.w * 0.5, y + 28.0),
        15.0,
        MUTED,
    );
    button(
        ctx,
        Rect::new(CARD.right() - 130.0, y, 116.0, 46.0),
        rules_term(ctx, "ui_next", "Next"),
        page + 1 < ids.len(),
        false,
    )
    .then(|| NotificationAction::SelectGroupItem(ids[page + 1]))
}

fn draw_subject_actions(
    ctx: &Context<'_>,
    item: &RenderedNotification,
) -> Option<NotificationAction> {
    let (map_available, route_available, route_action) = subject_actions(ctx, item);
    let map_rect = Rect::new(CARD.x + 14.0, CARD.y + 490.0, 218.0, 48.0);
    let link_rect = Rect::new(CARD.x + 246.0, CARD.y + 490.0, 220.0, 48.0);
    if button(
        ctx,
        map_rect,
        rules_term(ctx, "ui_show_on_map", "Show on map"),
        map_available,
        true,
    ) {
        return Some(NotificationAction::ShowOnMap(item.id));
    }
    let link_label = if route_action == Some(NotificationAction::OpenRecords) {
        rules_term(ctx, "ui_records", "Records")
    } else {
        follow_label(ctx, item)
    };
    if button(ctx, link_rect, link_label, route_available, false) {
        return route_action;
    }
    let y = CARD.y + 548.0;
    if button(
        ctx,
        Rect::new(CARD.x + 14.0, y, 218.0, 48.0),
        rules_term(ctx, "ui_dismiss", "Dismiss"),
        true,
        false,
    ) {
        return Some(NotificationAction::Dismiss(item.id));
    }
    button(
        ctx,
        Rect::new(CARD.x + 246.0, y, 220.0, 48.0),
        rules_term(ctx, "ui_settings", "Settings"),
        true,
        false,
    )
    .then_some(NotificationAction::SetTab(NotificationTab::Settings))
}

fn subject_actions(
    ctx: &Context<'_>,
    item: &RenderedNotification,
) -> (bool, bool, Option<NotificationAction>) {
    let Some(view) = ctx.campaign_view else {
        return (false, false, None);
    };
    match item.subject.as_ref() {
        Some(NotificationSubjectSnapshot::Person(person)) => {
            let current = view
                .people
                .iter()
                .any(|current| current.id == person.id && current.faction == view.player);
            (
                current && person_site(view, person.id).is_some(),
                current,
                Some(NotificationAction::FollowSubject(item.id)),
            )
        }
        Some(NotificationSubjectSnapshot::Place(place)) => {
            let current = view.world.site(place.id);
            let known = current.is_some();
            let managed = current.is_some_and(|site| site.controller == Some(view.player));
            (
                known,
                managed,
                Some(NotificationAction::FollowSubject(item.id)),
            )
        }
        Some(NotificationSubjectSnapshot::Army(army)) => {
            let current = view
                .armies
                .iter()
                .find(|candidate| candidate.id == army.id && candidate.faction == view.player);
            (
                current.is_some_and(|candidate| view.world.site(candidate.site).is_some()),
                current.is_some(),
                Some(NotificationAction::FollowSubject(item.id)),
            )
        }
        Some(NotificationSubjectSnapshot::Construction(work)) => {
            let current = view
                .construction
                .iter()
                .find(|order| order.id == work.id && order.owner == view.player);
            match current.map(|order| order.target) {
                Some(kestrum::state::construction::ConstructionTarget::Site(site)) => {
                    let known = view.world.site(site).is_some();
                    let managed = view
                        .world
                        .site(site)
                        .is_some_and(|site| site.controller == Some(view.player));
                    (
                        known,
                        managed,
                        Some(NotificationAction::FollowSubject(item.id)),
                    )
                }
                Some(kestrum::state::construction::ConstructionTarget::Route(route)) => (
                    route_site(view, route).is_some(),
                    current.is_some(),
                    Some(NotificationAction::OpenRecords),
                ),
                None => (false, false, Some(NotificationAction::OpenRecords)),
            }
        }
        Some(NotificationSubjectSnapshot::Battle(battle)) => {
            let current = view.battles.iter().find(|entry| entry.id == battle.id);
            (
                current.is_some_and(|entry| view.world.site(entry.site).is_some()),
                true,
                Some(if current.is_some() {
                    NotificationAction::FollowSubject(item.id)
                } else {
                    NotificationAction::OpenRecords
                }),
            )
        }
        Some(NotificationSubjectSnapshot::History { .. }) | None => {
            (false, true, Some(NotificationAction::OpenRecords))
        }
        Some(NotificationSubjectSnapshot::Route(route)) => {
            let live = view
                .world
                .routes
                .iter()
                .any(|current| current.id == route.id);
            let known = live
                && (view.world.site(route.from.id).is_some()
                    || view.world.site(route.to.id).is_some());
            (known, true, Some(NotificationAction::OpenRecords))
        }
        Some(NotificationSubjectSnapshot::Faction { .. }) => {
            (false, true, Some(NotificationAction::OpenRecords))
        }
    }
}

fn route_site(view: &VisibleCampaign, route: kestrum::data::world::RouteId) -> Option<SiteId> {
    let route = view
        .world
        .routes
        .iter()
        .find(|current| current.id == route)?;
    [route.from, route.to]
        .into_iter()
        .find(|site| view.world.site(*site).is_some())
}

fn follow_label<'a>(ctx: &'a Context<'_>, item: &RenderedNotification) -> &'a str {
    match item.subject.as_ref() {
        Some(NotificationSubjectSnapshot::Person(_)) => {
            rules_term(ctx, "ui_view_person", "View person")
        }
        Some(NotificationSubjectSnapshot::Place(_))
        | Some(NotificationSubjectSnapshot::Construction(_)) => {
            rules_term(ctx, "ui_manage_place", "Manage place")
        }
        _ => rules_term(ctx, "ui_records", "Records"),
    }
}

fn person_site(view: &VisibleCampaign, id: kestrum::state::people::PersonId) -> Option<SiteId> {
    let person = view
        .people
        .iter()
        .find(|person| person.id == id && person.faction == view.player)?;
    let site = match person.assignment {
        PersonAssignment::Site { site }
        | PersonAssignment::Dependent { site }
        | PersonAssignment::Trainee { site } => site,
        PersonAssignment::Formation { formation } => army_site(view, formation)?,
        PersonAssignment::Dead => return None,
    };
    view.world.site(site).map(|site| site.id)
}

fn army_site(view: &VisibleCampaign, formation: FormationId) -> Option<SiteId> {
    view.armies
        .iter()
        .find(|army| army.faction == view.player && army.formation_ids().any(|id| id == formation))
        .map(|army| army.site)
}

fn person_snapshot(item: &RenderedNotification) -> Option<&PersonNotificationSnapshot> {
    match &item.detail {
        NotificationDetail::Person { person, .. } => Some(person),
        _ => match item.subject.as_ref()? {
            NotificationSubjectSnapshot::Person(person) => Some(person),
            _ => None,
        },
    }
}

fn wrap_text(ctx: &Context<'_>, value: &str, width: f32, size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in value.lines() {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if measure_text(&candidate, ctx.body_font(), size as u16, 1.0).width > width
                && !line.is_empty()
            {
                lines.push(line);
                line = word.to_owned();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
    }
    lines
}

fn category_mark(category: kestrum::state::notifications::NotificationCategory) -> &'static str {
    use kestrum::state::notifications::NotificationCategory::*;
    match category {
        People => "P",
        Places => "L",
        Security | Military => "!",
        Orders => "W",
        EconomyDiplomacy => "$",
        Remembrance => "✦",
    }
}

fn priority_name<'a>(ctx: &'a Context<'_>, priority: NotificationPriority) -> &'a str {
    use NotificationPriority::*;
    let (key, fallback) = match priority {
        Urgent => ("ui_urgent", "Urgent"),
        Warning => ("ui_warning", "Warning"),
        Information => ("ui_information", "Information"),
        History => ("ui_history", "History"),
    };
    rules_term(ctx, key, fallback)
}

pub(super) fn priority_color(priority: NotificationPriority) -> Color {
    use NotificationPriority::*;
    match priority {
        Urgent => Color::new(0.91, 0.39, 0.28, 1.0),
        Warning => Color::new(0.91, 0.66, 0.30, 1.0),
        Information => BRASS,
        History => MUTED,
    }
}
