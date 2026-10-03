//! One observation boundary for player interfaces and future strategic AI.

use crate::{
    data::{
        progression::HistoryRules,
        world::{FactionId, SiteId},
        GameData,
    },
    state::{
        battle::{BattleId, BattleReport},
        knowledge::{encounter_people, CampaignKnowledge, EncounteredPerson},
        people::{Person, PersonId},
        StrategicCampaign,
    },
};
use std::collections::{BTreeMap, BTreeSet};

/// History and known-person queries share the documented 50-record page boundary.
pub const KNOWLEDGE_PAGE_SIZE: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersonKnowledge {
    CurrentOwn(Box<Person>),
    LastEncountered {
        snapshot: Box<EncounteredPerson>,
        available_report: Option<BattleId>,
    },
}

impl PersonKnowledge {
    pub fn id(&self) -> PersonId {
        match self {
            Self::CurrentOwn(person) => person.id,
            Self::LastEncountered { snapshot, .. } => snapshot.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::CurrentOwn(person) => &person.name,
            Self::LastEncountered { snapshot, .. } => &snapshot.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPeoplePage {
    pub people: Vec<PersonKnowledge>,
    pub total: usize,
    pub page: usize,
}

/// Only presence is returned: no army identity, faction, count or composition.
pub fn hostile_presence(campaign: &StrategicCampaign, observer: FactionId) -> BTreeSet<SiteId> {
    let mut range = BTreeSet::new();
    for army in campaign
        .armies
        .values()
        .filter(|army| army.faction == observer)
    {
        range.insert(army.site);
        range.extend(campaign.world.adjacent_sites(army.site));
    }
    campaign
        .armies
        .values()
        .filter(|army| {
            range.contains(&army.site) && super::retreat::hostile(campaign, observer, army.faction)
        })
        .map(|army| army.site)
        .collect()
}

pub fn person_knowledge(
    campaign: &StrategicCampaign,
    observer: FactionId,
    id: PersonId,
) -> Option<PersonKnowledge> {
    if !campaign.factions.contains_key(&observer) {
        return None;
    }
    if let Some(person) = campaign
        .people
        .get(&id)
        .filter(|person| person.faction == observer)
    {
        return Some(PersonKnowledge::CurrentOwn(Box::new(person.clone())));
    }
    let snapshot = campaign
        .knowledge
        .observers
        .get(&observer)?
        .people
        .get(&id)?
        .clone();
    let available_report = campaign
        .battles
        .get(&snapshot.battle)
        .filter(|report| {
            report
                .participant_factions()
                .any(|faction| faction == observer)
        })
        .map(|report| report.id);
    Some(PersonKnowledge::LastEncountered {
        snapshot: Box::new(snapshot),
        available_report,
    })
}

/// Stable identity order and a bounded result; an unknown query never searches hidden state.
pub fn known_people(
    campaign: &StrategicCampaign,
    observer: FactionId,
    search: &str,
    page: usize,
) -> KnownPeoplePage {
    let mut ids: BTreeSet<_> = campaign
        .people
        .values()
        .filter(|person| person.faction == observer)
        .map(|person| person.id)
        .collect();
    if let Some(knowledge) = campaign.knowledge.observers.get(&observer) {
        ids.extend(knowledge.people.keys().copied());
    }
    let search = search.trim().to_lowercase();
    let mut result = KnownPeoplePage {
        people: Vec::new(),
        total: 0,
        page,
    };
    let offset = page.saturating_mul(KNOWLEDGE_PAGE_SIZE);
    for person in ids
        .into_iter()
        .filter_map(|id| person_knowledge(campaign, observer, id))
    {
        if !person.name().to_lowercase().contains(&search) {
            continue;
        }
        if result.total >= offset && result.people.len() < KNOWLEDGE_PAGE_SIZE {
            result.people.push(person);
        }
        result.total += 1;
    }
    result
}

/// Called once inside the same atomic candidate that commits a real encounter.
pub(crate) fn observe_battle(knowledge: &mut CampaignKnowledge, report: &BattleReport) {
    let Some(defender) = report.defender.faction_side() else {
        return;
    };
    for (observer, enemy) in [
        (report.attacker.faction, defender),
        (defender.faction, &report.attacker),
    ] {
        let known = &mut knowledge.observers.entry(observer).or_default().people;
        for snapshot in encounter_people(report, enemy) {
            if known.get(&snapshot.id).is_none_or(|old| {
                (old.completed_rounds, old.battle) < (snapshot.completed_rounds, snapshot.battle)
            }) {
                known.insert(snapshot.id, snapshot);
            }
        }
    }
}

/// Older supported saves reveal only their retained, genuinely witnessed encounters.
pub(crate) fn restore_battle_knowledge(campaign: &mut StrategicCampaign) {
    for report in campaign.battles.values() {
        observe_battle(&mut campaign.knowledge, report);
    }
}

pub(crate) fn prune_knowledge(campaign: &mut StrategicCampaign, data: &GameData) {
    prune_observations(
        &mut campaign.knowledge,
        campaign.completed_rounds,
        &data.history,
    );
}

pub(crate) fn prune_observations(
    knowledge: &mut CampaignKnowledge,
    round: u32,
    rules: &HistoryRules,
) {
    let maximum_age = rules.knowledge_max_age_rounds;
    for observer in knowledge.observers.values_mut() {
        observer
            .people
            .retain(|_, person| round.saturating_sub(person.completed_rounds) <= maximum_age);
    }
    let mut order = BTreeMap::new();
    for (&observer, known) in &knowledge.observers {
        for person in known.people.values() {
            order.insert(
                (person.completed_rounds, person.battle, observer, person.id),
                (observer, person.id),
            );
        }
    }
    let excess = order.len().saturating_sub(rules.knowledge_max_entries);
    for (observer, person) in order.into_values().take(excess) {
        knowledge
            .observers
            .get_mut(&observer)
            .expect("recorded observer")
            .people
            .remove(&person);
    }
    knowledge
        .observers
        .retain(|_, known| !known.people.is_empty());
}
