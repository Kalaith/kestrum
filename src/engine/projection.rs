//! Public geography and observer-owned information for presentation and planning.

use super::RuleError;
use crate::{
    data::{
        economy::Resources,
        rules::Emblem,
        world::{FactionId, SiteId},
    },
    state::{world::CampaignWorld, CampaignId, CampaignPhase, FactionStatus, StrategicCampaign},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleFaction {
    pub id: FactionId,
    pub name: String,
    pub emblem: Emblem,
    pub status: FactionStatus,
    pub resources: Option<Resources>,
    pub headquarters: Option<SiteId>,
    pub capital: Option<SiteId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisibleCampaign {
    pub campaign_id: CampaignId,
    pub completed_rounds: u32,
    pub player: FactionId,
    pub active_faction: FactionId,
    pub active_faction_name: String,
    pub player_turn: bool,
    pub npc_paused: bool,
    pub factions: Vec<VisibleFaction>,
    pub world: CampaignWorld,
}

pub fn project(
    campaign: &StrategicCampaign,
    observer: FactionId,
) -> Result<VisibleCampaign, RuleError> {
    if !campaign.factions.contains_key(&observer) {
        return Err(RuleError::UnknownActor);
    }
    let active_faction = campaign.active_faction();
    let active = campaign
        .factions
        .get(&active_faction)
        .ok_or(RuleError::UnknownActor)?;
    Ok(VisibleCampaign {
        campaign_id: campaign.campaign_id,
        completed_rounds: campaign.completed_rounds,
        player: campaign.player,
        active_faction,
        active_faction_name: active.name.clone(),
        player_turn: active_faction == campaign.player,
        npc_paused: matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. }),
        factions: campaign
            .factions
            .values()
            .map(|faction| {
                let owned = faction.id == observer;
                VisibleFaction {
                    id: faction.id,
                    name: faction.name.clone(),
                    emblem: faction.emblem,
                    status: faction.status,
                    resources: owned.then_some(faction.resources),
                    headquarters: owned.then_some(faction.headquarters),
                    capital: owned.then_some(faction.capital),
                }
            })
            .collect(),
        world: campaign.world.clone(),
    })
}
