//! Destination taps issue immediate orders; route reviews remain presentation state.

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
            planned_destination: campaign
                .movement_plans
                .iter()
                .find(|plan| plan.armies.contains(&army))
                .and_then(|plan| plan.path.last())
                .copied(),
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
        if matches!(self.navigation.scope(), kestrum::navigation::MapScope::Region(region)
            if campaign.world.site(site).is_none_or(|site| site.marker != region))
        {
            self.navigation
                .focus_army_site(&campaign.world, site, &mut self.view);
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
        self.movement.reviewing_plan = false;
        self.movement.status.clear();
        self.movement.planned_destination = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .and_then(|campaign| {
                campaign
                    .movement_plans
                    .iter()
                    .find(|plan| {
                        plan.armies
                            .iter()
                            .any(|army| self.movement.armies.contains(army))
                    })
                    .and_then(|plan| plan.path.last())
            })
            .copied();
    }

    pub(super) fn choose_move_destination(&mut self) {
        if self.movement.armies.is_empty() {
            return;
        }
        self.movement.stage = ui::MoveStage::Map;
        self.movement.destination = None;
        self.movement.preview = None;
        self.movement.reviewing_plan = false;
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
        self.movement.planned_destination = campaign
            .movement_plans
            .iter()
            .find(|plan| {
                plan.armies
                    .iter()
                    .any(|army| self.movement.armies.contains(army))
            })
            .and_then(|plan| plan.path.last())
            .copied();
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
        if self.movement.reviewing_plan {
            self.movement.preview = None;
            if self.movement.planned_destination.is_some() {
                self.review_move();
            } else {
                self.choose_move_destination();
            }
        } else if let Some(destination) = self.movement.destination {
            self.select_move_destination(destination);
        }
    }

    pub(super) fn refresh_move_options(&mut self) {
        self.movement.nearby.clear();
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        let Some(origin) = self.movement.site else {
            return;
        };
        if self.navigation.scope() == kestrum::navigation::MapScope::World {
            let Some(marker) = campaign.world.site(origin).map(|site| site.marker) else {
                return;
            };
            let neighbors: std::collections::BTreeSet<_> = campaign
                .world
                .routes
                .iter()
                .filter_map(|route| route.major_connection)
                .filter(|pair| pair.contains(&marker))
                .flat_map(|pair| pair.into_iter().filter(|id| *id != marker))
                .collect();
            for destination in neighbors {
                if let Ok(preview) = engine::world_movement_preview(
                    campaign,
                    &self.data,
                    campaign.player,
                    &self.movement.armies,
                    destination,
                ) {
                    if preview.blocked.is_none() && preview.stop.is_none() {
                        if let Some(site) = preview.order.path.last() {
                            self.movement.nearby.insert(*site, preview.total_cost);
                        }
                    }
                }
            }
            return;
        }
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
        let preview = engine::map_movement_preview(
            campaign,
            &self.data,
            campaign.player,
            &self.movement.armies,
            site,
        );
        self.set_move_preview(Some(site), preview);
    }

    pub(super) fn select_world_destination(&mut self, marker: kestrum::data::world::MarkerId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        if self
            .movement
            .site
            .and_then(|id| campaign.world.site(id))
            .is_some_and(|site| site.marker == marker)
        {
            self.movement.destination = None;
            self.movement.preview = None;
            self.movement.status.clear();
            if campaign.world.physical_site(marker).is_none() {
                self.enter_region(marker);
            }
            return;
        }
        let preview = engine::world_movement_preview(
            campaign,
            &self.data,
            campaign.player,
            &self.movement.armies,
            marker,
        );
        let site = preview
            .as_ref()
            .ok()
            .and_then(|preview| preview.order.path.last())
            .copied();
        self.set_move_preview(site, preview);
    }

    fn set_move_preview(
        &mut self,
        site: Option<SiteId>,
        result: Result<engine::MovementPreview, engine::RuleError>,
    ) {
        self.movement.destination = site;
        self.movement.route_page = 0;
        self.movement.reviewing_plan = false;
        match result {
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
            let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
                return;
            };
            let Some(plan) = campaign.movement_plans.iter().find(|plan| {
                plan.armies
                    .iter()
                    .any(|army| self.movement.armies.contains(army))
            }) else {
                return;
            };
            let order = engine::MoveOrder {
                armies: plan.armies.clone(),
                path: plan.path.clone(),
            };
            let preview =
                engine::movement_order_preview(campaign, &self.data, campaign.player, &order);
            self.movement.armies = order.armies;
            self.set_move_preview(order.path.last().copied(), preview);
            self.movement.reviewing_plan = true;
        }
        if self.movement.preview.is_none() {
            return;
        }
        self.movement.stage = ui::MoveStage::Review;
        self.state.overlay = Overlay::MoveReview;
        self.notice = None;
    }

    pub(super) fn confirm_move(&mut self) {
        if self.movement.reviewing_plan {
            return;
        }
        let Some(preview) = &self.movement.preview else {
            return;
        };
        let result = self
            .state
            .command(&self.data, Command::Move(preview.order.clone()));
        match result {
            Ok(outcome) => {
                if outcome.battle_pending {
                    self.movement = ui::MoveView::default();
                    self.navigation.clear_selection();
                    self.open_pending_battlefield();
                    return;
                }
                if let Some(movement) = outcome.movement {
                    let selected = movement.armies;
                    let message = if movement.planned_destination.is_some() {
                        self.data
                            .presentation
                            .text("move_planned_notice")
                            .to_string()
                    } else if let Some(stop) = movement.stop {
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
                    if let Some(army) = selected.first() {
                        self.begin_move(*army);
                        self.movement.armies = selected;
                        self.refresh_move_options();
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
