//! Guide persistence and command receipts without fabricating simulation history.

use kestrum::{
    data::{
        economy::{Habitation, Resources},
        generation::ProductionSetup,
        rules::Emblem,
        world::{FactionId, SiteId},
        GameData,
    },
    engine::{self, Command, MoveOrder, RuleError},
    state::{
        campaign::DomainFactKind,
        development::DevelopmentReceipt,
        military::ArmyId,
        threat::ThreatStatus,
        tutorial::{TutorialProgress, TutorialStep, TUTORIAL_STEPS},
        Campaign, GameState, Overlay, StrategicCampaign,
    },
};

fn fixture() -> (GameData, GameState) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    let mut state = GameState::default();
    state.new_game(&data).unwrap();
    (data, state)
}

fn production_fixture() -> (GameData, GameState) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    let setup = ProductionSetup {
        kingdom_name: "Tutorial Rose".into(),
        emblem: Emblem::Rose,
        factions: data.rules.min_factions,
        seed: data.production_layout.default_seed,
    };
    let mut campaign = StrategicCampaign::new_production(&data, &setup).unwrap();
    let player = campaign.player;
    for threat in campaign.threats.values_mut() {
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
    campaign.reconcile_region_control();
    let capital = campaign.factions[&player].capital;
    campaign.factions.get_mut(&player).unwrap().resources = Resources {
        gold: 10_000,
        wood: 10_000,
        stone: 10_000,
    };
    clear_foreign_armies_near(&mut campaign, player, capital);
    campaign.validate(&data).unwrap();
    let mut state = GameState::default();
    state
        .load_campaign(Campaign::Strategic(Box::new(campaign)), &data)
        .unwrap();
    (data, state)
}

fn campaign_mut(state: &mut GameState) -> &mut StrategicCampaign {
    match state.campaign.as_mut().expect("campaign") {
        Campaign::Strategic(campaign) => campaign,
        Campaign::Shell(_) => panic!("production fixture is strategic"),
    }
}

fn guide_mut(state: &mut GameState) -> &mut TutorialProgress {
    &mut campaign_mut(state).tutorial
}

fn clear_foreign_armies_near(campaign: &mut StrategicCampaign, player: FactionId, center: SiteId) {
    let mut local = campaign.world.adjacent_sites(center);
    local.insert(center);
    let outside: Vec<_> = campaign
        .world
        .sites
        .iter()
        .map(|site| site.id)
        .filter(|site| !local.contains(site))
        .collect();
    for army in campaign.armies.values_mut() {
        if army.faction != player && local.contains(&army.site) {
            if let Some(refuge) = outside.first() {
                army.site = *refuge;
            }
        }
    }
}

fn author_second_capital(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    old_capital: SiteId,
) -> SiteId {
    let player = campaign.player;
    let faction_sites: std::collections::BTreeSet<_> = campaign
        .factions
        .values()
        .flat_map(|faction| [faction.capital, faction.headquarters])
        .collect();
    assert!(campaign
        .world
        .supplied_sites(player, campaign.factions[&player].headquarters)
        .contains(&old_capital));
    let old_neighbors = campaign.world.adjacent_sites(old_capital);
    let path = old_neighbors.iter().find_map(|middle| {
        if faction_sites.contains(middle)
            || campaign.site_is_ruined(*middle)
            || data.development.geography_caps[&campaign.world.site(*middle)?.geography]
                < Habitation::Village
        {
            return None;
        }
        campaign
            .world
            .adjacent_sites(*middle)
            .into_iter()
            .find_map(|target| {
                let site = campaign.world.site(target)?;
                (target != old_capital
                    && !old_neighbors.contains(&target)
                    && !faction_sites.contains(&target)
                    && !campaign.site_is_ruined(target)
                    && data.development.geography_caps[&site.geography] >= Habitation::Town
                    && !campaign
                        .world
                        .adjacent_sites(target)
                        .iter()
                        .any(|neighbor| {
                            campaign
                                .world
                                .site(*neighbor)
                                .is_some_and(|neighbor| neighbor.habitation >= Habitation::City)
                        }))
                .then_some((*middle, target))
            })
    });
    let (middle, candidate) = path.expect("production atlas has a two-route capital option");
    for (site_id, habitation) in [(middle, Habitation::Village), (candidate, Habitation::Town)] {
        let site = campaign
            .world
            .sites
            .iter_mut()
            .find(|site| site.id == site_id)
            .expect("selected connected site");
        site.controller = Some(player);
        site.habitation = habitation;
        campaign
            .world
            .development
            .entry(site_id)
            .or_default()
            .ruined = false;
        campaign
            .world
            .population
            .insert(site_id, data.construction.population.minimum[&habitation]);
        campaign.world.site_damage.remove(&site_id);
        campaign.world.occupation.remove(&site_id);
        campaign.world.contested_sites.remove(&site_id);
    }
    clear_foreign_armies_near(campaign, player, candidate);
    campaign.reconcile_region_control();
    campaign
        .validate(data)
        .expect("authored Town capital fixture");
    assert!(campaign.world.adjacent_sites(old_capital).contains(&middle));
    assert!(campaign.world.adjacent_sites(middle).contains(&candidate));
    assert!(!old_neighbors.contains(&candidate));
    candidate
}

fn guide(state: &GameState) -> &TutorialProgress {
    &state
        .campaign
        .as_ref()
        .unwrap()
        .strategic()
        .unwrap()
        .tutorial
}

fn movement(destination: u32) -> Command {
    Command::Move(MoveOrder {
        armies: vec![ArmyId(1)],
        path: vec![SiteId(1), SiteId(destination)],
    })
}

#[test]
fn new_campaigns_start_guidance_and_older_saves_opt_in_without_other_changes() {
    let (data, state) = fixture();
    assert_eq!(guide(&state).current(), Some(TutorialStep::Headquarters));
    let original = state.campaign.unwrap().strategic().unwrap().clone();
    let mut old = serde_json::to_value(&original).unwrap();
    old.as_object_mut().unwrap().remove("tutorial");
    let mut loaded: StrategicCampaign = serde_json::from_value(old).unwrap();
    loaded.validate(&data).unwrap();
    assert_eq!(loaded.tutorial.current(), None);
    loaded.tutorial.reopen();
    assert_eq!(loaded, original);
}

#[test]
fn out_of_order_completion_is_idempotent_and_dismissal_does_not_complete_actions() {
    let mut progress = TutorialProgress::new();
    progress.record(TutorialStep::Career);
    progress.record(TutorialStep::Career);
    progress.record(TutorialStep::Headquarters);
    assert_eq!(progress.current(), Some(TutorialStep::CityDevelopment));
    progress.dismiss();
    assert_eq!(progress.current(), None);
    assert!(!progress.completed(TutorialStep::Movement));
    let mut loaded: TutorialProgress =
        serde_json::from_str(&serde_json::to_string(&progress).unwrap()).unwrap();
    loaded.reopen();
    assert_eq!(loaded.current(), Some(TutorialStep::CityDevelopment));
    for step in TUTORIAL_STEPS {
        loaded.record(step);
    }
    assert!(loaded.is_complete());
    assert_eq!(loaded.current(), None);
    loaded.reopen();
    assert_eq!(loaded.current(), Some(TutorialStep::Headquarters));
    assert!(!loaded.is_complete());
}

#[test]
fn lesson_order_places_paid_city_investment_between_headquarters_and_region() {
    assert_eq!(
        TUTORIAL_STEPS,
        [
            TutorialStep::Headquarters,
            TutorialStep::CityDevelopment,
            TutorialStep::Region,
            TutorialStep::WorldMap,
            TutorialStep::Movement,
            TutorialStep::Career,
            TutorialStep::Household,
            TutorialStep::FirstTurn,
            TutorialStep::Records,
        ]
    );
    let (data, mut state) = production_fixture();
    let player = state.campaign.as_ref().unwrap().strategic().unwrap().player;
    let capital = state
        .campaign
        .as_ref()
        .unwrap()
        .strategic()
        .unwrap()
        .factions[&player]
        .capital;
    campaign_mut(&mut state)
        .tutorial
        .record(TutorialStep::Headquarters);
    assert_eq!(guide(&state).current(), Some(TutorialStep::CityDevelopment));

    state.overlay = Overlay::Settlement;
    let before = state
        .campaign
        .as_ref()
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    let starting = before.factions[&before.player].resources;
    let outcome = state
        .command(&data, Command::DevelopCity { site: capital })
        .unwrap();
    let cost = data.development.city_development.cost;
    let invested = state.campaign.as_ref().unwrap().strategic().unwrap();
    assert_eq!(
        invested.factions[&invested.player].resources,
        Resources {
            gold: starting.gold - cost.gold,
            wood: starting.wood - cost.wood,
            stone: starting.stone - cost.stone,
        }
    );
    assert!(outcome.facts.iter().any(|fact| matches!(
        &fact.kind,
        DomainFactKind::DevelopmentChanged {
            receipt: DevelopmentReceipt::HabitationChanged {
                site,
                owner: Some(owner),
                from: Habitation::Village,
                to: Habitation::City,
            }
        } if *site == capital && *owner == player
    )));
    assert!(guide(&state).completed(TutorialStep::CityDevelopment));
    assert_eq!(guide(&state).current(), Some(TutorialStep::Region));
    assert_eq!(
        before.world.site(capital).unwrap().habitation,
        Habitation::Village
    );
}

#[test]
fn obstructed_and_unaffordable_city_actions_leave_campaign_and_lesson_unchanged() {
    let (data, mut state) = production_fixture();
    let player = state.campaign.as_ref().unwrap().strategic().unwrap().player;
    let capital = state
        .campaign
        .as_ref()
        .unwrap()
        .strategic()
        .unwrap()
        .factions[&player]
        .capital;
    campaign_mut(&mut state)
        .tutorial
        .record(TutorialStep::Headquarters);
    let untouched = state.campaign.clone();
    state.overlay = Overlay::Help;
    assert!(matches!(
        state.command(&data, Command::DevelopCity { site: capital }),
        Err(RuleError::PlayObstructed)
    ));
    assert_eq!(state.campaign, untouched);
    assert_eq!(guide(&state).current(), Some(TutorialStep::CityDevelopment));

    state.overlay = Overlay::Settlement;
    campaign_mut(&mut state)
        .factions
        .get_mut(&player)
        .unwrap()
        .resources
        .gold = data.development.city_development.cost.gold - 1;
    let poor_before = state.campaign.clone();
    let option = engine::city_development_option(
        state.campaign.as_ref().unwrap().strategic().unwrap(),
        &data,
        player,
        capital,
    )
    .unwrap();
    assert!(option.blocked.is_some());
    assert!(matches!(
        state.command(&data, Command::DevelopCity { site: capital }),
        Err(RuleError::InsufficientResources { .. })
    ));
    assert_eq!(state.campaign, poor_before);
    assert_eq!(guide(&state).current(), Some(TutorialStep::CityDevelopment));
}

#[test]
fn only_the_current_capital_investment_receives_city_progress_after_capital_moves() {
    let (data, mut state) = production_fixture();
    let (old_capital, player) = {
        let campaign = campaign_mut(&mut state);
        campaign.tutorial.record(TutorialStep::Headquarters);
        (campaign.factions[&campaign.player].capital, campaign.player)
    };
    let new_capital = author_second_capital(campaign_mut(&mut state), &data, old_capital);
    state.overlay = Overlay::Settlement;
    state
        .command(&data, Command::MoveCapital { site: new_capital })
        .expect("validated connected Town can become capital");
    assert_eq!(
        state
            .campaign
            .as_ref()
            .unwrap()
            .strategic()
            .unwrap()
            .factions[&player]
            .capital,
        new_capital
    );
    assert!(!guide(&state).completed(TutorialStep::CityDevelopment));

    state
        .command(&data, Command::DevelopCity { site: old_capital })
        .expect("former capital accepts ordinary city investment");
    assert_eq!(
        state
            .campaign
            .as_ref()
            .unwrap()
            .strategic()
            .unwrap()
            .world
            .site(old_capital)
            .unwrap()
            .habitation,
        Habitation::City
    );
    assert!(!guide(&state).completed(TutorialStep::CityDevelopment));
    assert_eq!(guide(&state).current(), Some(TutorialStep::CityDevelopment));

    state
        .command(&data, Command::DevelopCity { site: new_capital })
        .expect("current capital meets city spacing and investment rules");
    assert!(guide(&state).completed(TutorialStep::CityDevelopment));
    assert_eq!(guide(&state).current(), Some(TutorialStep::Region));
    let saved: Campaign =
        serde_json::from_str(&serde_json::to_string(state.campaign.as_ref().unwrap()).unwrap())
            .unwrap();
    let mut resumed = GameState::default();
    resumed.load_campaign(saved, &data).unwrap();
    assert!(guide(&resumed).completed(TutorialStep::CityDevelopment));
    guide_mut(&mut resumed).reopen();
    assert_eq!(guide(&resumed).current(), Some(TutorialStep::Region));
}

#[test]
fn invalid_and_obstructed_orders_never_advance_the_guide() {
    let (data, mut state) = fixture();
    let untouched = state.campaign.clone();
    assert!(state.command(&data, movement(999_999)).is_err());
    assert_eq!(state.campaign, untouched);
    state.overlay = Overlay::Help;
    assert!(state.command(&data, movement(5)).is_err());
    assert!(state.end_turn(&data).is_err());
    assert_eq!(state.campaign, untouched);
    assert!(!guide(&state).completed(TutorialStep::Movement));
    assert!(!guide(&state).completed(TutorialStep::FirstTurn));
}

#[test]
fn actual_movement_and_end_turn_are_saved_and_do_not_mark_other_lessons() {
    let (data, mut state) = fixture();
    state.overlay = Overlay::MoveReview;
    let result = state.command(&data, movement(5)).unwrap();
    assert_eq!(result.movement.unwrap().path, [SiteId(1), SiteId(5)]);
    assert!(guide(&state).completed(TutorialStep::Movement));
    state.overlay = Overlay::None;
    state.end_turn(&data).unwrap();
    assert!(guide(&state).completed(TutorialStep::FirstTurn));
    assert!(!guide(&state).completed(TutorialStep::Headquarters));
    assert!(!guide(&state).completed(TutorialStep::Records));
    let saved: Campaign =
        serde_json::from_str(&serde_json::to_string(state.campaign.as_ref().unwrap()).unwrap())
            .unwrap();
    let mut resumed = GameState::default();
    resumed.load_campaign(saved, &data).unwrap();
    assert_eq!(guide(&resumed), guide(&state));
    let before = resumed.campaign.clone();
    assert!(resumed.end_turn(&data).is_err());
    assert_eq!(resumed.campaign, before);
}

#[test]
fn guide_progress_is_local_to_the_saved_campaign() {
    let (data, mut state) = fixture();
    state.command(&data, movement(5)).unwrap();
    let previous = state.campaign.clone().unwrap();
    state.new_game(&data).unwrap();
    assert!(!guide(&state).completed(TutorialStep::Movement));
    state.load_campaign(previous, &data).unwrap();
    assert!(guide(&state).completed(TutorialStep::Movement));
}
