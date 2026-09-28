//! Historical participants may expire; owner visibility and stable identities may not.
use super::*;
use crate::data::world::{FactionId, PersonClass};
use crate::state::people::PersonId;

impl StrategicCampaign {
    pub(super) fn validate_life_kind(
        &self,
        owner: FactionId,
        person: PersonId,
        event: &LifeEvent,
    ) -> Result<(), String> {
        ensure(
            self.factions.contains_key(&owner) && person.0 > 0 && person < self.next_ids.person,
            "invalid life-event owner or person",
        )?;
        ensure(
            self.people
                .get(&person)
                .is_none_or(|entry| entry.faction == owner),
            "foreign life-event person",
        )?;
        match event {
            LifeEvent::Recognized { epithet, .. } => {
                ensure(valid_label(epithet), "invalid recognition label")
            }
            LifeEvent::ClassCompleted { class } => {
                ensure(*class != PersonClass::Recruit, "invalid completed class")
            }
            LifeEvent::MentorshipStarted { mentor, .. }
            | LifeEvent::MentorshipCompleted { mentor, .. } => ensure(
                mentor.0 > 0
                    && *mentor < self.next_ids.person
                    && *mentor != person
                    && self
                        .people
                        .get(mentor)
                        .is_none_or(|entry| entry.faction == owner),
                "invalid historical teacher",
            ),
            LifeEvent::HouseholdFormed { household }
            | LifeEvent::HouseholdEnded { household, .. } => ensure(
                household.0 > 0 && *household < self.next_ids.household,
                "invalid historical household",
            ),
            _ => Ok(()),
        }
    }

    pub(super) fn validate_life_record(
        &self,
        record: &HistoryRecord,
        owner: FactionId,
        person: PersonId,
        event: &LifeEvent,
    ) -> Result<(), String> {
        ensure(
            record.source_fact.is_none()
                && record.visible_to == BTreeSet::from([owner])
                && record.people.iter().any(|entry| entry.id == person)
                && record.people.iter().all(|entry| {
                    self.people
                        .get(&entry.id)
                        .is_none_or(|entry| entry.faction == owner)
                })
                && record.armies.iter().all(|entry| {
                    self.armies
                        .get(&entry.id)
                        .is_none_or(|entry| entry.faction == owner)
                }),
            "life narrative needs its own private participants",
        )?;
        if let LifeEvent::MentorshipStarted { mentor, .. }
        | LifeEvent::MentorshipCompleted { mentor, .. } = event
        {
            ensure(
                record.people.iter().any(|entry| entry.id == *mentor),
                "life narrative has no teacher label",
            )?;
        }
        Ok(())
    }
}
