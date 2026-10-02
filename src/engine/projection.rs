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
        legacy::LegacyItem,
        mentorship::Mentorship,
        military::{Army, ArmyId, EconomyStatement, Formation, RecoveryStatement},
        people::{Person, PersonId},
        relationships::{Household, PersonFamily, SuccessorRegister},
        siege::SiegeId,
        world::CampaignWorld,
        CampaignId, CampaignPhase, FactionStatus, StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

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
    pub movement_plans: Vec<crate::state::movement::MovementPlan>,
    pub threats: Vec<super::threats::VisibleThreat>,
    pub sieges: Vec<VisibleSiege>,
    pub construction: Vec<crate::state::construction::ConstructionOrder>,
    /// Encounter snapshots stay participant-only except on a full observer map.
    pub battles: Vec<BattleReport>,
    pub campaign_id: CampaignId,
    pub completed_rounds: u32,
    pub player: FactionId,
    pub observer: FactionId,
    /// The campaign is running without player commands.
    pub observer_mode: bool,
    /// This map projection can inspect the whole campaign.
    pub full_map_visibility: bool,
    /// Discovery boundary retained even by projections used outside the map.
    pub known_sites: BTreeSet<SiteId>,
    /// Derived only for the observer; foreign functional HQ roles stay private.
    pub supplied_sites: BTreeSet<SiteId>,
    /// Observer map projections include supply by every faction.
    pub faction_supplied_sites: BTreeMap<FactionId, BTreeSet<SiteId>>,
    /// Army supply uses the existing besieger endpoint rule; full maps include every army.
    pub supplied_armies: BTreeSet<ArmyId>,
    /// Known hostile locations without faction-specific counts in ordinary views.
    pub hostile_presence: BTreeSet<SiteId>,
    pub active_faction: FactionId,
    pub active_faction_name: String,
    pub player_turn: bool,
    pub npc_paused: bool,
    pub factions: Vec<VisibleFaction>,
    pub world: CampaignWorld,
    /// Exact military and personnel records are private unless this is a full observer map.
    pub armies: Vec<Army>,
    pub formations: Vec<Formation>,
    pub people: Vec<Person>,
    /// Household and succession records belong to the observer, unless fully inspected.
    pub households: Vec<Household>,
    pub families: BTreeMap<PersonId, PersonFamily>,
    pub successors: SuccessorRegister,
    pub apprentice_last_invited_year: Option<u32>,
    pub apprentice_invited_years: BTreeMap<FactionId, u32>,
    /// Mentorship assignments are private unless this is a full observer map.
    pub mentorships: BTreeMap<PersonId, Mentorship>,
    /// Items and custody details are private unless this is a full observer map.
    pub legacy_items: Vec<LegacyItem>,
    pub era_label: String,
}

pub fn project(
    campaign: &StrategicCampaign,
    observer: FactionId,
) -> Result<VisibleCampaign, RuleError> {
    project_with_visibility(campaign, observer, false)
}

pub(super) fn project_observer_map(
    campaign: &StrategicCampaign,
    observer: FactionId,
) -> Result<VisibleCampaign, RuleError> {
    project_with_visibility(campaign, observer, true)
}

fn project_with_visibility(
    campaign: &StrategicCampaign,
    observer: FactionId,
    full_map_visibility: bool,
) -> Result<VisibleCampaign, RuleError> {
    let full_map_visibility = full_map_visibility && campaign.is_observer();
    let active_faction = campaign.active_faction();
    let active = campaign
        .factions
        .get(&active_faction)
        .ok_or(RuleError::UnknownActor)?;
    if !campaign.factions.contains_key(&observer) {
        return Err(RuleError::UnknownActor);
    }
    let map = map_records(campaign, observer, full_map_visibility);
    let roster = roster_records(campaign, observer, full_map_visibility);
    Ok(VisibleCampaign {
        movement_plans: map.movement_plans,
        threats: map.threats,
        sieges: map.sieges,
        construction: map.construction,
        battles: map.battles,
        campaign_id: campaign.campaign_id,
        completed_rounds: campaign.completed_rounds,
        player: campaign.player,
        observer,
        observer_mode: campaign.is_observer(),
        full_map_visibility,
        known_sites: map.known_sites,
        supplied_sites: map.supplied_sites,
        faction_supplied_sites: map.faction_supplied_sites,
        supplied_armies: map.supplied_armies,
        hostile_presence: map.hostile_presence,
        active_faction,
        active_faction_name: active.name.clone(),
        player_turn: !campaign.is_observer() && active_faction == campaign.player,
        npc_paused: matches!(campaign.phase, CampaignPhase::NpcTurn { paused: true, .. }),
        factions: map.factions,
        world: observed_world(campaign, observer, full_map_visibility),
        armies: roster.armies,
        formations: roster.formations,
        people: roster.people,
        households: roster.households,
        families: roster.families,
        successors: roster.successors,
        apprentice_last_invited_year: roster.apprentice_last_invited_year,
        apprentice_invited_years: roster.apprentice_invited_years,
        mentorships: roster.mentorships,
        legacy_items: roster.legacy_items,
        era_label: super::history::current_era(campaign),
    })
}

struct MapRecords {
    movement_plans: Vec<crate::state::movement::MovementPlan>,
    threats: Vec<super::threats::VisibleThreat>,
    sieges: Vec<VisibleSiege>,
    construction: Vec<crate::state::construction::ConstructionOrder>,
    battles: Vec<BattleReport>,
    known_sites: BTreeSet<SiteId>,
    supplied_sites: BTreeSet<SiteId>,
    faction_supplied_sites: BTreeMap<FactionId, BTreeSet<SiteId>>,
    supplied_armies: BTreeSet<ArmyId>,
    hostile_presence: BTreeSet<SiteId>,
    factions: Vec<VisibleFaction>,
}

fn map_records(
    campaign: &StrategicCampaign,
    observer: FactionId,
    full_map_visibility: bool,
) -> MapRecords {
    MapRecords {
        movement_plans: campaign
            .movement_plans
            .iter()
            .filter(|plan| full_map_visibility || plan_belongs_to(campaign, plan, observer))
            .cloned()
            .collect(),
        threats: if full_map_visibility {
            all_active_threats(campaign)
        } else {
            super::threats::visible_threats(campaign, observer)
        },
        sieges: if full_map_visibility {
            all_visible_sieges(campaign)
        } else {
            visible_sieges(campaign, observer)
        },
        construction: campaign
            .construction
            .values()
            .filter(|order| full_map_visibility || order.owner == observer)
            .cloned()
            .collect(),
        battles: if full_map_visibility {
            campaign.battles.values().cloned().collect()
        } else {
            super::battle_reports(campaign, observer)
        },
        known_sites: if full_map_visibility {
            campaign.world.sites.iter().map(|site| site.id).collect()
        } else {
            super::explored_sites(campaign, observer)
        },
        supplied_sites: campaign.supplied_sites(observer),
        faction_supplied_sites: if full_map_visibility {
            campaign
                .factions
                .keys()
                .map(|faction| (*faction, campaign.supplied_sites(*faction)))
                .collect()
        } else {
            BTreeMap::new()
        },
        supplied_armies: campaign
            .armies
            .values()
            .filter(|army| {
                (full_map_visibility || army.faction == observer)
                    && campaign.army_is_supplied(army.id)
            })
            .map(|army| army.id)
            .collect(),
        hostile_presence: if full_map_visibility {
            campaign
                .factions
                .keys()
                .flat_map(|faction| super::hostile_presence(campaign, *faction))
                .collect()
        } else {
            super::hostile_presence(campaign, observer)
        },
        factions: visible_factions(campaign, observer, full_map_visibility),
    }
}

fn visible_factions(
    campaign: &StrategicCampaign,
    observer: FactionId,
    full_map_visibility: bool,
) -> Vec<VisibleFaction> {
    campaign
        .factions
        .values()
        .map(|faction| {
            let visible = full_map_visibility || faction.id == observer;
            VisibleFaction {
                id: faction.id,
                name: faction.name.clone(),
                emblem: faction.emblem,
                status: faction.status,
                resources: visible.then_some(faction.resources),
                deficit: visible.then_some(faction.deficit),
                last_economy: visible.then(|| faction.last_economy.clone()).flatten(),
                last_recovery: visible.then(|| faction.last_recovery.clone()).flatten(),
                headquarters: visible.then_some(faction.headquarters),
                capital: visible.then_some(faction.capital),
            }
        })
        .collect()
}

struct RosterRecords {
    armies: Vec<Army>,
    formations: Vec<Formation>,
    people: Vec<Person>,
    households: Vec<Household>,
    families: BTreeMap<PersonId, PersonFamily>,
    successors: SuccessorRegister,
    apprentice_last_invited_year: Option<u32>,
    apprentice_invited_years: BTreeMap<FactionId, u32>,
    mentorships: BTreeMap<PersonId, Mentorship>,
    legacy_items: Vec<LegacyItem>,
}

fn roster_records(
    campaign: &StrategicCampaign,
    observer: FactionId,
    full_map_visibility: bool,
) -> RosterRecords {
    RosterRecords {
        armies: campaign
            .armies
            .values()
            .filter(|army| full_map_visibility || army.faction == observer)
            .cloned()
            .collect(),
        formations: campaign
            .formations
            .values()
            .filter(|formation| full_map_visibility || formation.faction == observer)
            .cloned()
            .collect(),
        people: campaign
            .people
            .values()
            .filter(|person| full_map_visibility || person.faction == observer)
            .cloned()
            .collect(),
        households: campaign
            .households
            .values()
            .filter(|household| full_map_visibility || household.faction == observer)
            .cloned()
            .collect(),
        families: campaign
            .families
            .iter()
            .filter(|(id, _)| full_map_visibility || person_belongs_to(campaign, **id, observer))
            .map(|(id, family)| (*id, family.clone()))
            .collect(),
        successors: campaign
            .successors
            .iter()
            .filter(|(id, _)| full_map_visibility || person_belongs_to(campaign, **id, observer))
            .map(|(id, entries)| (*id, entries.clone()))
            .collect(),
        apprentice_last_invited_year: campaign
            .apprentice_last_invited_year
            .get(&observer)
            .copied(),
        apprentice_invited_years: if full_map_visibility {
            campaign.apprentice_last_invited_year.clone()
        } else {
            BTreeMap::new()
        },
        mentorships: visible_mentorships(campaign, observer, full_map_visibility),
        legacy_items: campaign
            .legacy_items
            .values()
            .filter(|item| full_map_visibility || item.faction == observer)
            .cloned()
            .collect(),
    }
}

fn person_belongs_to(campaign: &StrategicCampaign, person: PersonId, faction: FactionId) -> bool {
    campaign
        .people
        .get(&person)
        .is_some_and(|person| person.faction == faction)
}

fn visible_mentorships(
    campaign: &StrategicCampaign,
    observer: FactionId,
    full_map_visibility: bool,
) -> BTreeMap<PersonId, Mentorship> {
    campaign
        .mentorships
        .iter()
        .filter(|(learner, mentorship)| {
            full_map_visibility
                || (person_belongs_to(campaign, **learner, observer)
                    && person_belongs_to(campaign, mentorship.mentor, observer))
        })
        .map(|(learner, mentorship)| (*learner, mentorship.clone()))
        .collect()
}

fn plan_belongs_to(
    campaign: &StrategicCampaign,
    plan: &crate::state::movement::MovementPlan,
    observer: FactionId,
) -> bool {
    plan.armies
        .first()
        .and_then(|id| campaign.armies.get(id))
        .is_some_and(|army| army.faction == observer)
}

fn all_active_threats(campaign: &StrategicCampaign) -> Vec<super::threats::VisibleThreat> {
    campaign
        .threats
        .values()
        .filter(|threat| threat.status == crate::state::threat::ThreatStatus::Active)
        .map(|threat| super::threats::VisibleThreat {
            id: threat.id,
            site: threat.site,
            kind: threat.kind,
            name: threat.name.clone(),
            headcount: Some(threat.headcount),
        })
        .collect()
}

fn all_visible_sieges(campaign: &StrategicCampaign) -> Vec<VisibleSiege> {
    campaign
        .sieges
        .values()
        .map(|siege| VisibleSiege {
            id: siege.id,
            site: siege.site,
            role: SiegeRole::Observer,
            elapsed_steps: siege.elapsed_steps,
            fort_damage: campaign
                .world
                .fort_damage
                .get(&siege.site)
                .copied()
                .unwrap_or(0),
            own_armies: Vec::new(),
            defender: Some(siege.defender),
            besieger: Some(siege.besieger),
            defending_armies: siege.defending.clone(),
            besieging_armies: siege.besieging.clone(),
        })
        .collect()
}

fn observed_world(
    campaign: &StrategicCampaign,
    observer: FactionId,
    full_map_visibility: bool,
) -> CampaignWorld {
    let mut world = campaign.world.clone();
    if full_map_visibility {
        return world;
    }
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
    world.fort_damage.retain(|id, _| {
        owned(id)
            || campaign
                .sieges
                .get(id)
                .is_some_and(|siege| siege.besieger == observer)
    });
    world
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiegeRole {
    Defender,
    Besieger,
    Observer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleSiege {
    pub id: SiegeId,
    pub site: SiteId,
    pub role: SiegeRole,
    pub elapsed_steps: u32,
    pub fort_damage: u32,
    pub own_armies: Vec<ArmyId>,
    /// These details are present only on a full observer map projection.
    pub defender: Option<FactionId>,
    pub besieger: Option<FactionId>,
    pub defending_armies: Vec<ArmyId>,
    pub besieging_armies: Vec<ArmyId>,
}

pub(super) fn visible_sieges(
    campaign: &StrategicCampaign,
    observer: FactionId,
) -> Vec<VisibleSiege> {
    campaign
        .sieges
        .values()
        .filter_map(|siege| {
            let (role, own_armies) = if siege.defender == observer {
                (SiegeRole::Defender, siege.defending.clone())
            } else if siege.besieger == observer {
                (SiegeRole::Besieger, siege.besieging.clone())
            } else {
                return None;
            };
            Some(VisibleSiege {
                id: siege.id,
                site: siege.site,
                role,
                elapsed_steps: siege.elapsed_steps,
                fort_damage: campaign
                    .world
                    .fort_damage
                    .get(&siege.site)
                    .copied()
                    .unwrap_or(0),
                own_armies,
                defender: None,
                besieger: None,
                defending_armies: Vec::new(),
                besieging_armies: Vec::new(),
            })
        })
        .collect()
}
