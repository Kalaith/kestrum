//! Retained reports are copied only when the authoritative campaign changes.

use super::*;

impl Game {
    pub(super) fn invalidate_projection(&mut self) {
        self.projection = None;
        self.projection_revision = None;
    }

    pub(super) fn refresh_projection(&mut self) -> bool {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            let changed = self.projection.is_some();
            self.invalidate_projection();
            return changed;
        };
        let revision = (campaign.campaign_id, campaign.accepted_sequence);
        let changed = self.projection_revision != Some(revision);
        if changed {
            self.projection = engine::project(campaign, campaign.player).ok();
            self.projection_revision = Some(revision);
        }
        changed
    }
}
