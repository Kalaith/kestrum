//! Public place changes and private population transfers keep the same dated facts.

use super::*;
use crate::state::{development::DevelopmentReceipt, StrategicCampaign};

fn sites(receipt: &DevelopmentReceipt) -> BTreeSet<SiteId> {
    use DevelopmentReceipt::*;
    match receipt {
        HabitationChanged { site, .. } | Ruined { site, .. } | SiteRenamed { site, .. } => {
            [*site].into()
        }
        PopulationMoved { from, to, .. }
        | CapitalMoved { from, to, .. }
        | HeadquartersMoved { from, to, .. } => [*from, *to].into(),
    }
}

fn audience(receipt: &DevelopmentReceipt, campaign: &StrategicCampaign) -> BTreeSet<FactionId> {
    match receipt {
        DevelopmentReceipt::PopulationMoved { owner, .. } => [*owner].into(),
        _ => campaign.factions.keys().copied().collect(),
    }
}

impl HistoryRecord {
    pub(crate) fn development(
        id: HistoryId,
        completed_rounds: u32,
        fact: FactId,
        receipt: &DevelopmentReceipt,
        campaign: &StrategicCampaign,
    ) -> Self {
        Self {
            id,
            completed_rounds,
            source_fact: Some(fact),
            kind: HistoryKind::Development {
                receipt: receipt.clone(),
            },
            sites: sites(receipt)
                .into_iter()
                .map(|id| EntityLabel {
                    id,
                    name: campaign
                        .world
                        .site(id)
                        .expect("development place")
                        .name
                        .clone(),
                })
                .collect(),
            armies: Vec::new(),
            people: Vec::new(),
            formations: Vec::new(),
            items: Vec::new(),
            related_events: Vec::new(),
            visible_to: audience(receipt, campaign),
        }
    }
}

impl StrategicCampaign {
    pub(crate) fn validate_development_record(
        &self,
        record: &HistoryRecord,
        receipt: &DevelopmentReceipt,
    ) -> Result<(), String> {
        let valid = record.source_fact.is_some()
            && record.visible_to == audience(receipt, self)
            && record
                .sites
                .iter()
                .map(|site| site.id)
                .collect::<BTreeSet<_>>()
                == sites(receipt)
            && record.armies.is_empty()
            && record.people.is_empty()
            && record.formations.is_empty();
        if !valid {
            return Err(
                "history: development narrative contradicts its location or audience".into(),
            );
        }
        if let DevelopmentReceipt::SiteRenamed { site, new_name, .. } = receipt {
            if !record
                .sites
                .iter()
                .any(|label| label.id == *site && label.name == *new_name)
            {
                return Err("history: rename narrative lost its resulting label".into());
            }
        }
        Ok(())
    }
}
