//! World orders resolve to physical sites; regions are entered through known gates.

use super::*;
use crate::data::world::{MarkerId, MarkerLocation};

pub fn world_movement_preview(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    armies: &[ArmyId],
    destination: MarkerId,
) -> Result<MovementPreview, RuleError> {
    let (origin, _) = validate_group(campaign, data, observer, armies)?;
    if campaign
        .world
        .site(origin)
        .is_some_and(|site| site.marker == destination)
    {
        return Err(RuleError::InvalidRoute);
    }
    let Some(marker) = campaign.world.marker(destination) else {
        return Err(RuleError::InvalidRoute);
    };
    let destinations = match &marker.location {
        MarkerLocation::Site { site } => BTreeSet::from([*site]),
        MarkerLocation::Region { entrances, .. } => {
            entrances.iter().map(|entrance| entrance.site).collect()
        }
    };
    let mut best: Option<MovementPreview> = None;
    for site in destinations {
        let Ok(preview) = map_movement_preview(campaign, data, observer, armies, site) else {
            continue;
        };
        if best.as_ref().is_none_or(|current| {
            rank(campaign, observer, &preview) < rank(campaign, observer, current)
        }) {
            best = Some(preview);
        }
    }
    best.ok_or(RuleError::InvalidRoute)
}

fn rank<'a>(
    campaign: &StrategicCampaign,
    observer: FactionId,
    preview: &'a MovementPreview,
) -> (bool, u32, &'a [SiteId]) {
    // Prefer an open gate even when the army needs another season to reach it.
    let blocked = preview
        .order
        .path
        .iter()
        .skip(1)
        .any(|site| public_block(campaign, observer, *site).is_some());
    (blocked, preview.total_cost, &preview.order.path)
}
