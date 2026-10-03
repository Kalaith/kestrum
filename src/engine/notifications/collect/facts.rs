//! Typed, own-side domain facts preserve the accepted transaction's source IDs.

use super::*;
use crate::state::{
    campaign::{DomainFact, DomainFactKind},
    development::DevelopmentReceipt,
    diplomacy::DiplomacyReceipt,
    siege::SiegeChange,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
) -> Result<(), String> {
    let mut construction_states = BTreeMap::new();
    let mut used_movement_results = BTreeSet::new();
    for fact in &outcome.facts {
        match &fact.kind {
            DomainFactKind::DevelopmentChanged { receipt } => {
                development(before, candidate, data, fact, receipt)?;
            }
            DomainFactKind::SiegeChanged { siege, change } => {
                let involved =
                    siege.defender == candidate.player || siege.besieger == candidate.player;
                if !involved {
                    continue;
                }
                let kind = match change {
                    SiegeChange::Established => NotificationKind::SiegeStarted,
                    SiegeChange::Lifted => NotificationKind::SiegeLifted,
                    SiegeChange::Reinforced | SiegeChange::Progressed => continue,
                };
                if let Some(place) = super::place(candidate, siege.site) {
                    append(
                        candidate,
                        data,
                        NotificationDraft {
                            source: NotificationSourceId::Fact { id: fact.id },
                            kind,
                            round: fact.completed_rounds,
                            sequence: fact.sequence,
                            subject: Some(NotificationSubjectSnapshot::Place(place.clone())),
                            detail: NotificationDetail::Place {
                                place,
                                before: None,
                                after: None,
                                cause: Some(format!("siege_{change:?}").to_lowercase()),
                                forecast_round: None,
                                conditions: Vec::new(),
                            },
                            active: false,
                        },
                    )?;
                }
            }
            DomainFactKind::ConstructionChanged { order } => {
                let previous = construction_states
                    .get(&order.id)
                    .copied()
                    .unwrap_or_else(|| before.construction.get(&order.id).map(|work| work.status));
                construction(candidate, data, fact, order, previous)?;
                construction_states.insert(order.id, Some(order.status));
            }
            DomainFactKind::BattleResolved {
                battle: battle_id, ..
            } => {
                battle(before, candidate, data, fact, *battle_id)?;
            }
            DomainFactKind::DiplomacyChanged { receipt } => {
                diplomacy(candidate, data, fact, receipt)?;
            }
            DomainFactKind::ArmiesMoved { faction, .. } if *faction == candidate.player => {
                movement(
                    before,
                    candidate,
                    data,
                    outcome,
                    fact,
                    &mut used_movement_results,
                )?;
            }
            _ => {}
        }
    }
    unmatched_blocked_movements(before, candidate, data, outcome, &used_movement_results)?;
    Ok(())
}

fn unmatched_blocked_movements(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
    used_results: &BTreeSet<usize>,
) -> Result<(), String> {
    let results = outcome
        .movement
        .iter()
        .chain(outcome.continued_movements.iter())
        .collect::<Vec<_>>();
    let mut emitted = Vec::new();
    for (index, result) in results.iter().enumerate() {
        if used_results.contains(&index)
            || results
                .iter()
                .enumerate()
                .any(|(used_index, prior)| used_results.contains(&used_index) && *prior == *result)
            || emitted.contains(result)
        {
            continue;
        }
        let Some(stop) = &result.stop else {
            continue;
        };
        let snapshots = result
            .armies
            .iter()
            .filter_map(|id| {
                super::army_snapshot(candidate, *id).or_else(|| super::army_snapshot(before, *id))
            })
            .collect::<Vec<_>>();
        let Some(army) = snapshots.first() else {
            continue;
        };
        emitted.push(*result);
        let ordinal = u32::try_from(index)
            .map_err(|_| "too many movement results in one action".to_owned())?;
        let destination_id = result
            .requested_destination
            .or_else(|| result.path.last().copied())
            .or(Some(stop.site));
        let destination = destination_id
            .and_then(|id| super::place(candidate, id).or_else(|| super::place(before, id)));
        let source = NotificationSourceId::Transition {
            accepted_sequence: candidate.accepted_sequence.max(1),
            kind: NotificationKind::MovementBlocked,
            subject: NotificationEntity::Army(army.id),
            ordinal,
        };
        append(
            candidate,
            data,
            NotificationDraft {
                source,
                kind: NotificationKind::MovementBlocked,
                round: candidate.completed_rounds,
                sequence: candidate.accepted_sequence,
                subject: Some(NotificationSubjectSnapshot::Army(army.clone())),
                detail: NotificationDetail::Movement {
                    armies: snapshots,
                    destination,
                    cause: Some(format!("movement_{:?}", stop.reason).to_lowercase()),
                },
                active: false,
            },
        )?;
    }
    Ok(())
}

fn development(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    fact: &DomainFact,
    receipt: &DevelopmentReceipt,
) -> Result<(), String> {
    let player = candidate.player;
    let (kind, site, before_text, after_text, owner) = match receipt {
        DevelopmentReceipt::HabitationChanged {
            site,
            owner,
            from,
            to,
        } => (
            NotificationKind::HabitationChanged,
            *site,
            Some(format!("{from:?}").to_lowercase()),
            Some(format!("{to:?}").to_lowercase()),
            *owner,
        ),
        DevelopmentReceipt::Ruined { site, owner, .. } => {
            (NotificationKind::SiteRuined, *site, None, None, *owner)
        }
        DevelopmentReceipt::CapitalMoved { owner, from, to } if *owner == player => (
            NotificationKind::CapitalRelocated,
            *to,
            site_name(before, candidate, *from),
            site_name(before, candidate, *to),
            Some(*owner),
        ),
        DevelopmentReceipt::HeadquartersMoved { owner, from, to } if *owner == player => (
            NotificationKind::HeadquartersRelocated,
            *to,
            site_name(before, candidate, *from),
            site_name(before, candidate, *to),
            Some(*owner),
        ),
        _ => return Ok(()),
    };
    if owner != Some(player) {
        return Ok(());
    }
    let Some(place) = super::place(candidate, site).or_else(|| super::place(before, site)) else {
        return Ok(());
    };
    append(
        candidate,
        data,
        NotificationDraft {
            source: NotificationSourceId::Fact { id: fact.id },
            kind,
            round: fact.completed_rounds,
            sequence: fact.sequence,
            subject: Some(NotificationSubjectSnapshot::Place(place.clone())),
            detail: NotificationDetail::Place {
                place,
                before: before_text,
                after: after_text,
                cause: Some(development_key(receipt)),
                forecast_round: None,
                conditions: Vec::new(),
            },
            active: false,
        },
    )?;
    Ok(())
}

fn site_name(
    before: &StrategicCampaign,
    candidate: &StrategicCampaign,
    id: crate::data::world::SiteId,
) -> Option<String> {
    super::place(candidate, id)
        .or_else(|| super::place(before, id))
        .map(|place| place.name)
}

fn development_key(receipt: &DevelopmentReceipt) -> String {
    match receipt {
        DevelopmentReceipt::HabitationChanged { .. } => "habitation_changed",
        DevelopmentReceipt::Ruined { .. } => "ruined",
        DevelopmentReceipt::PopulationMoved { .. } => "population_moved",
        DevelopmentReceipt::SiteRenamed { .. } => "site_renamed",
        DevelopmentReceipt::CapitalMoved { .. } => "capital_moved",
        DevelopmentReceipt::HeadquartersMoved { .. } => "headquarters_moved",
    }
    .into()
}

fn construction(
    candidate: &mut StrategicCampaign,
    data: &GameData,
    fact: &DomainFact,
    order: &crate::state::construction::ConstructionOrder,
    previous: Option<crate::state::construction::ConstructionStatus>,
) -> Result<(), String> {
    if order.owner != candidate.player {
        return Ok(());
    }
    let (kind, cause) = match (previous, order.status) {
        (None, crate::state::construction::ConstructionStatus::Active) => return Ok(()),
        (
            Some(crate::state::construction::ConstructionStatus::Active),
            crate::state::construction::ConstructionStatus::Paused { reason },
        ) => (
            NotificationKind::ConstructionBlocked,
            Some(format!("paused_{reason:?}").to_lowercase()),
        ),
        (
            Some(crate::state::construction::ConstructionStatus::Paused { .. }),
            crate::state::construction::ConstructionStatus::Active,
        ) => (NotificationKind::ConstructionResumed, None),
        (_, crate::state::construction::ConstructionStatus::Completed { .. }) => {
            (NotificationKind::ConstructionCompleted, None)
        }
        (
            _,
            crate::state::construction::ConstructionStatus::Cancelled {
                reason: crate::state::construction::CancellationReason::ControlLost,
                ..
            },
        ) => (
            NotificationKind::ConstructionLost,
            Some("control_lost".into()),
        ),
        _ => return Ok(()),
    };
    let snapshot = super::construction(candidate, order);
    let subject = NotificationSubjectSnapshot::Construction(snapshot.clone());
    append(
        candidate,
        data,
        NotificationDraft {
            source: NotificationSourceId::Fact { id: fact.id },
            kind,
            round: fact.completed_rounds,
            sequence: fact.sequence,
            subject: Some(subject),
            detail: NotificationDetail::Work {
                work: snapshot,
                before: previous,
                cause,
                forecast_round: None,
            },
            active: false,
        },
    )?;
    Ok(())
}

fn battle(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    fact: &DomainFact,
    id: crate::state::battle::BattleId,
) -> Result<(), String> {
    if before
        .pending_battle
        .as_ref()
        .is_some_and(|pending| pending.report.id == id)
    {
        return Ok(());
    }
    let Some(report) = candidate.battles.get(&id) else {
        return Ok(());
    };
    let Some(own_side) = report
        .faction_sides()
        .find(|side| side.faction == candidate.player)
    else {
        return Ok(());
    };
    let mut own_armies = Vec::new();
    let mut retreated = Vec::new();
    let mut destroyed = Vec::new();
    for army in &own_side.armies {
        let site_id = army.final_site.unwrap_or(report.site);
        let snapshot = ArmyNotificationSnapshot {
            id: army.id,
            faction: candidate.player,
            name: army.name.clone(),
            site: super::place(candidate, site_id).or_else(|| super::place(before, site_id)),
        };
        if army.final_site.is_none() {
            destroyed.push(snapshot.clone());
        } else if army.final_site != Some(report.site) {
            retreated.push(snapshot.clone());
        }
        own_armies.push(snapshot);
    }
    let Some(site) =
        super::place(candidate, report.site).or_else(|| super::place(before, report.site))
    else {
        return Ok(());
    };
    let snapshot = BattleNotificationSnapshot {
        id,
        completed_rounds: report.completed_rounds,
        outcome: report.outcome,
        site: site.clone(),
        own_armies,
        retreated: retreated.clone(),
        destroyed_armies: destroyed.clone(),
    };
    append(
        candidate,
        data,
        NotificationDraft {
            source: NotificationSourceId::Fact { id: fact.id },
            kind: NotificationKind::BattleResolved,
            round: fact.completed_rounds,
            sequence: fact.sequence,
            subject: Some(NotificationSubjectSnapshot::Battle(snapshot.clone())),
            detail: NotificationDetail::Battle { battle: snapshot },
            active: false,
        },
    )?;
    for (kind, entries) in [
        (NotificationKind::ArmyRetreated, retreated),
        (NotificationKind::ArmyDestroyed, destroyed),
    ] {
        for army in entries {
            let destination = army.site.clone();
            let source =
                super::transition_source(candidate, kind, NotificationEntity::Army(army.id));
            append(
                candidate,
                data,
                NotificationDraft {
                    source,
                    kind,
                    round: fact.completed_rounds,
                    sequence: fact.sequence,
                    subject: Some(NotificationSubjectSnapshot::Army(army.clone())),
                    detail: NotificationDetail::Movement {
                        armies: vec![army],
                        destination,
                        cause: Some("battle_resolution".into()),
                    },
                    active: false,
                },
            )?;
        }
    }
    Ok(())
}

fn diplomacy(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    fact: &DomainFact,
    receipt: &DiplomacyReceipt,
) -> Result<(), String> {
    let involved = match receipt {
        DiplomacyReceipt::WarDeclared { factions }
        | DiplomacyReceipt::PeaceAgreed { factions, .. } => factions.contains(&campaign.player),
        DiplomacyReceipt::PeaceOffered {
            proposer,
            recipient,
        }
        | DiplomacyReceipt::PeaceRejected {
            proposer,
            recipient,
        } => *proposer == campaign.player || *recipient == campaign.player,
        DiplomacyReceipt::ArmyWithdrawn { faction, .. } => *faction == campaign.player,
        DiplomacyReceipt::DefeatPending { faction, victor } => {
            *faction == campaign.player || *victor == campaign.player
        }
        DiplomacyReceipt::FactionResolved {
            faction, victor, ..
        } => *faction == campaign.player || *victor == Some(campaign.player),
        DiplomacyReceipt::CampaignEnded { .. } => !campaign.observer_mode,
    };
    if !involved {
        return Ok(());
    }
    let (key, faction_id) = match receipt {
        DiplomacyReceipt::WarDeclared { factions } => (
            "war_declared",
            factions.iter().copied().find(|id| *id != campaign.player),
        ),
        DiplomacyReceipt::PeaceOffered {
            proposer,
            recipient,
        } => (
            "peace_offered",
            Some(if *proposer == campaign.player {
                *recipient
            } else {
                *proposer
            }),
        ),
        DiplomacyReceipt::PeaceRejected {
            proposer,
            recipient,
        } => (
            "peace_rejected",
            Some(if *proposer == campaign.player {
                *recipient
            } else {
                *proposer
            }),
        ),
        DiplomacyReceipt::PeaceAgreed { factions, .. } => (
            "peace_agreed",
            factions.iter().copied().find(|id| *id != campaign.player),
        ),
        DiplomacyReceipt::ArmyWithdrawn { .. } => ("army_withdrawn", None),
        DiplomacyReceipt::DefeatPending { faction, .. } => ("defeat_pending", Some(*faction)),
        DiplomacyReceipt::FactionResolved { faction, .. } => ("faction_resolved", Some(*faction)),
        DiplomacyReceipt::CampaignEnded { .. } => ("campaign_ended", None),
    };
    let faction = faction_id.and_then(|id| {
        campaign
            .factions
            .get(&id)
            .map(|entry| (id, entry.name.clone()))
    });
    let details = BTreeMap::from([("outcome".into(), key.into())]);
    append(
        campaign,
        data,
        NotificationDraft {
            source: NotificationSourceId::Fact { id: fact.id },
            kind: NotificationKind::DiplomacyOutcome,
            round: fact.completed_rounds,
            sequence: fact.sequence,
            subject: faction
                .as_ref()
                .map(|(id, name)| NotificationSubjectSnapshot::Faction {
                    id: *id,
                    name: name.clone(),
                }),
            detail: NotificationDetail::Diplomacy {
                faction,
                outcome_key: key.into(),
                details,
            },
            active: false,
        },
    )?;
    Ok(())
}

fn movement(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
    fact: &DomainFact,
    used_results: &mut BTreeSet<usize>,
) -> Result<(), String> {
    let DomainFactKind::ArmiesMoved {
        armies,
        path,
        spent,
        ..
    } = &fact.kind
    else {
        return Ok(());
    };
    let Some((index, result)) = outcome
        .movement
        .iter()
        .chain(outcome.continued_movements.iter())
        .enumerate()
        .find(|(index, result)| {
            !used_results.contains(index)
                && result.armies.as_slice() == armies
                && result.path.as_slice() == path
                && result.spent == *spent
        })
    else {
        return Ok(());
    };
    used_results.insert(index);
    let snapshots = result
        .armies
        .iter()
        .filter_map(|id| {
            super::army_snapshot(candidate, *id).or_else(|| super::army_snapshot(before, *id))
        })
        .collect::<Vec<_>>();
    if snapshots.is_empty() {
        return Ok(());
    }
    let arrived = result.requested_destination.is_some_and(|destination| {
        result.armies.iter().all(|id| {
            candidate
                .armies
                .get(id)
                .is_some_and(|army| army.site == destination)
        })
    });
    let destination_id = result
        .requested_destination
        .or_else(|| result.stop.as_ref().map(|stop| stop.site))
        .or_else(|| result.path.last().copied());
    let destination = destination_id
        .and_then(|id| super::place(candidate, id).or_else(|| super::place(before, id)));
    let (kind, cause) = match &result.stop {
        Some(stop)
            if result.planned_destination.is_some()
                && matches!(
                    &stop.reason,
                    crate::engine::MovementBlock::InsufficientMovement { .. }
                ) =>
        {
            return Ok(())
        }
        Some(stop) => (
            NotificationKind::MovementBlocked,
            Some(format!("movement_{:?}", stop.reason).to_lowercase()),
        ),
        None if arrived => (NotificationKind::MovementArrived, None),
        None => return Ok(()),
    };
    append(
        candidate,
        data,
        NotificationDraft {
            source: NotificationSourceId::Fact { id: fact.id },
            kind,
            round: fact.completed_rounds,
            sequence: fact.sequence,
            subject: Some(NotificationSubjectSnapshot::Army(snapshots[0].clone())),
            detail: NotificationDetail::Movement {
                armies: snapshots,
                destination,
                cause,
            },
            active: false,
        },
    )?;
    Ok(())
}
