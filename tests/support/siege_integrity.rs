//! Actual movement fixtures and the production indexed-save loading seam.

use super::*;
use kestrum::state::persistence::{SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE};
use macroquad_toolkit::persistence::{
    IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};
use std::collections::BTreeMap;

fn fixture(fort: bool, weak: bool) -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .set_site_control(&data, SiteId(5), Some(FactionId(3)), false)
        .unwrap();
    let site = campaign
        .world
        .sites
        .iter_mut()
        .find(|site| site.id == SiteId(5))
        .unwrap();
    site.habitation = kestrum::data::economy::Habitation::Outpost;
    site.military = if fort {
        MilitaryLayer::Fort
    } else {
        MilitaryLayer::None
    };
    campaign.world.population.insert(SiteId(5), 50);
    campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(5);
    if weak {
        for id in campaign.armies[&ArmyId(3)]
            .formation_ids()
            .collect::<Vec<_>>()
        {
            campaign.formations.get_mut(&id).unwrap().headcount = 1;
        }
    }
    campaign.validate(&data).unwrap();
    let contact = apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(1), SiteId(5)],
        }),
    )
    .unwrap();
    if contact.battle_pending {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::StartPendingBattle,
        )
        .unwrap();
    }
    (data, campaign)
}

pub(super) fn siege_fixture(weak: bool) -> (GameData, StrategicCampaign) {
    fixture(true, weak)
}
pub(super) fn field_fixture() -> (GameData, StrategicCampaign) {
    fixture(false, true)
}

pub(super) fn finish(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_npc(campaign, data).unwrap();
    }
}

pub(super) fn serialized(campaign: &StrategicCampaign) -> Value {
    serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap()
}

pub(super) fn strip_siege(value: &mut Value) {
    value.as_object_mut().unwrap().remove("sieges");
    value["next_ids"].as_object_mut().unwrap().remove("siege");
    value["world"]
        .as_object_mut()
        .unwrap()
        .remove("fort_damage");
    for report in value["battles"].as_object_mut().unwrap().values_mut() {
        for field in [
            "context",
            "wall_permille",
            "fort_damage_added",
            "road_damage",
        ] {
            report.as_object_mut().unwrap().remove(field);
        }
        for exchange in report["exchanges"].as_array_mut().unwrap() {
            exchange.as_object_mut().unwrap().remove("wall_permille");
        }
    }
}

pub(super) fn assert_partial_report_groups(old: &Value, modern: &Value) {
    let id = modern["battles"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap();
    for field in [
        "context",
        "wall_permille",
        "fort_damage_added",
        "road_damage",
    ] {
        let mut partial = modern.clone();
        partial["battles"][id]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(
            serde_json::from_value::<Campaign>(partial).is_err(),
            "missing {field}"
        );
        let mut mixed = old.clone();
        mixed["battles"][id][field] = modern["battles"][id][field].clone();
        assert!(
            serde_json::from_value::<Campaign>(mixed).is_err(),
            "mixed {field}"
        );
    }
    let mut partial = modern.clone();
    partial["battles"][id]["exchanges"][0]
        .as_object_mut()
        .unwrap()
        .remove("wall_permille");
    assert!(serde_json::from_value::<Campaign>(partial).is_err());
    for path in ["/sieges", "/next_ids/siege", "/world/fort_damage"] {
        let mut invalid = modern.clone();
        *invalid.pointer_mut(path).unwrap() = Value::Null;
        assert!(
            serde_json::from_value::<Campaign>(invalid).is_err(),
            "null {path}"
        );
    }
}

pub(super) fn corrupt_partition(campaign: &mut StrategicCampaign, kind: &str) {
    let siege = campaign.sieges.get_mut(&SiteId(5)).unwrap();
    match kind {
        "id" => siege.id.0 = 0,
        "counter" => campaign.next_ids.siege = siege.id,
        "participant" => siege.defending[0] = ArmyId(999),
        "duplicate" => siege.defending.push(siege.defending[0]),
        "side" => siege.defending[0] = ArmyId(1),
        "empty" => siege.besieging.clear(),
        "location" => campaign.armies.get_mut(&ArmyId(3)).unwrap().site = SiteId(3),
        "control" => {
            campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == SiteId(5))
                .unwrap()
                .controller = Some(FactionId(1))
        }
        "fort" => {
            campaign
                .world
                .sites
                .iter_mut()
                .find(|site| site.id == SiteId(5))
                .unwrap()
                .military = MilitaryLayer::None
        }
        "future" => siege.last_progress_round = Some(campaign.completed_rounds),
        "progress" => siege.elapsed_steps = 100,
        "damage" => {
            campaign.world.fort_damage.insert(SiteId(5), 101);
        }
        "unlisted" => campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(5),
        "inactive" => {
            campaign.factions.get_mut(&FactionId(3)).unwrap().status =
                kestrum::state::FactionStatus::Eliminated
        }
        _ => unreachable!(),
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
        name: "Siege integrity".into(),
        kind: SaveKind::Manual,
        campaign_id: campaign.campaign_id,
        completed_rounds: campaign.completed_rounds,
        schema_version: 2,
        content_version: 1,
        success_order,
    };
    let bytes = raw.to_string();
    let saved = catalogue
        .write(
            &mut store,
            "siege_integrity",
            None,
            metadata,
            &bytes,
            |_, _| Ok(()),
        )
        .unwrap();
    let mut library = SaveLibrary::open(&mut store, data).unwrap();
    let loaded = library.load(&mut store, data, saved.entry_id).unwrap();
    assert_eq!(
        catalogue.load(&store, saved.entry_id).unwrap(),
        bytes,
        "loading must not rewrite the old payload"
    );
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
