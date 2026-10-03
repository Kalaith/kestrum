//! Own-people notices derive only from newly accepted life records and eligibility.

use super::*;
use crate::state::{
    history::{AnniversarySubject, HistoryKind, LifeEvent},
    legacy::{LegacyItemCustody, LegacyItemId},
};
use std::collections::BTreeMap;

pub(super) fn collect(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
) -> Result<(), String> {
    for id in &outcome.life_events {
        let Some(record) = candidate.history.events.get(id) else {
            continue;
        };
        let HistoryKind::Life {
            owner,
            person,
            event,
        } = &record.kind
        else {
            continue;
        };
        if *owner != candidate.player {
            continue;
        }
        let Some(snapshot) =
            super::person(candidate, *person).or_else(|| super::person(before, *person))
        else {
            continue;
        };
        let kind = match event {
            LifeEvent::Emerged { .. } => Some(NotificationKind::NewHero),
            LifeEvent::Arrived { .. } => Some(NotificationKind::PersonArrived),
            LifeEvent::ClassCompleted { .. } => Some(NotificationKind::PersonClassCompleted),
            LifeEvent::Recognized { .. } => Some(NotificationKind::PersonRecognized),
            LifeEvent::Retired => Some(NotificationKind::PersonRetired),
            LifeEvent::NaturalDeath => Some(NotificationKind::PersonDied),
            _ => None,
        };
        let Some(kind) = kind else { continue };
        let (reason_key, reason) = match event {
            LifeEvent::Emerged { troop } => (
                Some("emerged".into()),
                Some(format!("troop_{troop:?}").to_lowercase()),
            ),
            LifeEvent::Arrived { origin } => (
                Some("arrived".into()),
                Some(format!("origin_{origin:?}").to_lowercase()),
            ),
            LifeEvent::ClassCompleted { class } => (
                Some("class_completed".into()),
                Some(format!("class_{class:?}").to_lowercase()),
            ),
            LifeEvent::Recognized { epithet, cause } => (
                Some("recognized".into()),
                Some(format!("{epithet}|{cause:?}")),
            ),
            LifeEvent::Retired => (Some("retired".into()), None),
            LifeEvent::NaturalDeath => (Some("natural_death".into()), None),
            _ => (None, None),
        };
        let source = NotificationSourceId::History { id: record.id };
        append(
            candidate,
            data,
            NotificationDraft {
                source,
                kind,
                round: record.completed_rounds,
                sequence: candidate.accepted_sequence,
                subject: Some(NotificationSubjectSnapshot::Person(Box::new(
                    snapshot.clone(),
                ))),
                detail: NotificationDetail::Person {
                    person: Box::new(snapshot),
                    reason_key,
                    reason,
                    opportunities: Vec::new(),
                },
                active: false,
            },
        )?;
    }
    succession(before, candidate, data, outcome)?;
    career_opportunities(before, candidate, data)?;
    anniversaries(candidate, data, outcome)?;
    legacy_transfers(candidate, data, outcome, &outcome.legacy_items_changed)?;
    Ok(())
}

fn anniversaries(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
) -> Result<(), String> {
    for subject in &outcome.anniversary_reminders {
        let Some(record) = campaign
            .history
            .events
            .values()
            .find(|record| {
                record.visible_to.contains(&campaign.player)
                    && matches!(
                        &record.kind,
                        HistoryKind::Anniversary { subject: candidate, .. } if candidate == subject
                    )
            })
            .cloned()
        else {
            continue;
        };
        let HistoryKind::Anniversary { years, .. } = record.kind else {
            continue;
        };
        let subject_snapshot = match subject {
            AnniversarySubject::Person(id) => super::person(campaign, *id)
                .map(|snapshot| NotificationSubjectSnapshot::Person(Box::new(snapshot))),
            AnniversarySubject::Site(id) => {
                super::place(campaign, *id).map(NotificationSubjectSnapshot::Place)
            }
        };
        let Some(subject_snapshot) = subject_snapshot else {
            continue;
        };
        append(
            campaign,
            data,
            NotificationDraft {
                source: NotificationSourceId::History { id: record.id },
                kind: NotificationKind::Remembrance,
                round: record.completed_rounds,
                sequence: campaign.accepted_sequence,
                subject: Some(subject_snapshot.clone()),
                detail: NotificationDetail::Remembrance {
                    subject: subject_snapshot,
                    years: Some(years),
                    category: None,
                },
                active: false,
            },
        )?;
    }
    Ok(())
}

fn legacy_transfers(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
    changed: &[LegacyItemId],
) -> Result<(), String> {
    for item_id in changed {
        let Some(item) = campaign
            .legacy_items
            .get(item_id)
            .filter(|item| item.faction == campaign.player)
        else {
            continue;
        };
        let Some(record) = campaign
            .history
            .events
            .values()
            .find(|record| {
                record.visible_to.contains(&campaign.player)
                    && matches!(
                        &record.kind,
                        HistoryKind::ItemCustodyChanged { item, .. } if item == item_id
                    )
            })
            .cloned()
        else {
            continue;
        };
        let HistoryKind::ItemCustodyChanged { from, to, .. } = record.kind else {
            continue;
        };
        let details = BTreeMap::from([
            ("item".into(), item.name.clone()),
            ("from".into(), custody_key(from)),
            ("to".into(), custody_key(to)),
        ]);
        append(
            campaign,
            data,
            NotificationDraft {
                source: NotificationSourceId::History { id: record.id },
                kind: NotificationKind::LegacyTransfer,
                round: record.completed_rounds,
                sequence: outcome.accepted_sequence,
                subject: Some(NotificationSubjectSnapshot::History {
                    id: record.id,
                    label: item.name.clone(),
                }),
                detail: NotificationDetail::Facts { values: details },
                active: false,
            },
        )?;
    }
    Ok(())
}

fn custody_key(custody: LegacyItemCustody) -> String {
    match custody {
        LegacyItemCustody::Person(id) => format!("person_{}", id.0),
        LegacyItemCustody::SiteEstate(id) => format!("site_{}", id.0),
    }
}

fn succession(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
) -> Result<(), String> {
    for (ordinal, notice) in outcome.succession.iter().enumerate() {
        let Some(previous) = before.people.get(&notice.predecessor) else {
            continue;
        };
        if previous.faction != candidate.player {
            continue;
        }
        let Some(army) = before
            .armies
            .get(&notice.army)
            .filter(|army| army.faction == candidate.player)
            .or_else(|| {
                candidate
                    .armies
                    .get(&notice.army)
                    .filter(|army| army.faction == candidate.player)
            })
        else {
            continue;
        };
        let predecessor = super::person(candidate, notice.predecessor)
            .or_else(|| super::person(before, notice.predecessor));
        let Some(predecessor) = predecessor else {
            continue;
        };
        let kind = if notice.successor.is_some() {
            NotificationKind::Succession
        } else {
            NotificationKind::VacantCommand
        };
        let mut values = BTreeMap::from([
            (
                "event".into(),
                if notice.successor.is_some() {
                    "successor_assigned"
                } else {
                    "command_vacant"
                }
                .into(),
            ),
            ("army".into(), army.name.clone()),
        ]);
        if let Some(successor) = notice
            .successor
            .and_then(|id| super::person(candidate, id).or_else(|| super::person(before, id)))
        {
            values.insert("successor".into(), successor.name);
        }
        let source = NotificationSourceId::Transition {
            accepted_sequence: candidate.accepted_sequence.max(1),
            kind,
            subject: NotificationEntity::Person(notice.predecessor),
            ordinal: ordinal as u32,
        };
        let round = candidate.completed_rounds;
        let sequence = candidate.accepted_sequence;
        append(
            candidate,
            data,
            NotificationDraft {
                source,
                kind,
                round,
                sequence,
                subject: Some(NotificationSubjectSnapshot::Person(Box::new(
                    predecessor.clone(),
                ))),
                detail: NotificationDetail::Facts { values },
                active: false,
            },
        )?;
    }
    Ok(())
}

fn career_opportunities(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
) -> Result<(), String> {
    let mut newly_eligible = Vec::new();
    for person in candidate
        .people
        .values()
        .filter(|person| person.faction == candidate.player && person.is_alive())
    {
        let previous = before.people.get(&person.id);
        let before_options = if previous.is_some() {
            crate::engine::progression::career_options(before, data, person.id)
                .map_err(|error| error.to_string())?
        } else {
            Vec::new()
        };
        let after = crate::engine::progression::career_options(candidate, data, person.id)
            .map_err(|error| error.to_string())?;
        let classes: Vec<_> = after
            .iter()
            .filter(|option| {
                if option.class == person.class
                    || person.age_years(candidate.completed_rounds) < 17
                    || !requirements_satisfied(&option.requirements)
                {
                    return false;
                }
                let previously_satisfied = before_options
                    .iter()
                    .find(|old| old.class == option.class)
                    .is_some_and(|old| requirements_satisfied(&old.requirements));
                !previously_satisfied
            })
            .map(|option| option.class)
            .collect();
        if !classes.is_empty() {
            newly_eligible.push((person.id, classes));
        }
    }
    for (id, opportunities) in newly_eligible {
        let Some(person) = super::person(candidate, id) else {
            continue;
        };
        let source = NotificationSourceId::Transition {
            accepted_sequence: candidate.accepted_sequence.max(1),
            kind: NotificationKind::CareerOpportunity,
            subject: NotificationEntity::Person(id),
            ordinal: 0,
        };
        let round = candidate.completed_rounds;
        let sequence = candidate.accepted_sequence;
        append(
            candidate,
            data,
            NotificationDraft {
                source,
                kind: NotificationKind::CareerOpportunity,
                round,
                sequence,
                subject: Some(NotificationSubjectSnapshot::Person(Box::new(
                    person.clone(),
                ))),
                detail: NotificationDetail::Person {
                    person: Box::new(person),
                    reason_key: Some("prerequisites_met".into()),
                    reason: None,
                    opportunities,
                },
                active: false,
            },
        )?;
    }
    Ok(())
}

fn requirements_satisfied(requirements: &[crate::engine::progression::CareerRequirement]) -> bool {
    let paths = requirements
        .iter()
        .map(|requirement| requirement.path)
        .collect::<std::collections::BTreeSet<_>>();
    paths.into_iter().any(|path| {
        requirements
            .iter()
            .filter(|requirement| requirement.path == path)
            .all(|requirement| requirement.current >= requirement.required)
    })
}
