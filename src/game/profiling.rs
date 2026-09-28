//! Native scripted profiling uses ordinary commands and never opens player saves.
use super::*;

impl Game {
    pub fn begin_profile(&mut self) -> Result<(), String> {
        self.setup.factions =
            macroquad_toolkit::capture::env_u32("KESTRUM_PROFILE_FACTIONS", 8) as usize;
        self.setup.seed = 180_400 + self.setup.factions as u64;
        let target = macroquad_toolkit::capture::env_u32("KESTRUM_PROFILE_ROUND", 0);
        let mut campaign = kestrum::state::StrategicCampaign::new_production(
            &self.data,
            &self.setup.campaign_setup(),
        )?;
        while campaign.completed_rounds < target {
            if campaign.phase == kestrum::state::CampaignPhase::PlayerTurn {
                engine::apply(
                    &mut campaign,
                    &self.data,
                    engine::Actor::Player,
                    Command::EndTurn,
                )
                .map_err(|error| error.to_string())?;
            } else {
                engine::advance_npc(&mut campaign, &self.data)
                    .map_err(|error| error.to_string())?;
            }
        }
        self.state
            .load_campaign(Campaign::Strategic(Box::new(campaign)), &self.data)?;
        self.invalidate_projection();
        info!("K18_PROFILE_READY {}", self.profile_context());
        Ok(())
    }

    pub fn profile_order(&mut self) {
        if self.state.overlay != Overlay::None {
            return;
        }
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        if campaign.diplomacy.is_blocked() {
            return;
        }
        let result = if campaign.phase == kestrum::state::CampaignPhase::PlayerTurn {
            self.state.command(&self.data, Command::EndTurn)
        } else {
            self.state.advance_npc(&self.data)
        };
        self.handle_campaign_result(result);
    }

    pub fn profile_context(&self) -> String {
        let context = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .map(|campaign| {
                format!(
                    "factions={} round={} people={} events={} overlay={:?}",
                    campaign.factions.len(),
                    campaign.completed_rounds,
                    campaign.people.len(),
                    campaign.history.events.len(),
                    self.state.overlay
                )
            })
            .unwrap_or_else(|| "title".into());
        #[cfg(target_arch = "wasm32")]
        let context = format!(
            "{context} wasm_linear_bytes={}",
            core::arch::wasm32::memory_size(0) * 65_536
        );
        format!(
            "{context} canvas={}x{} dpi={}",
            screen_width(),
            screen_height(),
            screen_dpi_scale()
        )
    }
}
