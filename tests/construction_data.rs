//! The real toolkit loading seam rejects ambiguous or unsupported construction data.

use kestrum::data::{
    construction::{ConstructionRules, SOURCE},
    economy::Habitation,
    world::{Facility, SiteTag},
    GameData,
};
use kestrum::{
    engine::{apply, Actor, Command, RuleError},
    state::{
        construction::{ConstructionKind, ConstructionTarget},
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
