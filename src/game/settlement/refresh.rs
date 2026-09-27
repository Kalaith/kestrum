//! Bounded previews refresh only on state or navigation changes.

use super::*;
use kestrum::state::{
    construction::{ConstructionOrder, Focus},
    StrategicCampaign,
};

impl Game {
    pub(in crate::game) fn refresh_settlement(&mut self) {
        if self.state.overlay != Overlay::Settlement {
            return;
        }
        let Some(site) = self.settlement.site else {
            return;
        };
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        if !campaign
            .world
            .site(site)
            .is_some_and(|s| s.controller == Some(campaign.player))
        {
            self.state.overlay = Overlay::None;
            return;
        }
        let targets = std::iter::once(ConstructionTarget::Site(site)).chain(
            campaign
                .world
                .routes
                .iter()
                .filter(|route| route.other_endpoint(site).is_some())
                .map(|route| ConstructionTarget::Route(route.id)),
        );
        self.settlement.options = targets
            .flat_map(|target| {
                engine::construction_options(campaign, &self.data, campaign.player, target)
                    .into_iter()
                    .map(move |option| ui::BuildChoice { target, option })
            })
            .collect();
        let order = self
            .settlement
            .order
            .and_then(|id| campaign.construction.get(&id));
        if let Some(order) = order {
            self.settlement.builder = order.builder;
        }
        self.settlement.blocked = selection_command(&self.settlement).and_then(|command| {
            engine::preview(campaign, &self.data, engine::Actor::Player, command)
                .err()
                .map(|e| e.to_string())
        });
        self.settlement.refund = self.settlement.order.and_then(|order| {
            engine::construction_refund(campaign, &self.data, campaign.player, order).ok()
        });
        if self.settlement.focus.is_none() {
            self.settlement.focus = campaign.world.focus.get(&site).copied();
        }
        self.refresh_builders_and_focus();
        self.clamp_settlement_page();
    }

    fn refresh_builders_and_focus(&mut self) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        if self.settlement.mode == SettlementMode::Builders {
            let order = self
                .settlement
                .order
                .and_then(|id| campaign.construction.get(&id));
            let target = order
                .map(|order| order.target)
                .or(self.settlement.choice.map(|c| c.0));
            self.settlement.builders = campaign
                .armies
                .values()
                .filter(|army| {
                    army.faction == campaign.player
                        && target.is_some_and(|target| at_target(campaign, target, army.site))
                })
                .map(|army| {
                    let command = if let Some(order) = order {
                        Command::ReassignBuilder {
                            order: order.id,
                            builder: army.id,
                        }
                    } else {
                        let (target, kind) = self.settlement.choice.expect("builder choice");
                        Command::StartConstruction {
                            target,
                            kind,
                            builder: army.id,
                        }
                    };
                    ui::BuilderChoice {
                        id: army.id,
                        name: army.name.clone(),
                        location: campaign
                            .world
                            .site(army.site)
                            .map(|s| s.name.clone())
                            .unwrap_or_default(),
                        blocked: engine::preview(
                            campaign,
                            &self.data,
                            engine::Actor::Player,
                            command,
                        )
                        .err()
                        .map(|e| e.to_string()),
                    }
                })
                .collect();
        }
        if self.settlement.mode == SettlementMode::Focus {
            if let Some(site) = self.settlement.site {
                self.settlement.focuses = [
                    Focus::Growth,
                    Focus::Fortification,
                    Focus::TroopTraining,
                    Focus::Gold,
                    Focus::Wood,
                    Focus::Stone,
                ]
                .into_iter()
                .map(|focus| ui::FocusChoice {
                    focus,
                    blocked: engine::preview(
                        campaign,
                        &self.data,
                        engine::Actor::Player,
                        Command::SetFocus { site, focus },
                    )
                    .err()
                    .map(|e| e.to_string()),
                })
                .collect();
            }
        }
    }

    fn clamp_settlement_page(&mut self) {
        let count = match self.settlement.mode {
            SettlementMode::Builders => self.settlement.builders.len(),
            SettlementMode::Build | SettlementMode::Roads => self
                .settlement
                .options
                .iter()
                .filter(|choice| {
                    matches!(choice.target, ConstructionTarget::Route(_))
                        == (self.settlement.mode == SettlementMode::Roads)
                })
                .count(),
            SettlementMode::Overview => self
                .state
                .campaign
                .as_ref()
                .and_then(Campaign::strategic)
                .map(|campaign| {
                    campaign
                        .construction
                        .values()
                        .filter(|order| {
                            order.owner == campaign.player
                                && self
                                    .settlement
                                    .site
                                    .is_some_and(|site| order_at_site(campaign, order, site))
                        })
                        .count()
                })
                .unwrap_or(0),
            _ => 1,
        };
        self.settlement.page = self
            .settlement
            .page
            .min(count.div_ceil(ui::SETTLEMENT_PAGE_SIZE).saturating_sub(1));
    }
}

fn selection_command(view: &ui::SettlementView) -> Option<Command> {
    match view.mode {
        SettlementMode::Cancel => Some(Command::CancelConstruction { order: view.order? }),
        SettlementMode::Review => {
            let (target, kind) = view.choice?;
            Some(Command::StartConstruction {
                target,
                kind,
                builder: view.builder?,
            })
        }
        _ => None,
    }
}

fn at_target(campaign: &StrategicCampaign, target: ConstructionTarget, site: SiteId) -> bool {
    match target {
        ConstructionTarget::Site(id) => id == site,
        ConstructionTarget::Route(id) => campaign
            .world
            .route(id)
            .is_some_and(|r| r.other_endpoint(site).is_some()),
    }
}

fn order_at_site(campaign: &StrategicCampaign, order: &ConstructionOrder, site: SiteId) -> bool {
    at_target(campaign, order.target, site)
}
