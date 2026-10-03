//! Stable portrait allocation, finite-space fallbacks, and catalog transitions.

use kestrum::{
    data::{
        portraits::{AppearanceDescriptor, PortraitCatalog},
        GameData,
    },
    engine::portraits::{allocate_for_person, allocate_from_registry, migrate_legacy},
    state::{appearance::AppearanceRegistry, people::PersonId, Campaign, StrategicCampaign},
};
use serde_json::{json, Value};

fn current_catalog() -> PortraitCatalog {
    let catalog = GameData::load().expect("load game data").portraits;
    catalog.validate().expect("validate portrait catalog");
    catalog
}

fn registry(catalog: &PortraitCatalog, seed: u64) -> AppearanceRegistry {
    AppearanceRegistry::for_campaign_seed(
        seed,
        catalog.catalog_revision,
        catalog.allocation_revision,
    )
}

fn allocate(
    registry: &mut AppearanceRegistry,
    catalog: &PortraitCatalog,
    id: u32,
    living: &[AppearanceDescriptor],
) -> AppearanceDescriptor {
    allocate_from_registry(registry, catalog, PersonId(id), id, living)
        .unwrap_or_else(|error| panic!("allocate portrait for {id}: {error}"))
}

fn tiny_catalog(two_hair_styles: bool) -> PortraitCatalog {
    let mut catalog = current_catalog();
    let face_id = "face_oval";
    let nose_id = "nose_straight";
    let eyes_id = "eyes_open";
    let bald_id = "hair_bald";
    catalog.faces.retain(|feature| feature.id == face_id);
    catalog.noses.retain(|feature| feature.id == nose_id);
    catalog.eyes.retain(|feature| feature.id == eyes_id);
    catalog.hair.retain(|feature| {
        feature.id == bald_id || (two_hair_styles && feature.id == "hair_cropped")
    });
    for feature in &mut catalog.faces {
        feature.silhouette_group = "shared".into();
    }
    for feature in &mut catalog.hair {
        feature.silhouette_group = "shared".into();
    }
    let retained_hair = catalog
        .hair
        .iter()
        .map(|feature| feature.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    catalog.geometry.retain(|tuple| {
        tuple.face_id == face_id
            && tuple.nose_id == nose_id
            && tuple.eyes_id == eyes_id
            && retained_hair.contains(tuple.hair_id.as_str())
    });
    catalog.skin_palettes.truncate(1);
    catalog.eye_palettes.truncate(1);
    catalog
        .hair_palettes
        .retain(|palette| palette.id == "none" || (two_hair_styles && palette.id == "hair_ebony"));
    for palette in &mut catalog.hair_palettes {
        if palette.id != "none" {
            palette.value_group = "shared".into();
        }
    }
    catalog.geometry.shrink_to_fit();
    catalog.validate().expect("validate tiny portrait catalog");
    catalog
}

#[test]
fn deterministic_assignment_does_not_advance_gameplay_rng_streams() {
    let data = GameData::load().expect("load game data");
    let mut first = StrategicCampaign::new(&data).expect("new campaign");
    let mut second = first.clone();
    let id = first.next_ids.person;
    let before = first.rng.states();
    let first_appearance = allocate_for_person(&mut first, &data.portraits, id).unwrap();
    let second_appearance = allocate_for_person(&mut second, &data.portraits, id).unwrap();

    assert_eq!(first_appearance, second_appearance);
    assert_eq!(first.rng.states(), before);
    assert_eq!(second.rng.states(), before);
}

#[test]
fn living_roster_prefers_two_feature_groups_and_a_silhouette_change() {
    let catalog = current_catalog();
    let mut reservations = registry(&catalog, 712);
    let founder = allocate(&mut reservations, &catalog, 1, &[]);
    let next = allocate(
        &mut reservations,
        &catalog,
        2,
        std::slice::from_ref(&founder),
    );
    let index = catalog.validation_index().unwrap();
    let old_groups = index.distinctiveness(&founder).unwrap();
    let new_groups = index.distinctiveness(&next).unwrap();
    let differences = old_groups
        .iter()
        .zip(&new_groups)
        .filter(|(left, right)| left != right)
        .count();

    assert_ne!(founder.signature, next.signature);
    assert!(differences >= 2);
    assert!(old_groups[0] != new_groups[0] || old_groups[1] != new_groups[1]);
}

#[test]
fn unused_exact_portraits_survive_bounded_preference_relaxation_before_reuse() {
    let catalog = tiny_catalog(true);
    let mut reservations = registry(&catalog, 23);
    let first = allocate(&mut reservations, &catalog, 1, &[]);
    let second = allocate(&mut reservations, &catalog, 2, std::slice::from_ref(&first));

    assert_ne!(first.signature, second.signature);
    assert_eq!(reservations.reservations.len(), 2);
    assert_eq!(reservations.fallback_usage.near_duplicate_relaxed, 1);
    assert_eq!(reservations.fallback_usage.living_signature_reuse, 0);

    let third = allocate(
        &mut reservations,
        &catalog,
        3,
        &[first.clone(), second.clone()],
    );
    assert_eq!(third.signature, first.signature);
    assert_eq!(reservations.fallback_usage.living_signature_reuse, 1);
}

#[test]
fn exhausted_space_reuses_deceased_before_living_signatures() {
    let catalog = tiny_catalog(false);
    let mut reservations = registry(&catalog, 991);
    let first = allocate(&mut reservations, &catalog, 1, &[]);
    let second = allocate(&mut reservations, &catalog, 2, &[]);
    assert_eq!(first.signature, second.signature);
    assert_eq!(reservations.fallback_usage.deceased_signature_reuse, 1);

    let third = allocate(
        &mut reservations,
        &catalog,
        3,
        std::slice::from_ref(&second),
    );
    assert_eq!(third.signature, first.signature);
    assert_eq!(reservations.fallback_usage.living_signature_reuse, 1);
    assert!(reservations.validate(991, &catalog).is_ok());
}

#[test]
fn counter_overflow_rejects_allocation_without_mutating_reservations() {
    let catalog = tiny_catalog(false);
    let mut reservations = registry(&catalog, 117);
    allocate(&mut reservations, &catalog, 1, &[]);
    let record = reservations.reservations.values_mut().next().unwrap();
    record.reuse_count = u32::MAX;
    reservations.fallback_usage.deceased_signature_reuse = u64::from(u32::MAX - 1);
    let before = reservations.clone();

    let result = allocate_from_registry(&mut reservations, &catalog, PersonId(2), 2, &[]);
    assert!(result.is_err());
    assert_eq!(reservations, before);
}

#[test]
fn fallback_counter_overflow_does_not_reserve_the_candidate() {
    let catalog = tiny_catalog(true);
    let mut reservations = registry(&catalog, 118);
    let first = allocate(&mut reservations, &catalog, 1, &[]);
    reservations.fallback_usage.near_duplicate_relaxed = u64::MAX;
    let before = reservations.clone();

    let result = allocate_from_registry(
        &mut reservations,
        &catalog,
        PersonId(2),
        2,
        std::slice::from_ref(&first),
    );
    assert!(result.is_err());
    assert_eq!(reservations, before);
}

#[test]
fn saved_descriptor_survives_explicit_catalog_revision_transition() {
    let old_catalog = tiny_catalog(true);
    let mut reservations = registry(&old_catalog, 19);
    let established = allocate(&mut reservations, &old_catalog, 1, &[]);
    let mut expanded = old_catalog.clone();
    expanded.catalog_revision = 2;
    expanded.allocation_revision = 2;
    expanded.supported_catalog_revisions.push(2);
    expanded.supported_allocation_revisions.push(2);
    expanded.hair.push(kestrum::data::portraits::HairFeature {
        id: "hair_braided".into(),
        visual_key: "hair-braided-v2".into(),
        silhouette_group: "braided".into(),
        is_bald: false,
        fits: Default::default(),
    });
    expanded
        .geometry
        .push(kestrum::data::portraits::GeometryTuple {
            face_id: "face_oval".into(),
            nose_id: "nose_straight".into(),
            eyes_id: "eyes_open".into(),
            hair_id: "hair_braided".into(),
        });
    expanded.validate().expect("validate expanded catalog");
    expanded
        .validate_descriptor(&established)
        .expect("old descriptor remains supported");

    reservations.transition_to_catalog(&expanded).unwrap();
    let later = allocate(
        &mut reservations,
        &expanded,
        2,
        std::slice::from_ref(&established),
    );
    assert_eq!(established.catalog_revision, 1);
    assert_eq!(later.catalog_revision, 2);
    assert!(reservations.validate(19, &expanded).is_ok());
}

#[test]
fn tampered_canonical_signatures_are_rejected() {
    let catalog = current_catalog();
    let mut reservations = registry(&catalog, 44);
    let mut appearance = allocate(&mut reservations, &catalog, 1, &[]);
    appearance.signature.push_str("tampered");

    assert!(catalog.validate_descriptor(&appearance).is_err());
}

#[test]
fn frozen_catalog_allocation_has_a_portable_golden_signature() {
    let catalog = PortraitCatalog::load_frozen().expect("load frozen portrait catalog");
    let mut reservations = registry(&catalog, 712);
    let appearance = allocate(&mut reservations, &catalog, 1, &[]);

    assert_eq!(
        appearance.signature,
        "19:human-bust-front-v1|12:face-oval-v1|16:nose-upturned-v1|14:eyes-lidded-v1|12:hair-bald-v1|13:skin-ochre-v1|-|13:eyes-hazel-v1"
    );
}

fn legacy_identity_union() -> Value {
    json!({
        "seed": 1234,
        "completed_rounds": 7,
        "people": { "1": { "id": 1 } },
        "knowledge": {
            "observers": {
                "9": { "people": { "2": { "id": 2 } } },
            },
        },
        "battles": {
            "4": {
                "attacker": { "armies": [{ "people": [{ "id": 3 }] }] },
                "defender": {
                    "kind": "faction",
                    "side": { "armies": [{ "people": [{ "id": 4 }] }] },
                },
            },
        },
        "pending_battle": {
            "report": {
                "attacker": {
                    "armies": [{ "people": [{ "id": 3 }, { "id": 5 }] }],
                },
                "defender": { "kind": "threat", "side": {} },
            },
        },
    })
}

fn descriptor_from(record: &Value) -> AppearanceDescriptor {
    serde_json::from_value(record["appearance"].clone()).expect("migrated appearance")
}

#[test]
fn migration_allocates_id_ordered_union_across_live_observer_and_battle_snapshots() {
    let mut legacy = legacy_identity_union();
    migrate_legacy(&mut legacy).expect("migrate legacy identity union");

    let person_one = descriptor_from(&legacy["people"]["1"]);
    let observer_two = descriptor_from(&legacy["knowledge"]["observers"]["9"]["people"]["2"]);
    let completed_attacker =
        descriptor_from(&legacy["battles"]["4"]["attacker"]["armies"][0]["people"][0]);
    let completed_defender =
        descriptor_from(&legacy["battles"]["4"]["defender"]["side"]["armies"][0]["people"][0]);
    let pending_same =
        descriptor_from(&legacy["pending_battle"]["report"]["attacker"]["armies"][0]["people"][0]);
    let pending_new =
        descriptor_from(&legacy["pending_battle"]["report"]["attacker"]["armies"][0]["people"][1]);

    assert_ne!(person_one.signature, observer_two.signature);
    assert_ne!(observer_two.signature, completed_attacker.signature);
    assert_eq!(completed_attacker, pending_same);
    assert_ne!(completed_defender.signature, pending_new.signature);
    let registry: AppearanceRegistry =
        serde_json::from_value(legacy["appearance_registry"].clone()).unwrap();
    assert_eq!(registry.reservations.len(), 5);
    let catalog = PortraitCatalog::load_frozen().unwrap();
    assert!(registry.validate(1234, &catalog).is_ok());

    let migrated_once = legacy.clone();
    migrate_legacy(&mut legacy).expect("modern campaign migration is idempotent");
    assert_eq!(legacy, migrated_once);
}

#[test]
fn migration_rejects_partial_required_portrait_state_but_accepts_absent_state() {
    let mut legacy = legacy_identity_union();
    migrate_legacy(&mut legacy).expect("wholly absent legacy state migrates");

    let mut descriptor_without_registry = legacy_identity_union();
    descriptor_without_registry["people"]["1"]["appearance"] =
        legacy["people"]["1"]["appearance"].clone();
    assert!(migrate_legacy(&mut descriptor_without_registry)
        .unwrap_err()
        .contains("descriptors exist without"));

    let mut explicit_null = legacy_identity_union();
    explicit_null["people"]["1"]["appearance"] = Value::Null;
    assert!(migrate_legacy(&mut explicit_null).is_err());

    let mut incomplete_modern = legacy;
    incomplete_modern["people"]["1"]
        .as_object_mut()
        .unwrap()
        .remove("appearance");
    assert!(migrate_legacy(&mut incomplete_modern)
        .unwrap_err()
        .contains("registry exists but"));
}

#[test]
fn migration_preserves_pruned_reservations_and_does_not_backfill_notifications() {
    let mut campaign = legacy_identity_union();
    campaign["notifications"] = json!({
        "receipts": [{
            "subject": { "kind": "person", "snapshot": { "id": 77, "appearance": null } },
            "detail": {
                "kind": "remembrance",
                "subject": { "kind": "person", "snapshot": { "id": 77 } },
            },
        }],
    });
    migrate_legacy(&mut campaign).expect("legacy optional notification snapshots are accepted");
    assert!(
        campaign["notifications"]["receipts"][0]["subject"]["snapshot"]["appearance"].is_null()
    );
    assert!(
        campaign["notifications"]["receipts"][0]["detail"]["subject"]["snapshot"]
            .get("appearance")
            .is_none()
    );
    let migrated_registry: AppearanceRegistry =
        serde_json::from_value(campaign["appearance_registry"].clone()).unwrap();
    assert_eq!(migrated_registry.reservations.len(), 5);

    let saved_registry = campaign["appearance_registry"].clone();
    campaign.as_object_mut().unwrap().remove("people");
    campaign.as_object_mut().unwrap().remove("knowledge");
    campaign.as_object_mut().unwrap().remove("battles");
    campaign.as_object_mut().unwrap().remove("pending_battle");
    migrate_legacy(&mut campaign).expect("pruned modern campaign remains valid");
    assert_eq!(campaign["appearance_registry"], saved_registry);

    let registry: AppearanceRegistry = serde_json::from_value(saved_registry).unwrap();
    let catalog = PortraitCatalog::load_frozen().unwrap();
    registry.validate(1234, &catalog).unwrap();
    let reserved: std::collections::BTreeSet<_> = registry.reservations.keys().cloned().collect();
    let mut after_pruning = registry;
    let fresh = allocate_from_registry(&mut after_pruning, &catalog, PersonId(6), 8, &[]).unwrap();
    assert!(!reserved.contains(&fresh.signature));
}

#[test]
fn notification_descriptor_without_registry_is_rejected_without_backfill() {
    let catalog = PortraitCatalog::load_frozen().unwrap();
    let mut reservations = registry(&catalog, 78);
    let descriptor = allocate(&mut reservations, &catalog, 1, &[]);
    let snapshot = json!({ "id": 1, "appearance": descriptor });
    let receipts = [
        json!({
            "subject": { "kind": "person", "snapshot": snapshot.clone() },
        }),
        json!({
            "detail": { "kind": "person", "person": snapshot.clone() },
        }),
        json!({
            "detail": {
                "kind": "remembrance",
                "subject": { "kind": "person", "snapshot": snapshot.clone() },
            },
        }),
    ];
    for receipt in receipts {
        let mut legacy = json!({
            "seed": 78,
            "completed_rounds": 0,
            "notifications": { "receipts": [receipt] },
        });
        assert!(migrate_legacy(&mut legacy)
            .unwrap_err()
            .contains("notification descriptors exist without"));
    }
}

fn strip_appearance_fields(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            fields.remove("appearance");
            for nested in fields.values_mut() {
                strip_appearance_fields(nested);
            }
        }
        Value::Array(entries) => {
            for nested in entries {
                strip_appearance_fields(nested);
            }
        }
        _ => {}
    }
}

#[test]
fn actual_v2_campaign_decode_migrates_all_portraits_without_advancing_rng_and_roundtrips() {
    let data = GameData::load().expect("load game data");
    let original = StrategicCampaign::new(&data).expect("new strategic campaign");
    let rng_before = original.rng.states();
    let mut legacy = serde_json::to_value(Campaign::Strategic(Box::new(original.clone())))
        .expect("serialize current v2 campaign");
    legacy
        .as_object_mut()
        .unwrap()
        .remove("appearance_registry");
    strip_appearance_fields(&mut legacy);

    let decoded: Campaign = serde_json::from_value(legacy).expect("decode legacy v2 campaign");
    decoded.validate(&data).expect("validate migrated campaign");
    let migrated = decoded.strategic().unwrap();
    assert_eq!(migrated.rng.states(), rng_before);
    assert_eq!(
        migrated.appearance_registry.reservations.len(),
        migrated.people.len()
    );
    assert!(migrated.people.values().all(|person| {
        person.appearance.schema_version == kestrum::data::portraits::APPEARANCE_SCHEMA_VERSION
    }));

    let once = serde_json::to_value(&decoded).expect("serialize migrated campaign");
    let twice: Campaign = serde_json::from_value(once.clone()).expect("decode migrated campaign");
    assert_eq!(
        serde_json::to_value(twice).expect("reserialize migrated campaign"),
        once,
        "portrait migration should be stable after the first v2 load"
    );
}

#[test]
fn actual_v2_campaign_decode_rejects_partial_and_future_portrait_state() {
    let data = GameData::load().expect("load game data");
    let campaign = StrategicCampaign::new(&data).expect("new strategic campaign");
    assert!(!campaign.people.is_empty());
    let modern = serde_json::to_value(Campaign::Strategic(Box::new(campaign)))
        .expect("serialize current v2 campaign");

    let mut descriptors_without_registry = modern.clone();
    descriptors_without_registry
        .as_object_mut()
        .unwrap()
        .remove("appearance_registry");
    let error = serde_json::from_value::<Campaign>(descriptors_without_registry)
        .unwrap_err()
        .to_string();
    assert!(error.contains("descriptors exist without"));

    let mut missing_descriptor = modern.clone();
    let person = missing_descriptor["people"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap();
    person.as_object_mut().unwrap().remove("appearance");
    let error = serde_json::from_value::<Campaign>(missing_descriptor)
        .unwrap_err()
        .to_string();
    assert!(error.contains("registry exists but"));

    let mut future_descriptor = modern;
    let person = future_descriptor["people"]
        .as_object_mut()
        .unwrap()
        .values_mut()
        .next()
        .unwrap();
    person["appearance"]["schema_version"] = json!(u32::MAX);
    let error = serde_json::from_value::<Campaign>(future_descriptor)
        .unwrap_err()
        .to_string();
    assert!(error.contains("malformed or unsupported descriptor"));
}
