//! Saved-route command and overview evidence for the M02 map layer.

use super::super::*;
use kestrum::{
    data::world::{MarkerId, SiteId},
    engine::{Actor, Command, MoveOrder, MovementBlock, RuleError},
    navigation::{MapScaleBand, MapScope, MapSelection},
    state::{military::ArmyId, CampaignPhase, StrategicCampaign},
};

impl Game {
    pub(super) fn capture_m02_routes_dismissed(&mut self) -> bool {
        assert!(self.capture_world_orders("world_move_queued"));
        self.audit_saved_route_round_trip();
        self.audit_route_cancel_replace_block();
        assert!(self.capture_world_orders("world_move_continued"));
        assert_eq!(self.live_campaign().armies[&ArmyId(1)].site, SiteId(1));
        assert!(!self
            .live_campaign()
            .movement_plans
            .iter()
            .any(|plan| plan.armies.contains(&ArmyId(1))));

        // Rebuild the visible fixture after command audits, then dismiss only the
        // transient movement card. The accepted plan remains campaign state.
        assert!(self.capture_world_orders("world_move_queued"));
        let (sequence, plans, position) = self.route_state();
        assert_eq!(self.movement.stage, ui::MoveStage::Map);
        self.apply(UiAction::CancelMove);
        let world = self.live_campaign().world.clone();
        assert_eq!(self.live_campaign().accepted_sequence, sequence);
        assert_eq!(self.live_campaign().movement_plans, plans);
        assert_eq!(self.live_campaign().armies[&ArmyId(1)].site, position);
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.movement.stage, ui::MoveStage::Inactive);
        assert_eq!(self.navigation.scope(), MapScope::World);
        self.navigation
            .select(&world, MapSelection::Marker(MarkerId(1)))
            .expect("the selected route endpoint is on the world map");
        self.notice = None;
        true
    }

    pub(super) fn capture_m02_routes_overview(&mut self) -> bool {
        self.capture_campaign();
        let second = {
            let (data, campaign) = self.live_campaign_data_mut();
            let formation = campaign.armies[&ArmyId(1)]
                .formation_ids()
                .nth(1)
                .expect("capture army has a second formation");
            engine::apply(
                campaign,
                data,
                Actor::Player,
                Command::SplitArmy { formation },
            )
            .expect("real formation split creates a second army")
            .split_army
            .expect("split returns the new army identity")
        };
        let armies = vec![ArmyId(1), second];
        {
            let (data, campaign) = self.live_campaign_data_mut();
            engine::apply(
                campaign,
                data,
                Actor::Player,
                Command::Move(MoveOrder {
                    armies: armies.clone(),
                    path: [1, 5, 6].map(SiteId).to_vec(),
                }),
            )
            .expect("both armies travel together to the known gate");
            for formation in campaign.formations.values_mut() {
                formation.movement_spent = 4;
            }
            engine::apply(
                campaign,
                data,
                Actor::Player,
                Command::Move(MoveOrder {
                    armies: armies.clone(),
                    path: [6, 5, 1].map(SiteId).to_vec(),
                }),
            )
            .expect("real exhausted armies queue a shared return route");
            let shared_position = campaign.armies[&ArmyId(1)].site;
            assert_eq!(campaign.armies[&second].site, shared_position);
            assert!(campaign.movement_plans.iter().any(|plan| {
                plan.armies == armies
                    && plan.path.first() == Some(&shared_position)
                    && plan.path.last() == Some(&SiteId(1))
            }));
        }
        self.invalidate_projection();
        self.refresh_projection();
        self.navigation.reset(&mut self.view);
        self.apply(UiAction::BeginMove(ArmyId(1)));
        self.apply(UiAction::ToggleMoveArmy(second));
        let mut selected = self.movement.armies.clone();
        selected.sort_unstable();
        let mut expected = armies.clone();
        expected.sort_unstable();
        assert_eq!(selected, expected);
        self.apply(UiAction::Overview);
        assert!(self.view.overview_active());
        assert_eq!(self.view.band(), MapScaleBand::Overview);
        assert_eq!(self.navigation.scope(), MapScope::World);
        assert_eq!(self.live_campaign().phase, CampaignPhase::PlayerTurn);
        self.notice = None;
        true
    }

    fn audit_saved_route_round_trip(&self) {
        let campaign = self.live_campaign();
        let bytes = serde_json::to_vec(campaign).expect("campaign serializes for map audit");
        let restored: StrategicCampaign =
            serde_json::from_slice(&bytes).expect("queued route reloads without replay");
        assert_eq!(restored.accepted_sequence, campaign.accepted_sequence);
        assert_eq!(restored.movement_plans, campaign.movement_plans);
        assert_eq!(
            restored
                .armies
                .values()
                .map(|army| (army.id, army.site))
                .collect::<Vec<_>>(),
            campaign
                .armies
                .values()
                .map(|army| (army.id, army.site))
                .collect::<Vec<_>>()
        );
        assert!(restored
            .movement_plans
            .iter()
            .any(|plan| { plan.armies == [ArmyId(1)] && plan.path == [5, 1].map(SiteId) }));

        let mut continued = restored;
        engine::apply(&mut continued, &self.data, Actor::Player, Command::EndTurn)
            .expect("loaded route advances through the real player command");
        for _ in 0..continued.factions.len() {
            if matches!(continued.phase, CampaignPhase::PlayerTurn) {
                break;
            }
            let actor = Actor::Npc(continued.active_faction());
            engine::apply(&mut continued, &self.data, actor, Command::EndTurn)
                .expect("loaded route advances through accepted rival end turns");
        }
        assert!(matches!(continued.phase, CampaignPhase::PlayerTurn));
        assert_eq!(continued.armies[&ArmyId(1)].site, SiteId(1));
        assert!(!continued
            .movement_plans
            .iter()
            .any(|plan| plan.armies.contains(&ArmyId(1))));
    }

    fn audit_route_cancel_replace_block(&mut self) {
        let original = self.live_campaign().clone();
        let route = original
            .movement_plans
            .iter()
            .find(|plan| plan.armies.contains(&ArmyId(1)))
            .expect("queued route fixture");
        let old_path = route.path.clone();
        let mut cancelled = original.clone();
        engine::apply(
            &mut cancelled,
            &self.data,
            Actor::Player,
            Command::CancelMovementPlan { army: ArmyId(1) },
        )
        .expect("real command cancels a saved route");
        assert!(!cancelled
            .movement_plans
            .iter()
            .any(|plan| plan.armies.contains(&ArmyId(1))));

        // The shared fixture first accepted [1, 5, 6], so [5, 6] is a known
        // legal continuation from this same position after the queued [5, 1]
        // order is cancelled.
        let replacement = MoveOrder {
            armies: vec![ArmyId(1)],
            path: [5, 6].map(SiteId).to_vec(),
        };
        engine::apply(
            &mut cancelled,
            &self.data,
            Actor::Player,
            Command::Move(replacement.clone()),
        )
        .expect("real command replaces the cancelled route");
        let replaced = cancelled
            .movement_plans
            .iter()
            .find(|plan| plan.armies.contains(&ArmyId(1)))
            .expect("replacement route is saved");
        assert_eq!(replaced.path, replacement.path);
        assert_ne!(replaced.path, old_path);

        assert!(self.capture_world_orders("world_move_peace"));
        let preview = self
            .movement
            .preview
            .as_ref()
            .expect("peace boundary preview is visible")
            .clone();
        let (data, campaign) = self.live_campaign_data_mut();
        let sequence = campaign.accepted_sequence;
        assert!(matches!(
            engine::apply(campaign, data, Actor::Player, Command::Move(preview.order),),
            Err(RuleError::MovementBlocked {
                reason: MovementBlock::PeaceBoundary,
                ..
            })
        ));
        assert_eq!(campaign.accepted_sequence, sequence);
        assert!(campaign.movement_plans.is_empty());
    }

    fn route_state(&self) -> (u64, Vec<kestrum::state::movement::MovementPlan>, SiteId) {
        let campaign = self.live_campaign();
        (
            campaign.accepted_sequence,
            campaign.movement_plans.clone(),
            campaign.armies[&ArmyId(1)].site,
        )
    }

    fn live_campaign(&self) -> &StrategicCampaign {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("strategic M02 capture")
    }

    fn live_campaign_data_mut(&mut self) -> (&GameData, &mut StrategicCampaign) {
        let data = &self.data;
        let campaign = match self.state.campaign.as_mut().expect("strategic M02 capture") {
            Campaign::Strategic(campaign) => campaign.as_mut(),
            Campaign::Shell(_) => panic!("M02 capture needs a strategic campaign"),
        };
        (data, campaign)
    }
}
