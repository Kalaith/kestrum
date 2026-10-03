//! Current known development risks remain discoverable when event delivery is muted.

use crate::{
    engine::{AttentionKind, AttentionTarget, MapAttention, MapOverview},
    state::{
        notifications::{NotificationKind, WarningSubject},
        StrategicCampaign,
    },
};

/// Extend the existing condition list from the collector's shared forecast state.
/// Episode state survives suppressed or omitted receipts; delivery choices never
/// decide whether a known current condition deserves attention.
pub fn extend_attention(campaign: &StrategicCampaign, overview: &mut MapOverview) {
    if campaign.observer_mode {
        return;
    }
    for episode in &campaign.notifications.warning_episodes {
        if !episode.is_present {
            continue;
        }
        let WarningSubject::Site(site) = episode.key.subject else {
            continue;
        };
        if !overview
            .sites
            .get(&site)
            .is_some_and(|site| site.controller == Some(campaign.player))
        {
            continue;
        }
        let kind = match episode.key.kind {
            NotificationKind::RuinRisk => AttentionKind::RuinRisk,
            NotificationKind::DeclineRisk => AttentionKind::DeclineRisk,
            _ => continue,
        };
        let entry = MapAttention {
            target: AttentionTarget::Site(site),
            kind,
        };
        if !overview.attention.contains(&entry) {
            overview.attention.push(entry);
        }
    }
    overview
        .attention
        .sort_by_key(|entry| (entry.kind, entry.target));
}
