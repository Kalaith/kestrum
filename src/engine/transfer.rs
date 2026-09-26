//! Free whole-formation and person transfers at a stable, co-located boundary.

use super::RuleError;
use crate::{
    data::world::{FactionId, SiteId},
    state::{
        campaign::DomainFactKind,
        military::{Army, ArmyId, FormationId},
        people::{PersonAssignment, PersonId},
        StrategicCampaign,
    },
};

pub(super) fn formation(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    formation: FormationId,
    to_army: ArmyId,
    to_slot: usize,
) -> Result<DomainFactKind, RuleError> {
    let source = formation_army(campaign, owner, formation)?;
    let target = campaign
        .armies
        .get(&to_army)
        .ok_or(RuleError::UnknownArmy { army: to_army })?;
    if target.faction != owner {
        return Err(RuleError::ArmyNotOwned { army: to_army });
    }
    if campaign.armies[&source].site != target.site {
        return Err(RuleError::NotColocated);
    }
    if to_slot >= 6 {
        return Err(RuleError::InvalidSlot);
    }
    if target.slots[to_slot].is_some() {
        return Err(RuleError::SlotOccupied);
    }
    let site = target.site;
    move_formation(campaign, formation, source, to_army, to_slot);
    Ok(DomainFactKind::FormationTransferred {
        faction: owner,
        formation,
        from_army: source,
        to_army,
        site,
    })
}

pub(super) fn split(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    formation: FormationId,
) -> Result<(ArmyId, DomainFactKind), RuleError> {
    let source = formation_army(campaign, owner, formation)?;
    let site = campaign.armies[&source].site;
    let id = campaign.next_ids.army;
    let next = ArmyId(id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "army identifiers",
    })?);
    campaign.armies.insert(
        id,
        Army {
            id,
            faction: owner,
            site,
            name: format!("Army {}", id.0),
            slots: [None; 6],
            commander: None,
        },
    );
    campaign.next_ids.army = next;
    move_formation(campaign, formation, source, id, 0);
    Ok((
        id,
        DomainFactKind::FormationTransferred {
            faction: owner,
            formation,
            from_army: source,
            to_army: id,
            site,
        },
    ))
}

pub(super) fn person(
    campaign: &mut StrategicCampaign,
    owner: FactionId,
    person: PersonId,
    to_formation: FormationId,
) -> Result<DomainFactKind, RuleError> {
    let target = formation_army(campaign, owner, to_formation)?;
    let selected = campaign
        .people
        .get(&person)
        .ok_or(RuleError::UnknownPerson { person })?;
    if selected.faction != owner {
        return Err(RuleError::PersonNotOwned { person });
    }
    if selected.assignment
        == (PersonAssignment::Formation {
            formation: to_formation,
        })
    {
        return Err(RuleError::TransferUnchanged);
    }
    let source = match selected.assignment {
        PersonAssignment::Formation { formation } => {
            Some(formation_army(campaign, owner, formation)?)
        }
        PersonAssignment::Site { .. } => None,
    };
    let source_site = match selected.assignment {
        PersonAssignment::Formation { .. } => {
            campaign.armies[&source.ok_or(RuleError::NotColocated)?].site
        }
        PersonAssignment::Site { site } => site,
    };
    if campaign.armies[&target].site != source_site {
        return Err(RuleError::NotColocated);
    }
    let carried_commander = source.filter(|id| campaign.armies[id].commander == Some(person));
    campaign
        .people
        .get_mut(&person)
        .ok_or(RuleError::UnknownPerson { person })?
        .assignment = PersonAssignment::Formation {
        formation: to_formation,
    };
    if let Some(source) = carried_commander {
        carry_commander(campaign, source, target, person);
    }
    Ok(DomainFactKind::PersonTransferred {
        faction: owner,
        person,
        to_formation,
        site: source_site,
    })
}

fn formation_army(
    campaign: &StrategicCampaign,
    owner: FactionId,
    formation: FormationId,
) -> Result<ArmyId, RuleError> {
    let entry = campaign
        .formations
        .get(&formation)
        .ok_or(RuleError::UnknownFormation { formation })?;
    if entry.faction != owner {
        return Err(RuleError::FormationNotOwned { formation });
    }
    campaign
        .armies
        .values()
        .find(|army| army.formation_ids().any(|id| id == formation))
        .map(|army| army.id)
        .ok_or_else(|| RuleError::InvalidState("Formation has no army".into()))
}

fn move_formation(
    campaign: &mut StrategicCampaign,
    formation: FormationId,
    source: ArmyId,
    target: ArmyId,
    slot: usize,
) {
    let commander = campaign.armies[&source].commander.filter(|id| {
        campaign
            .people
            .get(id)
            .is_some_and(|person| person.assignment == (PersonAssignment::Formation { formation }))
    });
    let source_army = campaign.armies.get_mut(&source).expect("validated source");
    for entry in &mut source_army.slots {
        if *entry == Some(formation) {
            *entry = None;
        }
    }
    campaign
        .armies
        .get_mut(&target)
        .expect("validated target")
        .slots[slot] = Some(formation);
    if let Some(person) = commander {
        carry_commander(campaign, source, target, person);
    }
    if campaign.armies[&source].is_empty() {
        campaign.armies.remove(&source);
    }
}

fn carry_commander(
    campaign: &mut StrategicCampaign,
    source: ArmyId,
    target: ArmyId,
    person: PersonId,
) {
    if source == target {
        return;
    }
    campaign
        .armies
        .get_mut(&source)
        .expect("validated source")
        .commander = None;
    let target = campaign.armies.get_mut(&target).expect("validated target");
    if target.commander.is_none() {
        target.commander = Some(person);
    }
}

/// Physical co-location for an observer-owned person, including people at a site.
pub fn person_site(campaign: &StrategicCampaign, person: PersonId) -> Option<SiteId> {
    match campaign.people.get(&person)?.assignment {
        PersonAssignment::Site { site } => Some(site),
        PersonAssignment::Formation { formation } => campaign
            .armies
            .values()
            .find(|army| army.formation_ids().any(|id| id == formation))
            .map(|army| army.site),
    }
}
