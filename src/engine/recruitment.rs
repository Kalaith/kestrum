//! Local recruitment requirements and creation of fresh, exhausted formations.

use super::{preview, Actor, Command, RuleError};
use crate::{
    data::{
        economy::{Habitation, Resources, TroopKind},
        world::{Facility, FactionId, MilitaryLayer, Site, SiteId, SiteTag},
        GameData,
    },
    state::{
        campaign::DomainFactKind,
        military::{Army, ArmyId, Formation, FormationId},
        StrategicCampaign,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecruitmentResult {
    pub army: ArmyId,
    pub formation: FormationId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecruitOption {
    pub kind: TroopKind,
    pub cost: Resources,
    pub upkeep_gold: i64,
    pub capacity: u32,
    pub movement_allowance: u32,
    pub blocked: Option<String>,
}

/// Build options outside the UI using exactly the command's read-only preview.
pub fn recruit_options(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    site: SiteId,
    army: Option<ArmyId>,
) -> Vec<RecruitOption> {
    let actor = if observer == campaign.player {
        Actor::Player
    } else {
        Actor::Npc(observer)
    };
    data.economy
        .formations
        .iter()
        .map(|(&kind, definition)| RecruitOption {
            kind,
            cost: definition.recruit_cost,
            upkeep_gold: definition.upkeep_gold,
            capacity: definition.capacity,
            movement_allowance: definition.movement_allowance,
            blocked: preview(campaign, data, actor, Command::Recruit { site, army, kind })
                .err()
                .map(|error| error.to_string()),
        })
        .collect()
}

pub(super) fn recruit(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    site: SiteId,
    army: Option<ArmyId>,
    kind: TroopKind,
) -> Result<RecruitmentResult, RuleError> {
    validate_recruit(campaign, data, site, army, kind)?;
    let owner = campaign.active_faction();
    let definition = &data.economy.formations[&kind];
    let formation = campaign.next_ids.formation;
    campaign.next_ids.formation =
        FormationId(formation.0.checked_add(1).ok_or(RuleError::Overflow {
            field: "formation identifiers",
        })?);
    let army = if let Some(army) = army {
        army
    } else {
        let army = campaign.next_ids.army;
        campaign.next_ids.army = ArmyId(army.0.checked_add(1).ok_or(RuleError::Overflow {
            field: "army identifiers",
        })?);
        campaign.armies.insert(
            army,
            Army {
                id: army,
                faction: owner,
                site,
                name: format!("Army {}", army.0),
                slots: [None; 6],
                commander: None,
            },
        );
        army
    };
    let receiving = campaign
        .armies
        .get_mut(&army)
        .expect("validated receiving army");
    let slot = receiving
        .slots
        .iter_mut()
        .find(|slot| slot.is_none())
        .expect("validated free slot");
    *slot = Some(formation);
    campaign.formations.insert(
        formation,
        Formation {
            service: Default::default(),
            id: formation,
            faction: owner,
            kind,
            headcount: definition.capacity,
            capacity: definition.capacity,
            movement_spent: definition.movement_allowance,
            created_round: campaign.completed_rounds,
        },
    );
    let balance = &mut campaign
        .factions
        .get_mut(&owner)
        .expect("validated actor")
        .resources;
    balance.gold -= definition.recruit_cost.gold;
    balance.wood -= definition.recruit_cost.wood;
    balance.stone -= definition.recruit_cost.stone;
    Ok(RecruitmentResult { army, formation })
}

pub(super) fn validate_recruit(
    campaign: &StrategicCampaign,
    data: &GameData,
    site: SiteId,
    army: Option<ArmyId>,
    kind: TroopKind,
) -> Result<(), RuleError> {
    let faction = &campaign.factions[&campaign.active_faction()];
    let location = campaign
        .world
        .site(site)
        .ok_or(RuleError::UnknownSite { site })?;
    if location.controller != Some(faction.id) {
        return Err(RuleError::SiteNotOwned { site });
    }
    if campaign.active_threat(site).is_some() {
        return Err(RuleError::InvalidState(
            "Clear the local threat before recruiting here.".into(),
        ));
    }
    if campaign.site_is_ruined(site) {
        return Err(RuleError::InvalidState(
            "Reclaim this ruined site before recruiting here.".into(),
        ));
    }
    if campaign.world.contested_sites.contains(&site) {
        return Err(RuleError::SiteContested { site });
    }
    if location.habitation < Habitation::Outpost {
        return Err(RuleError::RecruitingSiteRequired { site });
    }
    if !campaign.supplied_sites(faction.id).contains(&site) {
        return Err(RuleError::SiteUnsupplied { site });
    }
    if faction.deficit {
        return Err(RuleError::Deficit {
            faction: faction.id,
        });
    }
    if let Some(army) = army {
        let receiving = campaign
            .armies
            .get(&army)
            .ok_or(RuleError::UnknownArmy { army })?;
        if receiving.faction != faction.id {
            return Err(RuleError::ArmyNotOwned { army });
        }
        if receiving.site != site {
            return Err(RuleError::ArmyElsewhere { army, site });
        }
        if receiving.slots.iter().all(Option::is_some) {
            return Err(RuleError::ArmyFull { army });
        }
    }
    validate_facility(campaign, data, location, kind)?;
    let required = data.economy.formations[&kind].recruit_cost;
    let available = faction.resources;
    if available.gold < required.gold
        || available.wood < required.wood
        || available.stone < required.stone
    {
        return Err(RuleError::InsufficientResources {
            required,
            available,
        });
    }
    Ok(())
}

fn validate_facility(
    campaign: &StrategicCampaign,
    data: &GameData,
    site: &Site,
    kind: TroopKind,
) -> Result<(), RuleError> {
    let facility = match kind {
        TroopKind::Riders => Facility::Stable,
        TroopKind::Medics => Facility::Infirmary,
        TroopKind::SiegeEngines => Facility::Workshop,
        _ => return Ok(()),
    };
    if !site.facilities.contains(&facility) {
        return Err(RuleError::MissingFacility {
            site: site.id,
            facility,
        });
    }
    if campaign.world.structural_damage(site.id) >= data.economy.facility_failure_damage {
        return Err(RuleError::FacilityDamaged { site: site.id });
    }
    if kind == TroopKind::Riders && !site.tags.contains(&SiteTag::HorseAccess) {
        return Err(RuleError::MissingHorses { site: site.id });
    }
    if kind == TroopKind::SiegeEngines && site.military != MilitaryLayer::Fort {
        return Err(RuleError::FortRequired { site: site.id });
    }
    Ok(())
}

pub(super) fn disband(
    campaign: &mut StrategicCampaign,
    formation: FormationId,
) -> Result<DomainFactKind, RuleError> {
    let selected = campaign
        .formations
        .get(&formation)
        .ok_or(RuleError::UnknownFormation { formation })?;
    if selected.faction != campaign.active_faction() {
        return Err(RuleError::FormationNotOwned { formation });
    }
    let army = campaign
        .armies
        .values()
        .find(|army| army.slots.contains(&Some(formation)))
        .ok_or_else(|| RuleError::InvalidState("Formation has no containing army.".into()))?;
    let fact = DomainFactKind::FormationDisbanded {
        faction: selected.faction,
        army: army.id,
        formation,
        site: army.site,
        troop: selected.kind,
    };
    campaign
        .remove_formation(formation)
        .map_err(RuleError::InvalidState)?;
    Ok(fact)
}
