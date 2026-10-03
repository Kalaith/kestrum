//! City investment remains transactional, terrain-independent and spaced.

#[path = "support/phases.rs"]
mod phases;

use kestrum::{
    data::{
        economy::{Habitation, Resources},
        world::{DiplomaticState, FactionId, Geography, SiteId, SiteTag},
        GameData,
    },
    engine::{
        apply, city_development_option, development_view, preview, Actor, Command, RuleError,
    },
    state::{
        campaign::DomainFactKind, development::DevelopmentReceipt, CampaignPhase, StrategicCampaign,
    },
};

fn fixture(geography: Option<Geography>) -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    for site in &mut data.scenario.sites {
        site.tags.retain(|tag| *tag != SiteTag::Ruins);
        if [1, 5, 6, 8, 10, 11, 12].contains(&site.id.0) {
            site.controller = Some(FactionId(1));
        }
    }
    for relation in &mut data.scenario.relations {
        relation.state = DiplomaticState::Peace;
    }
    let target = data
        .scenario
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(5))
        .unwrap();
    target.habitation = Habitation::Village;
    if let Some(geography) = geography {
        target.geography = geography;
    }
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    for faction in campaign.factions.values_mut() {
        faction.resources = Resources {
            gold: 100_000,
            wood: 100_000,
            stone: 100_000,
        };
    }
    (data, campaign)
}

fn site_mut(campaign: &mut StrategicCampaign, id: SiteId) -> &mut kestrum::data::world::Site {
    campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == id)
        .unwrap()
}

fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while !matches!(campaign.phase, CampaignPhase::PlayerTurn) {
        phases::pass_npc(campaign, data).unwrap();
    }
}

fn assert_blocked_action(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    site: SiteId,
    reason: &str,
) {
    let option = city_development_option(campaign, data, FactionId(1), site).unwrap();
    assert!(option
        .blocked
        .is_some_and(|blocked| blocked.contains(reason)));
    let before = campaign.clone();
    assert!(apply(campaign, data, Actor::Player, Command::DevelopCity { site }).is_err());
    assert_eq!(*campaign, before);
}

#[test]
fn investment_pays_once_preserves_people_and_state_and_roundtrips() {
    let (data, mut campaign) = fixture(None);
    let site = SiteId(5);
    let cost = data.development.city_development.cost;
    let population = campaign.world.population[&site];
    let development = campaign.world.development[&site].clone();
    let starting_resources = campaign.factions[&FactionId(1)].resources;
    let before = campaign.clone();
    let option = city_development_option(&campaign, &data, FactionId(1), site).unwrap();
    assert_eq!(option.cost, cost);
    assert_eq!(option.blocked, None);
    preview(
        &campaign,
        &data,
        Actor::Player,
        Command::DevelopCity { site },
    )
    .unwrap();
    assert_eq!(campaign, before);

    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DevelopCity { site },
    )
    .unwrap();
    assert_eq!(
        campaign.world.site(site).unwrap().habitation,
        Habitation::City
    );
    assert_eq!(campaign.world.population[&site], population);
    assert_eq!(campaign.world.development[&site], development);
    assert_eq!(
        campaign.factions[&FactionId(1)].resources,
        Resources {
            gold: starting_resources.gold - cost.gold,
            wood: starting_resources.wood - cost.wood,
            stone: starting_resources.stone - cost.stone,
        }
    );
    assert!(outcome.facts.iter().any(|fact| matches!(
        &fact.kind,
        DomainFactKind::DevelopmentChanged {
            receipt: DevelopmentReceipt::HabitationChanged {
                site: changed,
                owner: Some(FactionId(1)),
                from: Habitation::Village,
                to: Habitation::City,
            }
        } if *changed == site
    )));
    let restored: StrategicCampaign =
        serde_json::from_str(&serde_json::to_string(&campaign).unwrap()).unwrap();
    assert_eq!(restored, campaign);
    restored.validate(&data).unwrap();
    let after_investment = campaign.clone();
    assert!(matches!(
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::DevelopCity { site }
        ),
        Err(RuleError::Development(_))
    ));
    assert_eq!(campaign, after_investment);

    let (poor_data, mut poor) = fixture(None);
    poor.factions.get_mut(&FactionId(1)).unwrap().resources = Resources {
        gold: cost.gold - 1,
        wood: cost.wood,
        stone: cost.stone,
    };
    let poor_before = poor.clone();
    assert!(
        city_development_option(&poor, &poor_data, FactionId(1), site)
            .unwrap()
            .blocked
            .is_some()
    );
    assert!(matches!(
        apply(
            &mut poor,
            &poor_data,
            Actor::Player,
            Command::DevelopCity { site }
        ),
        Err(RuleError::InsufficientResources { .. })
    ));
    assert_eq!(poor, poor_before);
}

#[test]
fn adjacent_city_blocks_investment_across_faction_ownership_and_city_tiers() {
    for neighbor in [(FactionId(1), SiteId(1)), (FactionId(2), SiteId(2))] {
        for habitation in [Habitation::City, Habitation::MajorCity] {
            let (data, mut campaign) = fixture(None);
            site_mut(&mut campaign, neighbor.1).habitation = habitation;
            let option =
                city_development_option(&campaign, &data, FactionId(1), SiteId(5)).unwrap();
            assert!(option
                .blocked
                .is_some_and(|reason| reason.contains("leave at least one site")));
        }
    }

    let (data, mut campaign) = fixture(None);
    site_mut(&mut campaign, SiteId(1)).habitation = Habitation::City;
    assert_eq!(
        city_development_option(&campaign, &data, FactionId(1), SiteId(6))
            .unwrap()
            .blocked,
        None,
        "one intervening physical site keeps the investment eligible"
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DevelopCity { site: SiteId(6) },
    )
    .unwrap();
    assert_eq!(
        campaign.world.site(SiteId(6)).unwrap().habitation,
        Habitation::City
    );
}

#[test]
fn every_geography_accepts_city_investment_and_keeps_city_capacity() {
    for geography in [
        Geography::Plains,
        Geography::Forest,
        Geography::Hill,
        Geography::River,
        Geography::Valley,
        Geography::Coast,
        Geography::Marsh,
        Geography::Pass,
        Geography::Island,
    ] {
        let (data, mut campaign) = fixture(Some(geography));
        let site = SiteId(5);
        assert_eq!(
            city_development_option(&campaign, &data, FactionId(1), site)
                .unwrap()
                .blocked,
            None,
            "{geography:?}"
        );
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::DevelopCity { site },
        )
        .unwrap();
        let view = development_view(&campaign, &data, FactionId(1), site).unwrap();
        assert_eq!(view.habitation, Habitation::City, "{geography:?}");
        assert!(view.maximum_habitation >= Habitation::City, "{geography:?}");
    }
}

#[test]
fn town_needs_investment_and_a_declined_city_can_be_invested_in_again() {
    let (data, mut campaign) = fixture(None);
    let site = SiteId(5);
    site_mut(&mut campaign, site).habitation = Habitation::Town;
    campaign.world.population.insert(
        site,
        data.construction.population.minimum[&Habitation::City],
    );
    campaign.world.development.get_mut(&site).unwrap().pressure = 24;
    finish_round(&mut campaign, &data);
    assert_eq!(
        campaign.world.site(site).unwrap().habitation,
        Habitation::Town
    );
    assert_eq!(campaign.world.development[&site].pressure, 24);
    assert!(development_view(&campaign, &data, FactionId(1), site)
        .unwrap()
        .growth_blocked
        .is_some_and(|reason| reason.contains("city investment")));

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DevelopCity { site },
    )
    .unwrap();
    campaign.world.development.get_mut(&site).unwrap().pressure = -24;
    finish_round(&mut campaign, &data);
    assert_eq!(
        campaign.world.site(site).unwrap().habitation,
        Habitation::Town
    );
    assert_eq!(
        city_development_option(&campaign, &data, FactionId(1), site)
            .unwrap()
            .blocked,
        None
    );
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DevelopCity { site },
    )
    .unwrap();
    assert_eq!(
        campaign.world.site(site).unwrap().habitation,
        Habitation::City
    );
}

#[test]
fn eligibility_reports_settlement_condition_blockers_without_partial_actions() {
    #[derive(Clone, Copy)]
    enum Blocker {
        BelowVillage,
        Ruined,
        Occupied,
        Damaged,
        Contested,
    }

    for (blocker, expected) in [
        (Blocker::BelowVillage, "Village or larger"),
        (Blocker::Ruined, "ruined site"),
        (Blocker::Occupied, "Occupation pressure"),
        (Blocker::Damaged, "structural damage"),
        (Blocker::Contested, "contested or besieged"),
    ] {
        let (data, mut campaign) = fixture(None);
        let site = SiteId(5);
        match blocker {
            Blocker::BelowVillage => {
                site_mut(&mut campaign, site).habitation = Habitation::Hamlet;
            }
            Blocker::Ruined => {
                let completed_rounds = campaign.completed_rounds;
                let state = campaign.world.development.get_mut(&site).unwrap();
                state.ruined = true;
                state.ruination = 1;
                state.ruined_round = Some(completed_rounds);
            }
            Blocker::Occupied => {
                campaign
                    .world
                    .occupation
                    .insert(site, data.development.conditions.occupation_threshold);
            }
            Blocker::Damaged => {
                campaign
                    .world
                    .site_damage
                    .insert(site, data.economy.facility_failure_damage);
            }
            Blocker::Contested => campaign
                .set_site_control(&data, site, Some(FactionId(1)), true)
                .unwrap(),
        }
        assert_blocked_action(&mut campaign, &data, site, expected);
    }

    let (data, mut disconnected) = fixture(None);
    let site = SiteId(8);
    site_mut(&mut disconnected, site).habitation = Habitation::Village;
    disconnected
        .set_site_control(&data, SiteId(5), None, false)
        .unwrap();
    assert_blocked_action(&mut disconnected, &data, site, "supply path");
}
