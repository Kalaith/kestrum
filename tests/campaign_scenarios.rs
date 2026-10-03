//! Long production campaigns record real engine, retention, replay, and save measurements.

#[path = "support/appearance.rs"]
mod appearance_support;
#[path = "support/campaign_scenarios.rs"]
mod support;

use kestrum::state::StrategicCampaign;
use support::{campaign, run_to_round};

#[test]
fn peaceful_four_faction_production_campaign_preserves_fifty_year_continuity() {
    let data = kestrum::data::GameData::load().unwrap();
    let starting = campaign(&data, 4, 180_018);
    assert_eq!(starting.world.markers.len(), 80);
    assert_eq!(starting.world.sites.len(), 152);

    let metrics = run_to_round(starting, &data, 200, false);
    assert_eq!(metrics.rounds, 200);
    assert_eq!(metrics.factions, 4);
}

#[test]
fn four_and_eight_faction_campaigns_retain_and_replay_through_four_hundred_rounds() {
    let data = kestrum::data::GameData::load().unwrap();
    for factions in [4, 8] {
        let seed = 180_400 + factions as u64;
        let starting = campaign(&data, factions, seed);
        let metrics = run_to_round(starting, &data, 400, true);
        let state = &metrics.state;
        let orphaned_family_rows = state
            .families
            .iter()
            .filter(|(person, family)| {
                !state.people.contains_key(person) && !family.links.is_empty()
            })
            .count();
        println!(
            "K18_RETAINED factions={} people={} living={} households={} families={} orphaned_family_rows={} armies={} formations={} history_events={} battles={}",
            metrics.factions,
            state.people.len(),
            state.people.values().filter(|person| person.is_alive()).count(),
            state.households.len(),
            state.families.len(),
            orphaned_family_rows,
            state.armies.len(),
            state.formations.len(),
            state.history.events.len(),
            state.battles.len(),
        );
        assert_eq!(metrics.rounds, 400);
        assert_eq!(metrics.factions, factions);
    }
}

#[test]
fn saved_production_campaigns_retain_their_authored_topology() {
    let data = kestrum::data::GameData::load().unwrap();
    for factions in [4, 8] {
        let original = campaign(&data, factions, 180_500 + factions as u64);
        let restored = support::reload(&original, &data);
        assert_eq!(restored, original);
        assert_eq!(restored.world.markers.len(), 80);
        assert_eq!(restored.world.sites.len(), 152);
        assert_eq!(restored.factions.len(), factions);
    }
}

#[test]
fn production_campaign_metrics_cover_full_engine_rounds_and_bound_retained_history() {
    let data = kestrum::data::GameData::load().unwrap();
    for factions in [4, 8] {
        let final_state: StrategicCampaign = run_to_round(
            campaign(&data, factions, 180_600 + factions as u64),
            &data,
            40,
            false,
        )
        .state;
        assert_eq!(final_state.completed_rounds, 40);
        assert!(final_state.history.events.len() <= data.history.detail_max_entries);
        assert!(final_state
            .history
            .person_notables
            .values()
            .all(|rows| rows.len() <= data.history.notable_max_entries));
    }
}

#[test]
fn active_pupil_item_heir_survives_mentor_death_at_a_campaign_boundary() {
    use kestrum::{
        data::{
            progression::TrainingDiscipline,
            world::{PersonClass, SiteId},
        },
        engine::{apply, Actor, Command},
        state::{
            legacy::{LegacyItemCustody, LegacyItemId},
            people::{PersonAssignment, PersonId, PersonStatus},
            relationships::{LegacyCategory, SuccessorLink},
            CampaignPhase,
        },
    };
    use macroquad_toolkit::rng::SeededRng;

    let data = kestrum::data::GameData::load().unwrap();
    let mut campaign = StrategicCampaign::new(&data).unwrap();
    let mentor = PersonId(1);
    let heir = campaign.next_ids.person;
    campaign.next_ids.person.0 += 1;

    campaign.people.get_mut(&mentor).unwrap().class = PersonClass::Infantry;
    campaign.people.get_mut(&mentor).unwrap().birth_round = -104;
    for _ in 0..4 {
        finish_round(&mut campaign, &data);
    }
    assert_eq!(campaign.completed_rounds, 4);
    assert_eq!(
        campaign.people[&mentor].career.discipline_service_seasons[&TrainingDiscipline::Infantry],
        4
    );

    let mut apprentice = campaign.people[&mentor].clone();
    apprentice.id = heir;
    apprentice.appearance =
        appearance_support::allocate(&mut campaign, &data.portraits, apprentice.id);
    apprentice.name = "K18 Apprentice".into();
    apprentice.class = PersonClass::Recruit;
    apprentice.birth_round = -64;
    apprentice.service_start_round = 0;
    apprentice.assignment = PersonAssignment::Site { site: SiteId(1) };
    apprentice.status = PersonStatus::Fit;
    apprentice.career = Default::default();
    apprentice.evidence = Default::default();
    campaign.people.insert(heir, apprentice);
    campaign.people.get_mut(&mentor).unwrap().birth_round = -235;
    campaign
        .people
        .get_mut(&mentor)
        .unwrap()
        .service_start_round = 0;

    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::StartMentorship {
            mentor,
            learner: heir,
            discipline: TrainingDiscipline::Infantry,
        },
    )
    .unwrap();
    assert!(campaign.is_pupil(mentor, heir));
    apply(
        &mut campaign,
        &data,
        Actor::Player,
        Command::DesignateSuccessor {
            predecessor: mentor,
            successor: heir,
            category: LegacyCategory::Item,
            link: SuccessorLink::Martial,
        },
    )
    .unwrap();
    let item: LegacyItemId = campaign
        .legacy_items
        .values()
        .find(|item| item.custody == LegacyItemCustody::Person(mentor))
        .unwrap()
        .id;
    assert!(campaign.successors[&mentor][&LegacyCategory::Item].link_witnessed);
    campaign.validate(&data).unwrap();

    let death_seed = (1..100_000)
        .find(|seed| {
            let mut rng = SeededRng::new(*seed);
            rng.below(1000) < 20
        })
        .unwrap();
    campaign.rng.people = SeededRng::new(death_seed);
    assert!(matches!(campaign.phase, CampaignPhase::PlayerTurn));
    finish_round(&mut campaign, &data);

    assert!(matches!(
        campaign.people[&mentor].status,
        PersonStatus::Dead { .. }
    ));
    assert!(!campaign.mentorships.contains_key(&heir));
    assert_eq!(
        campaign.legacy_items[&item].custody,
        LegacyItemCustody::Person(heir)
    );
    assert!(campaign.successors[&mentor].contains_key(&LegacyCategory::Item));
    campaign.validate(&data).unwrap();
}

#[test]
fn production_save_library_retains_more_than_five_campaigns() {
    use std::time::Instant;

    use kestrum::state::{persistence::SaveLibrary, Campaign};
    use macroquad_toolkit::persistence::{IndexedSaveStore, RawSaveStore, WriterStatus};

    #[derive(Default)]
    struct Store(std::collections::BTreeMap<String, String>);

    impl RawSaveStore for Store {
        fn read(&self, key: &str) -> Result<Option<String>, String> {
            Ok(self.0.get(key).cloned())
        }

        fn write(&mut self, key: &str, value: &str) -> Result<(), String> {
            self.0.insert(key.into(), value.into());
            Ok(())
        }
    }

    impl IndexedSaveStore for Store {
        fn writer_status(&mut self) -> Result<WriterStatus, String> {
            Ok(WriterStatus::Ready)
        }

        fn remove(&mut self, key: &str) -> Result<(), String> {
            self.0.remove(key);
            Ok(())
        }
    }

    let data = kestrum::data::GameData::load().unwrap();
    let mut strategic = campaign(&data, 4, 180_624);
    let mut store = Store::default();
    let mut library = SaveLibrary::open(&mut store, &data).unwrap();
    strategic.campaign_id = library.allocate_campaign_id(&mut store).unwrap();
    let started = Instant::now();
    for index in 0..24 {
        let state = Campaign::Strategic(Box::new(strategic.clone()));
        let name = format!("Production archive {}", index + 1);
        let save = library
            .prepare_manual(&mut store, &data, &state, &name, None)
            .unwrap();
        library.write(&mut store, &data, &save).unwrap();
    }
    let restarted = SaveLibrary::open(&mut store, &data).unwrap();
    assert_eq!(restarted.entries().len(), 24);
    assert_eq!(
        restarted.last_successful(),
        Some(kestrum::state::persistence::ContinueSelection::Indexed(
            restarted.entries()[23].id
        ))
    );
    let largest_payload = store.0.values().map(String::len).max().unwrap_or(0);
    println!(
        "K18_SAVE_LIBRARY entries={} backing_keys={} largest_value_bytes={} write_and_discovery_ms={:.3}",
        restarted.entries().len(),
        store.0.len(),
        largest_payload,
        started.elapsed().as_secs_f64() * 1000.0,
    );
}

fn finish_round(campaign: &mut StrategicCampaign, data: &kestrum::data::GameData) {
    let round = campaign.completed_rounds;
    while campaign.completed_rounds == round {
        let actor = match campaign.phase {
            kestrum::state::CampaignPhase::PlayerTurn => kestrum::engine::Actor::Player,
            kestrum::state::CampaignPhase::NpcTurn { faction, .. } => {
                kestrum::engine::Actor::Npc(faction)
            }
        };
        kestrum::engine::apply(campaign, data, actor, kestrum::engine::Command::EndTurn).unwrap();
    }
}
