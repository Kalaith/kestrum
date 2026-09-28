//! Durable construction scenes use ordinary commands for work and progress.

use super::*;
use kestrum::{
    data::{
        economy::{Resources, TroopKind},
        world::{Facility, MarkerId, SiteId},
    },
    engine::{Actor, MoveOrder},
    navigation::MapSelection,
    state::{
        construction::{ConstructionKind, ConstructionTarget, Focus, OrderId},
        military::ArmyId,
        CampaignPhase, StrategicCampaign,
    },
};

impl Game {
    pub(super) fn capture_settlement_scene(&mut self, scene: &str) -> bool {
        let scene = scene.trim_end_matches("_minimum");
        if scene == "help_construction" {
            self.state.overlay = Overlay::Help;
            self.help_page = 8;
            return true;
        }
        if !scene.starts_with("settlement_") {
            return false;
        }
        self.capture_campaign();
        match scene {
            "settlement_foreign" => self.capture_foreign_settlement(),
            "settlement_empty"
            | "settlement_outpost"
            | "settlement_road"
            | "settlement_road_complete" => {
                self.capture_frontier_work(scene);
            }
            "settlement_progress"
            | "settlement_paused"
            | "settlement_cancel_unstarted"
            | "settlement_cancel_started"
            | "settlement_completed_details"
            | "settlement_cancelled_details"
            | "settlement_builder" => self.capture_existing_work(scene),
            "settlement_dense" => self.capture_dense_work(),
            "settlement_facility_complete" => self.capture_completed_facility(),
            _ => self.capture_work_choices(scene),
        }
        self.invalidate_projection();
        true
    }

    fn capture_work_choices(&mut self, scene: &str) {
        if scene == "settlement_blocked" {
            let campaign = campaign_mut(&mut self.state);
            campaign
                .factions
                .get_mut(&campaign.player)
                .expect("player")
                .resources = Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            };
            campaign
                .validate(&self.data)
                .expect("valid poor starting scenario");
        }
        if scene == "settlement_offturn" {
            self.state
                .command(&self.data, Command::EndTurn)
                .expect("rival turn");
        }
        self.open_settlement(SiteId(1));
        let mode = match scene {
            "settlement_overview" => ui::SettlementMode::Overview,
            "settlement_focus" => ui::SettlementMode::Focus,
            "settlement_build" | "settlement_offturn" => ui::SettlementMode::Build,
            "settlement_facility" | "settlement_blocked" => ui::SettlementMode::Review,
            _ => panic!("unknown construction scene {scene}"),
        };
        self.apply_settlement_action(UiAction::SettlementTab(mode));
        if mode == ui::SettlementMode::Review {
            self.apply_settlement_action(UiAction::SelectConstruction(
                ConstructionTarget::Site(SiteId(1)),
                ConstructionKind::Facility(Facility::Stable),
            ));
        }
        if mode == ui::SettlementMode::Focus {
            self.apply_settlement_action(UiAction::SelectFocus(Focus::Growth));
        }
    }

    fn capture_existing_work(&mut self, scene: &str) {
        let campaign = campaign_mut(&mut self.state);
        let order = start(
            campaign,
            &self.data,
            ConstructionTarget::Site(SiteId(1)),
            ConstructionKind::Fort,
        );
        if scene == "settlement_progress" || scene == "settlement_cancel_started" {
            complete_round(campaign, &self.data);
        }
        if scene == "settlement_completed_details" {
            for _ in 0..3 {
                complete_round(campaign, &self.data);
            }
        }
        if scene == "settlement_cancelled_details" {
            engine::apply(
                campaign,
                &self.data,
                Actor::Player,
                Command::CancelConstruction { order },
            )
            .expect("real cancellation");
        }
        if scene == "settlement_paused" {
            march_to_gate(campaign, &self.data);
            complete_round(campaign, &self.data);
        }
        if scene == "settlement_builder" {
            engine::apply(
                campaign,
                &self.data,
                Actor::Player,
                Command::Recruit {
                    site: SiteId(1),
                    army: None,
                    kind: TroopKind::Warriors,
                },
            )
            .expect("new local builder");
        }
        self.open_settlement(SiteId(1));
        self.apply_settlement_action(UiAction::OpenConstructionOrder(order));
        if scene.starts_with("settlement_cancel_") {
            self.apply_settlement_action(UiAction::AskCancelConstruction);
        } else if scene == "settlement_builder" {
            self.apply_settlement_action(UiAction::ChooseBuilder);
        }
    }

    fn capture_frontier_work(&mut self, scene: &str) {
        let campaign = campaign_mut(&mut self.state);
        march_to_gate(campaign, &self.data);
        let route = campaign
            .world
            .routes
            .iter()
            .find(|route| route.other_endpoint(SiteId(1)) == Some(SiteId(5)))
            .expect("existing road")
            .id;
        if scene == "settlement_road_complete" {
            start(
                campaign,
                &self.data,
                ConstructionTarget::Route(route),
                ConstructionKind::Road,
            );
            complete_round(campaign, &self.data);
            complete_round(campaign, &self.data);
        }
        self.open_settlement(SiteId(5));
        match scene {
            "settlement_outpost" => self.apply_settlement_action(UiAction::SelectConstruction(
                ConstructionTarget::Site(SiteId(5)),
                ConstructionKind::Outpost,
            )),
            "settlement_road" => self.apply_settlement_action(UiAction::SelectConstruction(
                ConstructionTarget::Route(route),
                ConstructionKind::Road,
            )),
            "settlement_road_complete" => {
                self.apply_settlement_action(UiAction::SettlementTab(ui::SettlementMode::Roads))
            }
            _ => {}
        }
    }

    fn capture_completed_facility(&mut self) {
        let campaign = campaign_mut(&mut self.state);
        start(
            campaign,
            &self.data,
            ConstructionTarget::Site(SiteId(1)),
            ConstructionKind::Facility(Facility::Stable),
        );
        complete_round(campaign, &self.data);
        complete_round(campaign, &self.data);
        self.open_settlement(SiteId(1));
    }

    fn capture_dense_work(&mut self) {
        let campaign = campaign_mut(&mut self.state);
        // Author a supplied starting position at the map's four-route junction.
        for site in 5..=14 {
            campaign
                .set_site_control(&self.data, SiteId(site), Some(campaign.player), false)
                .expect("authored local control");
        }
        campaign.armies.get_mut(&ArmyId(1)).expect("builder").site = SiteId(10);
        campaign
            .world
            .sites
            .iter_mut()
            .find(|s| s.id == SiteId(10))
            .expect("site")
            .name = "The Roseward Settlement of the Long Western March".into();
        campaign
            .factions
            .get_mut(&campaign.player)
            .expect("player")
            .resources = Resources {
            gold: 10_000,
            wood: 10_000,
            stone: 10_000,
        };
        campaign
            .validate(&self.data)
            .expect("valid dense starting scenario");
        start(
            campaign,
            &self.data,
            ConstructionTarget::Site(SiteId(10)),
            ConstructionKind::Facility(Facility::Infirmary),
        );
        let routes: Vec<_> = campaign
            .world
            .routes
            .iter()
            .filter(|route| route.other_endpoint(SiteId(10)).is_some())
            .map(|route| route.id)
            .collect();
        for route in routes {
            let outcome = engine::apply(
                campaign,
                &self.data,
                Actor::Player,
                Command::Recruit {
                    site: SiteId(10),
                    army: None,
                    kind: TroopKind::Warriors,
                },
            )
            .expect("recruit local builder");
            let builder = outcome.recruited.expect("builder recruited").army;
            engine::apply(
                campaign,
                &self.data,
                Actor::Player,
                Command::StartConstruction {
                    target: ConstructionTarget::Route(route),
                    kind: ConstructionKind::Road,
                    builder,
                },
            )
            .expect("distinct connected road order");
        }
        assert_eq!(campaign.construction.len(), 5);
        self.open_settlement(SiteId(10));
        self.apply_settlement_action(UiAction::SettlementPage(1));
    }

    fn capture_foreign_settlement(&mut self) {
        let campaign = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("campaign");
        self.navigation
            .select(&campaign.world, MapSelection::Marker(MarkerId(2)))
            .expect("foreign selection");
    }
}

fn start(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    target: ConstructionTarget,
    kind: ConstructionKind,
) -> OrderId {
    engine::apply(
        campaign,
        data,
        Actor::Player,
        Command::StartConstruction {
            target,
            kind,
            builder: ArmyId(1),
        },
    )
    .expect("real construction order");
    *campaign
        .construction
        .last_key_value()
        .expect("placed order")
        .0
}

fn march_to_gate(campaign: &mut StrategicCampaign, data: &GameData) {
    engine::apply(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(1), SiteId(5)],
        }),
    )
    .expect("real gate occupation");
}

fn complete_round(campaign: &mut StrategicCampaign, data: &GameData) {
    engine::apply(campaign, data, Actor::Player, Command::EndTurn).expect("end work season");
    while !matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        engine::advance_npc(campaign, data).expect("rival work season");
    }
}

fn campaign_mut(state: &mut GameState) -> &mut StrategicCampaign {
    match state.campaign.as_mut().expect("campaign") {
        Campaign::Strategic(campaign) => campaign,
        Campaign::Shell(_) => panic!("construction needs strategic campaign"),
    }
}
