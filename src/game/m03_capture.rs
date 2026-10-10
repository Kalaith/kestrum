//! M03 captures combine authored geography with ordinary campaign commands.

use super::*;
use kestrum::{
    data::{
        economy::Habitation,
        world::{RouteId, SiteId},
    },
    engine::{Actor, Command, MoveOrder, MovementBlock},
    navigation::MapScope,
    state::{
        construction::{ConstructionKind, ConstructionTarget},
        CampaignPhase, StrategicCampaign,
    },
};

const NORTHWATCH_HQ: SiteId = SiteId(1005);
const ALDER_HQ: SiteId = SiteId(1015);
const WESTMERE_HQ: SiteId = SiteId(1025);
const NORTHWATCH_CAPTURE_PATH: &[SiteId] = &[
    SiteId(1005),
    SiteId(1004),
    SiteId(1009),
    SiteId(1008),
    SiteId(1007),
    SiteId(1008),
    SiteId(1007),
    SiteId(1002),
];
const ALDER_CAPTURE_PATH: &[SiteId] = &[
    SiteId(1015),
    SiteId(1014),
    SiteId(1013),
    SiteId(1012),
    SiteId(1013),
    SiteId(1012),
    SiteId(1017),
];
const WESTMERE_CAPTURE_PATH: &[SiteId] = &[
    SiteId(1025),
    SiteId(1026),
    SiteId(1027),
    SiteId(1026),
    SiteId(1027),
    SiteId(1022),
];

impl Game {
    pub(super) fn capture_m03(&mut self, requested: &str) -> bool {
        match requested.trim_end_matches("_minimum") {
            "m03_northwatch" => self.capture_m03_landscape("Northwatch", NORTHWATCH_HQ),
            "m03_alder" => self.capture_m03_landscape("Alder Vale", ALDER_HQ),
            "m03_westmere" => self.capture_m03_landscape("Westmere", WESTMERE_HQ),
            "m03_country" => self.capture_m03_country(false),
            "m03_early_fog" => self.capture_m03_country(true),
            "m03_legacy" => self.capture_m03_legacy(),
            _ => false,
        }
    }

    fn capture_m03_landscape(&mut self, name: &str, headquarters: SiteId) -> bool {
        self.start_m03_at(headquarters, true);
        self.invest_m03_city(headquarters);
        let campaign = self.live_m03_campaign();
        let marker = campaign.world.site(headquarters).unwrap().marker;
        let local = campaign.world.region_sites(marker);
        assert!(
            m03_capture_path(headquarters)
                .iter()
                .all(|site| local.contains(site)),
            "the {name} capture route stays in its paid city neighborhood"
        );
        self.exercise_m03_routes(headquarters, name);
        self.enter_and_return_m03_region(headquarters);
        self.apply(UiAction::Notification(ui::NotificationAction::MarkAllRead));
        self.apply(UiAction::Notification(ui::NotificationAction::DismissRead));
        self.navigation.clear_selection();
        self.notice = None;
        true
    }

    fn start_m03_at(&mut self, headquarters: SiteId, require_route: bool) {
        assert!(
            self.data
                .production_layout
                .headquarters_candidates
                .contains(&headquarters),
            "the named M03 landscape needs an authored headquarters candidate"
        );
        self.setup.factions = 8;
        let base_seed = self.data.production_layout.default_seed;
        for offset in 0..256_u64 {
            self.setup.seed = base_seed.wrapping_add(offset);
            self.start_game();
            let campaign = self.live_m03_campaign();
            if campaign.factions[&campaign.player].capital == headquarters
                && (!require_route || m03_capture_path_clear(campaign, &self.data, headquarters))
            {
                assert!(campaign.world.layout_revision >= 5);
                return;
            }
        }
        panic!("the deterministic M03 seed search did not select {headquarters:?}");
    }

    fn capture_m03_country(&mut self, early_fog: bool) -> bool {
        if early_fog {
            self.start_m03_at(NORTHWATCH_HQ, false);
            self.navigation.reset(&mut self.view);
            self.focus_initial_home();
            let campaign = self.live_m03_campaign();
            eprintln!(
                "M03_CAPTURE topology={} layout={} view=player_frontier seed={} known_sites={} terrain_features={}",
                campaign.world.topology_revision,
                campaign.world.layout_revision,
                campaign.seed,
                engine::explored_sites(campaign, campaign.player).len(),
                campaign.world.terrain_features.len()
            );
        } else {
            self.setup.factions = self.data.rules.max_factions;
            self.setup.seed = self.data.production_layout.default_seed;
            self.apply(UiAction::OpenObserverSetup);
            assert_eq!(self.state.overlay, Overlay::ObserverSetup);
            self.apply(UiAction::StartObserver);
            assert!(self.is_observer());
            assert_eq!(self.navigation.scope(), MapScope::World);
            let campaign = self.live_m03_campaign();
            let projection = self
                .projection
                .as_ref()
                .expect("Observer projection is current");
            assert_eq!(projection.world.sites.len(), campaign.world.sites.len());
            assert_eq!(
                projection.world.terrain_features.len(),
                campaign.world.terrain_features.len()
            );
            eprintln!(
                "M03_CAPTURE topology={} layout={} view=observer_overview seed={} sites={} terrain_features={}",
                campaign.world.topology_revision,
                campaign.world.layout_revision,
                campaign.seed,
                projection.world.sites.len(),
                projection.world.terrain_features.len()
            );
        }
        self.navigation.clear_selection();
        self.notice = None;
        true
    }

    fn invest_m03_city(&mut self, site: SiteId) {
        let (before, cost) = {
            let campaign = self.live_m03_campaign();
            (
                campaign.factions[&campaign.player].resources,
                self.data.development.city_development.cost,
            )
        };
        let (data, campaign) = self.m03_campaign_data_mut();
        engine::apply(campaign, data, Actor::Player, Command::DevelopCity { site })
            .expect("the selected authored headquarters accepts paid city investment");
        assert_eq!(
            campaign.factions[&campaign.player].resources,
            kestrum::data::economy::Resources {
                gold: before.gold - cost.gold,
                wood: before.wood - cost.wood,
                stone: before.stone - cost.stone,
            },
            "the M03 landscape capture pays the normal city investment cost"
        );
        assert_eq!(
            campaign.world.site(site).unwrap().habitation,
            Habitation::City
        );
        self.invalidate_projection();
        self.refresh_projection();
    }

    fn exercise_m03_routes(&mut self, home: SiteId, landscape: &str) {
        let capture_path = m03_capture_path(home);
        assert_eq!(capture_path[0], home);
        self.reveal_m03_move(home, capture_path[1]);
        self.advance_m03_round();
        match home {
            NORTHWATCH_HQ => {
                self.reveal_m03_move(SiteId(1004), SiteId(1009));
                self.advance_m03_round();
                self.reveal_m03_move(SiteId(1009), SiteId(1008));
                self.advance_m03_round();
                self.reveal_m03_move(SiteId(1008), SiteId(1007));
                self.reveal_m03_move(SiteId(1007), SiteId(1008));
                self.advance_m03_round();
            }
            ALDER_HQ => {
                self.reveal_m03_move(SiteId(1014), SiteId(1013));
                self.advance_m03_round();
                self.reveal_m03_move(SiteId(1013), SiteId(1012));
                self.reveal_m03_move(SiteId(1012), SiteId(1013));
                self.advance_m03_round();
            }
            WESTMERE_HQ => {
                self.reveal_m03_move(SiteId(1026), SiteId(1027));
                self.reveal_m03_move(SiteId(1027), SiteId(1026));
                self.advance_m03_round();
            }
            _ => unreachable!("M03 capture is registered for the three authored areas"),
        }
        let path = self.m03_partial_path(home, landscape);
        let expected_end = *path.last().expect("partial route has a destination");
        let (army, expected_reachable) = {
            let campaign = self.live_m03_campaign();
            let army = campaign
                .armies
                .values()
                .find(|army| army.faction == campaign.player && army.site == path[0])
                .expect("the partial route starts at the moved player army");
            let preview = engine::movement_order_preview(
                campaign,
                &self.data,
                campaign.player,
                &MoveOrder {
                    armies: vec![army.id],
                    path: path.clone(),
                },
            )
            .expect("the selected M03 partial route previews");
            assert!(preview.blocked.is_none());
            assert!(preview.stop.as_ref().is_some_and(|stop| {
                matches!(stop.reason, MovementBlock::InsufficientMovement { .. })
            }));
            (army.id, preview.reachable_site)
        };
        let added_route = {
            let campaign = self.live_m03_campaign();
            campaign
                .world
                .connected_route(path[path.len() - 2], expected_end)
                .expect("the queued final edge is a physical route")
                .id
        };
        self.apply_m03_move(path);
        {
            let campaign = self.live_m03_campaign();
            let army = campaign.armies.get(&army).expect("the player army remains");
            assert_eq!(army.site, expected_reachable);
            assert!(
                campaign.movement_plans.iter().any(|plan| {
                    plan.armies.contains(&army.id) && plan.path.last() == Some(&expected_end)
                }),
                "the chosen {landscape} route queues its unaffordable remainder"
            );
            let supplied = campaign.supplied_sites(campaign.player);
            assert!(
                supplied.contains(&home),
                "the invested city remains supplied"
            );
            let bytes = serde_json::to_vec(campaign).expect("M03 route campaign serializes");
            let restored: StrategicCampaign =
                serde_json::from_slice(&bytes).expect("M03 queued route campaign round-trips");
            restored
                .validate(&self.data)
                .expect("M03 queued route remains valid after reload");
            assert_eq!(restored.world, campaign.world);
            assert_eq!(restored.knowledge, campaign.knowledge);
            assert_eq!(restored.history, campaign.history);
            assert_eq!(restored.construction, campaign.construction);
            assert_eq!(restored.battles, campaign.battles);
            assert_eq!(restored.movement_plans, campaign.movement_plans);
            assert!(restored.supplied_sites(restored.player).contains(&home));
        }
        self.advance_m03_round();
        let (topology_revision, layout_revision, seed) = {
            let campaign = self.live_m03_campaign();
            let army_state = campaign.armies.get(&army).expect("the player army remains");
            assert_eq!(army_state.site, expected_end);
            assert!(!campaign
                .movement_plans
                .iter()
                .any(|plan| plan.armies.contains(&army)));
            assert!(campaign.supplied_sites(campaign.player).contains(&home));
            (
                campaign.world.topology_revision,
                campaign.world.layout_revision,
                campaign.seed,
            )
        };
        let paid_road = self.improve_m03_added_route(army, added_route);
        if landscape == "Northwatch" {
            assert!(
                paid_road,
                "Northwatch traverses and pays for its added bypass"
            );
        }
        eprintln!(
            "M03_CAPTURE topology={} layout={} landscape={landscape} seed={} added_route={} paid_road={paid_road}",
            topology_revision,
            layout_revision,
            seed,
            added_route.0
        );
        self.refresh_projection();
    }

    fn reveal_m03_move(&mut self, from: SiteId, to: SiteId) {
        {
            let campaign = self.live_m03_campaign();
            let known = engine::explored_sites(campaign, campaign.player);
            assert!(known.contains(&to), "M03 reveal move targets a known site");
            assert!(
                neutral_open_site(campaign, to),
                "M03 reveal route stays safe"
            );
            let army = campaign
                .armies
                .values()
                .find(|army| army.faction == campaign.player && army.site == from)
                .expect("M03 reveal route starts at the player army");
            let preview = engine::movement_order_preview(
                campaign,
                &self.data,
                campaign.player,
                &MoveOrder {
                    armies: vec![army.id],
                    path: vec![from, to],
                },
            )
            .expect("M03 reveal route previews through the normal movement rules");
            assert!(preview.blocked.is_none());
            assert_eq!(preview.reachable_steps, 1);
            assert!(preview.encounter.is_none());
            assert!(preview.stop.is_none());
        }
        self.apply_m03_move(vec![from, to]);
        let campaign = self.live_m03_campaign();
        assert!(campaign
            .armies
            .values()
            .any(|army| army.faction == campaign.player && army.site == to));
    }

    fn improve_m03_added_route(
        &mut self,
        army_id: kestrum::state::military::ArmyId,
        route_id: RouteId,
    ) -> bool {
        let candidate = {
            let campaign = self.live_m03_campaign();
            campaign.world.routes.iter().find_map(|route| {
                if route.id != route_id {
                    return None;
                }
                let is_added = !self
                    .data
                    .production_layout
                    .routes
                    .iter()
                    .any(|base| base.id == route.id);
                if !is_added || route.road.improved {
                    return None;
                }
                let player = campaign.player;
                let controlled = [route.from, route.to].iter().all(|site| {
                    campaign
                        .world
                        .site(*site)
                        .is_some_and(|site| site.controller == Some(player))
                });
                let army_at_endpoint = campaign.armies.get(&army_id).is_some_and(|army| {
                    army.faction == player && [route.from, route.to].contains(&army.site)
                });
                if !controlled || !army_at_endpoint {
                    return None;
                }
                let target = ConstructionTarget::Route(route.id);
                let road = engine::construction_options(campaign, &self.data, player, target)
                    .into_iter()
                    .find(|option| {
                        option.kind == ConstructionKind::Road
                            && option.blocked.is_none()
                            && option.builders.contains(&army_id)
                    })?;
                Some((route.id, road.cost, player))
            })
        };
        let Some((route, cost, player)) = candidate else {
            return false;
        };
        let before = self.live_m03_campaign().factions[&player].resources;
        let (data, campaign) = self.m03_campaign_data_mut();
        engine::apply(
            campaign,
            data,
            Actor::Player,
            Command::StartConstruction {
                target: ConstructionTarget::Route(route),
                kind: ConstructionKind::Road,
                builder: army_id,
            },
        )
        .expect("the added route accepts normal paid road construction");
        {
            let order = campaign
                .construction
                .values()
                .find(|order| {
                    order.target == ConstructionTarget::Route(route)
                        && order.kind == ConstructionKind::Road
                        && order.builder == Some(army_id)
                })
                .expect("normal route construction persists its paid work order");
            assert_eq!(order.paid, cost);
            assert_eq!(
                campaign.factions[&player].resources,
                kestrum::data::economy::Resources {
                    gold: before.gold - cost.gold,
                    wood: before.wood - cost.wood,
                    stone: before.stone - cost.stone,
                }
            );
        }
        for _ in 0..self.data.construction.road_steps {
            self.advance_m03_round();
        }
        let campaign = self.live_m03_campaign();
        assert!(campaign.world.route(route).unwrap().road.improved);
        campaign
            .validate(&self.data)
            .expect("completed M03 route construction validates");
        let bytes = serde_json::to_vec(campaign).expect("completed M03 campaign serializes");
        let restored: StrategicCampaign =
            serde_json::from_slice(&bytes).expect("completed M03 campaign round-trips");
        restored
            .validate(&self.data)
            .expect("completed M03 campaign remains valid after reload");
        assert_eq!(restored.world, campaign.world);
        assert_eq!(restored.construction, campaign.construction);
        true
    }

    fn apply_m03_move(&mut self, path: Vec<SiteId>) {
        let campaign = self.live_m03_campaign();
        let player = campaign.player;
        let army = campaign
            .armies
            .values()
            .find(|army| army.faction == player && army.site == path[0])
            .expect("the M03 route starts at the player army")
            .id;
        let (data, campaign) = self.m03_campaign_data_mut();
        engine::apply(
            campaign,
            data,
            Actor::Player,
            Command::Move(MoveOrder {
                armies: vec![army],
                path,
            }),
        )
        .expect("the selected M03 route is a legal player movement command");
        self.invalidate_projection();
        self.refresh_projection();
    }

    fn m03_partial_path(&self, home: SiteId, landscape: &str) -> Vec<SiteId> {
        let campaign = self.live_m03_campaign();
        let army = campaign
            .armies
            .values()
            .find(|army| army.faction == campaign.player && army.site != home)
            .expect("the first M03 move reached a neutral site");
        let known = engine::explored_sites(campaign, campaign.player);
        let local = campaign
            .world
            .region_sites(campaign.world.site(home).unwrap().marker);
        let capture_path = m03_capture_path(home);
        let start = capture_path
            .iter()
            .rposition(|site| *site == army.site)
            .expect("the reveal route ends on its authored partial path");
        let path = capture_path[start..].to_vec();
        assert!(
            path.len() > 2,
            "M03 partial route has a queued continuation"
        );
        assert!(path[1..].iter().all(|site| {
            known.contains(site) && local.contains(site) && neutral_open_site(campaign, *site)
        }));
        let max_base_route = self
            .data
            .production_layout
            .routes
            .iter()
            .map(|route| route.id.0)
            .max()
            .unwrap_or(0);
        assert!(
            campaign
                .world
                .connected_route(path[path.len() - 2], *path.last().unwrap())
                .is_some_and(|route| route.id.0 > max_base_route),
            "the {landscape} partial route queues its authored added bypass"
        );
        let preview = engine::movement_order_preview(
            campaign,
            &self.data,
            campaign.player,
            &MoveOrder {
                armies: vec![army.id],
                path: path.clone(),
            },
        )
        .expect("the selected M03 partial route previews");
        assert!(preview.blocked.is_none());
        assert!(preview.reachable_steps > 0);
        assert!(preview.stop.as_ref().is_some_and(|stop| {
            matches!(stop.reason, MovementBlock::InsufficientMovement { .. })
        }));
        assert!(preview.encounter.is_none());
        assert_eq!(preview.reachable_steps + 1, path.len() - 1);
        path
    }

    fn advance_m03_round(&mut self) {
        self.apply_m03_end_turn(Actor::Player);
        for _ in 0..self.live_m03_campaign().factions.len() + 1 {
            if matches!(self.live_m03_campaign().phase, CampaignPhase::PlayerTurn) {
                break;
            }
            let faction = self.live_m03_campaign().active_faction();
            self.apply_m03_end_turn(Actor::Npc(faction));
        }
        assert!(matches!(
            self.live_m03_campaign().phase,
            CampaignPhase::PlayerTurn
        ));
    }

    fn apply_m03_end_turn(&mut self, actor: Actor) {
        let (data, campaign) = self.m03_campaign_data_mut();
        engine::apply(campaign, data, actor, Command::EndTurn)
            .expect("normal faction turns advance the M03 queued route");
    }

    fn enter_and_return_m03_region(&mut self, home: SiteId) {
        let marker = self.live_m03_campaign().world.site(home).unwrap().marker;
        let world = self.live_m03_campaign().world.clone();
        self.navigation
            .enter_region(&world, marker, &mut self.view)
            .expect("paid M03 city opens its connected countryside");
        assert!(matches!(self.navigation.scope(), MapScope::Region(id) if id == marker));
        self.navigation.show_world(&mut self.view);
        assert_eq!(self.navigation.scope(), MapScope::World);
        self.navigation
            .enter_region(&world, marker, &mut self.view)
            .expect("the M03 city region can be revisited after returning");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn capture_m03_legacy(&mut self) -> bool {
        let Some(path) = std::env::var_os("KESTRUM_M03_LEGACY_SAVE") else {
            self.error = Some("Set KESTRUM_M03_LEGACY_SAVE to the existing campaign JSON.".into());
            return true;
        };
        let bytes =
            std::fs::read(path).expect("the explicitly supplied M03 legacy save is readable");
        let campaign: Campaign = serde_json::from_slice(&bytes)
            .expect("the existing M03 legacy campaign uses the normal campaign parser");
        campaign
            .validate(&self.data)
            .expect("the existing M03 legacy campaign validates unchanged");
        let before = serde_json::to_value(&campaign).expect("legacy campaign normalizes to JSON");
        let round_trip = serde_json::to_vec(&campaign).expect("legacy campaign serializes");
        let restored: Campaign = serde_json::from_slice(&round_trip)
            .expect("legacy campaign normalizes through the ordinary parser");
        restored
            .validate(&self.data)
            .expect("round-tripped legacy campaign validates");
        let after = serde_json::to_value(&restored).expect("round-tripped campaign serializes");
        for field in [
            "world",
            "knowledge",
            "history",
            "movement_plans",
            "construction",
            "battles",
            "sieges",
            "pending_battle",
        ] {
            assert_eq!(
                before[field], after[field],
                "legacy {field} survives reload"
            );
        }
        self.state
            .load_campaign(restored, &self.data)
            .expect("normal GameState loading accepts the preserved legacy save");
        self.initialize_notification_campaign();
        self.invalidate_projection();
        self.refresh_projection();
        self.focus_initial_home();
        let campaign = self.live_m03_campaign();
        let home = campaign.factions[&campaign.player].headquarters;
        let marker = campaign.world.site(home).unwrap().marker;
        if campaign.world.is_region_available(marker) {
            let world = campaign.world.clone();
            if self.navigation.scope() != MapScope::Region(marker) {
                self.navigation.show_world(&mut self.view);
                self.navigation
                    .enter_region(&world, marker, &mut self.view)
                    .expect("the actual legacy home region remains readable");
            }
            assert_eq!(self.navigation.scope(), MapScope::Region(marker));
        }
        self.navigation.clear_selection();
        self.notice = None;
        true
    }

    #[cfg(target_arch = "wasm32")]
    fn capture_m03_legacy(&mut self) -> bool {
        false
    }

    fn live_m03_campaign(&self) -> &StrategicCampaign {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("M03 capture uses a strategic campaign")
    }

    fn m03_campaign_data_mut(&mut self) -> (&GameData, &mut StrategicCampaign) {
        let data = &self.data;
        let campaign = match self
            .state
            .campaign
            .as_mut()
            .expect("M03 strategic campaign")
        {
            Campaign::Strategic(campaign) => campaign.as_mut(),
            Campaign::Shell(_) => panic!("M03 capture needs a strategic campaign"),
        };
        (data, campaign)
    }
}

fn neutral_open_site(campaign: &StrategicCampaign, site: SiteId) -> bool {
    campaign.world.site(site).is_some_and(|site_state| {
        (site_state.controller.is_none() || site_state.controller == Some(campaign.player))
            && site_state.military == kestrum::data::world::MilitaryLayer::None
            && !campaign.world.contested_sites.contains(&site)
            && !campaign.sieges.contains_key(&site)
            && campaign.active_threat(site).is_none()
    })
}

fn m03_capture_path(home: SiteId) -> &'static [SiteId] {
    match home {
        NORTHWATCH_HQ => NORTHWATCH_CAPTURE_PATH,
        ALDER_HQ => ALDER_CAPTURE_PATH,
        WESTMERE_HQ => WESTMERE_CAPTURE_PATH,
        _ => unreachable!("M03 captures use only the authored headquarters"),
    }
}

fn m03_capture_path_clear(
    campaign: &StrategicCampaign,
    data: &kestrum::data::GameData,
    home: SiteId,
) -> bool {
    let path = m03_capture_path(home);
    if campaign.factions[&campaign.player].capital != home || path.first() != Some(&home) {
        return false;
    }
    if path[1..].iter().any(|site| {
        !neutral_open_site(campaign, *site)
            || campaign.armies.values().any(|army| army.site == *site)
    }) {
        return false;
    }
    if !path
        .windows(2)
        .all(|edge| campaign.world.connected_route(edge[0], edge[1]).is_some())
    {
        return false;
    }
    let max_base_route = data
        .production_layout
        .routes
        .iter()
        .map(|route| route.id.0)
        .max()
        .unwrap_or(0);
    if !campaign
        .world
        .connected_route(path[path.len() - 2], path[path.len() - 1])
        .is_some_and(|route| route.id.0 > max_base_route)
    {
        return false;
    }
    let known = engine::explored_sites(campaign, campaign.player);
    if !known.contains(&path[1]) {
        return false;
    }
    let Some(army) = campaign
        .armies
        .values()
        .find(|army| army.faction == campaign.player && army.site == home)
    else {
        return false;
    };
    engine::movement_order_preview(
        campaign,
        data,
        campaign.player,
        &MoveOrder {
            armies: vec![army.id],
            path: vec![home, path[1]],
        },
    )
    .is_ok_and(|preview| {
        preview.blocked.is_none()
            && preview.reachable_steps == 1
            && preview.encounter.is_none()
            && preview.stop.is_none()
    })
}
