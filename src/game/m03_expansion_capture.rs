//! Captures the expanded M03 city neighborhoods through ordinary campaign actions.

use super::*;
use kestrum::{
    data::{
        economy::{Habitation, Resources},
        world::{MarkerId, MarkerLocation, MilitaryLayer, SiteId},
    },
    engine::{Actor, Command, MoveOrder, MovementBlock},
    navigation::MapScope,
    state::{Campaign, CampaignPhase, StrategicCampaign},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const INNER_AREAS: [InnerArea; 5] = [
    InnerArea {
        name: "Riverfold",
        marker: MarkerId(76),
        headquarters: SiteId(1035),
    },
    InnerArea {
        name: "Lakeward",
        marker: MarkerId(77),
        headquarters: SiteId(1045),
    },
    InnerArea {
        name: "Rosevale",
        marker: MarkerId(78),
        headquarters: SiteId(1055),
    },
    InnerArea {
        name: "Eastfold",
        marker: MarkerId(79),
        headquarters: SiteId(1065),
    },
    InnerArea {
        name: "Southplain",
        marker: MarkerId(80),
        headquarters: SiteId(1075),
    },
];

const MAJOR_CITIES: [MajorCity; 3] = [
    MajorCity {
        scene: "m03_northwatch_major",
        name: "Northwatch",
        headquarters: SiteId(1005),
        center: SiteId(5),
        landscape_sites: &[SiteId(3)],
    },
    MajorCity {
        scene: "m03_riverfold_major",
        name: "Riverfold",
        headquarters: SiteId(1035),
        center: SiteId(32),
        landscape_sites: &[SiteId(28), SiteId(31), SiteId(33)],
    },
    MajorCity {
        scene: "m03_lakeward_major",
        name: "Lakeward",
        headquarters: SiteId(1045),
        center: SiteId(41),
        landscape_sites: &[SiteId(37), SiteId(42), SiteId(44)],
    },
];

struct InnerArea {
    name: &'static str,
    marker: MarkerId,
    headquarters: SiteId,
}

struct MajorCity {
    scene: &'static str,
    name: &'static str,
    headquarters: SiteId,
    center: SiteId,
    landscape_sites: &'static [SiteId],
}

impl Game {
    pub(super) fn capture_m03_expansion(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        if scene == "m03_country" {
            return self.capture_m03_country_overview();
        }
        if let Some(area) = INNER_AREAS
            .iter()
            .find(|area| scene == format!("m03_{}", area.name.to_ascii_lowercase()))
        {
            return self.capture_m03_inner_city(area);
        }
        if let Some(city) = MAJOR_CITIES.iter().find(|city| city.scene == scene) {
            return self.capture_m03_major_city(city);
        }
        false
    }

    fn capture_m03_inner_city(&mut self, area: &InnerArea) -> bool {
        let seed = self.m03_observer_city_seed(area.headquarters, area.marker);
        self.start_m03_expansion_observer(seed);
        let owner = self
            .live_m03_expansion_campaign()
            .factions
            .values()
            .find(|faction| faction.headquarters == area.headquarters)
            .expect("the M03 Observer campaign assigns this authored HQ")
            .id;
        let site = self
            .live_m03_expansion_campaign()
            .world
            .site(area.headquarters);
        assert_eq!(site.map(|site| site.marker), Some(area.marker));
        self.advance_m03_observer_to(owner);
        self.invest_m03_expansion_city(area.headquarters, Actor::Npc(owner), owner);
        self.enter_m03_expansion_city(area.headquarters, area.marker, &[]);
        self.clear_m03_expansion_capture_feedback();
        let campaign = self.live_m03_expansion_campaign();
        eprintln!(
            "M03_CAPTURE topology={} layout={} inner={} marker={} headquarters={} seed={} owner={} view=observer city=paid",
            campaign.world.topology_revision,
            campaign.world.layout_revision,
            area.name,
            area.marker.0,
            area.headquarters.0,
            campaign.seed,
            owner.0
        );
        true
    }

    fn capture_m03_major_city(&mut self, city: &MajorCity) -> bool {
        let path = self.start_m03_expansion_campaign(city.headquarters, Some(city.center));
        self.travel_m03_expansion_path(&path);
        let player = self.live_m03_expansion_campaign().player;
        self.invest_m03_expansion_city(city.center, Actor::Player, player);
        let marker = self
            .live_m03_expansion_campaign()
            .world
            .site(city.center)
            .expect("M03 major-city center exists")
            .marker;
        self.enter_m03_expansion_city(city.center, marker, city.landscape_sites);
        self.clear_m03_expansion_capture_feedback();
        let campaign = self.live_m03_expansion_campaign();
        eprintln!(
            "M03_CAPTURE topology={} layout={} major_city={} headquarters={} center={} marker={} seed={} path={:?} city=paid",
            campaign.world.topology_revision,
            campaign.world.layout_revision,
            city.name,
            city.headquarters.0,
            city.center.0,
            marker.0,
            campaign.seed,
            path.iter().map(|site| site.0).collect::<Vec<_>>()
        );
        true
    }

    fn capture_m03_country_overview(&mut self) -> bool {
        self.start_m03_expansion_observer(self.data.production_layout.default_seed);
        self.navigation.clear_selection();
        self.notice = None;
        let campaign = self.live_m03_expansion_campaign();
        assert_eq!(campaign.world.topology_revision, 3);
        assert_eq!(campaign.world.layout_revision, 5);
        let projection = self
            .projection
            .as_ref()
            .expect("the full Observer projection is current");
        assert_eq!(projection.world.sites.len(), campaign.world.sites.len());
        assert_eq!(
            projection.world.terrain_features.len(),
            campaign.world.terrain_features.len()
        );
        for marker in 73..=80 {
            assert!(matches!(
                campaign
                    .world
                    .marker(MarkerId(marker))
                    .map(|entry| &entry.location),
                Some(MarkerLocation::Region { .. })
            ));
        }
        eprintln!(
            "M03_CAPTURE topology={} layout={} view=observer_overview seed={} sites={} terrain_features={}",
            campaign.world.topology_revision,
            campaign.world.layout_revision,
            campaign.seed,
            projection.world.sites.len(),
            projection.world.terrain_features.len()
        );
        true
    }

    fn start_m03_expansion_observer(&mut self, seed: u64) {
        self.setup.factions = self.data.rules.max_factions;
        self.setup.seed = seed;
        self.apply(UiAction::OpenObserverSetup);
        assert_eq!(self.state.overlay, Overlay::ObserverSetup);
        self.apply(UiAction::StartObserver);
        assert!(self.is_observer());
        assert_eq!(self.navigation.scope(), MapScope::World);
        let campaign = self.live_m03_expansion_campaign();
        assert_eq!(campaign.world.topology_revision, 3);
        assert_eq!(campaign.world.layout_revision, 5);
        let projection = self
            .projection
            .as_ref()
            .expect("the M03 Observer projection is current");
        assert_eq!(projection.world.sites.len(), campaign.world.sites.len());
        assert_eq!(
            projection.world.terrain_features.len(),
            campaign.world.terrain_features.len()
        );
    }

    fn m03_observer_city_seed(&self, headquarters: SiteId, marker: MarkerId) -> u64 {
        assert!(self
            .data
            .production_layout
            .headquarters_candidates
            .contains(&headquarters));
        let base_seed = self.data.production_layout.default_seed;
        for offset in 0..512_u64 {
            let mut setup = ui::SetupView::default().campaign_setup();
            setup.factions = self.data.rules.max_factions;
            setup.seed = base_seed.wrapping_add(offset);
            let campaign = StrategicCampaign::new_production_observer(&self.data, &setup)
                .expect("the M03 Observer seed generates a valid campaign");
            let owner = campaign
                .factions
                .values()
                .find(|faction| faction.headquarters == headquarters);
            if campaign
                .world
                .site(headquarters)
                .is_some_and(|site| site.marker == marker)
                && owner.is_some_and(|owner| {
                    engine::city_development_option(&campaign, &self.data, owner.id, headquarters)
                        .is_some_and(|option| option.blocked.is_none())
                })
            {
                return setup.seed;
            }
        }
        panic!("no deterministic M03 Observer seed permits paid development at {headquarters:?}");
    }

    fn advance_m03_observer_to(&mut self, owner: kestrum::data::world::FactionId) {
        assert!(self.is_observer());
        for _ in 0..self.live_m03_expansion_campaign().factions.len() {
            let campaign = self.live_m03_expansion_campaign();
            if campaign.active_faction() == owner {
                return;
            }
            let active = campaign.active_faction();
            assert!(matches!(campaign.phase, CampaignPhase::NpcTurn { .. }));
            self.apply_m03_expansion_end_turn(Actor::Npc(active));
        }
        assert_eq!(
            self.live_m03_expansion_campaign().active_faction(),
            owner,
            "ordinary Observer faction turns reach the HQ owner"
        );
    }

    fn start_m03_expansion_campaign(
        &mut self,
        headquarters: SiteId,
        target: Option<SiteId>,
    ) -> Vec<SiteId> {
        assert!(self
            .data
            .production_layout
            .headquarters_candidates
            .contains(&headquarters));
        self.setup.factions = 8;
        let base_seed = self.data.production_layout.default_seed;
        for offset in 0..512_u64 {
            self.setup.seed = base_seed.wrapping_add(offset);
            self.start_game();
            let campaign = self.live_m03_expansion_campaign();
            if campaign.world.topology_revision != 3 || campaign.world.layout_revision != 5 {
                continue;
            }
            let player = campaign.player;
            let faction = &campaign.factions[&player];
            if faction.capital != headquarters || faction.headquarters != headquarters {
                continue;
            }
            if let Some(center) = target {
                if let Some(path) = m03_safe_city_path(campaign, &self.data, headquarters, center) {
                    return path;
                }
            } else if engine::city_development_option(campaign, &self.data, player, headquarters)
                .is_some_and(|option| option.blocked.is_none())
            {
                return vec![headquarters];
            }
        }
        panic!("the deterministic M03 expansion seed search could not prepare {headquarters:?}");
    }

    fn invest_m03_expansion_city(
        &mut self,
        site: SiteId,
        actor: Actor,
        owner: kestrum::data::world::FactionId,
    ) {
        let (before, cost) = {
            let campaign = self.live_m03_expansion_campaign();
            assert_eq!(campaign.active_faction(), owner);
            let option = engine::city_development_option(campaign, &self.data, owner, site)
                .expect("the M03 city is controlled by its acting faction");
            assert!(option.blocked.is_none(), "M03 city investment is legal");
            (campaign.factions[&owner].resources, option.cost)
        };
        let (data, campaign) = self.m03_expansion_campaign_data_mut();
        engine::apply(campaign, data, actor, Command::DevelopCity { site })
            .expect("the captured M03 site accepts normal paid city development");
        assert_eq!(
            campaign.factions[&owner].resources,
            Resources {
                gold: before.gold - cost.gold,
                wood: before.wood - cost.wood,
                stone: before.stone - cost.stone,
            },
            "M03 city capture pays the authored city-investment cost"
        );
        assert_eq!(
            campaign.world.site(site).unwrap().habitation,
            Habitation::City
        );
        campaign
            .validate(data)
            .expect("the paid M03 city campaign validates");
        self.invalidate_projection();
        self.refresh_projection();
    }

    fn travel_m03_expansion_path(&mut self, path: &[SiteId]) {
        assert!(path.len() >= 2, "major city requires real movement");
        let campaign = self.live_m03_expansion_campaign();
        let player = campaign.player;
        let army = campaign
            .armies
            .values()
            .find(|army| army.faction == player && army.site == path[0])
            .expect("the M03 major-city route starts at its authored HQ")
            .id;
        for edge in path.windows(2) {
            self.travel_m03_expansion_edge(army, edge[0], edge[1]);
        }
        assert_eq!(
            self.live_m03_expansion_campaign().armies[&army].site,
            *path.last().unwrap()
        );
        assert_eq!(
            self.live_m03_expansion_campaign()
                .world
                .site(*path.last().unwrap())
                .unwrap()
                .controller,
            Some(player),
            "ordinary movement claims the major city site"
        );
    }

    fn travel_m03_expansion_edge(
        &mut self,
        army: kestrum::state::military::ArmyId,
        from: SiteId,
        to: SiteId,
    ) {
        assert_eq!(
            self.live_m03_expansion_campaign().armies[&army].site,
            from,
            "the selected M03 edge starts at the army's actual location"
        );
        let order = MoveOrder {
            armies: vec![army],
            path: vec![from, to],
        };
        let preview = {
            let campaign = self.live_m03_expansion_campaign();
            engine::movement_order_preview(campaign, &self.data, campaign.player, &order)
                .expect("the next discovered M03 route previews normally")
        };
        assert!(preview.blocked.is_none());
        assert!(preview.encounter.is_none());
        assert!(preview.stop.as_ref().is_none_or(|stop| {
            matches!(stop.reason, MovementBlock::InsufficientMovement { .. })
        }));
        let (data, campaign) = self.m03_expansion_campaign_data_mut();
        engine::apply(campaign, data, Actor::Player, Command::Move(order))
            .expect("the selected M03 country route is a legal player move");
        self.invalidate_projection();
        self.refresh_projection();
        for _ in 0..8 {
            if self.live_m03_expansion_campaign().armies[&army].site == to {
                return;
            }
            assert!(self
                .live_m03_expansion_campaign()
                .movement_plans
                .iter()
                .any(|plan| plan.armies.contains(&army)));
            self.advance_m03_expansion_round();
        }
        assert_eq!(
            self.live_m03_expansion_campaign().armies[&army].site,
            to,
            "queued M03 edge completes through ordinary seasonal movement"
        );
    }

    fn advance_m03_expansion_round(&mut self) {
        self.apply_m03_expansion_end_turn(Actor::Player);
        for _ in 0..self.live_m03_expansion_campaign().factions.len() + 1 {
            if matches!(
                self.live_m03_expansion_campaign().phase,
                CampaignPhase::PlayerTurn
            ) {
                break;
            }
            let faction = self.live_m03_expansion_campaign().active_faction();
            self.apply_m03_expansion_end_turn(Actor::Npc(faction));
        }
        assert!(matches!(
            self.live_m03_expansion_campaign().phase,
            CampaignPhase::PlayerTurn
        ));
    }

    fn apply_m03_expansion_end_turn(&mut self, actor: Actor) {
        let (data, campaign) = self.m03_expansion_campaign_data_mut();
        engine::apply(campaign, data, actor, Command::EndTurn)
            .expect("ordinary faction turns advance the M03 city route");
        self.invalidate_projection();
        self.refresh_projection();
    }

    fn enter_m03_expansion_city(
        &mut self,
        center: SiteId,
        marker: MarkerId,
        nearby_sites: &[SiteId],
    ) {
        let world = self.live_m03_expansion_campaign().world.clone();
        assert!(world.is_region_available(marker));
        let local_sites = world.region_sites(marker);
        assert!(local_sites.contains(&center));
        for site in nearby_sites {
            assert!(
                local_sites.contains(site),
                "the M03 city neighborhood contains its authored landscape site {site:?}"
            );
        }
        let local_terrain = world
            .terrain_features
            .iter()
            .filter(|feature| world.region_site_position(marker, feature.anchor).is_some())
            .collect::<Vec<_>>();
        assert!(
            !local_terrain.is_empty(),
            "the M03 city neighborhood contains anchored authored terrain"
        );
        self.navigation.show_world(&mut self.view);
        self.navigation
            .enter_region(&world, marker, &mut self.view)
            .expect("paid M03 city opens its connected countryside");
        assert!(matches!(self.navigation.scope(), MapScope::Region(id) if id == marker));
        self.navigation.clear_selection();
    }

    fn clear_m03_expansion_capture_feedback(&mut self) {
        self.apply(UiAction::Notification(ui::NotificationAction::MarkAllRead));
        self.apply(UiAction::Notification(ui::NotificationAction::DismissRead));
        self.navigation.clear_selection();
        self.notice = None;
    }

    fn live_m03_expansion_campaign(&self) -> &StrategicCampaign {
        self.state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("M03 expansion capture uses a strategic campaign")
    }

    fn m03_expansion_campaign_data_mut(&mut self) -> (&GameData, &mut StrategicCampaign) {
        let data = &self.data;
        let campaign = match self
            .state
            .campaign
            .as_mut()
            .expect("M03 expansion capture has a production campaign")
        {
            Campaign::Strategic(campaign) => campaign.as_mut(),
            Campaign::Shell(_) => panic!("M03 expansion capture needs a strategic campaign"),
        };
        (data, campaign)
    }
}

fn m03_safe_city_path(
    campaign: &StrategicCampaign,
    data: &GameData,
    headquarters: SiteId,
    center: SiteId,
) -> Option<Vec<SiteId>> {
    let player = campaign.player;
    let target = campaign.world.site(center)?;
    if target.habitation < data.development.city_development.minimum_habitation
        || data
            .development
            .geography_caps
            .get(&target.geography)
            .is_none_or(|cap| *cap < Habitation::City)
        || target.controller.is_some_and(|owner| owner != player)
        || target.military != MilitaryLayer::None
        || campaign.world.contested_sites.contains(&center)
        || campaign.sieges.contains_key(&center)
        || campaign.active_threat(center).is_some()
        || campaign
            .world
            .site_damage
            .get(&center)
            .copied()
            .unwrap_or_default()
            >= data.economy.facility_failure_damage
        || campaign
            .world
            .adjacent_sites(center)
            .into_iter()
            .any(|neighbor| {
                campaign
                    .world
                    .site(neighbor)
                    .is_some_and(|site| site.habitation >= Habitation::City)
            })
    {
        return None;
    }

    let mut pending = VecDeque::from([headquarters]);
    let mut previous = BTreeMap::new();
    let mut visited = BTreeSet::from([headquarters]);
    while let Some(site) = pending.pop_front() {
        if site == center {
            let mut path = vec![center];
            let mut current = center;
            while current != headquarters {
                current = *previous.get(&current)?;
                path.push(current);
            }
            path.reverse();
            return (path.len() <= 12).then_some(path);
        }
        for next in campaign.world.adjacent_sites(site) {
            if visited.contains(&next) || !m03_capture_route_site(campaign, next, headquarters) {
                continue;
            }
            visited.insert(next);
            previous.insert(next, site);
            pending.push_back(next);
        }
    }
    None
}

fn m03_capture_route_site(
    campaign: &StrategicCampaign,
    site_id: SiteId,
    headquarters: SiteId,
) -> bool {
    if site_id == headquarters {
        return true;
    }
    let Some(site) = campaign.world.site(site_id) else {
        return false;
    };
    site.controller.is_none()
        && site.military == MilitaryLayer::None
        && !campaign.world.contested_sites.contains(&site_id)
        && !campaign.sieges.contains_key(&site_id)
        && campaign.active_threat(site_id).is_none()
        && campaign
            .world
            .site_damage
            .get(&site_id)
            .copied()
            .unwrap_or_default()
            < 100
        && !campaign.armies.values().any(|army| army.site == site_id)
}
