//! Public political records retain no private enemy roster.
use super::*;
use crate::state::{diplomacy::DiplomacyReceipt, StrategicCampaign};

impl CampaignHistory {
    pub(crate) fn refresh_war_links(&mut self) {
        let wars = self
            .events
            .values()
            .filter_map(|record| match &record.kind {
                HistoryKind::Diplomacy {
                    receipt: DiplomacyReceipt::WarDeclared { factions },
                } => Some((record.id, *factions)),
                _ => None,
            })
            .collect::<Vec<_>>();
        let links = wars
            .iter()
            .map(|(id, factions)| {
                let previous = wars
                    .iter()
                    .filter(|(candidate, pair)| candidate < id && pair == factions)
                    .map(|(candidate, _)| *candidate)
                    .max();
                (*id, previous.into_iter().collect::<Vec<_>>())
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        for record in self.events.values_mut() {
            if matches!(
                record.kind,
                HistoryKind::Diplomacy {
                    receipt: DiplomacyReceipt::WarDeclared { .. }
                }
            ) {
                record.related_events = links.get(&record.id).cloned().unwrap_or_default();
            }
        }
    }
}

impl HistoryRecord {
    pub(crate) fn diplomacy(
        id: HistoryId,
        completed_rounds: u32,
        source_fact: FactId,
        receipt: &DiplomacyReceipt,
        campaign: &StrategicCampaign,
    ) -> Self {
        let (sites, armies, visible_to) = match receipt {
            DiplomacyReceipt::ArmyWithdrawn {
                faction,
                army,
                from,
                to,
            } => {
                let mut sites = vec![*from, *to];
                sites.sort();
                sites.dedup();
                let armies = campaign
                    .armies
                    .get(army)
                    .map(|army| EntityLabel {
                        id: army.id,
                        name: army.name.clone(),
                    })
                    .into_iter()
                    .collect();
                (sites, armies, BTreeSet::from([*faction]))
            }
            DiplomacyReceipt::PeaceOffered {
                proposer,
                recipient,
            }
            | DiplomacyReceipt::PeaceRejected {
                proposer,
                recipient,
            } => (
                Vec::new(),
                Vec::new(),
                BTreeSet::from([*proposer, *recipient]),
            ),
            _ => (
                Vec::new(),
                Vec::new(),
                campaign.factions.keys().copied().collect(),
            ),
        };
        let related_events = match receipt {
            DiplomacyReceipt::WarDeclared { factions } => campaign
                .history
                .events
                .values()
                .filter(|record| {
                    record.id < id
                        && matches!(
                            &record.kind,
                            HistoryKind::Diplomacy {
                                receipt: DiplomacyReceipt::WarDeclared { factions: previous }
                            } if previous == factions
                        )
                })
                .max_by_key(|record| (record.completed_rounds, record.id))
                .map(|record| vec![record.id])
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        Self {
            id,
            completed_rounds,
            source_fact: Some(source_fact),
            kind: HistoryKind::Diplomacy {
                receipt: receipt.clone(),
            },
            sites: sites
                .into_iter()
                .filter_map(|id| {
                    campaign.world.site(id).map(|site| EntityLabel {
                        id,
                        name: site.name.clone(),
                    })
                })
                .collect(),
            armies,
            people: Vec::new(),
            formations: Vec::new(),
            items: Vec::new(),
            related_events,
            visible_to,
        }
    }
}
impl StrategicCampaign {
    pub(crate) fn validate_diplomacy_record(
        &self,
        record: &HistoryRecord,
        receipt: &DiplomacyReceipt,
    ) -> Result<(), String> {
        let expected = HistoryRecord::diplomacy(
            record.id,
            record.completed_rounds,
            record
                .source_fact
                .ok_or("diplomacy history: source missing")?,
            receipt,
            self,
        );
        let valid = record.people.is_empty()
            && record.formations.is_empty()
            && record.items.is_empty()
            && record.related_events == expected.related_events
            && record.visible_to == expected.visible_to
            && record
                .sites
                .iter()
                .map(|site| site.id)
                .eq(expected.sites.iter().map(|site| site.id))
            && match receipt {
                DiplomacyReceipt::ArmyWithdrawn { army, .. } => {
                    record.armies.len() == 1 && record.armies[0].id == *army
                }
                _ => record.armies.is_empty(),
            };
        if valid {
            Ok(())
        } else {
            Err("diplomacy history: inconsistent participants or observers".into())
        }
    }
}
