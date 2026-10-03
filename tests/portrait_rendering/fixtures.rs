//! Shared data and encoded image fixtures for portrait rendering regressions.

use kestrum::{
    data::{portraits::AppearanceDescriptor, GameData},
    portrait_rendering::PortraitCompositor,
};
use std::{collections::BTreeMap, fs, path::PathBuf};

pub(super) fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn data() -> GameData {
    GameData::load().expect("load and validate current portrait catalog")
}

pub(super) fn parity_appearance() -> AppearanceDescriptor {
    let path = root().join("assets/art-source/portraits/illustrated/parity_v1.json");
    let fixture: serde_json::Value = serde_json::from_slice(
        &fs::read(&path)
            .unwrap_or_else(|error| panic!("read parity fixture {}: {error}", path.display())),
    )
    .expect("parse parity fixture");
    serde_json::from_value(fixture["descriptor"].clone()).expect("decode parity appearance")
}

pub(super) fn read_sources(compositor: &PortraitCompositor) -> BTreeMap<String, Vec<u8>> {
    compositor
        .asset_paths()
        .iter()
        .map(|path| {
            let file = root().join(path);
            (
                path.clone(),
                fs::read(&file).unwrap_or_else(|error| {
                    panic!("read portrait source {}: {error}", file.display())
                }),
            )
        })
        .collect()
}
