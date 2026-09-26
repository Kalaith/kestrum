//! Five K03 contracts exercised against the real catalogue with injected storage failures.

use std::collections::{BTreeMap, BTreeSet};

use kestrum::{
    data::GameData,
    engine::{advance_npc, apply, Actor, Command},
    state::{
        persistence::{
            load_legacy, ContinueSelection, SaveKind, SaveLibrary, MAX_SAVE_NAME_CHARS,
            SAVE_NAMESPACE,
        },
        Campaign, CampaignPhase, GameState, ShellCampaign,
    },
};
use macroquad_toolkit::persistence::{encode_slot, IndexedSaveStore, RawSaveStore, WriterStatus};

#[derive(Clone, Default)]
struct MemoryStore {
    values: BTreeMap<String, String>,
    operations: usize,
    fail_at: Option<usize>,
    fail_from: Option<usize>,
    fail_key: Option<String>,
    busy: bool,
}

impl MemoryStore {
    fn mutate(&mut self) -> Result<(), String> {
        self.operations += 1;
        if self.fail_at == Some(self.operations)
            || self.fail_from.is_some_and(|first| self.operations >= first)
        {
            Err("Injected storage failure".into())
        } else {
            Ok(())
        }
    }
}

impl RawSaveStore for MemoryStore {
    fn read(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.values.get(key).cloned())
    }

    fn write(&mut self, key: &str, content: &str) -> Result<(), String> {
        if self.busy {
            return Err("Another window holds the save writer".into());
        }
        self.mutate()?;
        if self.fail_key.as_deref() == Some(key) {
            return Err("Injected key write failure".into());
        }
        self.values.insert(key.to_owned(), content.to_owned());
        Ok(())
    }
}

impl IndexedSaveStore for MemoryStore {
    fn writer_status(&mut self) -> Result<WriterStatus, String> {
        Ok(if self.busy {
            WriterStatus::Busy
        } else {
            WriterStatus::Ready
        })
    }

    fn remove(&mut self, key: &str) -> Result<(), String> {
        if self.busy {
            return Err("Another window holds the save writer".into());
        }
        self.mutate()?;
        self.values.remove(key);
        Ok(())
    }
}

fn fixture() -> (GameData, MemoryStore, SaveLibrary, Campaign) {
    let data = GameData::load().unwrap();
    let mut store = MemoryStore::default();
    let mut library = SaveLibrary::open(&mut store, &data).unwrap();
    let mut state = GameState::default();
    state.new_game(&data).unwrap();
    let mut campaign = state.campaign.unwrap();
    if let Campaign::Strategic(strategic) = &mut campaign {
        strategic.campaign_id = library.allocate_campaign_id(&mut store).unwrap();
    }
    (data, store, library, campaign)
}

fn round(campaign: &mut Campaign, data: &GameData) {
    let Campaign::Strategic(strategic) = campaign else {
        panic!("strategic fixture")
    };
    apply(strategic, data, Actor::Player, Command::EndTurn).unwrap();
    while matches!(strategic.phase, CampaignPhase::NpcTurn { .. }) {
        advance_npc(strategic, data).unwrap();
    }
}

fn manual(
    library: &mut SaveLibrary,
    store: &mut MemoryStore,
    data: &GameData,
    campaign: &Campaign,
) -> u64 {
    let request = library
        .prepare_manual(store, data, campaign, "Same name", None)
        .unwrap();
    library.write(store, data, &request).unwrap().id
}

#[test]
fn named_saves_and_round_checkpoints_have_unique_unbounded_catalogue_identities() {
    let (data, mut store, mut library, mut campaign) = fixture();
    let first_id = campaign.strategic().unwrap().campaign_id;
    let second_id = library.allocate_campaign_id(&mut store).unwrap();
    assert_ne!(first_id, second_id);
    let rng = campaign.strategic().unwrap().rng.clone();
    let mut saved = BTreeSet::new();
    for _ in 0..16 {
        assert!(saved.insert(manual(&mut library, &mut store, &data, &campaign)));
    }
    for _ in 0..8 {
        round(&mut campaign, &data);
        let request = library
            .prepare_checkpoint(&mut store, &data, &campaign)
            .unwrap();
        assert_eq!(request.metadata.kind, SaveKind::RoundCheckpoint);
        assert!(saved.insert(library.write(&mut store, &data, &request).unwrap().id));
    }
    assert_eq!(campaign.strategic().unwrap().rng, rng);
    let mut restarted = SaveLibrary::open(&mut store, &data).unwrap();
    assert_eq!(restarted.entries().len(), 24);
    assert_eq!(
        restarted
            .entries()
            .iter()
            .map(|entry| entry.id)
            .collect::<BTreeSet<_>>(),
        saved
    );
    let path_label = "../../A label";
    let request = restarted
        .prepare_manual(&mut store, &data, &campaign, path_label, None)
        .unwrap();
    let entry = restarted.write(&mut store, &data, &request).unwrap();
    assert!(store.values.keys().all(|key| !key.contains(path_label)));
    assert_eq!(
        restarted
            .entries()
            .iter()
            .find(|saved| saved.id == entry.id)
            .unwrap()
            .metadata
            .name,
        path_label
    );
    for name in [
        "".to_owned(),
        "line\nbreak".to_owned(),
        "x".repeat(MAX_SAVE_NAME_CHARS + 1),
    ] {
        let before = store.values.clone();
        assert!(restarted
            .prepare_manual(&mut store, &data, &campaign, &name, None)
            .is_err());
        assert_eq!(store.values, before);
    }
}

#[test]
fn every_interrupted_write_stage_preserves_existing_saves_and_recovers_one_intent() {
    for overwrite in [false, true] {
        let (data, mut stored, mut original_library, mut campaign) = fixture();
        let old_campaign = campaign.clone();
        let old_id = manual(&mut original_library, &mut stored, &data, &campaign);
        round(&mut campaign, &data);
        let request = original_library
            .prepare_manual(
                &mut stored,
                &data,
                &campaign,
                "New season",
                overwrite.then_some(old_id),
            )
            .unwrap();
        let mut probe = stored.clone();
        let mut probe_library = SaveLibrary::open(&mut probe, &data).unwrap();
        let before = probe.operations;
        probe_library.write(&mut probe, &data, &request).unwrap();
        let stages = probe.operations - before;
        assert!(
            stages >= 4,
            "journal, payload, publication, and selection must be tested"
        );
        for interrupted in 1..=stages {
            let mut store = stored.clone();
            let mut library = SaveLibrary::open(&mut store, &data).unwrap();
            store.fail_from = Some(store.operations + interrupted);
            let attempt = library.write(&mut store, &data, &request);
            assert!(attempt.is_err() || !attempt.as_ref().unwrap().warnings.is_empty());
            assert_eq!(campaign.strategic().unwrap().completed_rounds, 1);
            store.fail_from = None;
            if !overwrite && matches!(interrupted, 3 | 4) {
                // Early K03 clients persisted only the optional selection.
                store.values.insert(
                    format!("{SAVE_NAMESPACE}_continue"),
                    serde_json::to_string(&Some(ContinueSelection::Indexed(old_id))).unwrap(),
                );
            }
            let mut restored = SaveLibrary::open(&mut store, &data).unwrap();
            let latest = restored
                .entries()
                .iter()
                .max_by_key(|entry| entry.metadata.success_order)
                .unwrap()
                .id;
            assert_eq!(
                restored.last_successful(),
                Some(ContinueSelection::Indexed(latest))
            );
            let existing = restored.load(&mut store, &data, old_id).unwrap();
            assert!(existing == old_campaign || (overwrite && existing == campaign));
            let retry = restored.write(&mut store, &data, &request).unwrap();
            assert_eq!(restored.entries().len(), if overwrite { 1 } else { 2 });
            assert_eq!(
                restored.load(&mut store, &data, retry.id).unwrap(),
                campaign
            );
            let repeated = restored.write(&mut store, &data, &request).unwrap();
            assert_eq!(repeated.id, retry.id);
            assert!(repeated.replayed);
            restored.load(&mut store, &data, old_id).unwrap();
            let deliberately_older = SaveLibrary::open(&mut store, &data).unwrap();
            assert_eq!(
                deliberately_older.last_successful(),
                Some(ContinueSelection::Indexed(old_id))
            );
        }
    }
    explicit_older_selection_survives_delayed_cleanup();
}

fn explicit_older_selection_survives_delayed_cleanup() {
    let (data, mut store, mut library, mut campaign) = fixture();
    let older = manual(&mut library, &mut store, &data, &campaign);
    round(&mut campaign, &data);
    let checkpoint = library
        .prepare_checkpoint(&mut store, &data, &campaign)
        .unwrap();
    // New write stages are journal reservation, payload, publication, then
    // journal cleanup. The outage also prevents the following Continue write.
    store.fail_from = Some(store.operations + 4);
    let committed = library.write(&mut store, &data, &checkpoint).unwrap();
    assert!(!committed.warnings.is_empty());
    store.fail_from = None;
    store.fail_key = Some(format!("{SAVE_NAMESPACE}_index"));
    library.load(&mut store, &data, older).unwrap();
    assert_eq!(
        library.last_successful(),
        Some(ContinueSelection::Indexed(older))
    );
    store.fail_key = None;
    let refreshed = SaveLibrary::open(&mut store, &data).unwrap();
    assert!(refreshed
        .entries()
        .iter()
        .any(|entry| entry.id == committed.id));
    assert_eq!(
        refreshed.last_successful(),
        Some(ContinueSelection::Indexed(older))
    );
}

#[test]
fn overwrite_delete_and_checkpoint_retry_require_explicit_stable_intents() {
    stale_unattempted_overwrite_preserves_a_newer_save();
    save_intents_commit_in_preparation_order();
    let (data, mut store, mut library, mut campaign) = fixture();
    let original = library
        .prepare_manual(&mut store, &data, &campaign, "Original", None)
        .unwrap();
    let first = library.write(&mut store, &data, &original).unwrap().id;
    let second = manual(&mut library, &mut store, &data, &campaign);
    assert_ne!(first, second);
    round(&mut campaign, &data);
    let overwrite = library
        .prepare_manual(&mut store, &data, &campaign, "Replaced", Some(first))
        .unwrap();
    assert_eq!(
        library.write(&mut store, &data, &overwrite).unwrap().id,
        first
    );
    assert_eq!(library.entries().len(), 2);
    assert!(library.write(&mut store, &data, &original).is_err());
    assert_eq!(library.load(&mut store, &data, first).unwrap(), campaign);
    assert_eq!(
        library
            .load(&mut store, &data, second)
            .unwrap()
            .strategic()
            .unwrap()
            .completed_rounds,
        0
    );
    let checkpoint = library
        .prepare_checkpoint(&mut store, &data, &campaign)
        .unwrap();
    let saved = library.write(&mut store, &data, &checkpoint).unwrap();
    round(&mut campaign, &data);
    assert_eq!(
        library.write(&mut store, &data, &checkpoint).unwrap().id,
        saved.id
    );
    assert_eq!(
        library
            .load(&mut store, &data, saved.id)
            .unwrap()
            .strategic()
            .unwrap()
            .completed_rounds,
        1
    );
    let mut probe = store.clone();
    let mut probe_library = SaveLibrary::open(&mut probe, &data).unwrap();
    let before = probe.operations;
    probe_library.delete(&mut probe, &data, first).unwrap();
    for stage in 1..=probe.operations - before {
        let mut interrupted = store.clone();
        let mut interrupted_library = SaveLibrary::open(&mut interrupted, &data).unwrap();
        interrupted.fail_at = Some(interrupted.operations + stage);
        let result = interrupted_library.delete(&mut interrupted, &data, first);
        assert!(result.is_err() || !result.unwrap().warnings.is_empty());
        interrupted.fail_at = None;
        let mut restarted = SaveLibrary::open(&mut interrupted, &data).unwrap();
        restarted.delete(&mut interrupted, &data, first).unwrap();
        assert!(restarted.entries().iter().all(|entry| entry.id != first));
        assert!(restarted.entries().iter().any(|entry| entry.id == second));
    }
    library.delete(&mut store, &data, first).unwrap();
    assert!(library.delete(&mut store, &data, first).unwrap().replayed);
    assert!(library.entries().iter().all(|entry| entry.id != first));
    assert!(library.entries().iter().any(|entry| entry.id == second));
    assert!(library.write(&mut store, &data, &overwrite).is_err());
    assert!(library
        .prepare_manual(
            &mut store,
            &data,
            &campaign,
            "No missing overwrite",
            Some(first)
        )
        .is_err());
}

fn save_intents_commit_in_preparation_order() {
    let (data, mut store, mut library, campaign) = fixture();
    let earlier = library
        .prepare_manual(&mut store, &data, &campaign, "Earlier intent", None)
        .unwrap();
    let later = library
        .prepare_manual(&mut store, &data, &campaign, "Later intent", None)
        .unwrap();
    let saved = library.write(&mut store, &data, &later).unwrap();
    let bytes = store.values.clone();
    let selection = library.last_successful();
    assert!(library
        .write(&mut store, &data, &earlier)
        .unwrap_err()
        .contains("newer save"));
    assert_eq!(store.values, bytes);
    assert_eq!(library.last_successful(), selection);
    let newest = library
        .prepare_manual(&mut store, &data, &campaign, "Newest intent", None)
        .unwrap();
    let newest_id = library.write(&mut store, &data, &newest).unwrap().id;
    let replay = library.write(&mut store, &data, &later).unwrap();
    assert!(replay.replayed);
    assert_eq!(replay.id, saved.id);
    library.delete(&mut store, &data, saved.id).unwrap();
    library.delete(&mut store, &data, newest_id).unwrap();
    let after_deletion = store.values.clone();
    assert!(library.write(&mut store, &data, &earlier).is_err());
    assert_eq!(store.values, after_deletion);
}

fn stale_unattempted_overwrite_preserves_a_newer_save() {
    let (data, mut store, mut library, mut campaign) = fixture();
    let target = manual(&mut library, &mut store, &data, &campaign);
    let stale = library
        .prepare_manual(&mut store, &data, &campaign, "Older request", Some(target))
        .unwrap();
    round(&mut campaign, &data);
    let newer = library
        .prepare_manual(&mut store, &data, &campaign, "Newer request", Some(target))
        .unwrap();
    library.write(&mut store, &data, &newer).unwrap();
    let bytes = store.values.clone();
    let selected = library.last_successful();
    assert!(library.write(&mut store, &data, &stale).is_err());
    assert_eq!(store.values, bytes);
    assert_eq!(library.last_successful(), selected);
    assert_eq!(library.load(&mut store, &data, target).unwrap(), campaign);
    let retry = library.write(&mut store, &data, &newer).unwrap();
    assert_eq!(retry.id, target);
    assert!(retry.replayed);
}

#[test]
fn compatibility_imports_preserve_originals_and_invalid_loads_preserve_live_play() {
    valid_load_survives_unavailable_continue_storage();
    let (data, mut store, mut library, campaign) = fixture();
    let good = manual(&mut library, &mut store, &data, &campaign);
    let mut live = GameState::default();
    live.load_campaign(campaign.clone(), &data).unwrap();
    let shell = Campaign::Shell(ShellCampaign {
        version: 1,
        turn: 37,
    });
    let original_shell = encode_slot("kestrum_campaign_v1", &shell, "1.0.0").unwrap();
    assert_eq!(load_legacy(&original_shell, &data).unwrap(), shell);
    assert!(library
        .import_legacy(&mut store, &data, &original_shell, "Legacy atlas")
        .is_err());
    let original_strategic = encode_slot("kestrum_strategic_v2", &campaign, "2").unwrap();
    let preserved_bytes = original_strategic.clone();
    let imported = library
        .import_legacy(&mut store, &data, &original_strategic, "Imported kingdom")
        .unwrap();
    assert_ne!(
        imported.metadata.campaign_id,
        campaign.strategic().unwrap().campaign_id
    );
    let import_id = library.write(&mut store, &data, &imported).unwrap().id;
    let migrated = library.load(&mut store, &data, import_id).unwrap();
    assert_eq!(
        migrated.strategic().unwrap().rng,
        campaign.strategic().unwrap().rng
    );
    assert_eq!(original_strategic, preserved_bytes);
    library
        .load_into(&mut store, &data, good, &mut live)
        .unwrap();
    store.values.insert(
        format!("{SAVE_NAMESPACE}_continue"),
        serde_json::to_string(&Some(ContinueSelection::Indexed(good))).unwrap(),
    );
    library.refresh(&mut store, &data).unwrap();
    assert_eq!(
        library.last_successful(),
        Some(ContinueSelection::Indexed(good))
    );
    let selection = library.last_successful();
    let valid_bytes = serde_json::to_string(&campaign).unwrap();
    let payload_key = store
        .values
        .iter()
        .find(|(_, value)| **value == valid_bytes)
        .unwrap()
        .0
        .clone();
    for corruption in [
        "{".to_owned(),
        valid_bytes.replacen("\"version\":2", "\"version\":999", 1),
    ] {
        store.values.insert(payload_key.clone(), corruption.clone());
        assert!(library
            .load_into(&mut store, &data, good, &mut live)
            .is_err());
        assert_eq!(live.campaign.as_ref(), Some(&campaign));
        assert_eq!(library.last_successful(), selection);
        assert_eq!(store.values[&payload_key], corruption);
    }
    store.values.insert(payload_key, valid_bytes);
    let unsupported_envelope =
        original_strategic.replacen("\"version\":\"2\"", "\"version\":\"999\"", 1);
    assert!(load_legacy(&unsupported_envelope, &data).is_err());
    library.remember_legacy_shell(&mut store).unwrap();
    let restarted = SaveLibrary::open(&mut store, &data).unwrap();
    assert_eq!(
        restarted.last_successful(),
        Some(ContinueSelection::LegacyShell)
    );
}

fn valid_load_survives_unavailable_continue_storage() {
    let (data, mut store, mut library, campaign) = fixture();
    let saved = manual(&mut library, &mut store, &data, &campaign);
    let mut current = campaign.clone();
    round(&mut current, &data);
    let mut state = GameState::default();
    for busy in [true, false] {
        state.load_campaign(current.clone(), &data).unwrap();
        store.busy = busy;
        if !busy {
            store.fail_at = Some(store.operations + 1);
        }
        let bytes = store.values.clone();
        let selection = library.last_successful();
        library
            .load_into(&mut store, &data, saved, &mut state)
            .unwrap();
        assert_eq!(state.campaign.as_ref(), Some(&campaign));
        assert_eq!(library.last_successful(), selection);
        assert_eq!(store.values, bytes);
        assert!(library
            .take_warnings()
            .iter()
            .any(|warning| warning.contains("Continue could not be updated")));
    }
}

#[test]
fn resumed_phases_and_branched_rounds_preserve_rng_without_reapplying_boundaries() {
    let (data, mut store, mut library, mut campaign) = fixture();
    let starting = campaign.clone();
    let Campaign::Strategic(strategic) = &mut campaign else {
        panic!("strategic fixture")
    };
    apply(strategic, &data, Actor::Player, Command::EndTurn).unwrap();
    apply(strategic, &data, Actor::Player, Command::SetNpcPaused(true)).unwrap();
    let stable_npc = campaign.clone();
    let before = store.values.clone();
    assert!(library
        .prepare_manual(&mut store, &data, &campaign, "Off turn", None)
        .is_err());
    assert_eq!(store.values, before);
    let raw = encode_slot("old_stable_phase", &campaign, "2").unwrap();
    let prepared = library
        .import_legacy(&mut store, &data, &raw, "Paused phase")
        .unwrap();
    let saved = library.write(&mut store, &data, &prepared).unwrap();
    let resumed = library.load(&mut store, &data, saved.id).unwrap();
    let actual = resumed.strategic().unwrap();
    let expected = stable_npc.strategic().unwrap();
    assert_eq!(actual.phase, expected.phase);
    assert_eq!(actual.acted, expected.acted);
    assert_eq!(actual.pending_facts, expected.pending_facts);
    assert_eq!(actual.rng, expected.rng);
    campaign = starting.clone();
    round(&mut campaign, &data);
    let first = library
        .prepare_checkpoint(&mut store, &data, &campaign)
        .unwrap();
    let first_id = library.write(&mut store, &data, &first).unwrap().id;
    let mut branch = starting;
    let Campaign::Strategic(branch_state) = &mut branch else {
        panic!("strategic fixture")
    };
    apply(branch_state, &data, Actor::Player, Command::EndTurn).unwrap();
    apply(
        branch_state,
        &data,
        Actor::Player,
        Command::SetNpcPaused(true),
    )
    .unwrap();
    for _ in 0..3 {
        apply(branch_state, &data, Actor::Player, Command::StepNpc).unwrap();
    }
    let second = library
        .prepare_checkpoint(&mut store, &data, &branch)
        .unwrap();
    let second_id = library.write(&mut store, &data, &second).unwrap().id;
    assert_ne!(first_id, second_id);
    assert_ne!(
        campaign.strategic().unwrap().accepted_sequence,
        branch.strategic().unwrap().accepted_sequence
    );
    assert_eq!(library.load(&mut store, &data, first_id).unwrap(), campaign);
    assert_eq!(library.load(&mut store, &data, second_id).unwrap(), branch);
    assert_eq!(campaign.strategic().unwrap().completed_rounds, 1);
    assert_eq!(branch.strategic().unwrap().completed_rounds, 1);
}
