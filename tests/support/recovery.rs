//! Recovery acceptance fixtures keep the balance under test explicit.

use super::*;
use kestrum::state::{
    persistence::{SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE},
    FactionStatus,
};
use macroquad_toolkit::persistence::{
    IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};
use std::collections::BTreeMap;

pub(super) fn budget_fixture(gold: i64) -> (GameData, StrategicCampaign) {
    let mut data = GameData::load().unwrap();
    data.economy.headquarters_income_bonus = Resources {
        gold: 0,
        wood: 0,
        stone: 0,
    };
    for income in data.economy.settlement_income.values_mut() {
        *income = data.economy.headquarters_income_bonus;
    }
    for definition in data.economy.formations.values_mut() {
        definition.upkeep_gold = 0;
    }
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources = Resources {
        gold,
        wood: 123,
        stone: 88,
    };
    (data, campaign)
}

pub(super) fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        pass_npc(campaign, data).unwrap();
    }
}

pub(super) fn assert_income_then_upkeep_then_recovery() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources
        .gold = 0;
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 21;
    assert_eq!(
        recovery_preview(&campaign, &data, campaign.player).unwrap()[0].restored,
        20
    );
    finish_round(&mut campaign, &data);
    let faction = &campaign.factions[&campaign.player];
    let economy = faction.last_economy.as_ref().unwrap();
    let recovery = faction.last_recovery.as_ref().unwrap();
    assert_eq!(
        (
            economy.income.gold,
            economy.upkeep_paid,
            economy.closing.gold
        ),
        (50, 30, 20)
    );
    assert_eq!(
        (
            recovery.opening_gold,
            recovery.gold_spent,
            recovery.closing_gold
        ),
        (20, 6, 14)
    );
    assert_eq!(faction.resources.gold, 14);
    assert_eq!(campaign.formations[&FormationId(1)].headcount, 41);
}

pub(super) fn assert_deficit_blocks_and_paid_boundary_clears() {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    for _ in 0..3 {
        apply(
            &mut campaign,
            &data,
            Actor::Player,
            Command::Recruit {
                site: SiteId(1),
                army: Some(ArmyId(1)),
                kind: TroopKind::Warriors,
            },
        )
        .unwrap();
    }
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources
        .gold = 0;
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 21;
    assert!(
        recovery_preview(&campaign, &data, campaign.player).unwrap()[0]
            .blocked
            .as_ref()
            .unwrap()
            .contains("Upkeep")
    );
    finish_round(&mut campaign, &data);
    assert!(campaign.factions[&campaign.player].deficit);
    assert_eq!(campaign.formations[&FormationId(1)].headcount, 21);
    assert!(campaign.factions[&campaign.player]
        .last_recovery
        .as_ref()
        .unwrap()
        .entries
        .is_empty());
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::Disband {
            formation: FormationId(13),
        },
    )
    .unwrap();
    campaign
        .factions
        .get_mut(&campaign.player)
        .unwrap()
        .resources
        .gold = 6;
    let previews = recovery_preview(&campaign, &data, campaign.player).unwrap();
    assert_eq!((previews[0].restored, previews[0].gold_cost), (20, 6));
    assert!(
        campaign.factions[&campaign.player].deficit,
        "preview cannot clear a live deficit"
    );
    finish_round(&mut campaign, &data);
    assert!(!campaign.factions[&campaign.player].deficit);
    assert_eq!(campaign.formations[&FormationId(1)].headcount, 41);
}

pub(super) fn assert_recovery_save_validation(data: &GameData, campaign: &StrategicCampaign) {
    let value = serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    for invalid in [
        "cost",
        "headcount",
        "duplicate",
        "balance",
        "date",
        "identity",
    ] {
        let mut corrupt = value.clone();
        let receipt = &mut corrupt["factions"]["1"]["last_recovery"];
        match invalid {
            "cost" => receipt["entries"][0]["gold_cost"] = serde_json::json!(5),
            "headcount" => receipt["entries"][0]["restored"] = serde_json::json!(21),
            "duplicate" => {
                let copy = receipt["entries"][0].clone();
                receipt["entries"].as_array_mut().unwrap().push(copy);
            }
            "balance" => receipt["closing_gold"] = serde_json::json!(-1),
            "date" => {
                receipt["completed_rounds"] = serde_json::json!(campaign.completed_rounds + 1)
            }
            _ => {
                receipt["entries"][0]["formation"] =
                    serde_json::json!(campaign.next_ids.formation.0)
            }
        }
        let restored: Campaign = serde_json::from_value(corrupt).unwrap();
        assert!(restored.validate(data).is_err(), "{invalid}");
    }
    let restored: Campaign = serde_json::from_value(value).unwrap();
    restored.validate(data).unwrap();
    assert_eq!(restored.strategic(), Some(campaign));
}

pub(super) fn assert_inactive_factions_cannot_recover() {
    use kestrum::state::people::{PersonAssignment, PersonStatus};
    for status in [
        FactionStatus::Eliminated,
        FactionStatus::Vassal {
            sovereign: FactionId(1),
        },
    ] {
        let (data, mut campaign) = budget_fixture(100);
        let inactive = FactionId(2);
        campaign.factions.get_mut(&inactive).unwrap().status = status;
        campaign
            .formations
            .get_mut(&FormationId(4))
            .unwrap()
            .headcount = 21;
        campaign.round_order = campaign.independent_order();
        assert!(
            campaign.validate(&data).is_err(),
            "inactive forces are invalid saved state"
        );
        assert!(recovery_preview(&campaign, &data, inactive).is_err());
        campaign.armies.retain(|_, army| army.faction != inactive);
        campaign
            .formations
            .retain(|_, formation| formation.faction != inactive);
        for person in campaign
            .people
            .values_mut()
            .filter(|person| person.faction == inactive)
        {
            let site = campaign.factions[&inactive].headquarters;
            person.assignment = PersonAssignment::Site { site };
            person.status = PersonStatus::Displaced {
                completed_rounds: campaign.completed_rounds,
                site,
            };
            person.movement_spent = 0;
        }
        let original_gold = campaign.factions[&inactive].resources.gold;
        assert!(recovery_preview(&campaign, &data, inactive)
            .unwrap()
            .is_empty());
        finish_round(&mut campaign, &data);
        assert!(!campaign.formations.contains_key(&FormationId(4)));
        assert_eq!(campaign.factions[&inactive].resources.gold, original_gold);
        let statement = campaign.factions[&inactive].last_recovery.as_ref().unwrap();
        assert_eq!(statement.completed_rounds, campaign.completed_rounds);
        assert!(statement.entries.is_empty());
    }
}

pub(super) fn assert_pre_recovery_save_migration(data: &GameData, campaign: &StrategicCampaign) {
    let mut value = serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    for faction in value["factions"].as_object_mut().unwrap().values_mut() {
        faction.as_object_mut().unwrap().remove("last_recovery");
    }
    let restored: Campaign = serde_json::from_value(value.clone()).unwrap();
    restored.validate(data).unwrap();
    assert_eq!(restored.strategic(), Some(campaign));
    assert_eq!(
        load_legacy(
            &encode_slot("kestrum_strategic_v2", &value, "2").unwrap(),
            data
        )
        .unwrap(),
        restored
    );
    assert_catalogue_migration(data, &value, &restored);
    let mut resumed = restored.strategic().unwrap().clone();
    finish_round(&mut resumed, data);
    assert_eq!(resumed.formations[&FormationId(1)].headcount, 41);
    assert_eq!(
        resumed.factions[&resumed.player]
            .last_recovery
            .as_ref()
            .unwrap()
            .entries
            .len(),
        1
    );
}

fn assert_catalogue_migration(data: &GameData, old: &serde_json::Value, expected: &Campaign) {
    let raw = serde_json::to_string(old).unwrap();
    let campaign = expected.strategic().unwrap();
    let mut store = MemoryStore::default();
    let mut catalogue = IndexedCatalogue::<SaveMetadata>::new(SAVE_NAMESPACE).unwrap();
    catalogue.refresh(&mut store, |_, _| Ok(())).unwrap();
    let success_order = catalogue.reserve_identity(&mut store).unwrap();
    let metadata = SaveMetadata {
        name: "Before seasonal recovery".into(),
        kind: SaveKind::Manual,
        campaign_id: campaign.campaign_id,
        completed_rounds: campaign.completed_rounds,
        schema_version: 2,
        content_version: 1,
        success_order,
    };
    let saved = catalogue
        .write(
            &mut store,
            "earlier_recovery",
            None,
            metadata.clone(),
            &raw,
            |_, _| Ok(()),
        )
        .unwrap();
    let mut library = SaveLibrary::open(&mut store, data).unwrap();
    assert_eq!(library.entries()[0].metadata, metadata);
    let restored = library.load(&mut store, data, saved.entry_id).unwrap();
    assert_eq!(&restored, expected);
    assert_eq!(catalogue.load(&store, saved.entry_id).unwrap(), raw);
    let mut resumed = restored.strategic().unwrap().clone();
    finish_round(&mut resumed, data);
    assert_eq!(resumed.formations[&FormationId(1)].headcount, 41);
    assert_eq!(catalogue.load(&store, saved.entry_id).unwrap(), raw);
}

#[derive(Default)]
struct MemoryStore(BTreeMap<String, String>);

impl RawSaveStore for MemoryStore {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.0.get(key).cloned())
    }

    fn write(&mut self, key: &str, content: &str) -> Result<(), String> {
        self.0.insert(key.to_owned(), content.to_owned());
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
