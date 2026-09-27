//! Named-person acceptance fixtures use fixed combat streams, never capture-time search.

use super::*;
use kestrum::state::persistence::{
    load_legacy, SaveKind, SaveLibrary, SaveMetadata, SAVE_NAMESPACE,
};
use macroquad_toolkit::persistence::{
    encode_slot, IndexedCatalogue, IndexedSaveStore, RawSaveStore, WriterStatus,
};
use std::collections::BTreeMap;

pub(super) fn fixture() -> (GameData, StrategicCampaign) {
    let data = GameData::load().unwrap();
    let campaign = StrategicCampaign::new(&data).unwrap();
    (data, campaign)
}

pub(super) fn context(campaign: &StrategicCampaign) -> PersonCombatContext {
    PersonCombatContext {
        site: SiteId(5),
        sides: vec![
            side(campaign, FactionId(1), vec![ArmyId(1)]),
            side(campaign, FactionId(2), vec![ArmyId(2)]),
        ],
    }
}

pub(super) fn side(
    campaign: &StrategicCampaign,
    faction: FactionId,
    armies: Vec<ArmyId>,
) -> PersonCombatSide {
    let starting_headcounts = armies
        .iter()
        .flat_map(|id| campaign.armies[id].formation_ids())
        .map(|id| (id, campaign.formations[&id].headcount))
        .collect();
    let commanders = armies
        .iter()
        .filter_map(|id| campaign.armies[id].commander)
        .collect();
    PersonCombatSide {
        faction,
        armies,
        starting_headcounts,
        commanders,
        refuges: vec![],
    }
}

pub(super) fn add_person(
    campaign: &mut StrategicCampaign,
    id: u32,
    faction: u32,
    formation: u32,
    age: u32,
) {
    let mut person = campaign.people[&PersonId(faction)].clone();
    person.id = PersonId(id);
    person.name = format!("Test officer {id}");
    person.birth_round = -4 * i64::from(age);
    person.assignment = PersonAssignment::Formation {
        formation: FormationId(formation),
    };
    campaign.people.insert(person.id, person);
    campaign.next_ids.person = PersonId(campaign.next_ids.person.0.max(id + 1));
}

pub(super) fn split_formation(
    campaign: &mut StrategicCampaign,
    formation: FormationId,
    target: ArmyId,
    commander: Option<PersonId>,
) {
    let source = campaign.armies.get_mut(&ArmyId(1)).unwrap();
    source
        .slots
        .iter_mut()
        .filter(|entry| **entry == Some(formation))
        .for_each(|entry| *entry = None);
    let mut army = source.clone();
    army.id = target;
    army.slots = [Some(formation), None, None, None, None, None];
    army.commander = commander;
    campaign.armies.insert(target, army);
    campaign.next_ids.army = ArmyId(target.0 + 1);
}

pub(super) fn wounded(remaining_steps: u32) -> PersonStatus {
    PersonStatus::Wounded {
        since_round: 0,
        remaining_steps,
    }
}

pub(super) fn clean_zeros(campaign: &mut StrategicCampaign, data: &GameData) {
    let removed: Vec<_> = campaign
        .formations
        .values()
        .filter(|formation| formation.headcount == 0)
        .map(|formation| formation.id)
        .collect();
    for id in removed {
        campaign.remove_formation(id).unwrap();
    }
    campaign.validate(data).unwrap();
}

pub(super) fn assert_only_combat_draws(
    before: &StrategicCampaign,
    after: &StrategicCampaign,
    draws: usize,
) {
    let mut expected = before.rng.clone();
    for _ in 0..draws {
        expected.combat.below(100);
    }
    assert_eq!(after.rng, expected);
}

pub(super) fn assert_wipe_probability_boundary() {
    for (seed, dies) in [(7, true), (22, false)] {
        // first rolls 24 and 25
        let (data, mut campaign) = fixture();
        let context = context(&campaign);
        campaign
            .formations
            .get_mut(&FormationId(1))
            .unwrap()
            .headcount = 0;
        campaign.rng.combat = SeededRng::new(seed);
        resolve_person_combat(&mut campaign, &data, &context).unwrap();
        assert_eq!(!campaign.people[&PersonId(1)].is_alive(), dies);
    }
}

pub(super) fn assert_commander_thresholds_and_no_per_hit_rolls() {
    for (remaining, seed, rolls, wounds) in
        [(81, 44, 0, false), (80, 0, 1, false), (80, 44, 1, true)]
    {
        let (data, mut campaign) = fixture();
        add_person(&mut campaign, 5, 1, 1, 30); // surviving ordinary attached people get no roll
        let context = context(&campaign);
        campaign
            .formations
            .get_mut(&FormationId(1))
            .unwrap()
            .headcount = remaining;
        campaign.rng.combat = SeededRng::new(seed);
        let before = campaign.clone();
        resolve_person_combat(&mut campaign, &data, &context).unwrap();
        assert_eq!(campaign.people[&PersonId(1)].status == wounded(2), wounds);
        assert_eq!(campaign.people[&PersonId(5)].status, PersonStatus::Fit);
        assert_only_combat_draws(&before, &campaign, rolls);
    }
}

pub(super) fn assert_real_commander_battle() {
    use kestrum::{data::world::DiplomaticState, engine::MoveOrder};
    let (data, mut campaign) = fixture();
    add_person(&mut campaign, 5, 1, 1, 30);
    campaign
        .relations
        .iter_mut()
        .find(|relation| relation.factions == [FactionId(1), FactionId(2)])
        .unwrap()
        .state = DiplomaticState::War;
    campaign.armies.get_mut(&ArmyId(2)).unwrap().site = SiteId(5);
    campaign
        .set_site_control(&data, SiteId(5), Some(FactionId(2)), false)
        .unwrap();
    campaign.rng.combat = SeededRng::new(4);
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
    let report = &campaign.battles[&outcome.battle.unwrap()];
    assert_eq!(campaign.people[&PersonId(1)].status, wounded(2));
    assert_eq!(campaign.armies[&ArmyId(1)].commander, Some(PersonId(5)));
    assert!(report
        .person_events
        .iter()
        .any(|event| event.person == PersonId(5)
            && event.outcome
                == PersonCombatOutcome::AssumedCommand {
                    army: ArmyId(1),
                    previous: PersonId(1)
                }));
    assert!(report.attacker.armies[0]
        .formations
        .iter()
        .all(|formation| formation.end > 0));
    let before = campaign.clone();
    let reports = kestrum::engine::battle_reports(&campaign, FactionId(1));
    assert_eq!(reports[0], *report);
    assert_eq!(campaign, before);
    campaign.validate(&data).unwrap();
}

pub(super) fn wound_first_commander(data: &GameData, campaign: &mut StrategicCampaign) {
    let context = context(campaign);
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 80;
    campaign.rng.combat = SeededRng::new(4);
    resolve_person_combat(campaign, data, &context).unwrap();
    assert_eq!(campaign.people[&PersonId(1)].status, wounded(2));
}

fn finish_round(campaign: &mut StrategicCampaign, data: &GameData) {
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(campaign.phase, CampaignPhase::NpcTurn { .. }) {
        advance_npc(campaign, data).unwrap();
    }
}

pub(super) fn finish_without_healing_effects(campaign: &mut StrategicCampaign, data: &GameData) {
    let mut unwounded = campaign.clone();
    unwounded.people.get_mut(&PersonId(1)).unwrap().status = PersonStatus::Fit;
    finish_round(campaign, data);
    finish_round(&mut unwounded, data);
    assert_eq!(campaign.factions, unwounded.factions);
    assert_eq!(campaign.formations, unwounded.formations);
    assert_eq!(campaign.rng, unwounded.rng);
}

pub(super) fn assert_dead_people_never_heal_or_transfer() {
    let (data, mut campaign) = fixture();
    let context = context(&campaign);
    campaign
        .formations
        .get_mut(&FormationId(1))
        .unwrap()
        .headcount = 0;
    campaign.rng.combat = SeededRng::new(7);
    resolve_person_combat(&mut campaign, &data, &context).unwrap();
    clean_zeros(&mut campaign, &data);
    let dead = campaign.people[&PersonId(1)].clone();
    for _ in 0..3 {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(campaign.people[&PersonId(1)], dead);
    assert_eq!(dead.age_years(400), dead.age_years(0));
    let before = campaign.clone();
    assert!(apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::TransferPerson {
            person: PersonId(1),
            to_formation: FormationId(2)
        }
    )
    .is_err());
    assert_eq!(campaign, before);
}

pub(super) fn assert_rejected_snapshot_is_atomic(data: &GameData) {
    for invalid in [
        "duplicate_army",
        "duplicate_commander",
        "increased_headcount",
        "foreign_commander",
    ] {
        let (_, mut campaign) = fixture();
        let mut context = context(&campaign);
        match invalid {
            "duplicate_army" => context.sides[0].armies.push(ArmyId(1)),
            "duplicate_commander" => context.sides[0].commanders.push(PersonId(1)),
            "increased_headcount" => {
                context.sides[0]
                    .starting_headcounts
                    .insert(FormationId(1), 99);
            }
            _ => context.sides[0].commanders = vec![PersonId(2)],
        }
        let before = campaign.clone();
        assert!(
            resolve_person_combat(&mut campaign, data, &context).is_err(),
            "{invalid}"
        );
        assert_eq!(campaign, before);
    }
}

pub(super) fn assert_status_validation(data: &GameData, campaign: &StrategicCampaign) {
    let encoded = serde_json::to_value(Campaign::Strategic(Box::new(campaign.clone()))).unwrap();
    for invalid in [
        "zero_steps",
        "too_many_steps",
        "future_wound",
        "live_dead_assignment",
        "dead_live_assignment",
        "future_death",
        "unknown_death_site",
        "wounded_commander",
    ] {
        let mut value = encoded.clone();
        match invalid {
            "zero_steps" => {
                value["people"]["1"]["status"]["remaining_steps"] = serde_json::json!(0)
            }
            "too_many_steps" => {
                value["people"]["1"]["status"]["remaining_steps"] = serde_json::json!(3)
            }
            "future_wound" => value["people"]["1"]["status"]["since_round"] = serde_json::json!(1),
            "live_dead_assignment" => {
                value["people"]["1"]["assignment"] = serde_json::json!({"kind":"dead"})
            }
            "dead_live_assignment" => {
                value["people"]["2"]["assignment"] = serde_json::json!({"kind":"site","site":2})
            }
            "future_death" => {
                value["people"]["2"]["status"]["completed_rounds"] = serde_json::json!(1)
            }
            "unknown_death_site" => value["people"]["2"]["status"]["site"] = serde_json::json!(999),
            _ => value["armies"]["1"]["commander"] = serde_json::json!(1),
        }
        let restored: Campaign = serde_json::from_value(value).unwrap();
        assert!(restored.validate(data).is_err(), "{invalid}");
    }
}

pub(super) fn assert_fit_migration(data: &GameData, campaign: &StrategicCampaign) {
    let expected = Campaign::Strategic(Box::new(campaign.clone()));
    let mut value = serde_json::to_value(&expected).unwrap();
    for person in value["people"].as_object_mut().unwrap().values_mut() {
        person.as_object_mut().unwrap().remove("status");
    }
    let restored: Campaign = serde_json::from_value(value.clone()).unwrap();
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
    let raw = serde_json::to_string(&value).unwrap();
    let mut store = MemoryStore::default();
    let mut catalogue = IndexedCatalogue::<SaveMetadata>::new(SAVE_NAMESPACE).unwrap();
    catalogue.refresh(&mut store, |_, _| Ok(())).unwrap();
    let success_order = catalogue.reserve_identity(&mut store).unwrap();
    let saved = catalogue
        .write(
            &mut store,
            "before_wounds",
            None,
            SaveMetadata {
                name: "Before combat wounds".into(),
                kind: SaveKind::Manual,
                campaign_id: campaign.campaign_id,
                completed_rounds: campaign.completed_rounds,
                schema_version: 2,
                content_version: 1,
                success_order,
            },
            &raw,
            |_, _| Ok(()),
        )
        .unwrap();
    let mut library = SaveLibrary::open(&mut store, data).unwrap();
    assert_eq!(
        library.load(&mut store, data, saved.entry_id).unwrap(),
        expected
    );
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
