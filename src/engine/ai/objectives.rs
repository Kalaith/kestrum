//! Explainable objective order: distance, strategic need, witnessed rivalry, stable identity.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiTarget {
    pub site: SiteId,
    pub travel_cost: u32,
    pub strategic_priority: u8,
}

/// Order already reachable candidates using only this observer's retained facts.
pub fn rank_targets(
    campaign: &StrategicCampaign,
    view: &VisibleCampaign,
    mut targets: Vec<AiTarget>,
) -> Vec<AiTarget> {
    targets.sort_by_key(|target| {
        (
            target.travel_cost,
            target.strategic_priority,
            std::cmp::Reverse(rival_score(campaign, view, target.site)),
            target.site,
        )
    });
    targets
}

fn rival_score(campaign: &StrategicCampaign, view: &VisibleCampaign, site: SiteId) -> u32 {
    // A last encounter is a dated hint, never a lookup of the enemy's current location.
    if !view.hostile_presence.contains(&site) {
        return 0;
    }
    let Some(known) = campaign.knowledge.observers.get(&view.observer) else {
        return 0;
    };
    view.people
        .iter()
        .flat_map(|person| &person.career.relationships)
        .filter(|(id, relation)| {
            relation.mutual_combat_rounds >= 2
                && known
                    .people
                    .get(id)
                    .is_some_and(|encounter| encounter.site == site)
        })
        .map(|(_, relation)| relation.mutual_combat_rounds)
        .max()
        .unwrap_or(0)
}
