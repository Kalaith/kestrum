//! Public geography and observer-owned information for presentation and planning.

use super::RuleError;
use crate::{
    data::{
        economy::Resources,
        rules::Emblem,
        world::{FactionId, SiteId},
    },
    state::{
        battle::BattleReport,
        military::{Army, EconomyStatement, Formation, RecoveryStatement},
        people::Person,
        world::CampaignWorld,
        CampaignId, CampaignPhase, FactionStatus, StrategicCampaign,
    },
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleFaction {
    pub id: FactionId,
    pub name: String,
    pub emblem: Emblem,
    pub status: FactionStatus,
    pub resources: Option<Resources>,
    pub deficit: Option<bool>,
    pub last_economy: Option<EconomyStatement>,
    pub last_recovery: Option<RecoveryStatement>,
    pub headquarters: Option<SiteId>,
    pub capital: Option<SiteId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisibleCampaign {
    pub construction: Vec<crate::state::construction::ConstructionOrder>,
    /// Actual encounter snapshots, visible only to participants.
    pub battles: Vec<BattleReport>,
    pub campaign_id: CampaignId,
    pub completed_rounds: u32,
    pub player: FactionId,
    pub observer: FactionId,
    /// Derived only for the observer; foreign functional HQ roles stay private.
    pub supplied_sites: BTreeSet<SiteId>,
    /// Boolean hostile contact cues, without foreign army identities or counts.
    pub hostile_presence: BTreeSet<SiteId>,
    pub active_faction: FactionId,
    pub active_faction_name: String,
    pub player_turn: bool,
    pub npc_paused: bool,
    pub factions: Vec<VisibleFaction>,
    pub world: CampaignWorld,
    /// Exact military and personnel records are visible only to their own faction.
    pub armies: Vec<Army>,
    pub formations: Vec<Formation>,
    pub people: Vec<Person>,
}

pub fn project(
    campaign: &StrategicCampaign,
    observer: FactionId,
) -> Result<VisibleCampaign, RuleError> {
    let viewer = campaign
        .factions
        .get(&observer)
        .ok_or(RuleError::UnknownActor)?;
    let active_faction = campaign.active_faction();
    let active = campaign
        .factions
        .get(&active_faction)
        .ok_or(RuleError::UnknownActor)?;
    Ok(VisibleCampaign {
        construction: campaign
            .construction
            .values()
            .filter(|order| order.owner == observer)
            .cloned()
            .collect(),
        battles: super::battle_reports(campaign, observer),
        campaign_id: campaign.campaign_id,
        completed_rounds: campaign.completed_rounds,
        player: campaign.player,
        observer,
        supplied_sites: campaign.world.supplied_sites(observer, viewer.headquarters),
        hostile_presence: super::hostile_presence(campaign, observer),
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
                    deficit: owned.then_some(faction.deficit),
                    last_economy: owned.then(|| faction.last_economy.clone()).flatten(),
                    last_recovery: owned.then(|| faction.last_recovery.clone()).flatten(),
                    headquarters: owned.then_some(faction.headquarters),
                    capital: owned.then_some(faction.capital),
                }
            })
            .collect(),
        world: observed_world(campaign, observer),
        armies: campaign
            .armies
            .values()
            .filter(|army| army.faction == observer)
            .cloned()
            .collect(),
        formations: campaign
            .formations
            .values()
            .filter(|formation| formation.faction == observer)
            .cloned()
            .collect(),
        people: campaign
            .people
            .values()
            .filter(|person| person.faction == observer)
            .cloned()
            .collect(),
    })
}

fn observed_world(campaign: &StrategicCampaign, observer: FactionId) -> CampaignWorld {
    let mut world = campaign.world.clone();
    // Geography and visible structures remain public. Exact civilian pools and
    // selected development orders belong only to the current site controller.
    let owned = |id: &SiteId| {
        campaign
            .world
            .site(*id)
            .is_some_and(|site| site.controller == Some(observer))
    };
    world.population.retain(|id, _| owned(id));
    world.focus.retain(|id, _| owned(id));
    world
}
