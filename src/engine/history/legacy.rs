//! One immutable record per custody change and a bounded, factual reminder stream.

use super::records::{allocate, insert};
use crate::{
    data::world::FactionId,
    engine::RuleError,
    state::{
        history::{AnniversarySubject, EntityLabel, HistoryKind, HistoryRecord},
        legacy::{person_site, LegacyItemCustody},
        people::PersonId,
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn record_legacy_custody_changes(
    campaign: &mut StrategicCampaign,
    before: &StrategicCampaign,
) -> Result<Vec<crate::state::legacy::LegacyItemId>, RuleError> {
    let changes = campaign
        .legacy_items
        .values()
        .filter_map(|item| {
            let old = before.legacy_items.get(&item.id)?;
            (old.custody != item.custody).then_some((old.clone(), item.clone()))
        })
        .collect::<Vec<_>>();
    let mut recorded = Vec::new();
    for (old, item) in changes {
        let id = allocate(campaign)?;
        let site = custody_site(campaign, item.custody)
            .or_else(|| custody_site(before, old.custody))
            .ok_or_else(|| {
                RuleError::InvalidState("An heirloom has no physical location.".into())
            })?;
        let mut people = BTreeSet::new();
        collect_person(old.custody, &mut people);
        collect_person(item.custody, &mut people);
        let labels = people
            .into_iter()
            .map(|person| {
                let entry = campaign
                    .people
                    .get(&person)
                    .or_else(|| before.people.get(&person))
                    .ok_or_else(|| {
                        RuleError::InvalidState("An heirloom deed lost a person label.".into())
                    })?;
                Ok(EntityLabel {
                    id: person,
                    name: entry.name.clone(),
                })
            })
            .collect::<Result<Vec<_>, RuleError>>()?;
        let place = campaign.world.site(site).ok_or_else(|| {
            RuleError::InvalidState("An heirloom deed references an unknown site.".into())
        })?;
        insert(
            campaign,
            HistoryRecord {
                id,
                completed_rounds: campaign.completed_rounds,
                source_fact: None,
                kind: HistoryKind::ItemCustodyChanged {
                    item: item.id,
                    from: old.custody,
                    to: item.custody,
                },
                sites: vec![EntityLabel {
                    id: site,
                    name: place.name.clone(),
                }],
                armies: Vec::new(),
                people: labels,
                formations: Vec::new(),
                items: vec![EntityLabel {
                    id: item.id,
                    name: item.name,
                }],
                related_events: Vec::new(),
                visible_to: [item.faction].into_iter().collect(),
            },
        );
        recorded.push(item.id);
    }
    Ok(recorded)
}

pub(crate) fn record_anniversaries(
    campaign: &mut StrategicCampaign,
) -> Result<Vec<AnniversarySubject>, RuleError> {
    let mut due: BTreeMap<
        FactionId,
        Vec<(AnniversarySubject, u32, Option<crate::data::world::SiteId>)>,
    > = BTreeMap::new();
    for person in campaign.people.values().filter(|person| {
        person.is_alive()
            && matches!(
                person.assignment,
                crate::state::people::PersonAssignment::Formation { .. }
                    | crate::state::people::PersonAssignment::Site { .. }
            )
    }) {
        let years = campaign
            .completed_rounds
            .saturating_sub(person.service_start_round)
            / 4;
        let milestone = (years / 10) * 10;
        let subject = AnniversarySubject::Person(person.id);
        if milestone > 0
            && campaign
                .history
                .person_last_reminded
                .get(&person.id)
                .is_none_or(|last| *last < milestone)
            && person_site(campaign, person.id).is_some()
        {
            due.entry(person.faction).or_default().push((
                subject,
                milestone,
                person_site(campaign, person.id),
            ));
        }
    }
    for (site, founded) in &campaign.world.founded_rounds {
        let Some(faction) = campaign
            .world
            .site(*site)
            .and_then(|place| place.controller)
        else {
            continue;
        };
        let years = campaign.completed_rounds.saturating_sub(*founded) / 4;
        let milestone = (years / 10) * 10;
        let subject = AnniversarySubject::Site(*site);
        if milestone > 0
            && campaign
                .history
                .site_last_reminded
                .get(site)
                .is_none_or(|last| *last < milestone)
        {
            due.entry(faction)
                .or_default()
                .push((subject, milestone, Some(*site)));
        }
    }
    let mut recorded = Vec::new();
    for (faction, mut milestones) in due {
        milestones.sort_by_key(|(subject, _, _)| *subject);
        let Some((subject, years, site)) = milestones.into_iter().next() else {
            continue;
        };
        let id = allocate(campaign)?;
        let (people, sites) = match subject {
            AnniversarySubject::Person(person) => {
                let entry = &campaign.people[&person];
                let sites = site
                    .and_then(|id| {
                        campaign
                            .world
                            .site(id)
                            .map(|place| (id, place.name.clone()))
                    })
                    .map(|(id, name)| vec![EntityLabel { id, name }])
                    .unwrap_or_default();
                (
                    vec![EntityLabel {
                        id: person,
                        name: entry.name.clone(),
                    }],
                    sites,
                )
            }
            AnniversarySubject::Site(site) => {
                let place = &campaign
                    .world
                    .sites
                    .iter()
                    .find(|place| place.id == site)
                    .expect("validated foundation site");
                (
                    Vec::new(),
                    vec![EntityLabel {
                        id: site,
                        name: place.name.clone(),
                    }],
                )
            }
        };
        insert(
            campaign,
            HistoryRecord {
                id,
                completed_rounds: campaign.completed_rounds,
                source_fact: None,
                kind: HistoryKind::Anniversary { subject, years },
                sites,
                armies: Vec::new(),
                people,
                formations: Vec::new(),
                items: Vec::new(),
                related_events: Vec::new(),
                visible_to: [faction].into_iter().collect(),
            },
        );
        match subject {
            AnniversarySubject::Person(person) => {
                campaign.history.person_last_reminded.insert(person, years);
            }
            AnniversarySubject::Site(site) => {
                campaign.history.site_last_reminded.insert(site, years);
            }
        }
        recorded.push(subject);
    }
    Ok(recorded)
}

fn custody_site(
    campaign: &StrategicCampaign,
    custody: LegacyItemCustody,
) -> Option<crate::data::world::SiteId> {
    match custody {
        LegacyItemCustody::Person(id) => person_site(campaign, id),
        LegacyItemCustody::SiteEstate(site) => campaign.world.site(site).map(|place| place.id),
    }
}

fn collect_person(custody: LegacyItemCustody, people: &mut BTreeSet<PersonId>) {
    if let LegacyItemCustody::Person(person) = custody {
        people.insert(person);
    }
}

pub(crate) fn current_era(campaign: &StrategicCampaign) -> String {
    let active = campaign
        .relations
        .iter()
        .filter(|relation| {
            relation.state == crate::data::world::DiplomaticState::War
                && relation.factions.iter().all(|id| {
                    campaign.factions.get(id).is_some_and(|faction| {
                        faction.status == crate::state::FactionStatus::Independent
                    })
                })
        })
        .collect::<Vec<_>>();
    if !active.is_empty() {
        let labels = active
            .iter()
            .map(|relation| {
                let names = relation
                    .factions
                    .iter()
                    .filter_map(|id| {
                        campaign
                            .factions
                            .get(id)
                            .map(|faction| faction.name.as_str())
                    })
                    .collect::<Vec<_>>();
                let date = campaign
                    .diplomacy
                    .pair(relation.factions[0], relation.factions[1])
                    .and_then(|pair| pair.war_started_round)
                    .map(|round| format!(" · season {}", round + 1))
                    .unwrap_or_else(|| " · start not recorded".into());
                format!(
                    "{} ↔ {}{date}",
                    names.first().copied().unwrap_or("?"),
                    names.get(1).copied().unwrap_or("?")
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        return format!("War · {labels}");
    }
    let last_ended = campaign
        .diplomacy
        .pairs
        .iter()
        .filter_map(|pair| pair.war_ended_round.map(|round| (round, pair)))
        .max_by_key(|(round, pair)| (*round, pair.factions));
    if let Some((ended, pair)) = last_ended {
        if pair.factions.iter().any(|id| {
            campaign
                .factions
                .get(id)
                .is_some_and(|faction| faction.status != crate::state::FactionStatus::Independent)
        }) {
            return format!("After the fall · season {}", ended + 1);
        }
        if campaign.completed_rounds.saturating_sub(ended) < 4 {
            return format!("Armistice · season {}", ended + 1);
        }
        return format!("Peace · since season {}", ended + 1);
    }
    if campaign.diplomacy.era_history_complete {
        "Founding".into()
    } else {
        "Earlier era history not recorded".into()
    }
}
