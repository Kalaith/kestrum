//! Campaign-specific defence adjustments captured in the resolver opening.

use crate::{
    data::{economy::TroopKind, progression::FormationSpecialization, GameData},
    state::{military::FormationId, StrategicCampaign},
};

pub(super) fn guarded_resistance(
    campaign: &StrategicCampaign,
    data: &GameData,
    id: FormationId,
    kind: TroopKind,
    site: crate::data::world::SiteId,
    defending: bool,
) -> u32 {
    let resistance = data.troops.formations[&kind].resistance;
    if defending
        && is_anchor(campaign, site)
        && campaign.formations[&id].service.specialization
            == Some(FormationSpecialization::ShieldGuard)
    {
        let factor = data.progression.specializations[&FormationSpecialization::ShieldGuard]
            .defense_resistance_permille
            .unwrap_or(1000);
        resistance.saturating_mul(factor) / 1000
    } else {
        resistance
    }
}

fn is_anchor(campaign: &StrategicCampaign, site: crate::data::world::SiteId) -> bool {
    campaign
        .world
        .markers
        .iter()
        .any(|marker| match &marker.location {
            crate::data::world::MarkerLocation::Region { anchors, .. } => {
                anchor_contains(anchors, site)
            }
            crate::data::world::MarkerLocation::Site { .. } => false,
        })
}

fn anchor_contains(
    expression: &crate::data::world::AnchorExpression,
    site: crate::data::world::SiteId,
) -> bool {
    use crate::data::world::AnchorExpression;
    match expression {
        AnchorExpression::ControlledSite { site: id }
        | AnchorExpression::SuppliedEntrance { site: id } => *id == site,
        AnchorExpression::All { conditions } | AnchorExpression::Any { conditions } => conditions
            .iter()
            .any(|condition| anchor_contains(condition, site)),
    }
}
