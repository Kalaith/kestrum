//! Application-owned construction navigation submits the ordinary atomic commands.

mod development;
mod refresh;

use super::*;
use kestrum::{
    data::world::SiteId,
    state::{
        construction::{ConstructionKind, ConstructionTarget},
        military::ArmyId,
    },
};
use ui::SettlementMode;

impl Game {
    pub(super) fn open_settlement(&mut self, site: SiteId) {
        let Some(campaign) = self.state.campaign.as_ref().and_then(Campaign::strategic) else {
            return;
        };
        if !campaign
            .world
            .site(site)
            .is_some_and(|site| site.controller == Some(campaign.player))
        {
            return;
        }
        if matches!(
            campaign.phase,
            kestrum::state::CampaignPhase::NpcTurn { paused: false, .. }
        ) {
            self.apply_campaign_command(Command::SetNpcPaused(true));
        }
        self.settlement = ui::SettlementView {
            site: Some(site),
            ..Default::default()
        };
        self.state.overlay = Overlay::Settlement;
        self.error = None;
        self.notice = None;
        self.refresh_settlement();
    }

    pub(super) fn apply_settlement_action(&mut self, action: UiAction) {
        if self.apply_local_action(action) {
            return;
        }
        self.settlement.status.clear();
        match action {
            UiAction::OpenSettlement(site) => {
                self.open_settlement(site);
                return;
            }
            UiAction::SettlementTab(mode) => {
                self.settlement.mode = mode;
                self.settlement.page = 0;
                self.settlement.order = None;
                self.settlement.choice = None;
                self.settlement.builder = None;
            }
            UiAction::SettlementPage(delta) => {
                self.settlement.page = self.settlement.page.saturating_add_signed(delta as isize);
            }
            UiAction::SelectConstruction(target, kind) => self.select_construction(target, kind),
            UiAction::ChooseBuilder => {
                self.settlement.mode = SettlementMode::Builders;
                self.settlement.page = 0;
            }
            UiAction::SelectBuilder(builder) => self.select_builder(builder),
            UiAction::ConfirmConstruction => self.confirm_construction(),
            UiAction::OpenConstructionOrder(order) => {
                self.settlement.order = Some(order);
                self.settlement.mode = SettlementMode::Order;
                self.settlement.page = 0;
            }
            UiAction::AskCancelConstruction => self.settlement.mode = SettlementMode::Cancel,
            UiAction::ConfirmCancelConstruction => {
                if let Some(order) = self.settlement.order {
                    self.settlement_command(
                        Command::CancelConstruction { order },
                        "construction_cancel_success",
                    );
                }
            }
            UiAction::SelectFocus(focus) => self.settlement.focus = Some(focus),
            UiAction::ConfirmFocus => {
                if let (Some(site), Some(focus)) = (self.settlement.site, self.settlement.focus) {
                    self.settlement_command(Command::SetFocus { site, focus }, "focus_success");
                }
            }
            UiAction::SettlementBack => {
                self.settlement_back();
                return;
            }
            _ => unreachable!("construction action dispatch"),
        }
        self.refresh_settlement();
    }

    fn select_construction(&mut self, target: ConstructionTarget, kind: ConstructionKind) {
        self.settlement.choice = Some((target, kind));
        self.settlement.order = None;
        self.settlement.builder = self
            .settlement
            .options
            .iter()
            .find(|choice| choice.target == target && choice.option.kind == kind)
            .and_then(|choice| choice.option.builders.first().copied());
        self.settlement.mode = SettlementMode::Review;
    }

    fn select_builder(&mut self, builder: ArmyId) {
        if let Some(order) = self.settlement.order {
            self.settlement_command(
                Command::ReassignBuilder { order, builder },
                "construction_builder_success",
            );
            self.settlement.mode = SettlementMode::Order;
        } else {
            self.settlement.builder = Some(builder);
            self.settlement.mode = SettlementMode::Review;
        }
        self.settlement.page = 0;
    }

    fn confirm_construction(&mut self) {
        if let (Some((target, kind)), Some(builder)) =
            (self.settlement.choice, self.settlement.builder)
        {
            self.settlement_command(
                Command::StartConstruction {
                    target,
                    kind,
                    builder,
                },
                "construction_success",
            );
        }
    }

    fn settlement_command(&mut self, command: Command, success: &str) {
        match self.state.command(&self.data, command) {
            Ok(_) => {
                self.settlement.mode = SettlementMode::Overview;
                self.settlement.page = 0;
                self.settlement.status = self.data.presentation.text(success).into();
                self.invalidate_projection();
            }
            Err(error) => self.settlement.status = error.to_string(),
        }
    }

    pub(super) fn settlement_back(&mut self) {
        self.settlement.mode = match self.settlement.mode {
            SettlementMode::Overview
            | SettlementMode::Build
            | SettlementMode::Roads
            | SettlementMode::Focus => {
                self.state.overlay = Overlay::None;
                return;
            }
            SettlementMode::LocalActions => SettlementMode::Overview,
            SettlementMode::Rename | SettlementMode::Resettle => SettlementMode::LocalActions,
            SettlementMode::LocalReview => {
                if self.settlement.local_action == Some(ui::LocalAction::Resettle) {
                    SettlementMode::Resettle
                } else {
                    SettlementMode::LocalActions
                }
            }
            SettlementMode::Review => {
                if matches!(
                    self.settlement.choice,
                    Some((ConstructionTarget::Route(_), _))
                ) {
                    SettlementMode::Roads
                } else {
                    SettlementMode::Build
                }
            }
            SettlementMode::Builders if self.settlement.order.is_none() => SettlementMode::Review,
            SettlementMode::Builders | SettlementMode::Cancel => SettlementMode::Order,
            SettlementMode::Order => SettlementMode::Overview,
        };
        self.settlement.page = 0;
        self.settlement.status.clear();
        self.refresh_settlement();
    }
}
