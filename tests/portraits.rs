//! Stable portrait allocation, finite-space fallbacks, and catalog transitions.

use kestrum::{
    data::{
        portraits::{AppearanceDescriptor, PortraitCatalog},
        GameData,
    },
    engine::portraits::{allocate_for_person, allocate_from_registry},
    state::{appearance::AppearanceRegistry, people::PersonId, StrategicCampaign},
};

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
    let mut catalog = PortraitCatalog::load_frozen().expect("load frozen metadata-only catalog");
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
fn frozen_catalog_allocation_has_a_portable_golden_signature() {
    let catalog = PortraitCatalog::load_frozen().expect("load frozen portrait catalog");
    let mut reservations = registry(&catalog, 712);
    let appearance = allocate(&mut reservations, &catalog, 1, &[]);

    assert_eq!(
        appearance.signature,
        "19:human-bust-front-v1|12:face-oval-v1|16:nose-upturned-v1|14:eyes-lidded-v1|12:hair-bald-v1|13:skin-ochre-v1|-|13:eyes-hazel-v1"
    );
}
