//! Exercise decoded-source reuse, eviction, and reset through the compositor API.

use super::fixtures::{data, parity_appearance, read_sources};
use kestrum::portrait_rendering::PortraitCompositor;

#[test]
fn decoded_source_cache_reuses_bald_layers_and_evicts_within_32_mib() {
    const SOURCE_BUDGET: usize = 32 * 1024 * 1024;

    let data = data();
    let mut compositor = PortraitCompositor::new(&data.portraits).expect("build art manifest");
    let compressed = read_sources(&compositor);
    let mut bald = parity_appearance();
    bald.hair_id = "hair_bald".into();
    bald.hair_palette_id = "none".into();
    bald.signature = data
        .portraits
        .signature_for(&bald)
        .expect("canonicalize explicit bald fixture");
    data.portraits
        .validate_descriptor(&bald)
        .expect("explicit bald fixture is compatible");

    compositor
        .compose(&compressed, &bald, 128)
        .expect("compose bald portrait");
    let warm = compositor.source_metrics();
    assert!(warm.bytes <= SOURCE_BUDGET);
    assert!(warm.entries > 0);
    compositor
        .compose(&compressed, &bald, 128)
        .expect("reuse bald sources");
    let reused = compositor.source_metrics();
    assert!(reused.hits > warm.hits);
    assert_eq!(reused.decodes, warm.decodes);

    let hairy = parity_appearance();
    assert_ne!(hairy.hair_id, bald.hair_id);
    compositor
        .compose(&compressed, &hairy, 128)
        .expect("compose portrait with layered hair");
    let pressured = compositor.source_metrics();
    assert!(pressured.bytes <= SOURCE_BUDGET);
    assert!(pressured.evictions > warm.evictions);

    compositor.reset_decoded_sources();
    let reset = compositor.source_metrics();
    assert_eq!(reset.bytes, 0);
    assert_eq!(reset.entries, 0);
    compositor
        .compose(&compressed, &bald, 128)
        .expect("rebuild sources after reset");
    assert!(compositor.source_metrics().decodes > pressured.decodes);
}
