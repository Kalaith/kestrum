//! Public manifest construction rejects partial art without rejecting legacy metadata.

use super::fixtures::data;
use kestrum::{data::portraits::PortraitCatalog, portrait_rendering::PortraitCompositor};

#[test]
fn complete_catalog_enables_all_layers_and_missing_art_is_explicit() {
    let data = data();
    let complete = PortraitCompositor::new(&data.portraits).expect("build complete art manifest");
    assert!(complete.enabled());
    assert_eq!(complete.asset_paths().len(), 111);
    assert!(complete
        .asset_paths()
        .iter()
        .all(|path| path.starts_with("assets/portraits/") && path.ends_with(".png")));

    let metadata = PortraitCatalog::load_frozen().expect("load metadata-only frozen catalog");
    let legacy = PortraitCompositor::new(&metadata).expect("metadata-only fallback is supported");
    assert!(!legacy.enabled());
    assert!(legacy.asset_paths().is_empty());

    let mut partial = data.portraits.clone();
    let missing_face = partial.geometry[0].face_id.clone();
    partial
        .faces
        .iter_mut()
        .find(|face| face.id == missing_face)
        .expect("geometry face exists")
        .assets = None;
    assert!(
        PortraitCompositor::new(&partial).is_err(),
        "a partially authored catalog must not silently enable rendering"
    );
}
