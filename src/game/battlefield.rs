//! Presentation-only playback controls over an already resolved battle.

use super::*;

impl Game {
    pub(super) fn advance_battlefield(&mut self, delta_seconds: f32) {
        if let Some(resolution) = &self.battlefield {
            self.battlefield_view
                .advance(delta_seconds, resolution.events.len());
        }
    }

    pub(super) fn apply_battlefield_action(&mut self, action: ui::BattlefieldAction) {
        let event_count = self
            .battlefield
            .as_ref()
            .map_or(0, |resolution| resolution.events.len());
        self.battlefield_view.apply(action, event_count);
    }
}
