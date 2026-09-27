//! Bounded, observer-labeled narratives; these records never authorize rewards.

mod development;
mod diplomacy;
mod validation;

use super::{
    battle::{BattleId, BattleOutcome, BattleReport},
    campaign::FactId,
    construction::{ConstructionOrder, Focus},
    evidence::Veterancy,
    legacy::{LegacyItemCustody, LegacyItemId},
    military::{ArmyId, FormationId},
    people::PersonId,
    siege::{Siege, SiegeChange, SiegeId},
};
use crate::data::{
    economy::TroopKind,
    world::{FactionId, SiteId},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HistoryId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum HistorySubject {
    Site(SiteId),
    Army(ArmyId),
    Person(PersonId),
    Formation(FormationId),
    Item(LegacyItemId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum AnniversarySubject {
    Person(PersonId),
    Site(SiteId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityLabel<I> {
    pub id: I,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormationLabel {
    pub id: FormationId,
    pub kind: TroopKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HistoryKind {
    Diplomacy {
        receipt: super::diplomacy::DiplomacyReceipt,
    },
    Development {
        receipt: super::development::DevelopmentReceipt,
    },
    Siege {
        siege: SiegeId,
        defender: FactionId,
        besieger: FactionId,
        change: SiegeChange,
        elapsed_steps: u32,
    },
    Battle {
        battle: BattleId,
        outcome: BattleOutcome,
    },
    Recruited {
        troop: TroopKind,
    },
    Disbanded {
        troop: TroopKind,
    },
    Moved,
    FormationTransferred,
    PersonTransferred,
    VeterancyEarned {
        formation: FormationId,
        tier: Veterancy,
        xp: u32,
    },
    Construction {
        order: ConstructionOrder,
    },
    FocusChanged {
        focus: Focus,
    },
    ItemCustodyChanged {
        item: LegacyItemId,
        from: LegacyItemCustody,
        to: LegacyItemCustody,
    },
    Anniversary {
        subject: AnniversarySubject,
        years: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryKindFilter {
    Diplomacy,
    Development,
    Siege,
    Battle,
    Recruitment,
    Disbanding,
    Movement,
    Transfer,
    Veterancy,
    Construction,
    Focus,
    Memory,
}

impl HistoryKind {
    pub fn category(&self) -> HistoryKindFilter {
        match self {
            Self::Diplomacy { .. } => HistoryKindFilter::Diplomacy,
            Self::Development { .. } => HistoryKindFilter::Development,
            Self::Siege { .. } => HistoryKindFilter::Siege,
            Self::Battle { .. } => HistoryKindFilter::Battle,
            Self::Recruited { .. } => HistoryKindFilter::Recruitment,
            Self::Disbanded { .. } => HistoryKindFilter::Disbanding,
            Self::Moved => HistoryKindFilter::Movement,
            Self::FormationTransferred
            | Self::PersonTransferred
            | Self::ItemCustodyChanged { .. } => HistoryKindFilter::Transfer,
            Self::VeterancyEarned { .. } => HistoryKindFilter::Veterancy,
            Self::Construction { .. } => HistoryKindFilter::Construction,
            Self::FocusChanged { .. } => HistoryKindFilter::Focus,
            Self::Anniversary { .. } => HistoryKindFilter::Memory,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRecord {
    pub id: HistoryId,
    pub completed_rounds: u32,
    pub source_fact: Option<FactId>,
    pub kind: HistoryKind,
    pub sites: Vec<EntityLabel<SiteId>>,
    pub armies: Vec<EntityLabel<ArmyId>>,
    pub people: Vec<EntityLabel<PersonId>>,
    pub formations: Vec<FormationLabel>,
    #[serde(default)]
    pub items: Vec<EntityLabel<LegacyItemId>>,
    #[serde(default)]
    pub related_events: Vec<HistoryId>,
    pub visible_to: BTreeSet<FactionId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotableSummary {
    pub id: HistoryId,
    pub completed_rounds: u32,
    pub kind: HistoryKind,
    pub site: Option<EntityLabel<SiteId>>,
    pub visible_to: BTreeSet<FactionId>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignHistory {
    pub events: BTreeMap<HistoryId, HistoryRecord>,
    pub site_notables: BTreeMap<SiteId, Vec<NotableSummary>>,
    pub army_notables: BTreeMap<ArmyId, Vec<NotableSummary>>,
    pub person_notables: BTreeMap<PersonId, Vec<NotableSummary>>,
    /// Milestones already shown, independently bounded by living subject identity.
    #[serde(default)]
    pub person_last_reminded: BTreeMap<PersonId, u32>,
    #[serde(default)]
    pub site_last_reminded: BTreeMap<SiteId, u32>,
}

impl HistoryRecord {
    /// A siege's existence is known to its sides, but contact does not reveal
    /// the other side's roster. Keep those authoritative IDs out of narratives.
    pub(crate) fn siege(
        id: HistoryId,
        completed_rounds: u32,
        source_fact: FactId,
        siege: &Siege,
        change: SiegeChange,
        site_name: String,
    ) -> Self {
        Self {
            id,
            completed_rounds,
            source_fact: Some(source_fact),
            kind: HistoryKind::Siege {
                siege: siege.id,
                defender: siege.defender,
                besieger: siege.besieger,
                change,
                elapsed_steps: siege.elapsed_steps,
            },
            sites: vec![EntityLabel {
                id: siege.site,
                name: site_name,
            }],
            armies: Vec::new(),
            people: Vec::new(),
            formations: Vec::new(),
            items: Vec::new(),
            related_events: Vec::new(),
            visible_to: [siege.defender, siege.besieger].into_iter().collect(),
        }
    }

    pub(crate) fn battle(
        id: HistoryId,
        report: &BattleReport,
        source_fact: Option<FactId>,
    ) -> Self {
        let sides: Vec<_> = std::iter::once(&report.attacker)
            .chain(report.defender.faction_side())
            .collect();
        let mut armies: Vec<_> = sides
            .iter()
            .flat_map(|side| side.armies.iter())
            .map(|army| EntityLabel {
                id: army.id,
                name: army.name.clone(),
            })
            .collect();
        armies.sort_by_key(|entry| entry.id);
        let mut people: Vec<_> = sides
            .iter()
            .flat_map(|side| side.armies.iter())
            .flat_map(|army| army.people.iter())
            .map(|person| EntityLabel {
                id: person.id,
                name: person.name.clone(),
            })
            .collect();
        people.sort_by_key(|entry| entry.id);
        let mut formations: Vec<_> = sides
            .iter()
            .flat_map(|side| side.armies.iter())
            .flat_map(|army| army.formations.iter())
            .map(|formation| FormationLabel {
                id: formation.id,
                kind: formation.kind,
            })
            .collect();
        formations.sort_by_key(|entry| entry.id);
        Self {
            id,
            completed_rounds: report.completed_rounds,
            source_fact,
            kind: HistoryKind::Battle {
                battle: report.id,
                outcome: report.outcome,
            },
            sites: vec![EntityLabel {
                id: report.site,
                name: report.site_name.clone(),
            }],
            armies,
            people,
            formations,
            items: Vec::new(),
            related_events: Vec::new(),
            visible_to: std::iter::once(report.attacker.faction)
                .chain(report.defender.faction())
                .collect(),
        }
    }
    pub fn concerns(&self, subject: HistorySubject) -> bool {
        match subject {
            HistorySubject::Site(id) => self.sites.iter().any(|entry| entry.id == id),
            HistorySubject::Army(id) => self.armies.iter().any(|entry| entry.id == id),
            HistorySubject::Person(id) => self.people.iter().any(|entry| entry.id == id),
            HistorySubject::Formation(id) => self.formations.iter().any(|entry| entry.id == id),
            HistorySubject::Item(id) => self.items.iter().any(|entry| entry.id == id),
        }
    }
    pub(crate) fn notable(&self) -> NotableSummary {
        NotableSummary {
            id: self.id,
            completed_rounds: self.completed_rounds,
            kind: self.kind.clone(),
            site: self.sites.first().cloned(),
            visible_to: self.visible_to.clone(),
        }
    }
}
