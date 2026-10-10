//! Focused captures for persistent map orders and local outcome markers.

mod receipts;
mod routes;
mod work;

use super::*;

impl Game {
    fn map_status_input(&self) -> ui::MapStatusInput<'_> {
        ui::MapStatusInput {
            campaign: self.projection.as_ref().expect("map projection is current"),
            projection: self.notification_projection.as_ref(),
            notifications: &self.notifications,
            navigation: &self.navigation,
            view: &self.view,
            movement: &self.movement,
            overview: self.overview.as_ref(),
            overview_ui: &self.overview_ui,
            state: &self.state,
        }
    }
}

impl Game {
    pub(super) fn capture_m02_map(&mut self, scene: &str) -> bool {
        match scene.trim_end_matches("_minimum") {
            "m02_routes_dismissed" => self.capture_m02_routes_dismissed(),
            "m02_routes_overview" => self.capture_m02_routes_overview(),
            "m02_construction_supply" => self.capture_m02_construction_supply(),
            "m02_receipts_map" => self.capture_m02_receipts_map(),
            "m02_receipt_fallback" => self.capture_m02_receipt_fallback(),
            "m02_map_key" => self.capture_m02_map_key(),
            _ => false,
        }
    }

    fn capture_m02_map_key(&mut self) -> bool {
        self.capture_campaign();
        self.apply(UiAction::MapKey);
        assert_eq!(self.state.overlay, Overlay::Help);
        assert_eq!(self.help_page, ui::MAP_KEY_PAGE);
        self.notice = None;
        true
    }
}
