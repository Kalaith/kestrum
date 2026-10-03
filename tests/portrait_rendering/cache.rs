//! Public request queue bounds, failure suppression, and retry policy.

use super::fixtures::parity_appearance;
use kestrum::portrait_rendering::{PortraitRequestQueue, PortraitRequestStatus, PENDING_JOB_LIMIT};

#[test]
fn request_queue_is_bounded_and_clear_restores_retry() {
    let base = parity_appearance();
    let mut queue = PortraitRequestQueue::default();
    for index in 0..PENDING_JOB_LIMIT {
        let mut appearance = base.clone();
        appearance.signature = format!("pending-{index}");
        assert_eq!(
            queue.request(appearance, 128),
            PortraitRequestStatus::Enqueued
        );
    }

    let mut duplicate = base.clone();
    duplicate.signature = "pending-0".into();
    assert!(queue.is_full());
    assert!(queue.is_pending(&duplicate, 128));
    assert_eq!(
        queue.request(duplicate.clone(), 128),
        PortraitRequestStatus::AlreadyPending
    );
    let mut overflow = base.clone();
    overflow.signature = "overflow".into();
    assert_eq!(
        queue.request(overflow.clone(), 128),
        PortraitRequestStatus::AtCapacity
    );
    assert_eq!(
        queue.request(base.clone(), 64),
        PortraitRequestStatus::UnsupportedSize
    );

    let first = queue.take_next().expect("oldest queued request");
    assert_eq!(first.appearance(), &duplicate);
    assert_eq!(first.size(), 128);
    assert!(!queue.is_pending(&duplicate, 128));
    assert_eq!(
        queue.request(overflow.clone(), 128),
        PortraitRequestStatus::Enqueued
    );

    for index in 0..=PENDING_JOB_LIMIT {
        let mut appearance = base.clone();
        appearance.signature = format!("failed-{index}");
        queue.remember_failure(&appearance, 128);
    }
    let mut oldest_failure = base.clone();
    oldest_failure.signature = "failed-0".into();
    let mut newest_failure = base;
    newest_failure.signature = format!("failed-{PENDING_JOB_LIMIT}");
    assert_eq!(queue.failure_count(), PENDING_JOB_LIMIT);
    assert!(!queue.has_failure(&oldest_failure, 128));
    assert!(queue.has_failure(&newest_failure, 128));

    queue.clear();
    assert_eq!(queue.pending_len(), 0);
    assert_eq!(queue.failure_count(), 0);
    assert_eq!(
        queue.request(oldest_failure, 128),
        PortraitRequestStatus::Enqueued
    );
}
