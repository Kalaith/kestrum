//! Actual runtime artwork, interface copy, and semantic data validation.

use kestrum::data::GameData;
use std::{collections::BTreeSet, path::Path};

#[test]
fn runtime_artwork_is_registered_and_present() {
    let data = GameData::load().unwrap();
    let registry: serde_json::Value =
        macroquad_toolkit::include_json!("../asset_registry.json").unwrap();
    let registered: BTreeSet<_> = registry["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect();
    assert_eq!(registry["version"], 1);
    for path in [
        &data.presentation.map_path,
        &data.presentation.font_path,
        &data.presentation.body_font_path,
    ] {
        assert!(registered.contains(path.as_str()));
    }
    for path in registered {
        assert!(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(path).is_file(),
            "Missing asset: {path}"
        );
    }
}

#[test]
fn malformed_interface_data_is_rejected_before_play() {
    let data = GameData::load().unwrap();
    let mut missing_copy = data.game_text.clone();
    missing_copy.text.remove("new_game");
    let error = missing_copy
        .validate(&data.presentation.geography)
        .unwrap_err();
    assert!(error.starts_with("game_text.json:") && error.contains("new_game"));
    let mut wrong_seasons = data.game_text.clone();
    wrong_seasons.seasons.clear();
    assert!(wrong_seasons
        .validate(&data.presentation.geography)
        .unwrap_err()
        .starts_with("game_text.json:"));
    let mut invalid_position = data.presentation.clone();
    invalid_position.geography[0].position[0] = f32::NAN;
    assert!(invalid_position
        .validate()
        .unwrap_err()
        .starts_with("game_config.json:"));

    let mut duplicate_id = data.presentation.clone();
    duplicate_id.geography[1].id = duplicate_id.geography[0].id.clone();
    assert!(duplicate_id
        .validate()
        .unwrap_err()
        .starts_with("game_config.json:"));

    let mut missing_geography_text = data.game_text.clone();
    missing_geography_text.geography.remove("pale_range");
    assert!(missing_geography_text
        .validate(&data.presentation.geography)
        .unwrap_err()
        .starts_with("game_text.json:"));
    let mut extra_geography_text = data.game_text.clone();
    extra_geography_text
        .geography
        .insert("unknown_region".into(), "Unknown Region".into());
    assert!(extra_geography_text
        .validate(&data.presentation.geography)
        .unwrap_err()
        .starts_with("game_text.json:"));
    let mut blank_title = data.game_text.clone();
    blank_title.title.clear();
    assert!(blank_title
        .validate(&data.presentation.geography)
        .unwrap_err()
        .starts_with("game_text.json:"));
    let mut blank_season = data.game_text.clone();
    blank_season.seasons[0] = "  ".into();
    assert!(blank_season
        .validate(&data.presentation.geography)
        .unwrap_err()
        .starts_with("game_text.json:"));
}

#[test]
fn player_facing_copy_is_loaded_from_game_text() {
    let data = GameData::load().unwrap();
    let config: serde_json::Value =
        macroquad_toolkit::include_json!("../assets/data/game_config.json").unwrap();
    assert!(["title", "subtitle", "edition", "seasons", "text"]
        .iter()
        .all(|field| config.get(field).is_none()));
    assert!(config["geography"]
        .as_array()
        .unwrap()
        .iter()
        .all(|label| label.get("name").is_none() && label.get("id").is_some()));
    assert_eq!(data.game_text.title, "KESTRUM");
    assert_eq!(
        data.game_text.subtitle,
        "Every kingdom begins with a place."
    );
    assert_eq!(data.game_text.edition, "THE FIRST CHRONICLE");
    assert_eq!(
        data.game_text.seasons,
        vec![
            "Spring".to_owned(),
            "Summer".to_owned(),
            "Autumn".to_owned(),
            "Winter".to_owned()
        ]
    );
    assert_eq!(data.game_text.geography["pale_range"], "THE PALE RANGE");
}
