//! Confirmed physical routes and group selection are presentation state.

use super::*;
use kestrum::{data::world::SiteId, state::military::ArmyId};

impl Game {
    pub(super) fn begin_move(&mut self, army: ArmyId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let Some(force) = campaign
            .armies
            .get(&army)
            .filter(|force| force.faction == campaign.player)
        else {
            return;
        };
        if !matches!(campaign.phase, kestrum::state::CampaignPhase::PlayerTurn)
            || campaign.diplomacy.is_blocked()
        {
            return;
        }
        let site = force.site;
        self.movement = ui::MoveView {
            stage: ui::MoveStage::Map,
            site: Some(force.site),
            armies: vec![army],
            remaining: campaign
                .armies
                .values()
                .filter(|other| other.faction == campaign.player && other.site == force.site)
                .filter_map(|other| {
                    engine::army_remaining(campaign, &self.data, other.id)
                        .ok()
                        .map(|remaining| (other.id, remaining))
                })
                .collect(),
            ..Default::default()
        };
        self.navigation.clear_selection();
        if campaign.world.site(site).is_some_and(|site| {
            campaign.world.physical_site(site.marker).is_none()
                && self.navigation.scope() != kestrum::navigation::MapScope::Region(site.marker)
        }) {
            self.navigation
                .focus_site(&campaign.world, site, &mut self.view);
            self.navigation.clear_selection();
        }
        self.state.overlay = Overlay::None;
        self.notice = None;
        self.error = None;
        self.refresh_move_options();
    }

    pub(super) fn toggle_move_army(&mut self, army: ArmyId) {
        if !self.movement.remaining.contains_key(&army) {
            return;
        }
        if self.movement.armies.contains(&army) {
            self.movement.armies.retain(|id| *id != army);
        } else {
            self.movement.armies.push(army);
            self.movement.armies.sort_unstable();
        }
        self.movement.preview = None;
        self.movement.status.clear();
    }

    pub(super) fn choose_move_destination(&mut self) {
        if self.movement.armies.is_empty() {
            return;
        }
        self.movement.stage = ui::MoveStage::Map;
        self.movement.destination = None;
        self.movement.preview = None;
        self.movement.status.clear();
        self.navigation.clear_selection();
        self.state.overlay = Overlay::None;
        self.refresh_move_options();
    }

    pub(super) fn refresh_movement(&mut self) {
        if self.movement.stage == ui::MoveStage::Inactive {
            return;
        }
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        self.movement
            .armies
            .retain(|id| campaign.armies.contains_key(id));
        let site = self
            .movement
            .armies
            .first()
            .and_then(|id| campaign.armies.get(id))
            .map(|army| army.site);
        if site.is_none() || !matches!(campaign.phase, kestrum::state::CampaignPhase::PlayerTurn) {
            self.movement = ui::MoveView::default();
            return;
        }
        self.movement
            .armies
            .retain(|id| Some(campaign.armies[id].site) == site);
        if self.movement.site != site {
            self.movement.destination = None;
            self.movement.preview = None;
            self.navigation.clear_selection();
        }
        self.movement.site = site;
        self.movement.remaining = campaign
            .armies
            .values()
            .filter(|army| army.faction == campaign.player && Some(army.site) == site)
            .filter_map(|army| {
                engine::army_remaining(campaign, &self.data, army.id)
                    .ok()
                    .map(|left| (army.id, left))
            })
            .collect();
        self.refresh_move_options();
        if let Some(destination) = self.movement.destination {
            self.select_move_destination(destination);
        }
    }

    fn refresh_move_options(&mut self) {
        self.movement.nearby.clear();
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let Some(origin) = self.movement.site else {
            return;
        };
        for destination in campaign.world.adjacent_sites(origin) {
            if let Ok(preview) = engine::map_movement_preview(
                campaign,
                &self.data,
                campaign.player,
                &self.movement.armies,
                destination,
            ) {
                if preview.reachable_site == destination {
                    self.movement.nearby.insert(destination, preview.total_cost);
                }
            }
        }
    }

    pub(super) fn select_move_destination(&mut self, site: SiteId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        self.movement.destination = Some(site);
        self.movement.route_page = 0;
        match engine::map_movement_preview(
            campaign,
            &self.data,
            campaign.player,
            &self.movement.armies,
            site,
        ) {
            Ok(preview) => {
                self.movement.preview = Some(preview);
                self.movement.status.clear();
            }
            Err(error) => {
                self.movement.preview = None;
                self.movement.status = error.to_string();
            }
        }
    }

    pub(super) fn review_move(&mut self) {
        if self.movement.preview.is_none() {
            return;
        }
        self.movement.stage = ui::MoveStage::Review;
        self.state.overlay = Overlay::MoveReview;
    }

    pub(super) fn confirm_move(&mut self) {
        let Some(preview) = &self.movement.preview else {
            return;
        };
        let result = self
            .state
            .command(&self.data, Command::Move(preview.order.clone()));
        match result {
            Ok(outcome) => {
                if let Some(movement) = outcome.movement {
                    let selected = movement.armies.first().copied();
                    let message = if let Some(stop) = movement.stop {
                        format!("Movement interrupted: {}", stop.reason)
                    } else {
                        format!(
                            "Arrived. Each travelling member spent {} movement.",
                            movement.spent
                        )
                    };
                    self.movement = ui::MoveView::default();
                    self.state.overlay = Overlay::None;
                    self.navigation.clear_selection();
                    if let Some(army) = selected {
                        self.begin_move(army);
                    }
                    self.notice = Some((message, 5.0));
                }
                if outcome.battle.is_some() {
                    self.open_battle_reports();
                }
            }
            Err(error) => self.movement.status = error.to_string(),
        }
    }

    pub(super) fn cancel_move(&mut self) {
        self.movement = ui::MoveView::default();
        self.state.overlay = Overlay::None;
        self.navigation.clear_selection();
    }
}
