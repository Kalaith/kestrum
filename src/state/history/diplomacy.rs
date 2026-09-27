//! Public political records retain no private enemy roster.
use super::*;
use crate::state::{diplomacy::DiplomacyReceipt, StrategicCampaign};
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
