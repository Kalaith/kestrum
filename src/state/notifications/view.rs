//! Read-only card and rail presentation models.

use super::{
    NotificationCategory, NotificationDetail, NotificationId, NotificationKind,
    NotificationPriority, NotificationSubjectSnapshot,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedNotification {
    pub id: NotificationId,
    pub kind: NotificationKind,
    pub category: NotificationCategory,
    pub priority: NotificationPriority,
    pub completed_rounds: u32,
    pub occurred_order: u64,
    pub is_read: bool,
    pub is_dismissed: bool,
    pub is_active: bool,
    pub title: String,
    pub body: String,
    pub subject: Option<NotificationSubjectSnapshot>,
    pub detail: NotificationDetail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationGroup {
    pub kind: NotificationKind,
    pub category: NotificationCategory,
    pub priority: NotificationPriority,
    pub completed_rounds: u32,
    pub receipt_ids: Vec<NotificationId>,
    pub unread_count: usize,
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NotificationProjection {
    pub rail: Vec<NotificationGroup>,
    pub rail_overflow_count: usize,
    pub recent: Vec<RenderedNotification>,
    pub unread_count: usize,
    pub top_bar_unread_count: usize,
    pub omitted_count: u64,
    pub omitted_unread_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationTab {
    Recent,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationSettingsCategory {
    All,
    People,
    Places,
    Security,
    Orders,
    Military,
    EconomyDiplomacy,
    Remembrance,
}
