//! Exercise world army actions through the application's real dispatch and picking.

use super::*;
use kestrum::{
    data::world::{FactionId, MarkerId, SiteId},
    navigation::MapScope,
};

impl Game {
    pub(in crate::game) fn capture_world_orders(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        if !matches!(
            scene,
            "world_move_exit"
                | "world_move_entry"
                | "world_move_partial"
                | "world_move_queued"
                | "world_move_continued"
                | "world_move_peace"
                | "world_move_stack"
                | "world_army_details"
                | "world_move_review"
        ) {
            return false;
        }
        self.prepare_world_order_capture(scene);
        if matches!(scene, "world_move_stack" | "world_army_details") {
            self.capture_world_army_stack(scene);
            return true;
        }
        self.frame_world_order_army(SiteId(if scene == "world_move_entry" { 1 } else { 6 }));
        self.capture_army_tap(ArmyId(1));
        assert_eq!(self.navigation.scope(), MapScope::World);
        self.view.reset();
        self.apply(UiAction::SelectMap(MapSelection::Marker(MarkerId(
            if scene == "world_move_entry" { 5 } else { 1 },
        ))));
        let preview = self.movement.preview.as_ref().expect("world route preview");
        assert_eq!(
            preview.total_cost,
            if scene == "world_move_entry" { 2 } else { 4 }
        );
        if scene == "world_move_review" {
            self.apply(UiAction::ReviewMove);
        } else if matches!(
            scene,
            "world_move_partial" | "world_move_queued" | "world_move_continued"
        ) {
            self.apply(UiAction::ConfirmMove);
            assert_eq!(self.movement.site, Some(SiteId(5)));
            assert_eq!(self.navigation.scope(), MapScope::World);
            assert_eq!(self.movement.planned_destination, Some(SiteId(1)));
            match scene {
                "world_move_partial" => {
                    self.apply(UiAction::SelectMap(MapSelection::Marker(MarkerId(1))));
                    assert!(self.movement.preview.as_ref().unwrap().can_confirm());
                    assert_eq!(self.movement.preview.as_ref().unwrap().reachable_steps, 0);
                }
                "world_move_queued" => {
                    self.apply(UiAction::CancelMovementPlan(ArmyId(1)));
                    assert_eq!(self.movement.planned_destination, None);
                    self.apply(UiAction::SelectMap(MapSelection::Marker(MarkerId(1))));
                    self.apply(UiAction::ConfirmMove);
                    assert_eq!(self.movement.site, Some(SiteId(5)));
                    assert_eq!(self.movement.planned_destination, Some(SiteId(1)));
                }
                "world_move_continued" => {
                    self.apply(UiAction::CancelMove);
                    self.apply(UiAction::EndTurn);
                    for _ in 0..3 {
                        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
                            panic!("world movement capture needs a campaign");
                        };
                        let actor = engine::Actor::Npc(campaign.active_faction());
                        let outcome = engine::apply(campaign, &self.data, actor, Command::EndTurn);
                        self.handle_campaign_result(outcome);
                    }
                    self.refresh_projection();
                    self.begin_move(ArmyId(1));
                    assert_eq!(self.movement.site, Some(SiteId(1)));
                    assert_eq!(self.movement.planned_destination, None);
                }
                _ => {}
            }
        }
        true
    }

    fn prepare_world_order_capture(&mut self, scene: &str) {
        self.capture_campaign();
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            campaign.threats.clear();
        }
        self.navigation.reset(&mut self.view);
        if scene != "world_move_entry" {
            self.state
                .command(
                    &self.data,
                    Command::Move(engine::MoveOrder {
                        armies: vec![ArmyId(1)],
                        path: [1, 5, 6].map(SiteId).to_vec(),
                    }),
                )
                .expect("real travel into the region");
        }
        if scene == "world_move_peace" {
            if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                campaign
                    .set_site_control(&self.data, SiteId(5), Some(FactionId(2)), false)
                    .expect("peaceful foreign gate");
            }
        }
        if !matches!(
            scene,
            "world_move_partial"
                | "world_move_queued"
                | "world_move_continued"
                | "world_move_review"
        ) {
            if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                for formation in campaign.formations.values_mut() {
                    formation.movement_spent = 0;
                }
                for person in campaign.people.values_mut() {
                    person.movement_spent = 0;
                }
            }
        }
    }

    fn capture_world_army_stack(&mut self, scene: &str) {
        let campaign = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .unwrap();
        let formation = campaign.armies[&ArmyId(1)].formation_ids().nth(1).unwrap();
        let other = self
            .state
            .command(&self.data, Command::SplitArmy { formation })
            .unwrap()
            .split_army
            .unwrap();
        if scene == "world_move_stack" {
            self.state
                .command(
                    &self.data,
                    Command::Move(engine::MoveOrder {
                        armies: vec![other],
                        path: [6, 5].map(SiteId).to_vec(),
                    }),
                )
                .unwrap();
        }
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            campaign.armies.get_mut(&other).unwrap().name =
                "The Riverward Guard of the Silver Hawthorns".into();
        }
        self.frame_world_order_army(SiteId(6));
        self.capture_army_tap(ArmyId(1));
        self.apply(UiAction::BeginMove(other));
        assert_eq!(self.navigation.scope(), MapScope::World);
        self.apply(UiAction::Recenter);
        assert_eq!(self.navigation.scope(), MapScope::World);
        if scene == "world_army_details" {
            self.apply(UiAction::OpenArmies(SiteId(6)));
            assert_eq!(self.local_armies()[self.army.page], other);
            assert_eq!(self.navigation.scope(), MapScope::World);
        }
    }

    fn frame_world_order_army(&mut self, site: SiteId) {
        self.navigation.focus_army_site(
            &self
                .state
                .campaign
                .as_ref()
                .and_then(Campaign::strategic)
                .unwrap()
                .world,
            site,
            &mut self.view,
        );
        self.navigation.clear_selection();
        self.refresh_projection();
    }
}
