//! Production city and countryside capture states use the normal investment command.

use super::*;
use kestrum::{
    data::{
        economy::{Habitation, Resources},
        world::{FactionId, Geography, SiteId},
    },
    engine::Actor,
    navigation::MapSelection,
    state::{construction::Focus, threat::ThreatStatus, StrategicCampaign},
};

impl Game {
    pub(super) fn capture_city(&mut self, requested: &str) -> bool {
        let scene = requested.trim_end_matches("_minimum");
        match scene {
            "production_region" | "production_region_map" => {
                self.setup.factions = 8;
                self.setup.seed = self.data.production_layout.default_seed;
                self.start_game();
                let (marker, site) = {
                    let campaign = campaign_mut(&mut self.state);
                    let site = invest_in_capital(campaign, &self.data);
                    (campaign.world.site(site).unwrap().marker, site)
                };
                self.refresh_projection();
                let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
                self.navigation
                    .enter_region(&campaign.world, marker, &mut self.view)
                    .expect("paid production capital opens its city vicinity");
                if scene == "production_region" {
                    self.navigation
                        .select(&campaign.world, MapSelection::Site(site))
                        .expect("capital is a local city-view target");
                } else {
                    self.navigation.clear_selection();
                }
                self.notice = None;
                true
            }
            "city_dense" => {
                self.setup.factions = 8;
                self.setup.seed = self.data.production_layout.default_seed;
                self.start_game();
                let marker = {
                    let campaign = campaign_mut(&mut self.state);
                    let center = invest_in_capital(campaign, &self.data);
                    author_city_countryside(campaign, &self.data, center);
                    campaign.world.site(center).unwrap().marker
                };
                self.refresh_projection();
                let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
                self.navigation
                    .enter_region(&campaign.world, marker, &mut self.view)
                    .expect("developed capital opens its city vicinity");
                self.navigation.clear_selection();
                self.notice = None;
                true
            }
            "city_spacing_blocked" => {
                self.setup.factions = 8;
                self.setup.seed = self.data.production_layout.default_seed;
                self.start_game();
                let (marker, target, before) = {
                    let campaign = campaign_mut(&mut self.state);
                    let center = invest_in_capital(campaign, &self.data);
                    let target = author_spacing_block(campaign, &self.data, center);
                    let option = engine::city_development_option(
                        campaign,
                        &self.data,
                        campaign.player,
                        target,
                    )
                    .expect("owned neighboring Village has a city option");
                    let reason = option.blocked.expect("intervening site blocks investment");
                    assert!(
                        reason.contains("at least one site between them"),
                        "unexpected city spacing review: {reason}"
                    );
                    (
                        campaign.world.site(target).unwrap().marker,
                        target,
                        campaign.factions[&campaign.player].resources,
                    )
                };
                self.refresh_projection();
                let campaign = self.state.campaign.as_ref().unwrap().strategic().unwrap();
                self.navigation.show_world(&mut self.view);
                self.navigation
                    .select(&campaign.world, MapSelection::Marker(marker))
                    .expect("owned spacing fixture is visible on the world map");
                self.apply(UiAction::OpenSettlement(target));
                self.apply(UiAction::SettlementTab(ui::SettlementMode::LocalActions));
                self.apply(UiAction::SelectLocalAction(ui::LocalAction::DevelopCity));
                assert_eq!(self.state.overlay, Overlay::Settlement);
                assert_eq!(self.settlement.site, Some(target));
                assert_eq!(self.settlement.mode, ui::SettlementMode::LocalReview);
                assert!(self
                    .settlement
                    .blocked
                    .as_deref()
                    .is_some_and(|reason| reason.contains("at least one site between them")));
                assert_eq!(
                    self.state
                        .campaign
                        .as_ref()
                        .unwrap()
                        .strategic()
                        .unwrap()
                        .factions[&campaign_player(&self.state)]
                        .resources,
                    before,
                    "opening a blocked review spends nothing"
                );
                self.notice = None;
                true
            }
            _ => false,
        }
    }
}

fn campaign_mut(state: &mut GameState) -> &mut StrategicCampaign {
    let Campaign::Strategic(campaign) = state.campaign.as_mut().expect("production campaign")
    else {
        unreachable!("city captures use production campaigns")
    };
    campaign
}

fn campaign_player(state: &GameState) -> FactionId {
    state
        .campaign
        .as_ref()
        .and_then(Campaign::strategic)
        .expect("production campaign")
        .player
}

fn invest_in_capital(campaign: &mut StrategicCampaign, data: &GameData) -> SiteId {
    let site = campaign.factions[&campaign.player].capital;
    let before = campaign.factions[&campaign.player].resources;
    engine::apply(campaign, data, Actor::Player, Command::DevelopCity { site })
        .expect("the supplied production capital can receive a real city investment");
    let cost = data.development.city_development.cost;
    assert_eq!(
        campaign.factions[&campaign.player].resources,
        Resources {
            gold: before.gold - cost.gold,
            wood: before.wood - cost.wood,
            stone: before.stone - cost.stone,
        },
        "the production city capture pays the authored investment cost"
    );
    assert_eq!(
        campaign.world.site(site).unwrap().habitation,
        Habitation::City
    );
    site
}

fn author_spacing_block(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    center: SiteId,
) -> SiteId {
    let faction_sites: std::collections::BTreeSet<_> = campaign
        .factions
        .values()
        .flat_map(|faction| [faction.capital, faction.headquarters])
        .collect();
    let target = campaign
        .world
        .adjacent_sites(center)
        .into_iter()
        .find(|target| {
            !faction_sites.contains(target)
                && !campaign.site_is_ruined(*target)
                && campaign.world.site(*target).is_some_and(|site| {
                    site.habitation >= data.development.city_development.minimum_habitation
                        && site.habitation < Habitation::City
                        && data.development.geography_caps[&site.geography]
                            >= data.development.city_development.minimum_habitation
                })
        })
        .or_else(|| {
            campaign
                .world
                .adjacent_sites(center)
                .into_iter()
                .find(|target| {
                    !faction_sites.contains(target)
                        && !campaign.site_is_ruined(*target)
                        && campaign.world.site(*target).is_some_and(|site| {
                            data.development.geography_caps[&site.geography]
                                >= data.development.city_development.minimum_habitation
                        })
                })
        })
        .expect("production atlas has a usable Village beside its capital");
    let player = campaign.player;
    let target_site = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == target)
        .expect("neighbor site");
    target_site.controller = Some(player);
    target_site.habitation = Habitation::Village;
    campaign.world.population.insert(
        target,
        data.construction.population.minimum[&Habitation::Village],
    );
    campaign.world.contested_sites.remove(&target);
    campaign.world.site_damage.remove(&target);
    campaign.world.occupation.remove(&target);
    let target_neighborhood: std::collections::BTreeSet<_> = campaign
        .world
        .adjacent_sites(target)
        .into_iter()
        .chain([target])
        .collect();
    let local_threats: Vec<_> = campaign
        .threats
        .values()
        .filter(|threat| target_neighborhood.contains(&threat.site))
        .map(|threat| threat.id)
        .collect();
    for id in local_threats {
        let threat = campaign.threats.get_mut(&id).expect("local threat");
        threat.headcount = 0;
        threat.status = ThreatStatus::Cleared {
            round: campaign.completed_rounds,
            by: player,
            payout: Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            },
        };
    }
    relocate_foreign_armies(campaign, player, &target_neighborhood);
    campaign.reconcile_region_control();
    campaign
        .validate(data)
        .expect("validated city-spacing capture fixture");
    assert!(campaign.world.adjacent_sites(center).contains(&target));
    target
}

fn author_city_countryside(campaign: &mut StrategicCampaign, data: &GameData, center: SiteId) {
    let neighbors: Vec<_> = campaign
        .world
        .adjacent_sites(center)
        .into_iter()
        .filter(|site| !campaign.site_is_ruined(*site))
        .collect();
    let player = campaign.player;
    let field = neighbors
        .iter()
        .copied()
        .find(|id| {
            campaign.world.site(*id).is_some_and(|site| {
                matches!(
                    site.geography,
                    Geography::Plains | Geography::River | Geography::Valley
                ) && site.habitation < Habitation::City
                    && data.development.geography_caps[&site.geography] >= Habitation::Village
            })
        })
        .or_else(|| neighbors.iter().copied().next())
        .expect("production capital has neighboring countryside");
    for (index, id) in neighbors.iter().copied().enumerate() {
        let site = campaign
            .world
            .sites
            .iter_mut()
            .find(|site| site.id == id)
            .expect("neighbor site");
        let cap = data.development.geography_caps[&site.geography];
        let tier = if index % 2 == 0 && cap >= Habitation::Town {
            Habitation::Town
        } else {
            Habitation::Village.min(cap)
        };
        site.habitation = tier;
        if id == field {
            site.controller = Some(player);
        }
        campaign.world.focus.insert(id, Focus::Growth);
        campaign.world.development.entry(id).or_default().pressure = 3;
        campaign
            .world
            .population
            .insert(id, data.construction.population.minimum[&tier]);
    }
    campaign.world.contested_sites.remove(&field);
    campaign.world.site_damage.remove(&field);
    campaign.world.occupation.remove(&field);
    let mut field_neighborhood: std::collections::BTreeSet<_> = neighbors.iter().copied().collect();
    field_neighborhood.insert(center);
    let immediate: Vec<_> = field_neighborhood.iter().copied().collect();
    for site in immediate {
        field_neighborhood.extend(campaign.world.adjacent_sites(site));
    }
    let local_threats: Vec<_> = campaign
        .threats
        .values()
        .filter(|threat| field_neighborhood.contains(&threat.site))
        .map(|threat| threat.id)
        .collect();
    for id in local_threats {
        let threat = campaign.threats.get_mut(&id).expect("local threat");
        threat.headcount = 0;
        threat.status = ThreatStatus::Cleared {
            round: campaign.completed_rounds,
            by: player,
            payout: Resources {
                gold: 0,
                wood: 0,
                stone: 0,
            },
        };
    }
    relocate_foreign_armies(campaign, player, &field_neighborhood);
    campaign.reconcile_region_control();
    campaign
        .validate(data)
        .expect("validated developed countryside capture fixture");
}

fn relocate_foreign_armies(
    campaign: &mut StrategicCampaign,
    player: FactionId,
    forbidden: &std::collections::BTreeSet<SiteId>,
) {
    let refuges: std::collections::BTreeMap<_, _> = campaign
        .factions
        .values()
        .map(|faction| {
            let refuge = std::iter::once(faction.headquarters)
                .chain(
                    campaign
                        .world
                        .sites
                        .iter()
                        .filter(|site| site.controller == Some(faction.id))
                        .map(|site| site.id),
                )
                .chain(campaign.world.sites.iter().map(|site| site.id))
                .find(|site| !forbidden.contains(site))
                .expect("production world has a site outside the local fixture");
            (faction.id, refuge)
        })
        .collect();
    for army in campaign.armies.values_mut() {
        if army.faction != player && forbidden.contains(&army.site) {
            army.site = refuges[&army.faction];
        }
    }
}
