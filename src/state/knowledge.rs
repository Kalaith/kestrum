//! Dated witnessed identities; these labels never point into an enemy's live roster.

mod validation;

use super::{
    battle::{BattleId, BattleReport, BattleSideReport},
    military::ArmyId,
    people::{PersonId, PersonStatus},
};
use crate::data::world::{FactionId, FounderClass, SiteId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignKnowledge {
    #[serde(default)]
    pub contacts: BTreeMap<FactionId, std::collections::BTreeSet<FactionId>>,
    #[serde(default)]
    pub explored: BTreeMap<FactionId, std::collections::BTreeSet<SiteId>>,
    pub observers: BTreeMap<FactionId, ObserverKnowledge>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObserverKnowledge {
    pub people: BTreeMap<PersonId, EncounteredPerson>,
}

/// Only externally visible condition is learned, not injury dates or healing progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservedCondition {
    Fit,
    Wounded,
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EncounteredPerson {
    pub appearance: crate::data::portraits::AppearanceDescriptor,
    pub id: PersonId,
    pub name: String,
    pub class: FounderClass,
    pub completed_rounds: u32,
    pub site: SiteId,
    pub site_name: String,
    /// The army at the encounter, not an assertion about a current appointment.
    pub army: ArmyId,
    pub army_name: String,
    pub condition: ObservedCondition,
    /// A weak detail link. The dated identity remains valid after the report expires.
    pub battle: BattleId,
}

pub(crate) fn encounter_people(
    report: &BattleReport,
    side: &BattleSideReport,
) -> Vec<EncounteredPerson> {
    side.armies
        .iter()
        .flat_map(|army| {
            army.people.iter().filter_map(|person| {
                Some(EncounteredPerson {
                    appearance: person.appearance.clone(),
                    id: person.id,
                    name: person.name.clone(),
                    class: person.class,
                    completed_rounds: report.completed_rounds,
                    site: report.site,
                    site_name: report.site_name.clone(),
                    army: army.id,
                    army_name: army.name.clone(),
                    condition: match person.status {
                        PersonStatus::Fit => ObservedCondition::Fit,
                        PersonStatus::Wounded { .. } => ObservedCondition::Wounded,
                        PersonStatus::Dead { .. } => ObservedCondition::Dead,
                        PersonStatus::Displaced { .. } => return None,
                    },
                    battle: report.id,
                })
            })
        })
        .collect()
}
