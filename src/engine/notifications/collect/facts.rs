//! Typed, own-side domain facts preserve the accepted transaction's source IDs.

use super::*;
use crate::state::{
    campaign::{DomainFact, DomainFactKind},
    development::DevelopmentReceipt,
    diplomacy::DiplomacyReceipt,
    siege::SiegeChange,
};
use std::collections::BTreeMap;

pub(super) fn collect(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
) -> Result<(), String> {
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
                construction(before, candidate, data, fact, order)?;
            }
            DomainFactKind::BattleResolved {
                battle: battle_id, ..
            } => {
                battle(before, candidate, data, fact, *battle_id)?;
            }
            DomainFactKind::DiplomacyChanged { receipt } => {
                diplomacy(candidate, data, fact, receipt)?;
            }
            DomainFactKind::ArmiesMoved {
                faction,
                armies,
                path,
                ..
            } if *faction == candidate.player => {
                movement(before, candidate, data, outcome, fact, armies, path)?;
            }
            _ => {}
        }
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
            Some(format!("site_{}", from.0)),
            Some(format!("site_{}", to.0)),
            Some(*owner),
        ),
        DevelopmentReceipt::HeadquartersMoved { owner, from, to } if *owner == player => (
            NotificationKind::HeadquartersRelocated,
            *to,
            Some(format!("site_{}", from.0)),
            Some(format!("site_{}", to.0)),
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
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    fact: &DomainFact,
    order: &crate::state::construction::ConstructionOrder,
) -> Result<(), String> {
    if order.owner != candidate.player {
        return Ok(());
    }
    let previous = before.construction.get(&order.id);
    let (kind, cause) = match (previous.map(|work| work.status), order.status) {
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
                before: previous.map(|work| work.status),
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
                        destination: Some(site.clone()),
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
    armies: &[crate::state::military::ArmyId],
    path: &[crate::data::world::SiteId],
) -> Result<(), String> {
    let results = outcome
        .movement
        .iter()
        .chain(outcome.continued_movements.iter())
        .filter(|movement| movement.armies.iter().any(|id| armies.contains(id)));
    for (ordinal, result) in results.enumerate() {
        let snapshots = result
            .armies
            .iter()
            .filter_map(|id| {
                super::army_snapshot(candidate, *id).or_else(|| super::army_snapshot(before, *id))
            })
            .collect::<Vec<_>>();
        if snapshots.is_empty() {
            continue;
        }
        let destination = result
            .path
            .last()
            .or_else(|| path.last())
            .copied()
            .and_then(|id| super::place(candidate, id).or_else(|| super::place(before, id)));
        let (kind, cause) = match &result.stop {
            Some(stop) => (
                NotificationKind::MovementBlocked,
                Some(format!("movement_{:?}", stop.reason).to_lowercase()),
            ),
            None if result.planned_destination.is_some() => {
                (NotificationKind::MovementArrived, None)
            }
            None => continue,
        };
        let subject_id = snapshots[0].id;
        let source = if ordinal == 0 {
            NotificationSourceId::Fact { id: fact.id }
        } else {
            super::transition_source(candidate, kind, NotificationEntity::Army(subject_id))
        };
        append(
            candidate,
            data,
            NotificationDraft {
                source,
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
    }
    Ok(())
}
