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
        self.movement = ui::MoveView {
            stage: ui::MoveStage::Group,
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
        self.state.overlay = Overlay::MoveGroup;
        self.notice = None;
        self.error = None;
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
    }

    pub(super) fn select_move_destination(&mut self, site: SiteId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        self.movement.destination = Some(site);
        self.movement.route_page = 0;
        match engine::movement_preview(
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
                    let destination = movement.path.last().copied().or(self.movement.site);
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
                    if let Some(site) = destination {
                        self.open_armies(site);
                        self.army.status = message;
                        self.open_siege(site);
                    }
                }
                if outcome.battle.is_some() {
                    self.open_battle_reports();
                }
            }
            Err(error) => self.movement.status = error.to_string(),
        }
    }

    pub(super) fn cancel_move(&mut self) {
        let origin = self.movement.site;
        self.movement = ui::MoveView::default();
        self.state.overlay = Overlay::None;
        if let Some(site) = origin {
            self.open_armies(site);
        }
    }
}
