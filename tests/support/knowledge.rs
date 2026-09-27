//! Real encounters and the public save-library boundary, without private mutation hooks.

use super::*;
use kestrum::state::{
    battle::BattleReport,
    knowledge::EncounteredPerson,
    persistence::{load_legacy, SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE},
};
use macroquad_toolkit::persistence::{
    encode_slot, IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};
use std::collections::BTreeMap;

pub(super) fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign
        .people
        .retain(|id, _| [PersonId(1), PersonId(3)].contains(id));
    for (army, formation, site) in [(1, 1, 8), (3, 7, 10)] {
        let army = campaign.armies.get_mut(&ArmyId(army)).unwrap();
        army.site = SiteId(site);
        army.commander = None;
        army.slots = [Some(FormationId(formation)), None, None, None, None, None];
    }
    campaign.people.get_mut(&PersonId(1)).unwrap().name = "Own witness".into();
    campaign.people.get_mut(&PersonId(3)).unwrap().name = "Enemy witness 003".into();
    let mut remote = campaign.people[&PersonId(3)].clone();
    remote.id = PersonId(5);
    remote.name = "Hidden career".into();
    remote.assignment = PersonAssignment::Site { site: SiteId(3) };
    campaign.people.insert(remote.id, remote);
    campaign.next_ids.person = PersonId(6);
    campaign.validate(&data).unwrap();
    (data, campaign)
}

pub(super) fn fight(campaign: &mut StrategicCampaign, data: &GameData) -> BattleReport {
    let outcome = apply(
        campaign,
        data,
        Actor::Player,
        Command::Move(MoveOrder {
            armies: vec![ArmyId(1)],
            path: vec![SiteId(8), SiteId(10)],
        }),
    )
    .unwrap();
    campaign.battles[&outcome.battle.unwrap()].clone()
}

pub(super) fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        advance_npc(campaign, data).unwrap();
    }
}

pub(super) fn snapshot(
    campaign: &StrategicCampaign,
    observer: FactionId,
    id: PersonId,
) -> EncounteredPerson {
    match person_knowledge(campaign, observer, id).unwrap() {
        PersonKnowledge::LastEncountered { snapshot, .. } => snapshot,
        PersonKnowledge::CurrentOwn(_) => panic!("expected historical enemy knowledge"),
    }
}

pub(super) fn assert_corrupt_snapshots(data: &GameData, campaign: &StrategicCampaign) {
    for invalid in [
        "observer",
        "identity",
        "date",
        "site",
        "army",
        "battle",
        "name",
        "condition",
    ] {
        let mut candidate = campaign.clone();
        if invalid == "observer" {
            let known = candidate.knowledge.observers.remove(&FactionId(1)).unwrap();
            candidate.knowledge.observers.insert(FactionId(2), known);
        } else {
            let person = candidate
                .knowledge
                .observers
                .get_mut(&FactionId(1))
                .unwrap()
                .people
                .get_mut(&PersonId(3))
                .unwrap();
            match invalid {
                "identity" => person.id = PersonId(999),
                "date" => person.completed_rounds = 1,
                "site" => person.site = SiteId(999),
                "army" => person.army = ArmyId(999),
                "battle" => person.battle = BattleId(999),
                "name" => person.name = "Invented identity".into(),
                _ => person.condition = ObservedCondition::Dead,
            }
        }
        assert!(candidate.validate(data).is_err(), "{invalid}");
    }
    assert_corrupt_history(data, campaign);
}

fn assert_corrupt_history(data: &GameData, campaign: &StrategicCampaign) {
    use kestrum::state::{campaign::FactId, history::HistoryId};
    for invalid in [
        "identity",
        "date",
        "source",
        "visibility",
        "army_label",
        "notable",
        "duplicate",
    ] {
        let mut candidate = campaign.clone();
        let record = candidate.history.events.values_mut().next().unwrap();
        match invalid {
            "identity" => record.id = HistoryId(999),
            "date" => record.completed_rounds += 1,
            "source" => record.source_fact = Some(FactId(candidate.next_ids.fact.0)),
            "visibility" => {
                record.visible_to.insert(FactionId(2));
            }
            "army_label" => record.armies[0].name = "Invented battle name".into(),
            "notable" => {
                candidate.history.site_notables.values_mut().next().unwrap()[0].completed_rounds +=
                    1
            }
            _ => {
                let mut duplicate = record.clone();
                duplicate.id = candidate.next_ids.history;
                candidate.next_ids.history.0 += 1;
                candidate.history.events.insert(duplicate.id, duplicate);
            }
        }
        assert!(candidate.validate(data).is_err(), "history {invalid}");
    }
}

#[derive(Default)]
struct MemoryStore {
    values: BTreeMap<String, String>,
}
impl RawSaveStore for MemoryStore {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.values.get(key).cloned())
    }
    fn write(&mut self, key: &str, content: &str) -> Result<(), String> {
        self.values.insert(key.into(), content.into());
        Ok(())
    }
}
impl IndexedSaveStore for MemoryStore {
    fn writer_status(&mut self) -> Result<WriterStatus, String> {
        Ok(WriterStatus::Ready)
    }
    fn remove(&mut self, key: &str) -> Result<(), String> {
        self.values.remove(key);
        Ok(())
    }
}

pub(super) fn assert_catalogue(data: &GameData, campaign: &StrategicCampaign) {
    let expected = Campaign::Strategic(Box::new(campaign.clone()));
    let raw = serde_json::to_string(&expected).unwrap();
    assert_eq!(
        load_legacy(
            &encode_slot("kestrum_strategic_v2", &expected, "2").unwrap(),
            data
        )
        .unwrap(),
        expected
    );
    assert_raw_catalogue(data, campaign, &raw, &expected);
}

fn assert_raw_catalogue(
    data: &GameData,
    campaign: &StrategicCampaign,
    raw: &str,
    expected: &Campaign,
) {
    let mut store = MemoryStore::default();
    let mut catalogue = IndexedCatalogue::<SaveMetadata>::new(SAVE_NAMESPACE).unwrap();
    catalogue.refresh(&mut store, |_, _| Ok(())).unwrap();
    let success_order = catalogue.reserve_identity(&mut store).unwrap();
    let written = catalogue
        .write(
            &mut store,
            "knowledge_test",
            None,
            SaveMetadata {
                name: "Known encounters".into(),
                kind: SaveKind::Manual,
                campaign_id: campaign.campaign_id,
                completed_rounds: campaign.completed_rounds,
                schema_version: 2,
                content_version: 1,
                success_order,
            },
            raw,
            |_, _| Ok(()),
        )
        .unwrap();
    let before = store.values.clone();
    let mut library = SaveLibrary::open(&mut store, data).unwrap();
    assert_eq!(
        &library.load(&mut store, data, written.entry_id).unwrap(),
        expected
    );
    for (key, value) in before {
        assert_eq!(store.values[&key], value);
    }
}

pub(super) fn assert_earlier_save(data: &GameData, campaign: &StrategicCampaign) {
    let mut old = campaign.clone();
    old.people.get_mut(&PersonId(3)).unwrap().name = "Unknown later name".into();
    let mut value = serde_json::to_value(Campaign::Strategic(Box::new(old))).unwrap();
    strip_k08(&mut value);
    let raw = serde_json::to_string(&value).unwrap();
    let restored: Campaign = serde_json::from_str(&raw).unwrap();
    restored.validate(data).unwrap();
    let Campaign::Strategic(migrated) = &restored else {
        panic!("strategic migration")
    };
    assert_eq!(migrated.knowledge, campaign.knowledge);
    assert_eq!(migrated.rng, campaign.rng);
    assert_eq!(migrated.completed_rounds, campaign.completed_rounds);
    assert!(migrated
        .formations
        .values()
        .all(|formation| formation.service.xp == 0));
    assert!(migrated
        .people
        .values()
        .all(|person| person.evidence.counts.is_empty()));
    assert!(migrated
        .battles
        .values()
        .flat_map(|report| [&report.attacker, &report.defender])
        .flat_map(|side| &side.armies)
        .flat_map(|army| &army.people)
        .all(|person| person.starting_status.is_none()));
    assert_eq!(
        snapshot(migrated, FactionId(1), PersonId(3)).name,
        "Enemy witness 003"
    );
    assert_eq!(
        load_legacy(
            &encode_slot("kestrum_strategic_v2", &value, "2").unwrap(),
            data
        )
        .unwrap(),
        restored
    );
    assert_raw_catalogue(data, campaign, &raw, &restored);
    for field in ["knowledge", "history", "counter", "service", "evidence"] {
        let mut partial = value.clone();
        match field {
            "knowledge" => {
                partial["knowledge"] = serde_json::to_value(&campaign.knowledge).unwrap()
            }
            "history" => partial["history"] = serde_json::to_value(&campaign.history).unwrap(),
            "counter" => partial["next_ids"]["history"] = serde_json::json!(1),
            "service" => {
                partial["formations"]["1"]["service"] =
                    serde_json::to_value(&campaign.formations[&FormationId(1)].service).unwrap()
            }
            _ => {
                partial["people"]["1"]["evidence"] =
                    serde_json::to_value(&campaign.people[&PersonId(1)].evidence).unwrap()
            }
        }
        assert!(
            serde_json::from_value::<Campaign>(partial).is_err(),
            "{field}"
        );
    }
}

fn strip_k08(value: &mut serde_json::Value) {
    value.as_object_mut().unwrap().remove("knowledge");
    value.as_object_mut().unwrap().remove("history");
    value["next_ids"].as_object_mut().unwrap().remove("history");
    for formation in value["formations"].as_object_mut().unwrap().values_mut() {
        formation.as_object_mut().unwrap().remove("service");
    }
    for person in value["people"].as_object_mut().unwrap().values_mut() {
        person.as_object_mut().unwrap().remove("evidence");
    }
    for report in value["battles"].as_object_mut().unwrap().values_mut() {
        for side in ["attacker", "defender"] {
            for army in report[side]["armies"].as_array_mut().unwrap() {
                for person in army["people"].as_array_mut().unwrap() {
                    person.as_object_mut().unwrap().remove("starting_status");
                }
                for formation in army["formations"].as_array_mut().unwrap() {
                    formation
                        .as_object_mut()
                        .unwrap()
                        .remove("veterancy_permille");
                }
            }
        }
    }
}

pub(super) fn assert_expired_earlier_save(data: &GameData, campaign: &StrategicCampaign) {
    let mut old = campaign.clone();
    old.completed_rounds = 81;
    let mut value = serde_json::to_value(Campaign::Strategic(Box::new(old.clone()))).unwrap();
    strip_k08(&mut value);
    let raw = serde_json::to_string(&value).unwrap();
    let restored: Campaign = serde_json::from_str(&raw).unwrap();
    restored.validate(data).unwrap();
    let Campaign::Strategic(migrated) = &restored else {
        panic!("strategic migration")
    };
    assert!(migrated.knowledge.observers.is_empty());
    assert!(migrated.battles.is_empty());
    assert!(migrated.history.events.is_empty());
    assert!(person_knowledge(migrated, FactionId(1), PersonId(3)).is_none());
    assert!(migrated
        .formations
        .values()
        .all(|formation| formation.service.xp == 0));
    assert_eq!(migrated.rng, old.rng);
    assert_raw_catalogue(data, &old, &raw, &restored);
}

pub(super) fn assert_departed_budgets(data: &GameData) {
    let (_, mut campaign) = fixture();
    for formation in campaign.formations.values_mut() {
        formation.headcount = 1;
    }
    for site in campaign.world.adjacent_sites(SiteId(10)) {
        campaign.set_site_control(data, site, None, false).unwrap();
    }
    fight(&mut campaign, data);
    assert!(!campaign.people[&PersonId(1)].is_alive());
    assert!(!campaign.people[&PersonId(3)].is_alive());
    let departed = campaign.people[&PersonId(1)].clone();
    for id in 6..2006 {
        let mut person = departed.clone();
        person.id = PersonId(id);
        person.name = format!("Departed member {id}");
        person.evidence = Default::default();
        campaign.people.insert(person.id, person);
    }
    campaign.next_ids.person = PersonId(2006);
    campaign.validate(data).unwrap();
    // Migration must keep pending participants until their actual facts consume.
    let mut value = serde_json::to_value(Campaign::Strategic(Box::new(campaign))).unwrap();
    strip_k08(&mut value);
    let restored: Campaign = serde_json::from_value(value).unwrap();
    let Campaign::Strategic(mut campaign) = restored else {
        panic!("strategic migration")
    };
    campaign.validate(data).unwrap();
    assert!(
        campaign.people.contains_key(&PersonId(1)) && campaign.people.contains_key(&PersonId(3))
    );
    assert_eq!(
        campaign
            .people
            .values()
            .filter(|person| !person.is_alive())
            .count(),
        2002
    );
    finish_round(&mut campaign, data);
    assert!(
        !campaign.people.contains_key(&PersonId(1)) && !campaign.people.contains_key(&PersonId(3))
    );
    assert_eq!(
        campaign
            .people
            .values()
            .filter(|person| !person.is_alive())
            .count(),
        2000
    );
    assert_eq!(
        snapshot(&campaign, FactionId(1), PersonId(3)).condition,
        ObservedCondition::Dead
    );
    assert!(!battle_reports(&campaign, FactionId(1)).is_empty());
    assert_catalogue(data, &campaign);
    while campaign.completed_rounds < 80 {
        finish_round(&mut campaign, data);
    }
    assert_eq!(
        campaign
            .people
            .values()
            .filter(|person| !person.is_alive())
            .count(),
        2000
    );
    finish_round(&mut campaign, data);
    assert_eq!(campaign.people.len(), 1);
    assert!(campaign.people[&PersonId(5)].is_alive());
    assert_eq!(campaign.next_ids.person, PersonId(2006));
    assert!(campaign.history.person_notables.is_empty());
    assert_catalogue(data, &campaign);
}

pub(super) fn assert_history_visibility(data: &GameData, campaign: &StrategicCampaign) {
    use kestrum::{
        engine::{history_page, HistoryFilter},
        state::history::{HistoryKindFilter, HistorySubject},
    };
    let filter = HistoryFilter {
        subject: Some(HistorySubject::Person(PersonId(3))),
        kind: Some(HistoryKindFilter::Battle),
        from_round: Some(0),
        to_round: Some(0),
        page: 0,
    };
    let known = history_page(campaign, FactionId(1), &filter);
    assert_eq!(known.total_entries, 1);
    assert!(known.entries[0]
        .people
        .iter()
        .any(|label| label.id == PersonId(3) && label.name == "Enemy witness 003"));
    for observer in [FactionId(2), FactionId(4), FactionId(99)] {
        assert_eq!(history_page(campaign, observer, &filter).total_entries, 0);
    }
    let absent = HistoryFilter {
        subject: Some(HistorySubject::Person(PersonId(5))),
        ..filter.clone()
    };
    assert_eq!(
        history_page(campaign, FactionId(1), &absent).total_entries,
        0
    );
    let future = HistoryFilter {
        from_round: Some(1),
        to_round: None,
        ..filter.clone()
    };
    assert_eq!(
        history_page(campaign, FactionId(1), &future).total_entries,
        0
    );
    let wrong_kind = HistoryFilter {
        kind: Some(HistoryKindFilter::Movement),
        ..filter.clone()
    };
    assert_eq!(
        history_page(campaign, FactionId(1), &wrong_kind).total_entries,
        0
    );
    let mut hidden = campaign.clone();
    apply(&mut hidden, data, Actor::Player, Command::EndTurn).unwrap();
    advance_npc(&mut hidden, data).unwrap();
    assert_eq!(hidden.active_faction(), FactionId(3));
    apply(
        &mut hidden,
        data,
        Actor::Npc(FactionId(3)),
        Command::Disband {
            formation: FormationId(7),
        },
    )
    .unwrap();
    assert_eq!(history_page(&hidden, FactionId(1), &filter), known);
    assert_eq!(
        history_page(&hidden, FactionId(1), &HistoryFilter::default()).total_entries,
        1
    );
    assert_eq!(
        history_page(&hidden, FactionId(3), &HistoryFilter::default()).total_entries,
        2
    );
    assert_eq!(
        snapshot(&hidden, FactionId(1), PersonId(3)),
        snapshot(campaign, FactionId(1), PersonId(3))
    );
}
