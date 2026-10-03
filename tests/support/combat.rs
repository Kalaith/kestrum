//! Small physical battle fixtures and real in-memory catalogue compatibility checks.

#[path = "appearance_frozen.rs"]
mod appearance_support;

use super::*;
use kestrum::{
    data::world::FounderClass,
    state::{
        military::{Army, Formation},
        people::{Person, PersonStatus},
        persistence::{load_legacy, SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE},
    },
};
use macroquad_toolkit::persistence::{
    encode_slot, IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};
use std::collections::BTreeMap;

pub(super) fn fixture(first: u32, second: u32) -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    campaign
        .armies
        .retain(|id, _| [ArmyId(1), ArmyId(3)].contains(id));
    campaign
        .formations
        .retain(|id, _| [FormationId(1), FormationId(7)].contains(id));
    campaign.people.clear();
    campaign.legacy_items.clear();
    campaign.appearance_registry =
        kestrum::state::appearance::AppearanceRegistry::for_campaign_seed(
            campaign.seed,
            data.portraits.catalog_revision,
            data.portraits.allocation_revision,
        );
    for (army, formation, site, count) in [(1, 1, 8, first), (3, 7, 10, second)] {
        let army = campaign.armies.get_mut(&ArmyId(army)).unwrap();
        army.site = SiteId(site);
        army.commander = None;
        army.slots = [Some(FormationId(formation)), None, None, None, None, None];
        campaign
            .formations
            .get_mut(&FormationId(formation))
            .unwrap()
            .headcount = count;
    }
    campaign.validate(&data).unwrap();
    (data, campaign)
}

pub(super) fn person(campaign: &mut StrategicCampaign, id: u32, faction: u32, formation: u32) {
    let person_id = PersonId(id);
    let appearance = appearance_support::allocate_frozen(campaign, person_id);
    campaign.people.insert(
        person_id,
        Person {
            appearance,
            career: Default::default(),
            evidence: Default::default(),
            id: PersonId(id),
            faction: FactionId(faction),
            name: format!("Witness {id}"),
            birth_round: -96,
            service_start_round: 0,
            class: FounderClass::Officer,
            assignment: PersonAssignment::Formation {
                formation: FormationId(formation),
            },
            movement_spent: 0,
            status: PersonStatus::Fit,
        },
    );
    campaign.next_ids.person = PersonId(campaign.next_ids.person.0.max(id + 1));
}

pub(super) fn add_formation(campaign: &mut StrategicCampaign, id: u32, army: u32, source: u32) {
    let mut unit = campaign.formations[&FormationId(source)].clone();
    unit.id = FormationId(id);
    campaign.formations.insert(unit.id, unit);
    let army = campaign.armies.get_mut(&ArmyId(army)).unwrap();
    let slot = army.first_empty_slot().unwrap();
    army.slots[slot] = Some(FormationId(id));
    campaign.next_ids.formation.0 = campaign.next_ids.formation.0.max(id + 1);
}

pub(super) fn kind(
    campaign: &mut StrategicCampaign,
    data: &GameData,
    id: u32,
    kind: TroopKind,
    count: u32,
) {
    let formation = campaign.formations.get_mut(&FormationId(id)).unwrap();
    formation.kind = kind;
    formation.capacity = data.economy.formations[&kind].capacity;
    formation.headcount = count;
}

pub(super) fn add_army(
    campaign: &mut StrategicCampaign,
    id: u32,
    faction: u32,
    site: u32,
    formation: u32,
    count: u32,
) {
    campaign.armies.insert(
        ArmyId(id),
        Army {
            id: ArmyId(id),
            faction: FactionId(faction),
            site: SiteId(site),
            name: format!("Army {id}"),
            slots: [Some(FormationId(formation)), None, None, None, None, None],
            commander: None,
            battle_doctrine: None,
        },
    );
    campaign.formations.insert(
        FormationId(formation),
        Formation {
            battle_leader: None,
            tactics: None,
            tactics_override: Some(false),
            service: Default::default(),
            id: FormationId(formation),
            faction: FactionId(faction),
            kind: TroopKind::Warriors,
            headcount: count,
            capacity: 100,
            movement_spent: 0,
            created_round: 0,
        },
    );
    campaign.next_ids.army = ArmyId(campaign.next_ids.army.0.max(id + 1));
    campaign.next_ids.formation = FormationId(campaign.next_ids.formation.0.max(formation + 1));
}

pub(super) fn command() -> Command {
    order(&[1], &[8, 10])
}
pub(super) fn order(armies: &[u32], path: &[u32]) -> Command {
    Command::Move(MoveOrder {
        armies: armies.iter().copied().map(ArmyId).collect(),
        path: path.iter().copied().map(SiteId).collect(),
    })
}
pub(super) fn fight(campaign: &mut StrategicCampaign, data: &GameData) -> BattleReport {
    let outcome = apply(campaign, data, Actor::Player, command()).unwrap();
    let resolved = if outcome.battle_pending {
        apply(campaign, data, Actor::Player, Command::StartPendingBattle).unwrap()
    } else {
        outcome
    };
    campaign.battles[&resolved.battle.unwrap()].clone()
}
pub(super) fn loss(report: &BattleReport, _exchange: usize, formation: u32) -> u32 {
    report
        .exchanges
        .iter()
        .flat_map(|exchange| &exchange.losses)
        .filter(|loss| loss.formation == FormationId(formation))
        .map(|loss| loss.amount)
        .sum()
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
    let result = catalogue
        .write(
            &mut store,
            "combat_test",
            None,
            SaveMetadata {
                name: "Battle archive".into(),
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
        &library.load(&mut store, data, result.entry_id).unwrap(),
        expected
    );
    for (key, value) in before {
        assert_eq!(store.values[&key], value);
    }
}

pub(super) fn assert_earlier_save(data: &GameData) {
    let campaign = StrategicCampaign::new(data).unwrap();
    let expected = Campaign::Strategic(Box::new(campaign.clone()));
    let mut value = serde_json::to_value(&expected).unwrap();
    value.as_object_mut().unwrap().remove("battles");
    value["next_ids"].as_object_mut().unwrap().remove("battle");
    value["world"].as_object_mut().unwrap().remove("occupation");
    value["world"]["sites"][7]["tags"] = serde_json::json!([]);
    for person in value["people"].as_object_mut().unwrap().values_mut() {
        person.as_object_mut().unwrap().remove("status");
    }
    let raw = serde_json::to_string(&value).unwrap();
    let restored: Campaign = serde_json::from_str(&raw).unwrap();
    restored.validate(data).unwrap();
    assert_eq!(restored, expected);
    assert_eq!(
        load_legacy(
            &encode_slot("kestrum_strategic_v2", &value, "2").unwrap(),
            data
        )
        .unwrap(),
        expected
    );
    assert_raw_catalogue(data, &campaign, &raw, &expected);
    for field in ["battles", "counter", "occupation"] {
        let mut partial = value.clone();
        match field {
            "battles" => partial["battles"] = serde_json::json!({}),
            "counter" => partial["next_ids"]["battle"] = serde_json::json!(1),
            _ => partial["world"]["occupation"] = serde_json::json!({}),
        }
        assert!(
            serde_json::from_value::<Campaign>(partial).is_err(),
            "{field}"
        );
    }
    let mut malformed = serde_json::to_value(&expected).unwrap();
    malformed["battles"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<Campaign>(malformed).is_err());
}

pub(super) fn assert_bad_reports(data: &GameData, campaign: &StrategicCampaign) {
    for invalid in [
        "loss",
        "duplicate_loss",
        "unknown_form",
        "bad_counter",
        "date",
        "unknown_assignment",
        "wound_steps",
        "invented_event",
        "wrong_name",
        "remote_retreat",
        "winner_left",
        "starting_host",
    ] {
        let mut altered = campaign.clone();
        let report = altered.battles.values_mut().next().unwrap();
        match invalid {
            "loss" => report.attacker.armies[0].formations[0].combat_losses += 1,
            "duplicate_loss" => {
                let duplicate = report.exchanges[0].losses[0].clone();
                report.exchanges[0].losses.push(duplicate);
            }
            "unknown_form" => report.exchanges[0].losses[0].formation = FormationId(9999),
            "bad_counter" => altered.next_ids.battle = BattleId(1),
            "date" => report.completed_rounds += 1,
            "unknown_assignment" => {
                report.attacker.armies[0].people[0].assignment = PersonAssignment::Formation {
                    formation: FormationId(7),
                }
            }
            "wound_steps" => {
                report.attacker.armies[0].people[0].status = PersonStatus::Wounded {
                    since_round: 0,
                    remaining_steps: 0,
                }
            }
            "invented_event" => {
                report
                    .person_events
                    .push(kestrum::state::people::PersonCombatEvent {
                        person: PersonId(1),
                        name: "Witness 1".into(),
                        faction: FactionId(1),
                        outcome: kestrum::state::people::PersonCombatOutcome::Died {
                            reason: kestrum::state::people::PersonDeathReason::FormationDestroyed,
                        },
                    })
            }
            "wrong_name" => {
                report.attacker.armies[0].commander =
                    Some(kestrum::state::battle::BattleCommander {
                        id: PersonId(1),
                        name: "Wrong".into(),
                    });
            }
            "remote_retreat" => report.attacker.armies[0].final_site = Some(SiteId(1)),
            "winner_left" => {
                report.defender.faction_side_mut().unwrap().armies[0].final_site = Some(SiteId(11))
            }
            _ => report.attacker.armies[0].people[0].starting_formation = FormationId(7),
        }
        assert!(altered.validate(data).is_err(), "{invalid}");
    }
}

pub(super) fn assert_living_leadership() {
    let (data, mut campaign) = fixture(100, 100);
    let mut host = campaign.formations[&FormationId(1)].clone();
    host.id = FormationId(13);
    host.headcount = 1;
    campaign.formations.insert(host.id, host);
    campaign.next_ids.formation = FormationId(14);
    let army = campaign.armies.get_mut(&ArmyId(1)).unwrap();
    army.slots[0] = Some(FormationId(13));
    army.slots[1] = Some(FormationId(1));
    person(&mut campaign, 1, 1, 13);
    campaign.armies.get_mut(&ArmyId(1)).unwrap().commander = Some(PersonId(1));
    let report = fight(&mut campaign, &data);
    assert_eq!(report.exchanges[0].leadership[0].permille, 933);
    let unit = report
        .simulation
        .as_ref()
        .unwrap()
        .opening
        .armies
        .iter()
        .flat_map(|army| army.slots.iter().flatten())
        .find(|unit| {
            unit.id == kestrum::state::battle::simulation::BattleUnitId::Formation(FormationId(13))
        })
        .unwrap();
    assert!(unit.attack < data.troops.formations[&TroopKind::Warriors].attack);
}

pub(super) fn assert_peaceful_contact() {
    let (data, mut campaign) = fixture(100, 100);
    add_army(&mut campaign, 5, 2, 6, 13, 100);
    let result = apply(&mut campaign, &data, Actor::Player, order(&[1], &[8, 6])).unwrap();
    assert!(result.battle.is_none());
    assert_eq!(campaign.world.site(SiteId(6)).unwrap().controller, None);
    assert_eq!(campaign.armies[&ArmyId(1)].site, SiteId(6));
    assert_eq!(campaign.formations[&FormationId(1)].movement_spent, 2);
    let (_, mut mixed) = fixture(100, 100);
    add_army(&mut mixed, 5, 2, 10, 13, 100);
    let before = mixed.clone();
    assert!(apply(&mut mixed, &data, Actor::Player, command()).is_err());
    assert_eq!(mixed, before);
}

pub(super) fn assert_retreat_priorities() {
    let (data, mut supplied) = fixture(100, 40);
    supplied
        .set_site_control(&data, SiteId(9), None, false)
        .unwrap();
    supplied
        .set_site_control(&data, SiteId(11), Some(FactionId(3)), false)
        .unwrap();
    let supplied_report = fight(&mut supplied, &data);
    assert!(supplied_report.defender.armies()[0].final_site.is_some());
    assert_eq!(supplied.armies[&ArmyId(3)].site, SiteId(11)); // supplied beats lower neutral9
    for blocked in ["contested", "hostile"] {
        let (_, mut campaign) = fixture(100, 40);
        campaign
            .set_site_control(&data, SiteId(9), None, blocked == "contested")
            .unwrap();
        if blocked == "hostile" {
            add_army(&mut campaign, 5, 1, 9, 13, 100);
        }
        fight(&mut campaign, &data);
        assert_eq!(campaign.armies[&ArmyId(3)].site, SiteId(11), "{blocked}");
    }
    let (_, mut preferred) = fixture(100, 100);
    preferred
        .set_site_control(&data, SiteId(12), Some(FactionId(1)), false)
        .unwrap();
    preferred
        .factions
        .get_mut(&FactionId(1))
        .unwrap()
        .headquarters = SiteId(12);
    preferred.reconcile_region_control();
    fight(&mut preferred, &data);
    assert_eq!(preferred.armies[&ArmyId(1)].site, SiteId(8)); // neutral origin beats supplied12
}
