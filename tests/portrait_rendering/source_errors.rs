//! Missing and malformed encoded sources feed the public queue's negative cache.

use super::fixtures::{data, parity_appearance, read_sources};
use kestrum::portrait_rendering::{
    PortraitCompositor, PortraitFallback, PortraitRequestQueue, PortraitRequestStatus,
};
use std::collections::BTreeMap;

fn compose_and_remember_failure(
    compositor: &mut PortraitCompositor,
    compressed: &BTreeMap<String, Vec<u8>>,
    appearance: &kestrum::data::portraits::AppearanceDescriptor,
    queue: &mut PortraitRequestQueue,
) -> String {
    assert_eq!(
        queue.request(appearance.clone(), 128),
        PortraitRequestStatus::Enqueued
    );
    let job = queue.take_next().expect("queued composition request");
    let Err(error) = compositor.compose(compressed, job.appearance(), job.size()) else {
        panic!("invalid source unexpectedly composed");
    };
    queue.remember_failure(job.appearance(), job.size());
    assert!(queue.has_failure(appearance, 128));
    assert_eq!(
        queue.request(appearance.clone(), 128),
        PortraitRequestStatus::PreviouslyFailed
    );
    error
}

#[test]
fn missing_and_corrupt_sources_are_reported_and_negative_cached() {
    assert_eq!(
        PortraitFallback::for_snapshot(None, 17, true),
        Some(PortraitFallback::Unknown)
    );
    assert_eq!(
        PortraitFallback::for_snapshot(Some(16), 17, true),
        Some(PortraitFallback::Child)
    );
    assert_eq!(PortraitFallback::for_snapshot(Some(17), 17, true), None);
    assert_eq!(
        PortraitFallback::for_snapshot(Some(17), 17, false),
        Some(PortraitFallback::Unknown)
    );
    assert!(!PortraitFallback::Adult.uses_authored_fallback());
    assert!(PortraitFallback::Child.uses_authored_fallback());
    assert!(PortraitFallback::Unknown.uses_authored_fallback());

    let data = data();
    let appearance = parity_appearance();
    let face_source = data
        .portraits
        .faces
        .iter()
        .find(|face| face.id == appearance.face_id)
        .and_then(|face| face.assets.as_ref())
        .expect("parity face art exists")
        .base
        .source
        .clone();

    let mut missing_compositor =
        PortraitCompositor::new(&data.portraits).expect("build art manifest");
    let mut missing = read_sources(&missing_compositor);
    missing.remove(&face_source);
    let mut missing_queue = PortraitRequestQueue::default();
    let missing_error = compose_and_remember_failure(
        &mut missing_compositor,
        &missing,
        &appearance,
        &mut missing_queue,
    );
    assert!(missing_error.contains(&face_source));
    assert!(missing_error.contains("missing"));

    let mut corrupt_compositor =
        PortraitCompositor::new(&data.portraits).expect("build fresh art manifest");
    let mut corrupt = read_sources(&corrupt_compositor);
    corrupt.insert(face_source.clone(), vec![0, 1, 2]);
    let mut corrupt_queue = PortraitRequestQueue::default();
    let corrupt_error = compose_and_remember_failure(
        &mut corrupt_compositor,
        &corrupt,
        &appearance,
        &mut corrupt_queue,
    );
    assert!(corrupt_error.contains(&face_source));
}
