//! Snapshot labels at the action boundary, including entities removed by that action.

use super::*;
use crate::{
    engine::RuleError,
    state::{
        campaign::{DomainFact, DomainFactKind},
        evidence::Veterancy,
        military::{ArmyId, FormationId},
        people::PersonId,
    },
};
use std::collections::BTreeSet;

pub(crate) fn record_facts(
    campaign: &mut StrategicCampaign,
    before: &StrategicCampaign,
    facts: &[DomainFact],
) -> Result<(), RuleError> {
    for fact in facts {
        if matches!(fact.kind, DomainFactKind::FactionPassed { .. }) {
            continue;
        }
        let id = allocate(campaign)?;
        let record = if let DomainFactKind::DiplomacyChanged { ref receipt } = fact.kind {
            HistoryRecord::diplomacy(id, fact.completed_rounds, fact.id, receipt, campaign)
        } else if let DomainFactKind::DevelopmentChanged { ref receipt } = fact.kind {
            HistoryRecord::development(id, fact.completed_rounds, fact.id, receipt, campaign)
        } else if let DomainFactKind::BattleResolved { battle, .. } = fact.kind {
            HistoryRecord::battle(id, &campaign.battles[&battle], Some(fact.id))
        } else if let DomainFactKind::SiegeChanged { ref siege, change } = fact.kind {
            HistoryRecord::siege(
                id,
                fact.completed_rounds,
                fact.id,
                siege,
                change,
                site_label(campaign, siege.site).name,
            )
        } else {
            action_record(campaign, before, fact, id)?
        };
        insert(campaign, record);
    }
    Ok(())
}

pub(crate) fn record_veterancy(
    campaign: &mut StrategicCampaign,
    formation: FormationId,
    tier: Veterancy,
    xp: u32,
) -> Result<(), RuleError> {
    let id = allocate(campaign)?;
    let member = &campaign.formations[&formation];
    let army = campaign
        .armies
        .values()
        .find(|army| army.formation_ids().any(|id| id == formation))
        .expect("valid formation army");
    let record = HistoryRecord {
        id,
        completed_rounds: campaign.completed_rounds,
        source_fact: None,
        kind: HistoryKind::VeterancyEarned {
            formation,
            tier,
            xp,
        },
        sites: vec![site_label(campaign, army.site)],
        armies: vec![EntityLabel {
            id: army.id,
            name: army.name.clone(),
        }],
        people: Vec::new(),
        formations: vec![FormationLabel {
            id: formation,
            kind: member.kind,
        }],
        items: Vec::new(),
        related_events: Vec::new(),
        visible_to: [member.faction].into_iter().collect(),
    };
    insert(campaign, record);
    Ok(())
}

struct ActionParticipants {
    kind: HistoryKind,
    faction: crate::data::world::FactionId,
    sites: Vec<crate::data::world::SiteId>,
    armies: Vec<ArmyId>,
    people: Vec<PersonId>,
    formations: Vec<FormationLabel>,
}

fn action_record(
    campaign: &StrategicCampaign,
    before: &StrategicCampaign,
    fact: &DomainFact,
    id: HistoryId,
) -> Result<HistoryRecord, RuleError> {
    let ActionParticipants {
        kind,
        faction,
        sites,
        armies,
        people,
        mut formations,
    } = action_participants(campaign, before, &fact.kind)?;
    formations.sort_by_key(|entry| entry.id);
    Ok(HistoryRecord {
        id,
        completed_rounds: fact.completed_rounds,
        source_fact: Some(fact.id),
        kind,
        sites: sites
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| site_label(campaign, id))
            .collect(),
        armies: armies
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| army_label(campaign, before, id))
            .collect::<Result<_, _>>()?,
        people: people
            .into_iter()
            .map(|id| person_label(campaign, before, id))
            .collect::<Result<_, _>>()?,
        formations,
        items: Vec::new(),
        related_events: Vec::new(),
        visible_to: [faction].into_iter().collect(),
    })
}

fn action_participants(
    campaign: &StrategicCampaign,
    before: &StrategicCampaign,
    fact: &DomainFactKind,
) -> Result<ActionParticipants, RuleError> {
    match fact {
        DomainFactKind::ConstructionChanged { order } => Ok(ActionParticipants {
            kind: HistoryKind::Construction {
                order: order.clone(),
            },
            faction: order.owner,
            sites: construction_sites(campaign, order.target)?,
            armies: order.builder.into_iter().collect(),
            people: Vec::new(),
            formations: Vec::new(),
        }),
        DomainFactKind::FocusChanged {
            faction,
            site,
            focus,
        } => Ok(ActionParticipants {
            kind: HistoryKind::FocusChanged { focus: *focus },
            faction: *faction,
            sites: vec![*site],
            armies: Vec::new(),
            people: Vec::new(),
            formations: Vec::new(),
        }),
        _ => military_participants(campaign, before, fact),
    }
}

fn construction_sites(
    campaign: &StrategicCampaign,
    target: crate::state::construction::ConstructionTarget,
) -> Result<Vec<crate::data::world::SiteId>, RuleError> {
    use crate::state::construction::ConstructionTarget;
    match target {
        ConstructionTarget::Site(site) => Ok(vec![site]),
        ConstructionTarget::Route(route) => {
            let edge = campaign.world.route(route).ok_or_else(|| {
                RuleError::InvalidState("Construction history has an unknown route.".into())
            })?;
            Ok(vec![edge.from, edge.to])
        }
    }
}

fn military_participants(
    campaign: &StrategicCampaign,
    before: &StrategicCampaign,
    fact: &DomainFactKind,
) -> Result<ActionParticipants, RuleError> {
    let (kind, faction, sites, armies, people, formations) = match fact {
        DomainFactKind::FormationRecruited {
            faction,
            army,
            formation,
            site,
            troop,
        }
        | DomainFactKind::FormationDisbanded {
            faction,
            army,
            formation,
            site,
            troop,
        } => {
            let kind = if matches!(fact, DomainFactKind::FormationRecruited { .. }) {
                HistoryKind::Recruited { troop: *troop }
            } else {
                HistoryKind::Disbanded { troop: *troop }
            };
            (
                kind,
                *faction,
                vec![*site],
                vec![*army],
                vec![],
                vec![FormationLabel {
                    id: *formation,
                    kind: *troop,
                }],
            )
        }
        DomainFactKind::ArmiesMoved {
            faction,
            armies,
            path,
            movement,
            ..
        } => {
            let (people, formations) = movement_labels(before, movement.as_ref());
            (
                HistoryKind::Moved,
                *faction,
                path.clone(),
                armies.clone(),
                people,
                formations,
            )
        }
        DomainFactKind::FormationTransferred {
            faction,
            formation,
            from_army,
            to_army,
            site,
        } => (
            HistoryKind::FormationTransferred,
            *faction,
            vec![*site],
            vec![*from_army, *to_army],
            vec![],
            vec![formation_label(campaign, *formation)],
        ),
        DomainFactKind::PersonTransferred {
            faction,
            person,
            to_formation,
            site,
        } => (
            HistoryKind::PersonTransferred,
            *faction,
            vec![*site],
            vec![],
            vec![*person],
            vec![formation_label(campaign, *to_formation)],
        ),
        _ => {
            return Err(RuleError::InvalidState(
                "Unexpected narrative source.".into(),
            ))
        }
    };
    Ok(ActionParticipants {
        kind,
        faction,
        sites,
        armies,
        people,
        formations,
    })
}

fn formation_label(campaign: &StrategicCampaign, id: FormationId) -> FormationLabel {
    FormationLabel {
        id,
        kind: campaign.formations[&id].kind,
    }
}

fn movement_labels(
    before: &StrategicCampaign,
    movement: Option<&crate::state::evidence::MovementService>,
) -> (Vec<PersonId>, Vec<FormationLabel>) {
    movement
        .map(|receipt| {
            (
                receipt.people.clone(),
                receipt
                    .formations
                    .iter()
                    .filter_map(|id| before.formations.get(id))
                    .map(|formation| FormationLabel {
                        id: formation.id,
                        kind: formation.kind,
                    })
                    .collect(),
            )
        })
        .unwrap_or_default()
}

fn site_label(
    campaign: &StrategicCampaign,
    id: crate::data::world::SiteId,
) -> EntityLabel<crate::data::world::SiteId> {
    EntityLabel {
        id,
        name: campaign
            .world
            .site(id)
            .expect("validated site")
            .name
            .clone(),
    }
}
fn army_label(
    campaign: &StrategicCampaign,
    before: &StrategicCampaign,
    id: ArmyId,
) -> Result<EntityLabel<ArmyId>, RuleError> {
    let army = campaign
        .armies
        .get(&id)
        .or_else(|| before.armies.get(&id))
        .ok_or(RuleError::UnknownArmy { army: id })?;
    Ok(EntityLabel {
        id,
        name: army.name.clone(),
    })
}
fn person_label(
    campaign: &StrategicCampaign,
    before: &StrategicCampaign,
    id: PersonId,
) -> Result<EntityLabel<PersonId>, RuleError> {
    let person = campaign
        .people
        .get(&id)
        .or_else(|| before.people.get(&id))
        .ok_or(RuleError::UnknownPerson { person: id })?;
    Ok(EntityLabel {
        id,
        name: person.name.clone(),
    })
}
pub(super) fn allocate(campaign: &mut StrategicCampaign) -> Result<HistoryId, RuleError> {
    let id = campaign.next_ids.history;
    campaign.next_ids.history = HistoryId(id.0.checked_add(1).ok_or(RuleError::Overflow {
        field: "history identifiers",
    })?);
    Ok(id)
}

pub(super) fn insert(campaign: &mut StrategicCampaign, record: HistoryRecord) {
    let notable_siege = matches!(
        &record.kind,
        HistoryKind::Siege {
            change: crate::state::siege::SiegeChange::Established
                | crate::state::siege::SiegeChange::Lifted,
            ..
        }
    );
    let completed_construction = matches!(
        &record.kind,
        HistoryKind::Construction { order }
            if matches!(order.status, crate::state::construction::ConstructionStatus::Completed { .. })
    );
    if completed_construction
        || notable_siege
        || matches!(&record.kind, HistoryKind::Development { receipt }
            if !matches!(receipt, crate::state::development::DevelopmentReceipt::PopulationMoved { .. }))
        || matches!(
            record.kind,
            HistoryKind::Battle { .. }
                | HistoryKind::VeterancyEarned { .. }
                | HistoryKind::ItemCustodyChanged { .. }
                | HistoryKind::Anniversary { .. }
                | HistoryKind::Life { .. }
        )
    {
        let summary = record.notable();
        for site in &record.sites {
            campaign
                .history
                .site_notables
                .entry(site.id)
                .or_default()
                .push(summary.clone());
        }
        for army in &record.armies {
            if campaign.armies.contains_key(&army.id) {
                campaign
                    .history
                    .army_notables
                    .entry(army.id)
                    .or_default()
                    .push(summary.clone());
            }
        }
        for person in &record.people {
            if campaign.people.contains_key(&person.id) {
                campaign
                    .history
                    .person_notables
                    .entry(person.id)
                    .or_default()
                    .push(summary.clone());
            }
        }
    }
    campaign.history.events.insert(record.id, record);
}

pub(crate) fn restore_battle_history(campaign: &mut StrategicCampaign) {
    for report in campaign.battles.values().cloned().collect::<Vec<_>>() {
        let source = campaign
            .pending_facts
            .iter()
            .find(|fact| {
                matches!(fact.kind,
            DomainFactKind::BattleResolved{battle,..} if battle==report.id)
            })
            .map(|fact| fact.id);
        let id = campaign.next_ids.history;
        campaign.next_ids.history = HistoryId(id.0 + 1);
        insert(campaign, HistoryRecord::battle(id, &report, source));
    }
}
