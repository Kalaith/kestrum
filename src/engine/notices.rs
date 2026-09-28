//! Application feedback at the observer boundary; private lives stay with their faction.

use super::ActionOutcome;
use crate::{
    data::{world::FactionId, GameData},
    state::{history::AnniversarySubject, people::PersonId, StrategicCampaign},
};

/// Called only for an accepted action, never while projecting or loading a save.
pub fn action_notices(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    outcome: &ActionOutcome,
) -> Vec<String> {
    let mut messages = Vec::new();
    for (ids, key) in [
        (
            &outcome.automatic_retirements,
            "automatic_retirement_notice",
        ),
        (&outcome.new_people, "new_people_notice"),
    ] {
        let names = own_names(campaign, observer, ids);
        if !names.is_empty() {
            messages.push(data.presentation.text(key).replace("{names}", &names));
        }
    }
    messages.extend(life_notices(campaign, data, observer, outcome));
    messages.extend(succession_notices(campaign, data, observer, outcome));
    let items = outcome
        .legacy_items_changed
        .iter()
        .filter_map(|id| campaign.legacy_items.get(id))
        .filter(|item| item.faction == observer)
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    if !items.is_empty() {
        messages.push(
            data.presentation
                .text("legacy_item_transfer_notice")
                .replace("{names}", &items),
        );
    }
    messages.extend(anniversary_notices(campaign, data, observer, outcome));
    messages
}

fn own_names(campaign: &StrategicCampaign, observer: FactionId, ids: &[PersonId]) -> String {
    ids.iter()
        .filter_map(|id| campaign.people.get(id))
        .filter(|person| person.faction == observer)
        .map(|person| person.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn succession_notices(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    outcome: &ActionOutcome,
) -> Vec<String> {
    outcome
        .succession
        .iter()
        .filter_map(|notice| {
            let army = campaign
                .armies
                .get(&notice.army)
                .filter(|army| army.faction == observer)?;
            let predecessor = campaign
                .people
                .get(&notice.predecessor)
                .filter(|person| person.faction == observer)?;
            let successor = notice
                .successor
                .and_then(|id| campaign.people.get(&id))
                .filter(|person| person.faction == observer);
            let key = if successor.is_some() {
                "succession_notice"
            } else {
                "vacant_succession_notice"
            };
            Some(
                data.presentation
                    .text(key)
                    .replace("{predecessor}", &predecessor.name)
                    .replace(
                        "{successor}",
                        successor.map_or("", |person| person.name.as_str()),
                    )
                    .replace("{army}", &army.name),
            )
        })
        .collect()
}

fn anniversary_notices(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    outcome: &ActionOutcome,
) -> Vec<String> {
    outcome
        .anniversary_reminders
        .iter()
        .filter_map(|subject| {
            let (name, years, key) = match subject {
                AnniversarySubject::Person(id) => {
                    let person = campaign
                        .people
                        .get(id)
                        .filter(|person| person.faction == observer)?;
                    (
                        person.name.as_str(),
                        campaign.history.person_last_reminded.get(id)?,
                        "history_service_anniversary",
                    )
                }
                AnniversarySubject::Site(id) => {
                    let site = campaign
                        .world
                        .site(*id)
                        .filter(|site| site.controller == Some(observer))?;
                    (
                        site.name.as_str(),
                        campaign.history.site_last_reminded.get(id)?,
                        "history_foundation_anniversary",
                    )
                }
            };
            Some(
                data.presentation
                    .text("history_anniversary_notice")
                    .replace("{kind}", data.presentation.text(key))
                    .replace("{name}", name)
                    .replace("{years}", &years.to_string()),
            )
        })
        .collect()
}

fn life_notices(
    campaign: &StrategicCampaign,
    data: &GameData,
    observer: FactionId,
    outcome: &ActionOutcome,
) -> Vec<String> {
    use crate::state::history::{HistoryKind, LifeEvent};
    outcome
        .life_events
        .iter()
        .filter_map(|id| {
            let record = campaign.history.events.get(id)?;
            let HistoryKind::Life {
                owner,
                person,
                event,
            } = &record.kind
            else {
                return None;
            };
            if *owner != observer
                || !record.visible_to.contains(&observer)
                || !matches!(
                    event,
                    LifeEvent::Emerged { .. }
                        | LifeEvent::Recognized { .. }
                        | LifeEvent::ClassCompleted { .. }
                )
            {
                return None;
            }
            let name = record
                .people
                .iter()
                .find(|entry| entry.id == *person)?
                .name
                .as_str();
            Some(format!(
                "{name}: {}",
                data.presentation.life_event_text(event)
            ))
        })
        .collect()
}
