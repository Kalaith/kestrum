//! Own-people notices derive only from newly accepted life records and eligibility.

use super::*;
use crate::state::{
    history::{AnniversarySubject, HistoryKind, LifeEvent},
    legacy::{LegacyItemCustody, LegacyItemId},
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn collect(
    before: &StrategicCampaign,
    candidate: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
) -> Result<(), String> {
    let life_records = outcome
        .life_events
        .iter()
        .filter_map(|id| candidate.history.events.get(id).cloned())
        .collect::<Vec<_>>();
    let emerged_people = life_records
        .iter()
        .filter_map(|record| match &record.kind {
            HistoryKind::Life {
                person,
                event: LifeEvent::Emerged { .. },
                ..
            } => Some(*person),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let recognitions = life_records
        .iter()
        .filter_map(|record| match &record.kind {
            HistoryKind::Life {
                person,
                event: LifeEvent::Recognized { epithet, cause },
                ..
            } => Some((*person, (epithet.clone(), *cause))),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    for record in &life_records {
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
        if matches!(event, LifeEvent::Recognized { .. }) && emerged_people.contains(person) {
            continue;
        }
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
            LifeEvent::Emerged { troop } => recognitions
                .get(person)
                .map(|(epithet, _)| (Some("emerged_and_recognized".into()), Some(epithet.clone())))
                .unwrap_or_else(|| (Some("emerged".into()), Some(troop_key(*troop).into()))),
            LifeEvent::Arrived { origin } => {
                (Some("arrived".into()), Some(origin_key(*origin).into()))
            }
            LifeEvent::ClassCompleted { class } => (
                Some("class_completed".into()),
                Some(class_key(*class).into()),
            ),
            LifeEvent::Recognized { epithet, cause } => (
                Some(format!("recognized_{cause:?}").to_lowercase()),
                Some(epithet.clone()),
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
    anniversaries(before, candidate, data, outcome)?;
    legacy_transfers(
        before,
        candidate,
        data,
        outcome,
        &outcome.legacy_items_changed,
    )?;
    Ok(())
}

fn anniversaries(
    before: &StrategicCampaign,
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
) -> Result<(), String> {
    let records = campaign
        .history
        .events
        .values()
        .filter(|record| record.id >= before.next_ids.history)
        .filter(|record| {
            record.visible_to.contains(&campaign.player)
                && matches!(
                    &record.kind,
                    HistoryKind::Anniversary { subject, .. }
                        if outcome.anniversary_reminders.contains(subject)
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    for record in records {
        let HistoryKind::Anniversary { subject, years } = record.kind else {
            continue;
        };
        let subject_snapshot = match &subject {
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
    before: &StrategicCampaign,
    campaign: &mut StrategicCampaign,
    data: &GameData,
    outcome: &ActionOutcome,
    changed: &[LegacyItemId],
) -> Result<(), String> {
    let changed = changed
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let records = campaign
        .history
        .events
        .values()
        .filter(|record| record.id >= before.next_ids.history)
        .filter(|record| {
            record.visible_to.contains(&campaign.player)
                && matches!(
                    &record.kind,
                    HistoryKind::ItemCustodyChanged { item, .. } if changed.contains(item)
                )
        })
        .cloned()
        .collect::<Vec<_>>();
    for record in records {
        let HistoryKind::ItemCustodyChanged {
            item: item_id,
            from,
            to,
        } = record.kind
        else {
            continue;
        };
        let Some(item) = campaign
            .legacy_items
            .get(&item_id)
            .filter(|item| item.faction == campaign.player)
        else {
            continue;
        };
        let details = BTreeMap::from([
            ("item".into(), item.name.clone()),
            ("from".into(), custody_label(before, campaign, data, from)),
            ("to".into(), custody_label(before, campaign, data, to)),
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

fn custody_label(
    before: &StrategicCampaign,
    campaign: &StrategicCampaign,
    data: &GameData,
    custody: LegacyItemCustody,
) -> String {
    match custody {
        LegacyItemCustody::Person(id) => super::person(campaign, id)
            .or_else(|| super::person(before, id))
            .map(|person| person.name)
            .unwrap_or_else(|| authored_unknown_subject(data)),
        LegacyItemCustody::SiteEstate(id) => super::place(campaign, id)
            .or_else(|| super::place(before, id))
            .map(|place| place.name)
            .unwrap_or_else(|| authored_unknown_subject(data)),
    }
}

fn authored_unknown_subject(data: &GameData) -> String {
    data.notifications
        .terms
        .get("ui_unknown_subject")
        .cloned()
        .unwrap_or_else(|| "Unknown subject".into())
}

fn troop_key(troop: crate::data::economy::TroopKind) -> &'static str {
    use crate::data::economy::TroopKind;
    match troop {
        TroopKind::Warriors => "troop_warriors",
        TroopKind::Spearmen => "troop_spearmen",
        TroopKind::Archers => "troop_archers",
        TroopKind::Riders => "troop_riders",
        TroopKind::Medics => "troop_medics",
        TroopKind::SiegeEngines => "troop_siege_engines",
    }
}

fn origin_key(origin: crate::state::relationships::FamilyOrigin) -> &'static str {
    use crate::state::relationships::FamilyOrigin;
    match origin {
        FamilyOrigin::Birth => "origin_birth",
        FamilyOrigin::AdoptedWard => "origin_adopted_ward",
        FamilyOrigin::LocalApprentice => "origin_local_apprentice",
    }
}

fn class_key(class: crate::data::world::PersonClass) -> &'static str {
    use crate::data::world::PersonClass;
    match class {
        PersonClass::Recruit => "class_recruit",
        PersonClass::Infantry => "class_infantry",
        PersonClass::Archer => "class_archer",
        PersonClass::Scout => "class_scout",
        PersonClass::Cavalry => "class_cavalry",
        PersonClass::Medic => "class_medic",
        PersonClass::Officer => "class_officer",
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
    let minimum_age = data.households.service_minimum_age_years;
    for person in candidate
        .people
        .values()
        .filter(|person| person.faction == candidate.player && person.is_alive())
    {
        let previous = before.people.get(&person.id);
        let age = person.age_years(candidate.completed_rounds);
        let entered_service_age = previous.is_some_and(|previous| {
            previous.age_years(before.completed_rounds) < minimum_age && age >= minimum_age
        });
        if previous == Some(person) && !entered_service_age {
            continue;
        }
        if age < minimum_age {
            continue;
        }
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
                if option.class == person.class || !requirements_satisfied(&option.requirements) {
                    return false;
                }
                let previously_satisfied = before_options
                    .iter()
                    .find(|old| old.class == option.class)
                    .is_some_and(|old| requirements_satisfied(&old.requirements));
                !previously_satisfied || entered_service_age
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
