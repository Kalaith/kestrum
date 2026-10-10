//! Map-context action captures exercise visible inspector and compact-panel input.

use super::*;
use kestrum::{
    data::{
        economy::{OrderKind, Resources, TroopKind},
        world::SiteId,
    },
    navigation::{MapScope, MapSelection},
    state::{
        construction::{ConstructionKind, ConstructionTarget},
        movement::MovementPlan,
    },
};

mod build;

impl Game {
    pub(super) fn capture_context_scene(&mut self, scene: &str) -> bool {
        match scene {
            "context_place" => {
                self.select_home_marker();
            }
            "context_army_recruit" => self.capture_context_army_recruit(),
            "context_recruit" => self.capture_context_recruit(false),
            "context_recruit_blocked" => self.capture_context_recruit(true),
            "context_build" => self.capture_context_build(),
            "context_build_review" => self.capture_context_build_review(),
            "context_build_choices" => self.capture_context_build_choices(),
            "context_builders" => self.capture_context_builders(),
            "context_focus" => self.capture_context_focus(),
            "context_focus_selected" => self.capture_context_focus_selected(),
            _ => return false,
        }
        self.notice = None;
        true
    }

    fn select_home_marker(&mut self) -> SiteId {
        self.capture_campaign();
        self.refresh_projection();
        let (site, marker) = {
            let campaign = self
                .state
                .campaign
                .as_ref()
                .and_then(Campaign::strategic)
                .expect("context capture campaign");
            let site = campaign.factions[&campaign.player].headquarters;
            let location = campaign.world.site(site).expect("headquarters site");
            assert_eq!(location.controller, Some(campaign.player));
            (site, location.marker)
        };
        self.navigation
            .select(
                &self.projection.as_ref().expect("map projection").world,
                MapSelection::Marker(marker),
            )
            .expect("owned headquarters marker is selectable on the country map");
        self.refresh_army();
        self.refresh_settlement();
        assert_eq!(self.navigation.scope(), MapScope::World);
        assert_eq!(
            self.navigation.selection(),
            Some(MapSelection::Marker(marker))
        );
        site
    }

    fn capture_context_recruit(&mut self, stale_confirmation: bool) {
        let site = self.select_home_marker();
        self.set_player_resources(Resources {
            gold: 10_000,
            wood: 10_000,
            stone: 10_000,
        });
        self.refresh_projection();
        self.refresh_army();

        let entry = self.inspector_action_center(0);
        self.assert_rejected_pointer_origins(entry);
        let action = self.pointer_tap(entry);
        assert!(matches!(action, UiAction::ContextRecruit(id) if id == site));
        self.apply(action);
        self.assert_world_place_context(site);
        assert!(matches!(self.army.mode, ui::ArmyMode::Recruit { .. }));

        let recruit_pages = self.army.options.len().div_ceil(3);
        assert!(recruit_pages > 1, "capture offers a second recruit page");
        for _ in 1..recruit_pages {
            let next = self.panel_point(16.0 + 226.0 + 50.0, 504.0 + 21.0);
            let action = self.pointer_tap(next);
            assert!(matches!(action, UiAction::ContextRecruitPage(1)));
            self.apply(action);
            self.refresh_army();
        }
        assert_eq!(self.army.recruit_page, recruit_pages - 1);
        let option_index = self
            .army
            .options
            .iter()
            .enumerate()
            .rev()
            .find(|(_, option)| option.blocked.is_none())
            .map(|(index, _)| index)
            .expect("an available recruit option appears in the capture offer");
        while self.army.recruit_page > option_index / 3 {
            let previous = self.panel_point(16.0 + 50.0, 504.0 + 21.0);
            let action = self.pointer_tap(previous);
            assert!(matches!(action, UiAction::ContextRecruitPage(-1)));
            self.apply(action);
            self.refresh_army();
        }
        let kind = self.army.options[option_index].kind;
        let page_index = option_index % 3;
        let option_center =
            self.panel_point(16.0 + 163.0, 164.0 + page_index as f32 * 112.0 + 54.0);
        let action = self.pointer_tap(option_center);
        assert!(matches!(action, UiAction::SelectRecruit(selected) if selected == kind));
        self.apply(action);
        self.refresh_army();

        let confirm = self.panel_point(16.0 + 160.0 + 83.0, 552.0 + 24.0);
        let action = self.pointer_tap(confirm);
        assert!(matches!(action, UiAction::ConfirmRecruit));
        if stale_confirmation {
            self.set_player_resources(Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            });
            let before = campaign(&self.state).clone();
            self.apply(action);
            assert_eq!(campaign(&self.state), &before, "stale recruit is atomic");
            assert!(
                !self.army.status.is_empty(),
                "rejected confirmation is shown in the compact panel"
            );
        } else {
            let before = campaign(&self.state).clone();
            let before_resources = before.factions[&before.player].resources;
            let option = self
                .army
                .options
                .iter()
                .find(|option| option.kind == kind)
                .expect("selected warrior option");
            let cost = option.cost;
            self.apply(action);
            self.refresh_projection();
            self.refresh_army();
            let after = campaign(&self.state);
            let after_resources = after.factions[&after.player].resources;
            assert_resources_spent(before_resources, after_resources, cost);
            assert!(after.formations.len() > before.formations.len());
            assert!(after.armies.values().any(|army| {
                army.faction == after.player
                    && army.site == site
                    && army.formation_ids().any(|formation| {
                        after
                            .formations
                            .get(&formation)
                            .is_some_and(|entry| entry.kind == kind)
                    })
            }));
        }
        self.assert_world_place_context(site);
        assert_eq!(self.state.overlay, Overlay::None);
    }

    fn capture_context_army_recruit(&mut self) {
        let site = self.select_home_marker();
        let (army, destination) = {
            let campaign = campaign_mut(&mut self.state);
            let player = campaign.player;
            let army = campaign
                .armies
                .values()
                .find(|army| army.faction == player && army.site == site)
                .map(|army| army.id)
                .expect("owned headquarters has a player army");
            let destination = campaign
                .world
                .adjacent_sites(site)
                .into_iter()
                .find(|destination| campaign.world.connected_route(site, *destination).is_some())
                .expect("headquarters has a connected route for the saved plan");
            campaign.movement_plans.push(MovementPlan {
                armies: vec![army],
                path: vec![site, destination],
            });
            campaign
                .validate(&self.data)
                .expect("capture movement plan validates");
            (army, destination)
        };
        self.refresh_projection();
        let saved_plans = campaign(&self.state).movement_plans.clone();
        self.begin_move(army);
        assert_eq!(self.movement.stage, ui::MoveStage::Map);
        assert_eq!(self.movement.armies, [army]);
        assert_eq!(self.movement.planned_destination, Some(destination));
        assert_eq!(self.navigation.scope(), MapScope::World);
        let projection = self.projection.as_ref().expect("map projection");
        let panel = ui::movement_panel_bounds(
            &self.movement,
            &self.navigation,
            &projection.world,
            &self.view,
        );
        let recruit = vec2(panel.x + 263.0, 642.0);
        let action = self.pointer_tap(recruit);
        assert!(matches!(action, UiAction::ContextRecruitArmy(id) if id == army));
        self.apply(action);

        self.assert_world_place_context(site);
        assert_eq!(self.army.site, Some(site));
        assert!(
            matches!(self.army.mode, ui::ArmyMode::Recruit { army: Some(id), .. } if id == army)
        );
        assert_eq!(self.movement.stage, ui::MoveStage::Inactive);
        assert_eq!(campaign(&self.state).movement_plans, saved_plans);
        assert_eq!(
            campaign(&self.state).movement_plans[0].path,
            [site, destination]
        );
    }

    fn capture_context_build(&mut self) {
        assert!(self.capture_city("production_region"));
        let site = match self.navigation.selection() {
            Some(MapSelection::Site(site)) => site,
            selection => {
                panic!("production city capture should select a regional site: {selection:?}")
            }
        };
        self.set_player_resources(Resources {
            gold: 10_000,
            wood: 10_000,
            stone: 10_000,
        });
        self.prepare_selected_site(site);
        self.ensure_local_builders(site, 4);
        self.refresh_projection();
        self.refresh_settlement();
        assert!(matches!(self.navigation.scope(), MapScope::Region(_)));

        let entry = self.inspector_action_center(1);
        let action = self.pointer_tap(entry);
        assert!(
            matches!(action, UiAction::ContextSettlement(id, ui::SettlementMode::Build) if id == site)
        );
        self.apply(action);
        self.assert_region_place_context(site);
        assert_eq!(self.settlement.mode, ui::SettlementMode::Build);

        let choices: Vec<_> = self
            .settlement
            .options
            .iter()
            .filter(|choice| !matches!(choice.target, ConstructionTarget::Route(_)))
            .map(|choice| {
                (
                    choice.target,
                    choice.option.kind,
                    choice.option.cost,
                    choice.option.blocked.is_none() && !choice.option.builders.is_empty(),
                )
            })
            .collect();
        let choice_index = choices
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (target, _, _, available))| {
                *target == ConstructionTarget::Site(site) && *available
            })
            .map(|(index, _)| index)
            .expect("an available construction at the selected production city");
        let (target, kind, cost, _) = choices[choice_index];
        let build_pages = choices.len().div_ceil(3).max(1);
        for _ in 1..build_pages {
            let next = self.panel_point(16.0 + 218.0 + 54.0, 552.0 + 24.0);
            let action = self.pointer_tap(next);
            assert!(matches!(action, UiAction::SettlementPage(1)));
            self.apply(action);
            self.refresh_settlement();
        }
        assert_eq!(self.settlement.page, build_pages - 1);
        while self.settlement.page > choice_index / 3 {
            let previous = self.panel_point(16.0 + 109.0 + 50.0, 552.0 + 24.0);
            let action = self.pointer_tap(previous);
            assert!(matches!(action, UiAction::SettlementPage(-1)));
            self.apply(action);
            self.refresh_settlement();
        }
        let choice_center = self.panel_point(
            16.0 + 163.0,
            138.0 + (choice_index % 3) as f32 * 120.0 + 58.0,
        );
        let action = self.pointer_tap(choice_center);
        assert!(
            matches!(action, UiAction::SelectConstruction(selected, selected_kind) if selected == target && selected_kind == kind)
        );
        self.apply(action);
        self.refresh_settlement();

        let choose_builder = self.panel_point(16.0 + 108.0 + 109.0, 476.0 + 24.0);
        let action = self.pointer_tap(choose_builder);
        assert!(matches!(action, UiAction::ChooseBuilder));
        self.apply(action);
        self.refresh_settlement();
        let builder_pages = self.settlement.builders.len().div_ceil(3).max(1);
        assert!(builder_pages > 1, "capture offers a second builder page");
        for _ in 1..builder_pages {
            let next = self.panel_point(16.0 + 218.0 + 54.0, 552.0 + 24.0);
            let action = self.pointer_tap(next);
            assert!(matches!(action, UiAction::SettlementPage(1)));
            self.apply(action);
            self.refresh_settlement();
        }
        assert_eq!(self.settlement.page, builder_pages - 1);
        let builder_index = self
            .settlement
            .builders
            .iter()
            .enumerate()
            .rev()
            .find(|(_, builder)| builder.blocked.is_none())
            .map(|(index, _)| index)
            .expect("an available builder appears on the final page");
        while self.settlement.page > builder_index / 3 {
            let previous = self.panel_point(16.0 + 109.0 + 50.0, 552.0 + 24.0);
            let action = self.pointer_tap(previous);
            assert!(matches!(action, UiAction::SettlementPage(-1)));
            self.apply(action);
            self.refresh_settlement();
        }
        let builder = self.settlement.builders[builder_index].id;
        let builder_center = self.panel_point(
            16.0 + 163.0,
            138.0 + (builder_index % 3) as f32 * 120.0 + 58.0,
        );
        let action = self.pointer_tap(builder_center);
        assert!(matches!(action, UiAction::SelectBuilder(selected) if selected == builder));
        self.apply(action);
        self.refresh_settlement();

        let before = campaign(&self.state).factions[&campaign(&self.state).player].resources;
        let confirm = self.panel_point(16.0 + 163.0, 552.0 + 24.0);
        let action = self.pointer_tap(confirm);
        assert!(matches!(action, UiAction::ConfirmConstruction));
        self.apply(action);
        self.refresh_projection();
        self.refresh_settlement();
        let after = campaign(&self.state);
        assert_resources_spent(before, after.factions[&after.player].resources, cost);
        assert!(after.construction.values().any(|order| {
            order.target == target
                && order.kind == kind
                && order.builder
                    == if matches!(kind, ConstructionKind::Facility(_)) {
                        None
                    } else {
                        Some(builder)
                    }
        }));
        self.assert_region_place_context(site);
        assert_eq!(self.state.overlay, Overlay::None);
    }

    fn capture_context_build_review(&mut self) {
        assert!(self.capture_city("production_region"));
        let site = match self.navigation.selection() {
            Some(MapSelection::Site(site)) => site,
            selection => {
                panic!("production city capture should select a regional site: {selection:?}")
            }
        };
        self.set_player_resources(Resources {
            gold: 10_000,
            wood: 10_000,
            stone: 10_000,
        });
        self.prepare_selected_site(site);
        self.ensure_local_builders(site, 1);
        self.refresh_projection();
        self.refresh_settlement();

        let action = self.pointer_tap(self.inspector_action_center(1));
        assert!(
            matches!(action, UiAction::ContextSettlement(id, ui::SettlementMode::Build) if id == site)
        );
        self.apply(action);
        let choices: Vec<_> = self
            .settlement
            .options
            .iter()
            .filter(|choice| !matches!(choice.target, ConstructionTarget::Route(_)))
            .map(|choice| {
                (
                    choice.target,
                    choice.option.kind,
                    choice.option.blocked.is_none() && !choice.option.builders.is_empty(),
                )
            })
            .collect();
        let index = choices
            .iter()
            .position(|(target, _, available)| {
                *target == ConstructionTarget::Site(site) && *available
            })
            .expect("a build choice can be reviewed at the selected city");
        for _ in 0..index / 3 {
            let next = self.panel_point(16.0 + 218.0 + 54.0, 552.0 + 24.0);
            let action = self.pointer_tap(next);
            assert!(matches!(action, UiAction::SettlementPage(1)));
            self.apply(action);
            self.refresh_settlement();
        }
        let (target, kind, _) = choices[index];
        let center = self.panel_point(16.0 + 163.0, 138.0 + (index % 3) as f32 * 120.0 + 58.0);
        let action = self.pointer_tap(center);
        assert!(
            matches!(action, UiAction::SelectConstruction(selected, selected_kind) if selected == target && selected_kind == kind)
        );
        self.apply(action);
        self.assert_region_place_context(site);
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.settlement.mode, ui::SettlementMode::Review);
        assert!(self.settlement.builder.is_some());
    }

    fn capture_context_focus(&mut self) {
        assert!(self.capture_city("production_region"));
        let site = match self.navigation.selection() {
            Some(MapSelection::Site(site)) => site,
            selection => {
                panic!("production city capture should select a regional site: {selection:?}")
            }
        };
        self.set_player_resources(Resources {
            gold: 10_000,
            wood: 10_000,
            stone: 10_000,
        });
        self.prepare_selected_site(site);
        let previous = campaign(&self.state).world.focus.get(&site).copied();

        let focus_entry = self.inspector_action_center(2);
        let action = self.pointer_tap(focus_entry);
        assert!(
            matches!(action, UiAction::ContextSettlement(id, ui::SettlementMode::Focus) if id == site)
        );
        self.apply(action);
        self.assert_region_place_context(site);
        self.refresh_settlement();
        let (index, focus) = self
            .settlement
            .focuses
            .iter()
            .enumerate()
            .find(|(_, choice)| choice.blocked.is_none() && Some(choice.focus) != previous)
            .map(|(index, choice)| (index, choice.focus))
            .expect("a different unblocked focus at the selected city");
        let focus_center = self.panel_point(
            16.0 + (index % 2) as f32 * 166.0 + 80.0,
            138.0 + (index / 2) as f32 * 68.0 + 30.0,
        );
        let action = self.pointer_tap(focus_center);
        assert!(matches!(action, UiAction::SelectFocus(selected) if selected == focus));
        self.apply(action);
        self.refresh_settlement();

        let before = campaign(&self.state).factions[&campaign(&self.state).player].resources;
        let confirm = self.panel_point(16.0 + 160.0 + 83.0, 552.0 + 24.0);
        let action = self.pointer_tap(confirm);
        assert!(matches!(action, UiAction::ConfirmFocus));
        self.apply(action);
        self.refresh_projection();
        let after = campaign(&self.state);
        assert_eq!(after.world.focus.get(&site), Some(&focus));
        assert_resources_spent(
            before,
            after.factions[&after.player].resources,
            self.data.economy.orders[&OrderKind::ChangeFocus].cost,
        );
        self.assert_region_place_context(site);

        // Reopen Focus through its inspector button, then exercise its visible Back.
        let action = self.pointer_tap(self.inspector_action_center(2));
        assert!(
            matches!(action, UiAction::ContextSettlement(id, ui::SettlementMode::Focus) if id == site)
        );
        self.apply(action);
        let back = self.panel_point(16.0 + 73.0, 552.0 + 24.0);
        let action = self.pointer_tap(back);
        assert!(matches!(action, UiAction::SettlementBack));
        self.apply(action);
        self.assert_region_place_context(site);
        assert_eq!(self.state.overlay, Overlay::None);

        let end_turn = vec2(1804.0, 1032.0);
        let action = self.pointer_tap(end_turn);
        assert!(matches!(action, UiAction::EndTurn));
        self.apply(action);
        self.assert_region_place_context(site);
        self.refresh_projection();
        self.refresh_settlement();

        let close = self.inspector_close_center();
        let action = self.pointer_tap(close);
        assert!(matches!(action, UiAction::CloseSelection));
        self.apply(action);
        assert_eq!(self.navigation.selection(), None);
        assert_eq!(self.state.overlay, Overlay::None);
    }

    fn capture_context_focus_selected(&mut self) {
        assert!(self.capture_city("production_region"));
        let site = match self.navigation.selection() {
            Some(MapSelection::Site(site)) => site,
            selection => {
                panic!("production city capture should select a regional site: {selection:?}")
            }
        };
        self.set_player_resources(Resources {
            gold: 10_000,
            wood: 10_000,
            stone: 10_000,
        });
        self.prepare_selected_site(site);
        let previous = campaign(&self.state).world.focus.get(&site).copied();
        let action = self.pointer_tap(self.inspector_action_center(2));
        assert!(
            matches!(action, UiAction::ContextSettlement(id, ui::SettlementMode::Focus) if id == site)
        );
        self.apply(action);
        self.refresh_settlement();
        let (index, focus) = self
            .settlement
            .focuses
            .iter()
            .enumerate()
            .find(|(_, choice)| choice.blocked.is_none() && Some(choice.focus) != previous)
            .map(|(index, choice)| (index, choice.focus))
            .expect("a different unblocked focus at the selected city");
        let center = self.panel_point(
            16.0 + (index % 2) as f32 * 166.0 + 80.0,
            138.0 + (index / 2) as f32 * 68.0 + 30.0,
        );
        let action = self.pointer_tap(center);
        assert!(matches!(action, UiAction::SelectFocus(selected) if selected == focus));
        self.apply(action);
        self.assert_region_place_context(site);
        assert_eq!(self.settlement.mode, ui::SettlementMode::Focus);
    }

    fn prepare_selected_site(&mut self, site: SiteId) {
        self.state.overlay = Overlay::None;
        self.settlement = ui::SettlementView {
            site: Some(site),
            ..Default::default()
        };
        self.refresh_projection();
        self.refresh_army();
        self.refresh_settlement();
        self.assert_region_place_context(site);
    }

    fn set_player_resources(&mut self, resources: Resources) {
        let campaign = campaign_mut(&mut self.state);
        campaign
            .factions
            .get_mut(&campaign.player)
            .unwrap()
            .resources = resources;
        campaign
            .validate(&self.data)
            .expect("capture fixture resources validate");
        self.invalidate_projection();
    }

    fn assert_world_place_context(&self, site: SiteId) {
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(self.navigation.scope(), MapScope::World);
        let marker = campaign(&self.state).world.site(site).unwrap().marker;
        assert_eq!(
            self.navigation.selection(),
            Some(MapSelection::Marker(marker))
        );
    }

    fn assert_region_place_context(&self, site: SiteId) {
        assert_eq!(self.state.overlay, Overlay::None);
        let MapScope::Region(region) = self.navigation.scope() else {
            panic!("expected regional map context")
        };
        assert_eq!(self.navigation.selection(), Some(MapSelection::Site(site)));
        assert!(campaign(&self.state)
            .world
            .region_site_position(region, site)
            .is_some());
    }

    fn inspector_action_center(&self, index: usize) -> Vec2 {
        assert!(index < 3);
        let rect = self.inspector_bounds();
        vec2(
            rect.x + 16.0 + index as f32 * 111.0 + 52.0,
            rect.bottom() - 174.0 + 24.0,
        )
    }

    fn inspector_close_center(&self) -> Vec2 {
        let rect = self.inspector_bounds();
        vec2(rect.x + 16.0 + 163.0, rect.bottom() - 62.0 + 24.0)
    }

    fn panel_point(&self, x: f32, y: f32) -> Vec2 {
        let rect = self.inspector_bounds();
        vec2(rect.x + x, rect.y + y)
    }

    fn inspector_bounds(&self) -> Rect {
        let campaign = self.projection.as_ref().expect("map projection");
        ui::selection_bounds(&self.navigation, &campaign.world, &self.view)
            .expect("selected place has an inspector bound")
    }

    fn pointer_tap(&self, position: Vec2) -> UiAction {
        assert!(self
            .capture_sheet_pointer(position, Some(position), false)
            .is_none());
        self.capture_sheet_pointer(position, Some(position), true)
            .expect("visible contextual control accepts a matching-origin release")
    }

    fn assert_rejected_pointer_origins(&self, position: Vec2) {
        assert!(self.capture_sheet_pointer(position, None, true).is_none());
        assert!(self
            .capture_sheet_pointer(position, Some(position + vec2(180.0, 0.0)), true)
            .is_none());
    }

    fn ensure_local_builders(&mut self, site: SiteId, minimum: usize) {
        let (player, current) = {
            let campaign = campaign(&self.state);
            (
                campaign.player,
                campaign
                    .armies
                    .values()
                    .filter(|army| army.faction == campaign.player && army.site == site)
                    .count(),
            )
        };
        for _ in current..minimum {
            self.state
                .command(
                    &self.data,
                    Command::Recruit {
                        site,
                        army: None,
                        kind: TroopKind::Warriors,
                    },
                )
                .expect("capture recruits another local builder");
        }
        assert!(
            campaign(&self.state)
                .armies
                .values()
                .filter(|army| army.faction == player && army.site == site)
                .count()
                >= minimum
        );
        self.invalidate_projection();
    }
}

fn campaign(state: &GameState) -> &kestrum::state::StrategicCampaign {
    state
        .campaign
        .as_ref()
        .and_then(Campaign::strategic)
        .expect("strategic capture campaign")
}

fn campaign_mut(state: &mut GameState) -> &mut kestrum::state::StrategicCampaign {
    match state.campaign.as_mut().expect("capture campaign") {
        Campaign::Strategic(campaign) => campaign,
        Campaign::Shell(_) => panic!("strategic capture campaign"),
    }
}

fn assert_resources_spent(before: Resources, after: Resources, cost: Resources) {
    assert_eq!(after.gold, before.gold - cost.gold, "gold cost is paid");
    assert_eq!(after.wood, before.wood - cost.wood, "wood cost is paid");
    assert_eq!(after.stone, before.stone - cost.stone, "stone cost is paid");
}
