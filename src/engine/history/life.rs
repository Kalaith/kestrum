//! Compare accepted transitions once; projection and loading never synthesize past lives.
use super::records::{allocate, insert};
use crate::{
    engine::{person_site, RuleError},
    state::{
        history::{EntityLabel, HistoryId, HistoryKind, HistoryRecord, LifeEvent},
        people::{Person, PersonAssignment, PersonCombatOutcome, PersonId, PersonStatus},
        relationships::HouseholdStatus,
        StrategicCampaign,
    },
};
use std::collections::BTreeSet;

pub(crate) fn record_life_changes(
    campaign: &mut StrategicCampaign,
    before: &StrategicCampaign,
) -> Result<Vec<HistoryId>, RuleError> {
    let mut changes = Vec::new();
    for person in campaign.people.values() {
        let previous = before.people.get(&person.id);
        for event in transitions(campaign, before, person, previous) {
            changes.push((person.id, event));
        }
    }
    for household in campaign.households.values() {
        let event = match before.households.get(&household.id) {
            None => Some(LifeEvent::HouseholdFormed {
                household: household.id,
            }),
            Some(previous) if previous.status == HouseholdStatus::Active => {
                if let HouseholdStatus::Ended { reason, .. } = household.status {
                    Some(LifeEvent::HouseholdEnded {
                        household: household.id,
                        reason,
                    })
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(event) = event {
            changes.push((household.partners[0], event));
        }
    }
    changes
        .into_iter()
        .map(|(person, event)| record(campaign, before, person, event))
        .collect()
}

fn transitions(
    campaign: &StrategicCampaign,
    before: &StrategicCampaign,
    person: &Person,
    previous: Option<&Person>,
) -> Vec<LifeEvent> {
    let mut events = Vec::new();
    if previous.is_none() {
        if let Some(emergence) = &person.career.emergence {
            events.push(LifeEvent::Emerged {
                troop: emergence.source_troop,
            });
        } else if let Some(family) = campaign.families.get(&person.id) {
            events.push(LifeEvent::Arrived {
                origin: family.origin,
            });
        }
    }
    if previous.is_none_or(|old| old.career.recognition.is_none()) {
        if let Some(recognition) = &person.career.recognition {
            events.push(LifeEvent::Recognized {
                epithet: recognition.epithet.clone(),
                cause: recognition.cause,
            });
        }
    }
    if let Some(old) = previous {
        if old.class != person.class {
            events.push(LifeEvent::ClassCompleted {
                class: person.class,
            });
        }
        if !old.career.retired && person.career.retired {
            events.push(LifeEvent::Retired);
        }
        if old.is_alive() && !person.is_alive() && !battle_death(campaign, before, person.id) {
            events.push(LifeEvent::NaturalDeath);
        }
        if matches!(
            old.assignment,
            PersonAssignment::Dependent { .. } | PersonAssignment::Trainee { .. }
        ) && matches!(
            person.assignment,
            PersonAssignment::Site { .. } | PersonAssignment::Formation { .. }
        ) {
            events.push(LifeEvent::ServiceEntered);
        }
        for completion in person
            .career
            .completed_mentors
            .difference(&old.career.completed_mentors)
        {
            events.push(LifeEvent::MentorshipCompleted {
                mentor: completion.mentor,
                discipline: completion.discipline,
            });
        }
    }
    if let Some(link) = campaign.mentorships.get(&person.id) {
        if before.mentorships.get(&person.id).is_none_or(|old| {
            (old.mentor, old.discipline, old.started_round)
                != (link.mentor, link.discipline, link.started_round)
        }) {
            events.push(LifeEvent::MentorshipStarted {
                mentor: link.mentor,
                discipline: link.discipline,
            });
        }
    }
    events
}

fn battle_death(
    campaign: &StrategicCampaign,
    before: &StrategicCampaign,
    person: PersonId,
) -> bool {
    campaign
        .battles
        .range(before.next_ids.battle..)
        .any(|(_, battle)| {
            battle.person_events.iter().any(|event| {
                event.person == person && matches!(event.outcome, PersonCombatOutcome::Died { .. })
            })
        })
}

fn participants(
    campaign: &StrategicCampaign,
    person: PersonId,
    event: &LifeEvent,
) -> BTreeSet<PersonId> {
    let mut ids = BTreeSet::from([person]);
    match event {
        LifeEvent::MentorshipStarted { mentor, .. }
        | LifeEvent::MentorshipCompleted { mentor, .. } => {
            ids.insert(*mentor);
        }
        LifeEvent::HouseholdFormed { household } | LifeEvent::HouseholdEnded { household, .. } => {
            ids.extend(campaign.households[household].partners);
        }
        LifeEvent::Arrived { .. } => {
            if let Some(family) = campaign.families.get(&person) {
                ids.extend(family.links.keys().copied());
            }
        }
        _ => {}
    }
    ids
}

fn record(
    campaign: &mut StrategicCampaign,
    before: &StrategicCampaign,
    person: PersonId,
    event: LifeEvent,
) -> Result<HistoryId, RuleError> {
    let id = allocate(campaign)?;
    let entry = &campaign.people[&person];
    let people = participants(campaign, person, &event);
    let site = match &event {
        LifeEvent::Recognized { .. } => entry.career.recognition.as_ref().map(|record| record.site),
        LifeEvent::Emerged { .. } => entry.career.emergence.as_ref().map(|record| record.site),
        LifeEvent::HouseholdFormed { household } | LifeEvent::HouseholdEnded { household, .. } => {
            Some(campaign.households[household].home)
        }
        LifeEvent::NaturalDeath => {
            if let PersonStatus::Dead { site, .. } = entry.status {
                Some(site)
            } else {
                None
            }
        }
        _ => person_site(campaign, person).or_else(|| person_site(before, person)),
    }
    .ok_or_else(|| RuleError::InvalidState("A life transition has no physical place.".into()))?;
    let armies = related_armies(campaign, before, &people);
    let record = HistoryRecord {
        id,
        completed_rounds: campaign.completed_rounds,
        source_fact: None,
        kind: HistoryKind::Life {
            owner: entry.faction,
            person,
            event,
        },
        sites: vec![EntityLabel {
            id: site,
            name: campaign.world.site(site).expect("life place").name.clone(),
        }],
        armies,
        people: people
            .into_iter()
            .filter_map(|id| campaign.people.get(&id).or_else(|| before.people.get(&id)))
            .map(|entry| EntityLabel {
                id: entry.id,
                name: entry.name.clone(),
            })
            .collect(),
        formations: Vec::new(),
        items: Vec::new(),
        related_events: Vec::new(),
        visible_to: BTreeSet::from([entry.faction]),
    };
    insert(campaign, record);
    Ok(id)
}

fn related_armies(
    campaign: &StrategicCampaign,
    before: &StrategicCampaign,
    people: &BTreeSet<PersonId>,
) -> Vec<EntityLabel<crate::state::military::ArmyId>> {
    let mut armies = std::collections::BTreeMap::new();
    for snapshot in [before, campaign] {
        for person in people.iter().filter_map(|id| snapshot.people.get(id)) {
            if let PersonAssignment::Formation { formation } = person.assignment {
                for army in snapshot
                    .armies
                    .values()
                    .filter(|army| army.formation_ids().any(|id| id == formation))
                {
                    armies.insert(
                        army.id,
                        EntityLabel {
                            id: army.id,
                            name: army.name.clone(),
                        },
                    );
                }
            }
        }
    }
    armies.into_values().collect()
}
