//! The real toolkit loading seam rejects ambiguous or unsupported construction data.

use kestrum::data::{
    construction::{ConstructionRules, SOURCE},
    economy::{Habitation, OrderKind, Resources},
    world::{Facility, SiteTag},
    GameData,
};
use kestrum::{
    engine::{apply, construction_options, project, Actor, Command, RuleError},
    state::{
        construction::{ConstructionKind, ConstructionTarget, Focus},
        military::ArmyId,
        StrategicCampaign,
    },
};
use macroquad_toolkit::data_loader::parse_json_labeled;
use serde_json::{json, Value};

const RAW: &str = include_str!("../assets/data/construction_rules.json");

fn load(raw: &str) -> Result<ConstructionRules, String> {
    let rules: ConstructionRules = parse_json_labeled(SOURCE, raw)?;
    rules.validate()?;
    Ok(rules)
}

fn invalid(path: &str, value: Value, field: &str) {
    let mut data: Value = serde_json::from_str(RAW).unwrap();
    *data.pointer_mut(path).unwrap() = value;
    error_contains(load(&data.to_string()), field);
}

fn error_contains(result: Result<ConstructionRules, String>, field: &str) {
    let error = result.unwrap_err();
    assert!(error.contains(SOURCE), "missing source in {error}");
    assert!(error.contains(field), "expected {field} in {error}");
}

#[test]
fn startup_loads_the_complete_build_table_without_duplicating_order_prices() {
    let data = GameData::load().unwrap();
    let rules = &data.construction;
    assert_eq!(*rules, load(RAW).unwrap());
    assert_eq!(
        (rules.outpost_steps, rules.road_steps, rules.fort_steps),
        (3, 2, 3)
    );
    assert_eq!(rules.road_repair.steps, 1);
    assert_eq!(
        rules.road_repair.cost,
        Resources {
            gold: 20,
            wood: 15,
            stone: 0
        }
    );
    for (kind, expected) in [
        (OrderKind::EstablishOutpost, (80, 60, 40)),
        (OrderKind::ImproveRoad, (40, 30, 0)),
        (OrderKind::BuildFort, (100, 80, 100)),
        (OrderKind::ChangeFocus, (0, 0, 0)),
    ] {
        let cost = data.economy.orders[&kind].cost;
        assert_eq!((cost.gold, cost.wood, cost.stone), expected);
    }
    for (facility, cost, steps) in [
        (Facility::TrainingGround, (60, 40, 20), 2),
        (Facility::Stable, (100, 60, 40), 2),
        (Facility::Infirmary, (80, 40, 20), 2),
        (Facility::Workshop, (100, 80, 60), 3),
        (Facility::Temple, (80, 60, 40), 3),
    ] {
        let definition = &rules.facilities[&facility];
        assert_eq!(
            (
                definition.cost.gold,
                definition.cost.wood,
                definition.cost.stone
            ),
            cost
        );
        assert_eq!(definition.steps, steps);
    }
    let value = serde_json::to_value(rules).unwrap();
    for unsupported in [
        "orders",
        "outpost_cost",
        "income_focus_percent",
        "training_discount",
    ] {
        assert!(value.get(unsupported).is_none());
    }
}

#[test]
fn ambiguous_keys_unknown_ids_and_unimplemented_fields_are_rejected() {
    let duplicate_facility = RAW.replacen(
        "\"facilities\": {",
        "\"facilities\": {\"stable\": {\"cost\": {\"gold\": 1, \"wood\": 0, \"stone\": 0}, \"steps\": 1, \"minimum_habitation\": \"village\", \"required_tag\": \"horse_access\"},",
        1,
    );
    error_contains(load(&duplicate_facility), "duplicate");
    let duplicate_population = RAW.replacen("\"minimum\": {", "\"minimum\": {\"camp\": 100,", 1);
    error_contains(load(&duplicate_population), "duplicate");
    error_contains(
        load(&RAW.replace("\"temple\":", "\"wizard_tower\":")),
        "wizard_tower",
    );
    error_contains(
        load(&RAW.replacen('{', "{\"income_focus_percent\":25,", 1)),
        "income_focus_percent",
    );
    error_contains(
        load(&RAW.replacen("\"steps\": 1", "\"steps\": 1, \"instant\": true", 1)),
        "instant",
    );
    error_contains(load("{"), "EOF");
}

#[test]
fn costs_and_durations_cannot_create_free_or_nonterminating_work() {
    for path in [
        "/outpost_steps",
        "/road_steps",
        "/fort_steps",
        "/road_repair/steps",
        "/facilities/temple/steps",
    ] {
        invalid(path, json!(0), path.rsplit('/').next().unwrap());
        invalid(path, json!(101), path.rsplit('/').next().unwrap());
    }
    invalid("/schema_version", json!(2), "schema_version");
    for field in ["gold", "wood", "stone"] {
        invalid(&format!("/road_repair/cost/{field}"), json!(-1), field);
        invalid(
            &format!("/facilities/workshop/cost/{field}"),
            json!(-1),
            field,
        );
    }
    for path in ["/road_repair/cost", "/facilities/temple/cost"] {
        invalid(path, json!({"gold":0,"wood":0,"stone":0}), "cost");
    }
    check_refund_overflow_is_atomic();
}

fn check_refund_overflow_is_atomic() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let site = campaign.factions[&campaign.player].headquarters;
    let order = campaign.next_ids.order;
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartConstruction {
            target: ConstructionTarget::Site(site),
            kind: ConstructionKind::Facility(Facility::Temple),
            builder: ArmyId(1),
        },
    )
    .unwrap();
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources
        .gold = i64::MAX;
    campaign.validate(&data).unwrap();
    let before = campaign.clone();
    let result = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::CancelConstruction { order },
    );
    assert!(matches!(
        result,
        Err(RuleError::Overflow {
            field: "construction refund"
        })
    ));
    assert_eq!(
        campaign, before,
        "a rejected refund cannot partly credit resources or close the order"
    );
}

#[test]
fn facility_requirements_and_complete_tables_match_the_supported_rules() {
    let rules = load(RAW).unwrap();
    assert_eq!(
        rules.facilities[&Facility::Stable].required_tag,
        Some(SiteTag::HorseAccess)
    );
    assert_eq!(
        rules.facilities[&Facility::TrainingGround].minimum_habitation,
        Habitation::Outpost
    );
    for facility in ["stable", "infirmary", "workshop", "temple"] {
        invalid(
            &format!("/facilities/{facility}/minimum_habitation"),
            json!("camp"),
            "minimum_habitation",
        );
    }
    invalid(
        "/facilities/stable/required_tag",
        Value::Null,
        "required_tag",
    );
    invalid(
        "/facilities/temple/required_tag",
        json!("horse_access"),
        "required_tag",
    );
    let mut missing: Value = serde_json::from_str(RAW).unwrap();
    missing["facilities"]
        .as_object_mut()
        .unwrap()
        .remove("temple");
    error_contains(load(&missing.to_string()), "facilities");
    invalid("/facilities/training_ground", Value::Null, "invalid type");
}

#[test]
fn population_initialization_is_complete_bounded_and_monotonic() {
    let rules = load(RAW).unwrap();
    assert_eq!(
        rules
            .population
            .minimum
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec![0, 20, 50, 100, 200, 500, 1000, 1800]
    );
    assert_eq!(
        (
            rules.population.headquarters,
            rules.population.settler_limit
        ),
        (250, 50)
    );
    for (path, bad, field) in [
        ("/population/minimum/unsettled", 1, "Unsettled"),
        ("/population/minimum/camp", 0, "Camp"),
        ("/population/minimum/outpost", 20, "Outpost"),
        ("/population/minimum/city", 1_000_001, "City"),
        ("/population/headquarters", 199, "headquarters"),
        ("/population/headquarters", 1_000_001, "headquarters"),
        ("/population/settler_limit", 0, "settler_limit"),
        ("/population/settler_limit", 1_000_001, "settler_limit"),
    ] {
        invalid(path, json!(bad), field);
    }
    let mut missing: Value = serde_json::from_str(RAW).unwrap();
    missing["population"]["minimum"]
        .as_object_mut()
        .unwrap()
        .remove("major_city");
    error_contains(load(&missing.to_string()), "population.minimum");
    check_private_population_and_focus();
}

fn check_private_population_and_focus() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let player = campaign.player;
    let own_site = campaign.factions[&player].headquarters;
    let foreign_site = campaign
        .factions
        .values()
        .find(|faction| faction.id != player)
        .unwrap()
        .headquarters;
    campaign.world.focus.insert(own_site, Focus::Growth);
    let visible = project(&campaign, player).unwrap();
    let target = ConstructionTarget::Site(foreign_site);
    let options = construction_options(&campaign, &data, player, target);
    assert_eq!(
        visible.world.population[&own_site],
        data.construction.population.headquarters
    );
    assert_eq!(visible.world.focus[&own_site], Focus::Growth);
    assert!(!visible.world.population.contains_key(&foreign_site));
    assert!(!visible.world.focus.contains_key(&foreign_site));
    campaign.world.population.insert(foreign_site, 50_000);
    campaign
        .world
        .focus
        .insert(foreign_site, Focus::TroopTraining);
    campaign.validate(&data).unwrap();
    assert_eq!(project(&campaign, player).unwrap(), visible);
    assert_eq!(
        construction_options(&campaign, &data, player, target),
        options
    );
}
