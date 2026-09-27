//! Add K16 custody and era dates to earlier strategic saves without inventing deeds.

use serde::de::Error as _;
use serde_json::Value;

use crate::{data::world::DiplomaticState, state::history::HistoryKind};

pub(super) struct Migration {
    pub seed_founders: bool,
    pub era_dates_missing: bool,
    pub history_links_missing: bool,
}

pub(super) fn initialize(value: &mut Value) -> Result<Migration, serde_json::Error> {
    let campaign = value
        .as_object_mut()
        .ok_or_else(|| serde_json::Error::custom("campaign must be an object"))?;
    let has_items = campaign.contains_key("legacy_items");
    let has_item_id = campaign
        .get("next_ids")
        .and_then(Value::as_object)
        .is_some_and(|ids| ids.contains_key("legacy_item"));
    let seed_founders = match (has_items, has_item_id) {
        (false, false) => {
            campaign.insert("legacy_items".into(), Value::Object(Default::default()));
            campaign
                .get_mut("next_ids")
                .and_then(Value::as_object_mut)
                .ok_or_else(|| serde_json::Error::custom("campaign.next_ids is missing"))?
                .insert("legacy_item".into(), Value::from(1));
            true
        }
        (true, true) => false,
        _ => {
            return Err(serde_json::Error::custom(
                "campaign: incomplete K16 item custody state",
            ))
        }
    };
    let era_dates_missing = campaign
        .get("diplomacy")
        .and_then(Value::as_object)
        .is_some_and(|diplomacy| !diplomacy.contains_key("era_history_complete"));
    let history_links_missing = campaign
        .get("history")
        .and_then(|history| history.get("events"))
        .and_then(Value::as_object)
        .is_some_and(|events| {
            events.values().any(|record| {
                record.get("related_events").is_none()
                    && record
                        .get("kind")
                        .and_then(|kind| kind.get("kind"))
                        .and_then(Value::as_str)
                        == Some("diplomacy")
                    && record
                        .get("kind")
                        .and_then(|kind| kind.get("receipt"))
                        .and_then(|receipt| receipt.get("kind"))
                        .and_then(Value::as_str)
                        == Some("war_declared")
            })
        });
    Ok(Migration {
        seed_founders,
        era_dates_missing,
        history_links_missing,
    })
}

pub(super) fn restore_era_dates(campaign: &mut super::super::StrategicCampaign) {
    use crate::state::diplomacy::DiplomacyReceipt;

    for pair in &mut campaign.diplomacy.pairs {
        let mut wars = Vec::new();
        let mut peaces = Vec::new();
        for record in campaign.history.events.values() {
            let HistoryKind::Diplomacy { receipt } = &record.kind else {
                continue;
            };
            match receipt {
                DiplomacyReceipt::WarDeclared { factions } if *factions == pair.factions => {
                    wars.push((record.completed_rounds, record.id));
                }
                DiplomacyReceipt::PeaceAgreed { factions, .. } if *factions == pair.factions => {
                    peaces.push((record.completed_rounds, record.id));
                }
                _ => {}
            }
        }
        pair.war_started_round = wars.iter().map(|(round, _)| *round).max();
        pair.war_ended_round = peaces.iter().map(|(round, _)| *round).max();
        if campaign
            .relations
            .iter()
            .find(|relation| relation.factions == pair.factions)
            .is_some_and(|relation| relation.state == DiplomaticState::War)
            && pair
                .war_started_round
                .is_some_and(|start| pair.war_ended_round.is_some_and(|end| start <= end))
        {
            pair.war_started_round = None;
        }
    }
}
