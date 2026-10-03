//! Global delivery controls preserve the map card and use their own pager.

use super::*;
use crate::ui::NotificationSettingsCategory;

impl Game {
    pub(super) fn audit_notification_global_settings(&mut self) {
        let original = self.notifications.clone();
        let overlay = self.state.overlay;
        for card_open in [false, true] {
            self.notifications.is_open = card_open;
            self.notifications.tab = NotificationTab::Recent;
            self.notifications.settings_category = NotificationSettingsCategory::All;
            self.notifications.settings_page = 0;
            self.notifications.recent_page = 0;
            self.state.overlay = Overlay::Settings;
            self.apply(UiAction::Notification(
                NotificationAction::OpenGlobalSettings,
            ));
            assert!(self.notifications.global_settings_open);
            let next = self.capture_sheet_tap(vec2(1146.0, 621.0));
            assert!(matches!(
                next,
                UiAction::Notification(NotificationAction::Page(1))
            ));
            self.apply(next);
            assert_eq!(self.notifications.settings_page, 1);
            assert_eq!(self.notifications.recent_page, 0);
            self.apply(UiAction::Notification(
                NotificationAction::ToggleSettingsCategory(
                    NotificationSettingsCategory::Remembrance,
                ),
            ));
            let last_category_next = vec2(1152.0, 434.0);
            assert!(
                self.capture_sheet_pointer(last_category_next, Some(last_category_next), true)
                    .is_none(),
                "last category has no next action"
            );
            let back = self.capture_sheet_tap(vec2(960.0, 754.0));
            assert!(matches!(
                back,
                UiAction::Notification(NotificationAction::CloseGlobalSettings)
            ));
            self.apply(back);
            assert!(!self.notifications.global_settings_open);
            assert_eq!(self.notifications.is_open, card_open);
            assert_eq!(self.notifications.tab, NotificationTab::Recent);
        }
        self.notifications = original;
        self.state.overlay = overlay;
    }
}
