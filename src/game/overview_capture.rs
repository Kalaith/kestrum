//! Observer-safe overview stress scenes; authored conditions pass normal save validation.

use super::*;
use kestrum::{
    data::{
        economy::{Habitation, Resources},
        world::{MarkerId, MarkerLocation, SiteId},
    },
    engine::Actor,
    navigation::MapSelection,
    state::{
        military::ArmyId,
        threat::{Threat, ThreatStatus},
        CampaignPhase, StrategicCampaign,
    },
};

struct OverviewFixture {
    campaign: StrategicCampaign,
    region: MarkerId,
    threat: SiteId,
    army: ArmyId,
}

impl Game {
    pub(super) fn capture_overview(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        if !matches!(
            scene,
            "production_world"
                | "overview_urgent"
                | "overview_region"
                | "overview_attention"
                | "overview_siege"
                | "overview_deficit"
        ) {
            return false;
        }
        if matches!(scene, "overview_siege" | "overview_deficit") {
            if scene == "overview_siege" {
                assert!(self.capture_siege_scene("siege_defender"));
            } else {
                self.capture_army("army_deficit");
            }
            if let Some(Campaign::Strategic(campaign)) = &mut self.state.campaign {
                campaign.tutorial.dismiss();
            }
            self.state.overlay = Overlay::None;
            self.refresh_projection();
            let visible = self.projection.as_ref().unwrap();
            if scene == "overview_siege" {
                assert!(!visible.sieges.is_empty());
            } else {
                assert!(visible
                    .factions
                    .iter()
                    .find(|faction| faction.id == visible.observer)
                    .and_then(|faction| faction.last_economy.as_ref())
                    .is_some_and(|receipt| receipt.shortfall > 0));
            }
            self.navigation.show_world(&mut self.view);
            self.navigation.clear_selection();
            self.view.reset();
            self.notice = None;
            return true;
        }
        self.setup.factions = 8;
        self.setup.seed = self.data.production_layout.default_seed;
        self.start_game();
        if scene == "production_world" {
            self.capture_hidden_attention();
            return true;
        }
        let campaign = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .expect("production overview campaign")
            .clone();
        let fixture = overview_fixture(campaign, &self.data);
        self.state
            .load_campaign(Campaign::Strategic(Box::new(fixture.campaign)), &self.data)
            .expect("validated overview stress snapshot");
        self.invalidate_projection();
        self.refresh_projection();
        let visible = self.projection.as_ref().expect("overview projection");
        assert_eq!(visible.factions.len(), 8);
        assert!(visible
            .threats
            .iter()
            .any(|entry| entry.site == fixture.threat));
        assert!(!visible.hostile_presence.is_empty());
        assert_eq!(visible.armies.len(), 2);
        assert!(visible.armies.iter().any(|army| army.id == fixture.army));
        self.capture_attention_actions(fixture.threat, fixture.army);
        self.navigation.show_world(&mut self.view);
        self.navigation.clear_selection();
        self.view.reset();
        self.overview_ui.expanded = scene == "overview_urgent";
        if scene == "overview_region" {
            self.enter_region(fixture.region);
            self.view.reset();
        }
        if scene == "overview_attention" {
            self.apply(UiAction::FocusAttention(engine::AttentionTarget::Site(
                fixture.threat,
            )));
        }
        self.notice = None;
        true
    }

    fn capture_hidden_attention(&mut self) {
        self.refresh_projection();
        let visible = self.projection.as_ref().expect("early projection");
        let campaign = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .unwrap();
        let hidden = campaign
            .world
            .sites
            .iter()
            .find(|site| visible.world.site(site.id).is_none())
            .expect("early campaign still has undiscovered geography")
            .id;
        let original = campaign.clone();
        let selection = self.navigation.selection();
        let camera = self.view;
        self.apply(UiAction::FocusAttention(engine::AttentionTarget::Site(
            hidden,
        )));
        assert_eq!(self.navigation.selection(), selection);
        assert_eq!(self.view, camera);
        assert_eq!(
            self.state.campaign.as_ref().and_then(Campaign::strategic),
            Some(&original)
        );
    }

    fn capture_attention_actions(&mut self, site: SiteId, army: ArmyId) {
        let original = self
            .state
            .campaign
            .as_ref()
            .and_then(Campaign::strategic)
            .unwrap()
            .clone();
        self.apply(UiAction::FocusAttention(engine::AttentionTarget::Army(
            army,
        )));
        assert_eq!(self.movement.armies, vec![army]);
        assert_eq!(self.state.overlay, Overlay::None);
        self.apply(UiAction::FocusAttention(engine::AttentionTarget::Site(
            site,
        )));
        assert_eq!(self.navigation.selection(), Some(MapSelection::Site(site)));
        assert!(self.movement.armies.is_empty());
        assert_eq!(self.state.overlay, Overlay::None);
        assert_eq!(
            self.state.campaign.as_ref().and_then(Campaign::strategic),
            Some(&original)
        );
    }
}

fn overview_fixture(mut campaign: StrategicCampaign, data: &GameData) -> OverviewFixture {
    campaign.tutorial.dismiss();
    let player = campaign.player;
    let home = campaign.factions[&player].headquarters;
    let region = campaign.world.site(home).expect("capital site").marker;
    let army = campaign
        .armies
        .values()
        .find(|army| army.faction == player)
        .expect("founding force")
        .id;
    let second = campaign.armies[&army]
        .formation_ids()
        .nth(1)
        .expect("two founding formations");
    engine::apply(
        &mut campaign,
        data,
        Actor::Player,
        Command::SplitArmy { formation: second },
    )
    .expect("ordinary split for two owned banners");
    let local_threat = campaign
        .threats
        .values()
        .find(|threat| {
            campaign.world.site(threat.site).unwrap().marker == region
                && campaign.world.adjacent_sites(threat.site).contains(&home)
        })
        .expect("known local threat beside regional capital")
        .clone();
    author_developed_holdings(&mut campaign, data);
    // Economy is a real closed season for the authored holdings, never a forecast.
    loop {
        let actor = match campaign.phase {
            CampaignPhase::PlayerTurn => Actor::Player,
            CampaignPhase::NpcTurn { faction, .. } => Actor::Npc(faction),
        };
        engine::apply(&mut campaign, data, actor, Command::EndTurn)
            .expect("ordinary seasonal accounting in overview fixture");
        if campaign.completed_rounds > 0 {
            break;
        }
    }
    let threat = local_threat.site;
    author_local_pressure(&mut campaign, data, local_threat);
    campaign.armies.get_mut(&army).unwrap().name = "The Riverward Guard of Silver Hawthorns".into();
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == home)
        .unwrap()
        .name = "The High Seat of Silver Hawthorns".into();
    campaign
        .validate(data)
        .expect("eight-faction dense overview is a supported campaign state");
    OverviewFixture {
        campaign,
        region,
        threat,
        army,
    }
}

fn author_local_pressure(campaign: &mut StrategicCampaign, data: &GameData, local_threat: Threat) {
    let player = campaign.player;
    let home = campaign.factions[&player].headquarters;
    let region = campaign.world.site(home).unwrap().marker;
    let threat = local_threat.site;
    campaign.threats.insert(local_threat.id, local_threat);
    campaign.reconcile_region_control();
    let contact = campaign
        .world
        .adjacent_sites(home)
        .into_iter()
        .find(|site| {
            campaign.world.site(*site).unwrap().marker == region
                && campaign.active_threat(*site).is_none()
        })
        .expect("safe founding exit for observed contact");
    let enemy = campaign
        .armies
        .values()
        .find(|army| army.faction != player)
        .expect("rival force")
        .id;
    let rival = campaign.armies[&enemy].faction;
    campaign.armies.get_mut(&enemy).unwrap().site = contact;
    campaign
        .set_site_control(data, contact, Some(rival), false)
        .expect("local occupation does not transfer the region claim");
    campaign.world.occupation.insert(contact, 65);
    campaign.world.site_damage.insert(contact, 60);
    campaign
        .set_site_control(data, threat, Some(player), true)
        .expect("known contested internal location");
    engine::apply(
        campaign,
        data,
        Actor::Player,
        Command::DeclareWar { faction: rival },
    )
    .expect("ordinary declaration makes nearby contact hostile");
}

fn author_developed_holdings(campaign: &mut StrategicCampaign, data: &GameData) {
    let homes: Vec<_> = campaign
        .factions
        .values()
        .map(|faction| {
            let marker = campaign.world.site(faction.headquarters).unwrap().marker;
            (faction.id, campaign.world.marker(marker).unwrap().position)
        })
        .collect();
    let positions: std::collections::BTreeMap<_, _> = campaign
        .world
        .markers
        .iter()
        .map(|marker| (marker.id, marker.position))
        .collect();
    for site in &mut campaign.world.sites {
        let position = positions[&site.marker];
        site.controller = homes
            .iter()
            .min_by(|left, right| {
                distance_squared(left.1, position).total_cmp(&distance_squared(right.1, position))
            })
            .map(|(faction, _)| *faction);
        site.habitation = match site.id.0 % 4 {
            0 => Habitation::Town,
            1 => Habitation::Village,
            2 => Habitation::Camp,
            _ => Habitation::City,
        }
        .min(data.development.geography_caps[&site.geography]);
    }
    campaign.factions.get_mut(&campaign.player).unwrap().name =
        "Kingdom of the Silver Hawthorns".into();
    // This authored stress scene knows the atlas; production_world retains actual fog.
    campaign.knowledge.explored.insert(
        campaign.player,
        campaign.world.sites.iter().map(|site| site.id).collect(),
    );
    for threat in campaign.threats.values_mut() {
        threat.headcount = 0;
        threat.status = ThreatStatus::Cleared {
            round: 0,
            by: campaign.player,
            payout: Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            },
        };
    }
    campaign.reconcile_region_control();
    assert!(campaign
        .world
        .markers
        .iter()
        .filter(|marker| matches!(marker.location, MarkerLocation::Region { .. }))
        .all(|marker| campaign.world.region_control[&marker.id]
            .political_owner
            .is_some()));
    campaign
        .validate(data)
        .expect("authored developed holdings");
}

fn distance_squared(left: [f32; 2], right: [f32; 2]) -> f32 {
    (left[0] - right[0]).powi(2) + (left[1] - right[1]).powi(2)
}
