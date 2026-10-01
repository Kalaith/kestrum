//! Retained reports are copied only when the authoritative campaign changes.

use super::*;

impl Game {
    pub(super) fn invalidate_projection(&mut self) {
        self.projection = None;
        self.overview = None;
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
            self.projection = engine::project_map(campaign, campaign.player).ok();
            self.overview = self.projection.as_ref().map(engine::map_overview);
            self.projection_revision = Some(revision);
        }
        changed
    }
}
