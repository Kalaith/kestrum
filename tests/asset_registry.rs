//! Actual runtime artwork, interface copy, and semantic data validation.

use kestrum::data::GameData;
use std::{collections::BTreeSet, path::Path};

#[test]
fn runtime_artwork_is_registered_and_present() {
    let data = GameData::load().unwrap().presentation;
    let registry: serde_json::Value =
        macroquad_toolkit::include_json!("../asset_registry.json").unwrap();
    let registered: BTreeSet<_> = registry["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect();
    assert_eq!(registry["version"], 1);
    for path in [&data.map_path, &data.font_path, &data.body_font_path] {
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
    let data = GameData::load().unwrap().presentation;
    let mut missing_copy = data.clone();
    missing_copy.text.remove("new_game");
    assert!(missing_copy.validate().unwrap_err().contains("new_game"));
    let mut wrong_seasons = data.clone();
    wrong_seasons.seasons.clear();
    assert!(wrong_seasons.validate().is_err());
    let mut invalid_position = data;
    invalid_position.geography[0].position[0] = f32::NAN;
    assert!(invalid_position.validate().is_err());
}
