//! Real commands and indexed storage used by the K11 integration contracts.

use super::*;
use kestrum::state::persistence::{SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE};
use macroquad_toolkit::persistence::{
    IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};
use std::collections::BTreeMap;

pub(super) fn field_campaign() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(5);
    campaign
        .set_site_control(&data, SiteId(5), Some(FactionId(3)), false)
        .unwrap();
    let outcome = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(1), SiteId(5)],
        }),
    )
    .unwrap();
    assert!(outcome.battle_pending);
    assert!(outcome.battle.is_none());
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartPendingBattle,
    )
    .unwrap();
    (data, campaign)
}

pub(super) fn threat_campaign() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .formations
        .retain(|_, formation| formation.faction != FactionId(1) || formation.id == FormationId(1));
    let army = campaign.armies.get_mut(&ArmyId(1)).unwrap();
    army.slots = [Some(FormationId(1)), None, None, None, None, None];
    army.site = SiteId(12);
    campaign
        .set_site_control(&data, SiteId(12), Some(FactionId(1)), false)
        .unwrap();
    campaign.validate(&data).unwrap();
    (data, campaign)
}

pub(super) fn value(campaign: &StrategicCampaign) -> Value {
    serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap()
}

pub(super) fn reload(data: &GameData, campaign: &StrategicCampaign) -> StrategicCampaign {
    let restored: Campaign = serde_json::from_value(value(campaign)).unwrap();
    restored.validate(data).unwrap();
    restored.strategic().unwrap().clone()
}

pub(super) fn round(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_npc(campaign, data).unwrap();
    }
}

pub(super) fn strip_k11(value: &mut Value) {
    value.as_object_mut().unwrap().remove("threats");
    value["next_ids"].as_object_mut().unwrap().remove("threat");
    value["world"]
        .as_object_mut()
        .unwrap()
        .remove("development");
    for site in value["world"]["sites"].as_array_mut().unwrap() {
        site["tags"]
            .as_array_mut()
            .unwrap()
            .retain(|tag| tag != "ruins");
    }
    for faction in value["factions"].as_object_mut().unwrap().values_mut() {
        faction
            .as_object_mut()
            .unwrap()
            .remove("last_hq_relocation");
    }
    for report in value["battles"].as_object_mut().unwrap().values_mut() {
        report["defender"] = report["defender"]["side"].clone();
        for exchange in report["exchanges"].as_array_mut().unwrap() {
            exchange.as_object_mut().unwrap().remove("threat_losses");
        }
    }
}

pub(super) fn catalogue_load(data: &GameData, raw: &Value) -> Campaign {
    let expected: Campaign = serde_json::from_value(raw.clone()).unwrap();
    let campaign = expected.strategic().unwrap();
    let mut store = MemoryStore::default();
    let mut catalogue = IndexedCatalogue::<SaveMetadata>::new(SAVE_NAMESPACE).unwrap();
    catalogue.refresh(&mut store, |_, _| Ok(())).unwrap();
    let success_order = catalogue.reserve_identity(&mut store).unwrap();
    let metadata = SaveMetadata {
        name: "K11 compatibility".into(),
        kind: SaveKind::Manual,
        campaign_id: campaign.campaign_id,
        completed_rounds: campaign.completed_rounds,
        schema_version: 2,
        content_version: campaign.content_version,
        success_order,
    };
    let bytes = raw.to_string();
    let saved = catalogue
        .write(
            &mut store,
            "development_integrity",
            None,
            metadata,
            &bytes,
            |_, _| Ok(()),
        )
        .unwrap();
    let mut library = SaveLibrary::open(&mut store, data).unwrap();
    let loaded = library.load(&mut store, data, saved.entry_id).unwrap();
    assert_eq!(catalogue.load(&store, saved.entry_id).unwrap(), bytes);
    loaded
}

#[derive(Default)]
struct MemoryStore(BTreeMap<String, String>);
impl RawSaveStore for MemoryStore {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.0.get(key).cloned())
    }
    fn write(&mut self, key: &str, value: &str) -> Result<(), String> {
        self.0.insert(key.into(), value.into());
        Ok(())
    }
}
impl IndexedSaveStore for MemoryStore {
    fn writer_status(&mut self) -> Result<WriterStatus, String> {
        Ok(WriterStatus::Ready)
    }
    fn remove(&mut self, key: &str) -> Result<(), String> {
        self.0.remove(key);
        Ok(())
    }
}

pub(super) fn assert_development_history(data: &GameData, campaign: &StrategicCampaign) {
    let filter = HistoryFilter {
        kind: Some(HistoryKindFilter::Development),
        ..Default::default()
    };
    let visible = history_page(campaign, FactionId(2), &filter);
    assert_eq!(visible.total_entries, 2);
    assert!(visible.entries.iter().all(|event| matches!(
        &event.kind,
        HistoryKind::Development {
            receipt: DevelopmentReceipt::SiteRenamed { .. }
        }
    )));
    let migration = campaign
        .history
        .events
        .values()
        .find(|event| {
            matches!(
                &event.kind,
                HistoryKind::Development {
                    receipt: DevelopmentReceipt::PopulationMoved { .. }
                }
            )
        })
        .unwrap();
    let mut forged = campaign.clone();
    forged
        .history
        .events
        .get_mut(&migration.id)
        .unwrap()
        .visible_to
        .insert(FactionId(2));
    assert!(forged.validate(data).is_err());
}
