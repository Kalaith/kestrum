//! K01's five behavioral concerns, through the same toolkit/data seam as startup.

use kestrum::data::{
    economy::{self, Economy, Resources, TroopKind},
    rules::{self, CampaignRules},
    world::{self, AnchorExpression, FactionId, MarkerId, MarkerLocation, Scenario, SiteId},
    GameData,
};
use macroquad_toolkit::data_loader::parse_json_labeled;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn changed<T: Serialize + DeserializeOwned>(
    original: &T,
    source: &str,
    change: (&str, Value),
) -> Result<T, String> {
    let mut value = serde_json::to_value(original).unwrap();
    *value.pointer_mut(change.0).unwrap() = change.1;
    parse_json_labeled(source, &value.to_string())
}

fn expect_error(result: Result<(), String>, source: &str, field: &str) {
    let error = result.unwrap_err();
    assert!(error.contains(source), "missing source in {error}");
    assert!(error.contains(field), "expected {field} in {error}");
}

fn bad_scenarios(data: &GameData, cases: Vec<(&str, Value, &str)>) {
    for (path, value, field) in cases {
        let result = changed(&data.scenario, world::SOURCE, (path, value))
            .and_then(|scenario| scenario.validate(&data.rules, &data.economy));
        expect_error(result, world::SOURCE, field);
    }
}

#[test]
fn toolkit_loading_resolves_all_economy_and_scenario_references() {
    let data = GameData::load().unwrap();
    let scenario = &data.scenario;
    assert_eq!(
        (
            scenario.markers.len(),
            scenario.sites.len(),
            scenario.routes.len()
        ),
        (5, 14, 16)
    );
    assert_eq!(scenario.factions.len(), 4);
    assert_eq!(scenario.player, FactionId(1));
    assert_eq!(scenario.seed, 260926);
    check_economy_defaults(&data.economy);
    check_authored_topology(scenario);
    check_content_round_trip(&data);
}

fn check_economy_defaults(economy: &Economy) {
    assert_eq!(
        economy.starting_resources,
        Resources {
            gold: 500,
            wood: 200,
            stone: 150
        }
    );
    assert_eq!(
        economy.headquarters_income_bonus,
        Resources {
            gold: 40,
            wood: 15,
            stone: 10
        }
    );
    for (kind, capacity, gold, upkeep) in [
        (TroopKind::Warriors, 100, 60, 10),
        (TroopKind::Spearmen, 100, 70, 10),
        (TroopKind::Archers, 80, 80, 10),
        (TroopKind::Riders, 40, 120, 16),
        (TroopKind::Medics, 40, 60, 8),
        (TroopKind::SiegeEngines, 20, 120, 14),
    ] {
        let formation = &economy.formations[&kind];
        assert_eq!(
            (
                formation.capacity,
                formation.recruit_cost.gold,
                formation.upkeep_gold
            ),
            (capacity, gold, upkeep)
        );
    }
}

fn check_authored_topology(scenario: &Scenario) {
    // Exact topology is an acceptance assertion, not a second runtime layout.
    let edges: BTreeSet<_> = scenario
        .routes
        .iter()
        .filter(|route| route.major_connection.is_none())
        .map(|route| {
            (
                scenario.site(route.from).unwrap().key.as_str(),
                scenario.site(route.to).unwrap().key.as_str(),
            )
        })
        .collect();
    assert_eq!(
        edges,
        BTreeSet::from([
            ("west_gate", "milltown"),
            ("milltown", "orchard"),
            ("milltown", "bridge"),
            ("orchard", "hill"),
            ("hill", "high_fort"),
            ("bridge", "high_fort"),
            ("bridge", "city"),
            ("high_fort", "city"),
            ("city", "east_gate"),
            ("city", "quarry"),
            ("quarry", "ruined_hold"),
            ("ruined_hold", "east_gate"),
        ])
    );
}

fn check_content_round_trip(data: &GameData) {
    let scenario = &data.scenario;
    for faction in &scenario.factions {
        assert_eq!(
            faction.resources.resolve(&data.economy),
            data.economy.starting_resources
        );
        assert_eq!(
            scenario.site(faction.headquarters).unwrap().controller,
            Some(faction.id)
        );
        assert_eq!(
            faction
                .starting_formations
                .iter()
                .map(|kind| data.economy.formations[kind].capacity)
                .collect::<Vec<_>>(),
            [100, 100, 80]
        );
    }
    for site in scenario
        .sites
        .iter()
        .filter(|site| site.marker == MarkerId(5))
    {
        assert_eq!(
            site.controller,
            if ["city", "high_fort"].contains(&site.key.as_str()) {
                Some(FactionId(3))
            } else {
                None
            }
        );
    }
    // Every JSON field survives typed serialization; no silently ignored content.
    let economy_json: Value =
        macroquad_toolkit::include_json!("../assets/data/economy.json").unwrap();
    assert_eq!(serde_json::to_value(&data.economy).unwrap(), economy_json);
    let encoded = serde_json::to_string(scenario).unwrap();
    let restored: Scenario = parse_json_labeled(world::SOURCE, &encoded).unwrap();
    restored.validate(&data.rules, &data.economy).unwrap();
    assert_eq!(&restored, scenario);
    let restored_rules: CampaignRules =
        parse_json_labeled(rules::SOURCE, &serde_json::to_string(&data.rules).unwrap()).unwrap();
    assert_eq!(restored_rules, data.rules);
    let restored_economy: Economy = parse_json_labeled(
        economy::SOURCE,
        &serde_json::to_string(&data.economy).unwrap(),
    )
    .unwrap();
    assert_eq!(restored_economy, data.economy);
}

#[test]
fn duplicate_ids_and_missing_references_fail_with_source_and_field() {
    let data = GameData::load().unwrap();
    bad_scenarios(
        &data,
        vec![
            ("/sites/1/id", json!(1), "sites.id"),
            ("/markers/1/id", json!(1), "markers.id"),
            ("/routes/1/id", json!(1), "routes.id"),
            ("/factions/1/id", json!(1), "factions.id"),
            ("/sites/1/key", json!("rose_hq"), "sites.key"),
            ("/markers/1/key", json!("rose_hq"), "markers.key"),
            ("/routes/0/from", json!(999), ".from"),
            ("/routes/0/to", json!(999), ".to"),
            ("/sites/0/marker", json!(999), ".marker"),
            ("/sites/5/controller", json!(999), ".controller"),
            ("/markers/4/location/sites/0", json!(999), ".marker"),
            ("/player", json!(999), "player"),
            ("/relations/0/factions/1", json!(999), "relations.factions"),
            ("/relations/1/factions", json!([2, 1]), "relations.factions"),
            ("/routes/0/to", json!(5), "self-routes"),
            ("/routes/1/to", json!(5), "duplicate undirected"),
            ("/sites/0/id", json!(0), "sites.id"),
            ("/markers/0/id", json!(0), "markers.id"),
            ("/routes/0/id", json!(0), "routes.id"),
            ("/factions/0/id", json!(0), "factions.id"),
        ],
    );
    for (collection, key) in [
        ("formations", "medics"),
        ("settlement_income", "village"),
        ("orders", "build_fort"),
    ] {
        let mut value = serde_json::to_value(&data.economy).unwrap();
        value[collection].as_object_mut().unwrap().remove(key);
        let candidate: Economy = parse_json_labeled(economy::SOURCE, &value.to_string()).unwrap();
        expect_error(candidate.validate(), economy::SOURCE, collection);
    }
    for (collection, key, field) in [
        ("formations", "warriors", "Warriors"),
        ("settlement_income", "city", "City"),
        ("orders", "improve_road", "ImproveRoad"),
    ] {
        let value = serde_json::to_value(&data.economy).unwrap();
        let original = serde_json::to_string(&value).unwrap();
        let needle = format!("\"{collection}\":{{");
        let duplicate = format!("{needle}\"{key}\":{},", value[collection][key]);
        let encoded = original.replacen(&needle, &duplicate, 1);
        expect_error(
            parse_json_labeled::<Economy>(economy::SOURCE, &encoded).map(|_| ()),
            economy::SOURCE,
            field,
        );
    }
}

#[test]
fn gates_map_both_ways_and_every_headquarters_reaches_the_region() {
    let data = GameData::load().unwrap();
    let scenario = &data.scenario;
    let MarkerLocation::Region {
        sites, entrances, ..
    } = &scenario.marker(MarkerId(5)).unwrap().location
    else {
        panic!("region missing")
    };
    assert_eq!(sites.len(), 10);
    assert_eq!(entrances.len(), 4);
    assert_eq!(
        entrances.iter().map(|gate| gate.site).collect::<Vec<_>>(),
        [SiteId(5), SiteId(5), SiteId(11), SiteId(11)]
    );
    for gate in entrances {
        let route = scenario.route(gate.route).unwrap();
        let outside = route.other_endpoint(gate.site).unwrap();
        assert_eq!(route.other_endpoint(outside), Some(gate.site));
        assert_eq!(scenario.site(gate.site).unwrap().marker, MarkerId(5));
        assert_ne!(scenario.site(outside).unwrap().marker, MarkerId(5));
    }
    for faction in &scenario.factions {
        let reached = scenario.reachable_sites(faction.headquarters);
        assert_eq!(reached.len(), 14);
        assert!(sites.iter().all(|id| reached.contains(id)));
    }
    check_invalid_boundaries(&data);
    check_disconnected_graphs(&data);
}

fn check_invalid_boundaries(data: &GameData) {
    bad_scenarios(
        data,
        vec![
            (
                "/markers/4/location/entrances/0/site",
                json!(11),
                "entrances",
            ),
            (
                "/markers/4/location/entrances/0/route",
                json!(999),
                "entrances",
            ),
            (
                "/markers/4/location/entrances/0/route",
                json!(14),
                "entrances",
            ),
            (
                "/markers/4/location/entrances/0/site",
                json!(1),
                "entrances",
            ),
            (
                "/markers/4/location/entrances",
                json!([{"route":13,"site":5},{"route":15,"site":11},{"route":16,"site":11}]),
                "reverse entrance",
            ),
            (
                "/routes/12/major_connection",
                json!([5, 1]),
                "major_connection",
            ),
            (
                "/routes/0/major_connection",
                json!([5, 5]),
                "major_connection",
            ),
            (
                "/routes/12/major_connection",
                Value::Null,
                "major_connection",
            ),
        ],
    );
}

fn check_disconnected_graphs(data: &GameData) {
    let scenario = &data.scenario;
    let mut disconnected = scenario.clone();
    disconnected
        .routes
        .retain(|route| route.from != SiteId(14) && route.to != SiteId(14));
    expect_error(
        disconnected.validate(&data.rules, &data.economy),
        world::SOURCE,
        "internal graph must be connected",
    );
    let mut isolated_hq = scenario.clone();
    isolated_hq.routes.retain(|route| route.from != SiteId(1));
    if let MarkerLocation::Region { entrances, .. } = &mut isolated_hq.markers[4].location {
        entrances.retain(|gate| gate.route.0 != 13);
    }
    expect_error(
        isolated_hq.validate(&data.rules, &data.economy),
        world::SOURCE,
        "not satisfiable from every HQ",
    );
    // Vector order is not identity and reversing an undirected route is legal.
    let mut reordered = scenario.clone();
    reordered.sites.reverse();
    reordered.markers.reverse();
    reordered.factions.reverse();
    for route in &mut reordered.routes {
        std::mem::swap(&mut route.from, &mut route.to);
        if let Some(markers) = &mut route.major_connection {
            markers.swap(0, 1);
        }
    }
    reordered.validate(&data.rules, &data.economy).unwrap();
}

#[test]
fn capacities_resources_setup_and_supported_policies_reject_bad_values() {
    let data = GameData::load().unwrap();
    check_economy_bounds(&data.economy);
    check_setup_bounds(&data);
    check_scenario_bounds(&data);
}

fn check_economy_bounds(economy: &Economy) {
    for (path, value, field) in [
        ("/schema_version", json!(99), "schema_version"),
        ("/status", json!("final"), "final"),
        ("/period", json!("faction_turn"), "faction_turn"),
        ("/formations/warriors/capacity", json!(0), "capacity"),
        (
            "/formations/warriors/recruit_cost/gold",
            json!(0),
            "recruit_cost.gold",
        ),
        ("/formations/medics/upkeep_gold", json!(-1), "upkeep_gold"),
        (
            "/starting_resources/gold",
            json!(-1),
            "starting_resources.gold",
        ),
        (
            "/headquarters_income_bonus/wood",
            json!(-1),
            "headquarters_income_bonus.wood",
        ),
        ("/settlement_income/city/stone", json!(-1), "stone"),
        ("/orders/improve_road/cost/wood", json!(-1), "cost.wood"),
    ] {
        expect_error(
            changed(economy, economy::SOURCE, (path, value))
                .and_then(|candidate| candidate.validate()),
            economy::SOURCE,
            field,
        );
    }
    check_economy_policies(economy);
}

fn check_economy_policies(economy: &Economy) {
    for (path, value, field) in [
        (
            "/recovery/capacity_percent_per_round",
            json!(101),
            "capacity_percent_per_round",
        ),
        (
            "/recovery/full_replacement_recruit_gold_percent",
            json!(0),
            "full_replacement_recruit_gold_percent",
        ),
        ("/recovery/round_cost_up", json!(false), "round_cost_up"),
        ("/recovery/requires_supply", json!(false), "requires_supply"),
        (
            "/upkeep/named_character_gold",
            json!(1),
            "named_character_gold",
        ),
        (
            "/upkeep/charge_full_formation_rate",
            json!(false),
            "charge_full_formation_rate",
        ),
        (
            "/upkeep/carry_unpaid_debt",
            json!(true),
            "carry_unpaid_debt",
        ),
        (
            "/upkeep/block_recruitment_and_recovery_on_shortfall",
            json!(false),
            "block_recruitment",
        ),
        (
            "/upkeep/automatic_disbanding",
            json!(true),
            "automatic_disbanding",
        ),
        (
            "/refunds/unstarted_construction_percent",
            json!(101),
            "unstarted_construction_percent",
        ),
        (
            "/refunds/started_construction_percent",
            json!(50),
            "started_construction_percent",
        ),
        ("/refunds/disband_percent", json!(1), "disband_percent"),
        (
            "/recruitment_population/enabled",
            json!(true),
            "recruitment_population.enabled",
        ),
    ] {
        expect_error(
            changed(economy, economy::SOURCE, (path, value))
                .and_then(|candidate| candidate.validate()),
            economy::SOURCE,
            field,
        );
    }
}

fn check_scenario_bounds(data: &GameData) {
    bad_scenarios(
        data,
        vec![
            ("/schema_version", json!(2), "schema_version"),
            ("/content_version", json!(2), "content_version"),
            ("/kind", json!("production"), "production"),
            ("/routes/0/terrain_cost", json!(0), "terrain_cost"),
            ("/routes/0/road/damage", json!(101), "road.damage"),
            ("/routes/0/road/damage", json!(10), "road"),
            ("/factions/0/name", json!(" Rose "), ".name"),
            ("/factions/0/name", json!(""), ".name"),
            ("/factions/0/name", json!("🌹".repeat(33)), ".name"),
            ("/factions/0/resources", json!("free_gold"), "free_gold"),
            ("/factions/0/founder/age_years", json!(23), ".founder"),
            (
                "/factions/0/starting_formations",
                json!(["warriors", "dragons"]),
                "dragons",
            ),
            ("/sites/0/facilities", json!([]), "headquarters"),
            ("/sites/0/tags", json!([]), "headquarters"),
            ("/sites/0/controller", json!(2), "headquarters"),
            ("/sites/0/position/0", json!(1.1), ".position"),
        ],
    );
    let mut nonfinite = data.scenario.clone();
    nonfinite.markers[0].position[0] = f32::NAN;
    expect_error(
        nonfinite.validate(&data.rules, &data.economy),
        world::SOURCE,
        ".position",
    );
}

fn check_setup_bounds(data: &GameData) {
    for (path, value, field) in [
        ("/schema_version", json!(99), "schema_version"),
        ("/content_version", json!(99), "content_version"),
        ("/min_factions", json!(3), "min_factions"),
        ("/max_factions", json!(9), "max_factions"),
        ("/default_factions", json!(5), "default_factions"),
        (
            "/kingdom_name_max_chars",
            json!(0),
            "kingdom_name_max_chars",
        ),
        (
            "/ai_income_bonus_percent",
            json!(1),
            "ai_income_bonus_percent",
        ),
        ("/difficulty", json!("hard"), "hard"),
        ("/emblems/1/id", json!("rose"), "emblems.id"),
    ] {
        expect_error(
            changed(&data.rules, rules::SOURCE, (path, value)).and_then(|rules| rules.validate()),
            rules::SOURCE,
            field,
        );
    }
    let mut unicode_name = data.scenario.clone();
    unicode_name.factions[0].name = "🌹".repeat(32);
    unicode_name.validate(&data.rules, &data.economy).unwrap();
    for (source, mut value) in [
        (
            economy::SOURCE,
            serde_json::to_value(&data.economy).unwrap(),
        ),
        (rules::SOURCE, serde_json::to_value(&data.rules).unwrap()),
        (world::SOURCE, serde_json::to_value(&data.scenario).unwrap()),
    ] {
        value["unknown_policy"] = json!(true);
        let result = match source {
            economy::SOURCE => {
                parse_json_labeled::<Economy>(source, &value.to_string()).map(|_| ())
            }
            rules::SOURCE => {
                parse_json_labeled::<CampaignRules>(source, &value.to_string()).map(|_| ())
            }
            _ => parse_json_labeled::<Scenario>(source, &value.to_string()).map(|_| ()),
        };
        expect_error(result, source, "unknown_policy");
    }
}

#[test]
fn anchor_expressions_resolve_and_require_both_anchors_and_a_supplied_gate() {
    let data = GameData::load().unwrap();
    let MarkerLocation::Region { anchors, .. } = &data.scenario.markers[4].location else {
        panic!("region missing")
    };
    for (held, supplied, expected) in [
        (vec![9, 10, 5], vec![5], true),
        (vec![9, 10, 11], vec![11], true),
        (vec![9, 10, 5, 11], vec![5, 11], true),
        (vec![9, 10, 5], vec![], false),
        (vec![9, 10], vec![5], false),
        (vec![10, 5], vec![5], false),
        (vec![9, 11], vec![11], false),
        (vec![9, 10, 5], vec![11], false),
    ] {
        assert_eq!(
            anchors.is_satisfied(
                &held.into_iter().map(SiteId).collect(),
                &supplied.into_iter().map(SiteId).collect()
            ),
            expected
        );
    }
    for expression in [
        AnchorExpression::Any { conditions: vec![] },
        AnchorExpression::All { conditions: vec![] },
        AnchorExpression::ControlledSite { site: SiteId(999) },
        AnchorExpression::ControlledSite { site: SiteId(1) },
        AnchorExpression::SuppliedEntrance { site: SiteId(9) },
    ] {
        bad_scenarios(
            &data,
            vec![(
                "/markers/4/location/anchors",
                serde_json::to_value(expression).unwrap(),
                ".anchors",
            )],
        );
    }
    // Even one invalid alternative is rejected instead of silently ignoring it.
    bad_scenarios(
        &data,
        vec![(
            "/markers/4/location/anchors/conditions/2/conditions/1/site",
            json!(999),
            ".anchors",
        )],
    );
}
