//! Border discovery uses actual map picking and touch releases on visible controls.

use super::*;
use kestrum::{
    data::world::{DiplomaticState, FactionId, MarkerId, SiteId},
    navigation::MapScope,
    state::tutorial::TutorialStep,
};

impl Game {
    pub(in crate::game) fn capture_border_access(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        if !matches!(
            scene,
            "border_cutoff_world"
                | "border_cutoff_region"
                | "border_hostile_attack"
                | "border_region_queued"
                | "border_peace_blocked"
                | "border_peace_kingdom"
                | "border_peace_confirm"
                | "border_peace_entered"
                | "border_foreign_region"
                | "border_owned_occupation"
        ) {
            return false;
        }
        if matches!(scene, "border_foreign_region" | "border_owned_occupation") {
            self.capture_border_occupation(scene);
            return true;
        }
        if scene == "border_region_queued" {
            self.capture_queued_region_entry();
            return true;
        }
        self.prepare_border_capture(scene.starts_with("border_peace"));
        if scene == "border_cutoff_world" {
            return true;
        }
        if scene == "border_cutoff_region" {
            let world = &self.projection.as_ref().unwrap().world;
            self.navigation
                .enter_region(world, MarkerId(5), &mut self.view)
                .unwrap();
            self.view.focus([0.95, 0.95], self.view.working_zoom());
            assert!(self
                .navigation
                .army_targets(world, &self.view, &self.projection.as_ref().unwrap().armies)
                .iter()
                .all(|target| !target.armies.contains(&ArmyId(1))));
            self.navigation.show_world(&mut self.view);
            self.navigation.clear_selection();
        }
        let mut before = self.state.campaign.clone();
        if let Some(Campaign::Strategic(campaign)) = &mut before {
            campaign.tutorial.record(TutorialStep::Region);
        }
        let world_view = self.view;
        self.capture_region_entry_tap();
        assert_eq!(
            self.state.campaign, before,
            "entering a map issues no travel"
        );
        let target = self
            .navigation
            .army_targets(
                &self.projection.as_ref().unwrap().world,
                &self.view,
                &self.projection.as_ref().unwrap().armies,
            )
            .into_iter()
            .find(|target| target.armies.contains(&ArmyId(1)))
            .expect("region entry keeps the selected army on screen");
        assert!(!self.border_movement_panel().overlaps(&target.bounds));
        if scene == "border_cutoff_region" {
            let regional_view = self.view;
            self.apply(UiAction::WorldMap);
            assert_eq!(
                self.view, world_view,
                "region entry preserves the world camera"
            );
            self.capture_region_entry_tap();
            assert_eq!(
                self.view, regional_view,
                "an already visible army retains its regional camera"
            );
            return true;
        }
        if scene == "border_hostile_attack" {
            self.capture_border_site_tap(SiteId(7));
            let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
            assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(7));
            assert_eq!(
                campaign.world.site(SiteId(7)).unwrap().controller,
                Some(campaign.player)
            );
            assert_eq!(self.state.overlay, Overlay::None);
            assert_eq!(self.movement.armies, vec![ArmyId(1)]);
            self.refresh_projection();
            return true;
        }
        self.capture_border_site_tap(SiteId(5));
        assert!(matches!(
            self.movement
                .preview
                .as_ref()
                .unwrap()
                .blocked
                .as_ref()
                .unwrap()
                .reason,
            engine::MovementBlock::PeaceBoundary
        ));
        assert_eq!(self.movement.site, Some(SiteId(6)));
        if scene == "border_peace_blocked" {
            return true;
        }
        self.capture_border_diplomacy(scene);
        true
    }

    fn prepare_border_capture(&mut self, peaceful: bool) {
        self.capture_campaign();
        self.navigation.reset(&mut self.view);
        if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
            campaign.threats.clear();
        }
        self.state
            .command(
                &self.data,
                Command::Move(engine::MoveOrder {
                    armies: vec![ArmyId(1)],
                    path: [1, 5, 6].map(SiteId).to_vec(),
                }),
            )
            .expect("real travel into Rosemarch");
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            panic!("border capture requires a strategic campaign");
        };
        let foreign = FactionId(if peaceful { 2 } else { 3 });
        for site in [SiteId(5), SiteId(7)] {
            campaign
                .set_site_control(&self.data, site, Some(foreign), false)
                .expect("foreign sites cut the occupied town off from home");
        }
        for formation in campaign.formations.values_mut() {
            formation.movement_spent = 0;
        }
        for person in campaign.people.values_mut() {
            person.movement_spent = 0;
        }
        assert!(!campaign.army_is_supplied(ArmyId(1)));
        campaign
            .validate(&self.data)
            .expect("valid isolated border fixture");
        self.navigation
            .focus_army_site(&campaign.world, SiteId(6), &mut self.view);
        self.navigation.clear_selection();
        self.invalidate_projection();
        self.refresh_projection();
        self.capture_army_tap(ArmyId(1));
        assert_eq!(self.navigation.scope(), MapScope::World);
        assert_eq!(self.navigation.selection(), None);
        self.notice = None;
    }

    fn capture_region_entry_tap(&mut self) {
        let rect = self.border_movement_panel();
        let position = vec2(rect.center().x, rect.bottom() - 66.0);
        assert!(self
            .capture_sheet_pointer(position, Some(position), false)
            .is_none());
        assert!(self.capture_sheet_pointer(position, None, true).is_none());
        assert!(self
            .capture_sheet_pointer(position, Some(vec2(960.0, 900.0)), true)
            .is_none());
        let action = self.capture_sheet_tap(position);
        assert!(matches!(action, UiAction::EnterRegion(MarkerId(5))));
        self.apply(action);
        assert_eq!(self.navigation.scope(), MapScope::Region(MarkerId(5)));
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.movement.armies, vec![ArmyId(1)]);
    }

    fn capture_queued_region_entry(&mut self) {
        assert!(self.capture_world_orders("world_move_partial"));
        self.refresh_projection();
        let world_view = self.view;
        let mut before = self.state.campaign.clone();
        if let Some(Campaign::Strategic(campaign)) = &mut before {
            campaign.tutorial.record(TutorialStep::Region);
        }
        let destination = self.movement.planned_destination;
        self.capture_region_entry_tap();
        assert_eq!(
            self.state.campaign, before,
            "Enter Region retains queued travel"
        );
        assert_eq!(self.movement.planned_destination, destination);
        self.apply(UiAction::WorldMap);
        assert_eq!(self.view, world_view);
        if let Some(Campaign::Strategic(campaign)) = &mut before {
            campaign.tutorial.record(TutorialStep::WorldMap);
        }
        self.capture_border_map_tap(MapSelection::Marker(MarkerId(5)));
        assert_eq!(self.navigation.scope(), MapScope::Region(MarkerId(5)));
        assert_eq!(
            self.state.campaign, before,
            "same-region marker retains queued travel"
        );
        assert_eq!(self.movement.armies, vec![ArmyId(1)]);
        assert_eq!(self.movement.planned_destination, destination);
        self.notice = None;
    }

    fn capture_border_diplomacy(&mut self, scene: &str) {
        let rect = self.border_movement_panel();
        let action = self.capture_sheet_tap(vec2(rect.center().x, rect.y + 350.0));
        assert!(matches!(action, UiAction::OpenKingdom(Some(FactionId(2)))));
        self.apply(action);
        assert_eq!(self.state.overlay, Overlay::Kingdom);
        assert_eq!(self.kingdom.selected, Some(FactionId(2)));
        if scene == "border_peace_kingdom" {
            return;
        }
        let action = self.capture_sheet_tap(vec2(1200.0, 554.0));
        assert!(matches!(
            action,
            UiAction::ReviewDiplomacy(ui::KingdomIntent::DeclareWar)
        ));
        self.apply(action);
        assert_eq!(self.kingdom.intent, Some(ui::KingdomIntent::DeclareWar));
        assert!(self.kingdom.blocked.is_none());
        if scene == "border_peace_confirm" {
            return;
        }
        let action = self.capture_sheet_tap(vec2(1310.0, 830.0));
        assert!(matches!(action, UiAction::ConfirmDiplomacy));
        self.apply(action);
        let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
        assert!(campaign.relations.iter().any(|relation| {
            relation.factions == [campaign.player, FactionId(2)]
                && relation.state == DiplomaticState::War
        }));
        let action = self.capture_sheet_tap(vec2(522.0, 830.0));
        assert!(matches!(action, UiAction::KingdomBack));
        self.apply(action);
        self.refresh_projection();
        self.refresh_movement();
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.movement.armies, vec![ArmyId(1)]);
        self.capture_border_site_tap(SiteId(5));
        assert_eq!(self.movement.site, Some(SiteId(5)));
        let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
        assert_eq!(
            campaign.world.site(SiteId(5)).unwrap().controller,
            Some(campaign.player)
        );
        assert!(campaign.army_is_supplied(ArmyId(1)));
        self.refresh_projection();
    }

    fn border_movement_panel(&self) -> Rect {
        ui::movement_panel_bounds(
            &self.movement,
            &self.navigation,
            &self.projection.as_ref().unwrap().world,
            &self.view,
        )
    }

    fn capture_border_occupation(&mut self, scene: &str) {
        self.prepare_border_capture(false);
        let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign else {
            panic!("occupation capture requires a strategic campaign");
        };
        // Hawthorn already holds both anchors. Securing its adjacent entrance
        // supplies them from its headquarters and establishes the regional claim.
        campaign
            .set_site_control(&self.data, SiteId(11), Some(FactionId(3)), false)
            .expect("secure eastern supply establishes Hawthorn's claim");
        assert_eq!(
            campaign
                .world
                .region_control(MarkerId(5))
                .unwrap()
                .political_owner,
            Some(FactionId(3))
        );
        assert_eq!(
            campaign.world.site(SiteId(6)).unwrap().controller,
            Some(campaign.player)
        );
        // Regional claims are public only after every local site is discovered.
        // This inspector fixture represents a previously explored Rosemarch;
        // retain the ordinary projection rather than inserting a visible claim.
        campaign
            .knowledge
            .explored
            .entry(campaign.player)
            .or_default()
            .extend(
                campaign
                    .world
                    .sites
                    .iter()
                    .filter(|site| site.marker == MarkerId(5))
                    .map(|site| site.id),
            );
        campaign
            .validate(&self.data)
            .expect("valid known regional claim and isolated player occupation");
        self.apply(UiAction::CancelMove);
        self.invalidate_projection();
        self.refresh_projection();
        self.refresh_kingdom();
        let visible = self.projection.as_ref().unwrap();
        let site = visible
            .world
            .sites
            .iter()
            .find(|site| {
                site.controller == Some(visible.observer)
                    && visible
                        .world
                        .region_control(site.marker)
                        .and_then(|control| control.political_owner)
                        .is_some_and(|owner| owner != visible.observer)
            })
            .expect("the occupied town is visible under Hawthorn's regional claim");
        let (site, marker) = (site.id, site.marker);
        let owner = visible
            .world
            .region_control(marker)
            .unwrap()
            .political_owner
            .unwrap();
        if scene == "border_owned_occupation" {
            self.navigation
                .focus_site(&visible.world, site, &mut self.view);
            self.apply(UiAction::SelectMap(MapSelection::Site(site)));
            assert_eq!(self.navigation.scope(), MapScope::Region(marker));
            return;
        }
        self.view.focus(
            visible.world.marker(marker).unwrap().position,
            self.view.working_zoom(),
        );
        self.capture_border_map_tap(MapSelection::Marker(marker));
        let rect = ui::selection_bounds(
            &self.navigation,
            &self.projection.as_ref().unwrap().world,
            &self.view,
        )
        .unwrap();
        let action = self.capture_sheet_tap(vec2(rect.center().x, rect.bottom() - 94.0));
        assert!(matches!(action, UiAction::OpenKingdom(Some(found)) if found == owner));
        self.apply(action);
        assert_eq!(self.kingdom.selected, Some(owner));
        let action = self.capture_sheet_tap(vec2(522.0, 830.0));
        assert!(matches!(action, UiAction::KingdomBack));
        self.apply(action);
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(
            self.navigation.selection(),
            Some(MapSelection::Marker(marker))
        );
    }

    fn capture_border_site_tap(&mut self, site: SiteId) {
        let position = self
            .projection
            .as_ref()
            .unwrap()
            .world
            .site(site)
            .unwrap()
            .position;
        self.view.focus(position, self.view.working_zoom());
        self.capture_border_map_tap(MapSelection::Site(site));
    }

    fn capture_border_map_tap(&mut self, selection: MapSelection) {
        let position = self
            .navigation
            .targets(&self.projection.as_ref().unwrap().world, &self.view)
            .into_iter()
            .find(|target| target.selection == selection)
            .expect("destination is visible on the map")
            .center;
        self.origin = Some(position);
        self.map_gesture = true;
        let action = self.map_selection_action(Pointer {
            position,
            hovering: false,
            down: false,
            released: true,
        });
        assert!(matches!(action, Some(UiAction::SelectMap(found)) if found == selection));
        self.apply(action.unwrap());
        self.origin = None;
        self.map_gesture = false;
    }
}
