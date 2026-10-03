//! Compact, map-local notification rail and dated receipt card.

mod detail;
mod rail;
mod settings;

use crate::ui::Context;
use kestrum::state::notifications::{
    NotificationCategory, NotificationDelivery, NotificationGroup, NotificationId,
    NotificationKind, NotificationProjection, RenderedNotification,
};
use macroquad::prelude::{Rect, Vec2};

pub use kestrum::state::notifications::{NotificationSettingsCategory, NotificationTab};

const RAIL_X: f32 = 24.0;
const RAIL_Y: f32 = 96.0;
const RAIL_HEIGHT: f32 = 64.0;
const RAIL_BUTTON: f32 = 56.0;
const RAIL_GAP: f32 = 8.0;
const RAIL_MAX_GROUPS: usize = 8;
const MORE_WIDTH: f32 = 92.0;
const NOTIFICATIONS_WIDTH: f32 = 264.0;
const CARD: Rect = Rect::new(24.0, 160.0, 480.0, 620.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationAction {
    Toggle,
    Close,
    SetTab(NotificationTab),
    Open(NotificationId),
    OpenGroup(NotificationId),
    SelectGroupItem(NotificationId),
    Page(i32),
    OpenRecords,
    OpenGlobalSettings,
    CloseGlobalSettings,
    MarkAllRead,
    Dismiss(NotificationId),
    DismissRead,
    ShowOnMap(NotificationId),
    FollowSubject(NotificationId),
    SetDelivery(NotificationKind, NotificationDelivery),
    ToggleSettingsCategory(NotificationSettingsCategory),
    AllOff,
    RestoreDefaults,
}

#[derive(Debug, Clone)]
pub struct NotificationReturnContext {
    pub tab: NotificationTab,
    pub selected_id: Option<NotificationId>,
    pub selected: Option<RenderedNotification>,
    pub group_ids: Vec<NotificationId>,
    pub group_snapshots: Vec<RenderedNotification>,
    pub group_page: usize,
    pub recent_page: usize,
    pub detail_page: usize,
}

#[derive(Debug, Clone)]
pub struct NotificationSession {
    pub is_open: bool,
    pub tab: NotificationTab,
    pub selected_id: Option<NotificationId>,
    pub selected: Option<RenderedNotification>,
    pub group_ids: Vec<NotificationId>,
    pub group_snapshots: Vec<RenderedNotification>,
    pub group_page: usize,
    pub recent_page: usize,
    pub detail_page: usize,
    pub settings_category: NotificationSettingsCategory,
    pub settings_page: usize,
    pub global_settings_open: bool,
    pub return_context: Option<NotificationReturnContext>,
    pub pointer_captured: bool,
    pub release_pending: bool,
    pub frozen_projection: Option<NotificationProjection>,
}

impl Default for NotificationSession {
    fn default() -> Self {
        Self {
            is_open: false,
            tab: NotificationTab::Recent,
            selected_id: None,
            selected: None,
            group_ids: Vec::new(),
            group_snapshots: Vec::new(),
            group_page: 0,
            recent_page: 0,
            detail_page: 0,
            settings_category: NotificationSettingsCategory::All,
            settings_page: 0,
            global_settings_open: false,
            return_context: None,
            pointer_captured: false,
            release_pending: false,
            frozen_projection: None,
        }
    }
}

impl NotificationSession {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn visible_projection<'a>(
        &'a self,
        current: Option<&'a NotificationProjection>,
    ) -> Option<&'a NotificationProjection> {
        self.frozen_projection.as_ref().or(current)
    }

    pub fn remember_return_context(&mut self) {
        if self.return_context.is_none() {
            self.return_context = Some(NotificationReturnContext {
                tab: self.tab,
                selected_id: self.selected_id,
                selected: self.selected.clone(),
                group_ids: self.group_ids.clone(),
                group_snapshots: self.group_snapshots.clone(),
                group_page: self.group_page,
                recent_page: self.recent_page,
                detail_page: self.detail_page,
            });
        }
    }

    pub fn restore_return_context(&mut self) -> bool {
        let Some(context) = self.return_context.take() else {
            return false;
        };
        self.is_open = true;
        self.tab = context.tab;
        self.selected_id = context.selected_id;
        self.selected = context.selected;
        self.group_ids = context.group_ids;
        self.group_snapshots = context.group_snapshots;
        self.group_page = context.group_page;
        self.recent_page = context.recent_page;
        self.detail_page = context.detail_page;
        true
    }

    pub fn capture_rail_gesture(&mut self, projection: Option<&NotificationProjection>) {
        self.frozen_projection = projection.cloned();
        self.pointer_captured = true;
        self.release_pending = false;
    }

    pub fn finish_pointer_gesture(&mut self) {
        self.pointer_captured = false;
        self.release_pending = true;
    }

    pub fn clear_finished_gesture(&mut self) {
        if self.release_pending {
            self.release_pending = false;
            self.frozen_projection = None;
        }
    }

    pub fn snapshots_for_group(
        &self,
        group: &NotificationGroup,
        projection: Option<&NotificationProjection>,
    ) -> Vec<RenderedNotification> {
        let Some(projection) = self.visible_projection(projection) else {
            return Vec::new();
        };
        group
            .receipt_ids
            .iter()
            .filter_map(|id| {
                projection
                    .recent
                    .iter()
                    .find(|item| item.id == *id)
                    .cloned()
            })
            .collect()
    }

    pub fn select_group_item(&mut self, id: NotificationId) -> bool {
        let Some(index) = self.group_ids.iter().position(|candidate| *candidate == id) else {
            return false;
        };
        let Some(item) = self.group_snapshots.get(index).cloned() else {
            return false;
        };
        self.group_page = index;
        self.selected_id = Some(id);
        self.selected = Some(item);
        self.detail_page = 0;
        true
    }

    pub fn clear_card(&mut self) {
        self.is_open = false;
        self.selected_id = None;
        self.selected = None;
        self.group_ids.clear();
        self.group_snapshots.clear();
        self.group_page = 0;
        self.detail_page = 0;
        self.return_context = None;
        self.global_settings_open = false;
    }

    pub fn suspend_for_route(&mut self) {
        self.is_open = false;
        self.pointer_captured = false;
        self.release_pending = false;
        self.frozen_projection = None;
    }
}

pub fn rail_bounds(
    session: &NotificationSession,
    projection: Option<&NotificationProjection>,
) -> Rect {
    let (groups, overflow) = groups_for_rail(session, projection);
    Rect::new(
        RAIL_X,
        RAIL_Y,
        rail_width(groups.len(), overflow > 0),
        RAIL_HEIGHT,
    )
}

pub fn notification_controls_contain(
    point: Vec2,
    session: &NotificationSession,
    projection: Option<&NotificationProjection>,
) -> bool {
    rail_bounds(session, projection).contains(point) || (session.is_open && CARD.contains(point))
}

pub fn notification_reserved_rects(
    session: &NotificationSession,
    projection: Option<&NotificationProjection>,
) -> Vec<Rect> {
    let mut rects = vec![rail_bounds(session, projection)];
    if session.is_open {
        rects.push(CARD);
    }
    rects
}

pub fn is_open_for_map(ctx: &Context<'_>) -> bool {
    ctx.state.screen == kestrum::state::Screen::Campaign
        && ctx.state.overlay == kestrum::state::Overlay::None
        && ctx.notifications.is_open
        && ctx
            .campaign_view
            .is_some_and(|campaign| !campaign.observer_mode)
}

pub fn draw_notification_overlay(ctx: &Context<'_>) -> Option<super::UiAction> {
    if ctx.state.screen != kestrum::state::Screen::Campaign
        || ctx.state.overlay != kestrum::state::Overlay::None
        || !ctx
            .campaign_view
            .is_some_and(|campaign| !campaign.observer_mode)
    {
        return None;
    }

    let projection = ctx
        .notifications
        .visible_projection(ctx.notification_projection);
    if let Some(action) = rail::draw(ctx, projection) {
        return Some(super::UiAction::Notification(action));
    }
    if !ctx.notifications.is_open {
        return None;
    }
    let projection = projection.cloned().unwrap_or_default();
    let action = match ctx.notifications.tab {
        NotificationTab::Recent => detail::draw_card(ctx, &projection),
        NotificationTab::Settings => settings::draw_card(ctx),
    };
    action.map(super::UiAction::Notification)
}

pub fn draw_global_settings(ctx: &Context<'_>) -> Option<super::UiAction> {
    settings::draw_global(ctx).map(super::UiAction::Notification)
}

fn rail_width(group_count: usize, has_more: bool) -> f32 {
    let visible = group_count.min(RAIL_MAX_GROUPS);
    let group_width = visible as f32 * RAIL_BUTTON + visible.saturating_sub(1) as f32 * RAIL_GAP;
    let more_width = if has_more { RAIL_GAP + MORE_WIDTH } else { 0.0 };
    let notifications_gap = if visible > 0 || has_more {
        RAIL_GAP
    } else {
        0.0
    };
    group_width + more_width + notifications_gap + NOTIFICATIONS_WIDTH
}

/// Register changing receipt and settings copy before any visible glyphs are
/// submitted; resizing the atlas during the card draw invalidates earlier text.
pub fn prepare_text(ctx: &Context<'_>) {
    let Some(font) = ctx.body_font() else {
        return;
    };
    let projection = ctx
        .notifications
        .visible_projection(ctx.notification_projection);
    let mut body_samples = Vec::<(u16, String)>::new();
    let mut title_samples = Vec::<(u16, String)>::new();
    let add_body = |samples: &mut Vec<(u16, String)>, size, value: &str| {
        samples.push((size, value.to_owned()));
    };
    let add_title = |samples: &mut Vec<(u16, String)>, size, value: &str| {
        samples.push((size, value.to_owned()));
    };

    if let Some(projection) = projection {
        for item in projection
            .recent
            .iter()
            .skip(ctx.notifications.recent_page.saturating_mul(5))
            .take(5)
        {
            add_body(&mut body_samples, 17, &item.title);
            add_body(&mut body_samples, 14, &item.body);
        }
    }
    if let Some(item) = &ctx.notifications.selected {
        add_body(&mut body_samples, 19, &item.title);
        add_body(&mut body_samples, 16, &item.body);
        add_body(&mut body_samples, 17, &item.body);
        if let Some(subject) = &item.subject {
            match subject {
                kestrum::state::notifications::NotificationSubjectSnapshot::Person(person) => {
                    add_body(&mut body_samples, 18, &person.name);
                }
                kestrum::state::notifications::NotificationSubjectSnapshot::Place(place) => {
                    add_body(&mut body_samples, 18, &place.name);
                }
                kestrum::state::notifications::NotificationSubjectSnapshot::Army(army) => {
                    add_body(&mut body_samples, 18, &army.name);
                }
                kestrum::state::notifications::NotificationSubjectSnapshot::Route(route) => {
                    add_body(&mut body_samples, 16, &route.from.name);
                    add_body(&mut body_samples, 16, &route.to.name);
                }
                kestrum::state::notifications::NotificationSubjectSnapshot::Construction(work) => {
                    if let Some(site) = &work.site {
                        add_body(&mut body_samples, 16, &site.name);
                    }
                }
                kestrum::state::notifications::NotificationSubjectSnapshot::Battle(battle) => {
                    add_body(&mut body_samples, 16, &battle.site.name);
                }
                kestrum::state::notifications::NotificationSubjectSnapshot::Faction {
                    name,
                    ..
                } => {
                    add_body(&mut body_samples, 16, name);
                }
                kestrum::state::notifications::NotificationSubjectSnapshot::History {
                    label,
                    ..
                } => {
                    add_body(&mut body_samples, 16, label);
                }
            }
        }
    }

    for value in ctx.notification_rules.terms.values() {
        for size in [13, 14, 15, 16, 17, 19, 21] {
            add_body(&mut body_samples, size, value);
        }
    }
    for value in ctx.notification_rules.categories.values() {
        for size in [15, 17, 21] {
            add_body(&mut body_samples, size, value);
        }
    }
    for rule in ctx.notification_rules.kinds.values() {
        add_body(&mut body_samples, 15, &rule.label);
        add_body(&mut body_samples, 17, &rule.label);
    }
    add_title(
        &mut title_samples,
        22,
        rules_term(ctx, "ui_notifications", "Notifications"),
    );
    add_title(
        &mut title_samples,
        21,
        rules_term(ctx, "ui_notifications", "Notifications"),
    );
    add_title(
        &mut title_samples,
        21,
        rules_term(ctx, "ui_settings", "Settings"),
    );
    for category in categories() {
        add_title(
            &mut title_samples,
            18,
            settings::category_short_name(ctx, category),
        );
    }

    let body_samples = body_samples
        .iter()
        .map(|(size, value)| (*size, value.as_str()))
        .collect::<Vec<_>>();
    macroquad_toolkit::ui::prepare_font_text(font, &body_samples);
    if let Some(font) = ctx.font() {
        let title_samples = title_samples
            .iter()
            .map(|(size, value)| (*size, value.as_str()))
            .collect::<Vec<_>>();
        macroquad_toolkit::ui::prepare_font_text(font, &title_samples);
    }
}

pub(super) fn groups_for_rail(
    session: &NotificationSession,
    projection: Option<&NotificationProjection>,
) -> (Vec<NotificationGroup>, usize) {
    session.visible_projection(projection).map_or_else(
        || (Vec::new(), 0),
        |projection| (projection.rail.clone(), projection.rail_overflow_count),
    )
}

pub(super) fn categories() -> [NotificationSettingsCategory; 8] {
    [
        NotificationSettingsCategory::All,
        NotificationSettingsCategory::People,
        NotificationSettingsCategory::Places,
        NotificationSettingsCategory::Security,
        NotificationSettingsCategory::Orders,
        NotificationSettingsCategory::Military,
        NotificationSettingsCategory::EconomyDiplomacy,
        NotificationSettingsCategory::Remembrance,
    ]
}

pub(super) fn category_for(category: NotificationSettingsCategory) -> Option<NotificationCategory> {
    match category {
        NotificationSettingsCategory::All => None,
        NotificationSettingsCategory::People => Some(NotificationCategory::People),
        NotificationSettingsCategory::Places => Some(NotificationCategory::Places),
        NotificationSettingsCategory::Security => Some(NotificationCategory::Security),
        NotificationSettingsCategory::Orders => Some(NotificationCategory::Orders),
        NotificationSettingsCategory::Military => Some(NotificationCategory::Military),
        NotificationSettingsCategory::EconomyDiplomacy => {
            Some(NotificationCategory::EconomyDiplomacy)
        }
        NotificationSettingsCategory::Remembrance => Some(NotificationCategory::Remembrance),
    }
}

pub(super) fn rules_term<'a>(ctx: &'a Context<'_>, key: &str, fallback: &'a str) -> &'a str {
    ctx.notification_rules
        .terms
        .get(key)
        .map(String::as_str)
        .unwrap_or(fallback)
}
