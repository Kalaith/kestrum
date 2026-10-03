//! Deterministic headless production runs use the same player and NPC engine paths as play.

use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use kestrum::{
    data::{generation::ProductionSetup, rules::Emblem, GameData},
    engine::{self, apply, Actor, Command},
    state::{persistence::load_legacy, Campaign, CampaignPhase, StrategicCampaign},
};
use macroquad_toolkit::persistence::encode_slot;

const CHECKPOINTS: [u32; 2] = [200, 400];

#[derive(Debug, Clone, Default)]
struct Peaks {
    people: usize,
    alive_people: usize,
    households: usize,
    families: usize,
    armies: usize,
    formations: usize,
    items: usize,
    history_events: usize,
    battles: usize,
    population: u64,
}

struct CheckpointReport<'a> {
    factions: usize,
    rounds: u32,
    elapsed: Duration,
    round_times: &'a [Duration],
    npc_phase_times: &'a [Duration],
    npc_action_times: &'a [Duration],
    peaks: &'a Peaks,
    save_bytes: usize,
    reload_time: Duration,
}

#[derive(Debug)]
pub(super) struct RunMetrics {
    pub(super) factions: usize,
    pub(super) rounds: u32,
    pub(super) state: StrategicCampaign,
}

pub(super) fn campaign(data: &GameData, factions: usize, seed: u64) -> StrategicCampaign {
    StrategicCampaign::new_production(
        data,
        &ProductionSetup {
            kingdom_name: "Briarhold".into(),
            emblem: Emblem::Rose,
            factions,
            seed,
        },
    )
    .unwrap()
}

pub(super) fn reload(campaign: &StrategicCampaign, data: &GameData) -> StrategicCampaign {
    let raw = encode_slot(
        "kestrum_strategic_v2",
        &Campaign::Strategic(Box::new(campaign.clone())),
        "2",
    )
    .unwrap();
    load_legacy(&raw, data)
        .unwrap()
        .strategic()
        .unwrap()
        .clone()
}

pub(super) fn run_to_round(
    mut campaign: StrategicCampaign,
    data: &GameData,
    target: u32,
    replay_at_200: bool,
) -> RunMetrics {
    let factions = campaign.factions.len();
    let run_started = Instant::now();
    let mut round_times = Vec::new();
    let mut npc_phase_times = Vec::new();
    let mut npc_action_times = Vec::new();
    let mut peaks = Peaks::default();
    let mut checkpoint_sizes = BTreeMap::new();

    while campaign.completed_rounds < target {
        let before = campaign.completed_rounds;
        let round_started = Instant::now();
        begin_player_round(&mut campaign, data);
        let mut active_npc = None;
        let mut active_npc_started = None;
        let mut action_count = 0;

        while let CampaignPhase::NpcTurn { faction, .. } = campaign.phase {
            if active_npc != Some(faction) {
                finish_npc_phase(active_npc, active_npc_started, &mut npc_phase_times);
                active_npc = Some(faction);
                active_npc_started = Some(Instant::now());
            }
            let action_started = Instant::now();
            if let Err(error) = engine::advance_npc(&mut campaign, data) {
                let decision = engine::ai::propose(&campaign, data, faction)
                    .map(|decision| format!("{:?}", decision.command))
                    .unwrap_or_else(|proposal| format!("proposal error: {proposal}"));
                panic!(
                    "NPC phase failed at round {before}, faction {faction:?}, decision {decision}: {error}; successors={:?}",
                    campaign.successors
                );
            }
            start_player_pending_battle(&mut campaign, data);
            npc_action_times.push(action_started.elapsed());
            action_count += 1;
            assert!(action_count <= 195, "NPC turn failed to reach a boundary");
            assert!(
                campaign.diplomacy.ending.is_none(),
                "campaign ended before the continuity milestone at round {}",
                campaign.completed_rounds
            );
        }
        finish_npc_phase(active_npc, active_npc_started, &mut npc_phase_times);
        let boundary = campaign.completed_rounds;
        assert_eq!(
            boundary,
            before + 1,
            "the round did not resolve exactly once"
        );
        assert_eq!(
            campaign.diplomacy.ending.as_ref().map(|ending| ending.kind),
            None,
            "peaceful test setup unexpectedly ended"
        );
        campaign.validate(data).unwrap();
        round_times.push(round_started.elapsed());
        observe(&campaign, &mut peaks);

        if CHECKPOINTS.contains(&boundary) && boundary <= target {
            let save_started = Instant::now();
            let raw = encode_slot(
                "kestrum_strategic_v2",
                &Campaign::Strategic(Box::new(campaign.clone())),
                "2",
            )
            .unwrap();
            let restored = load_legacy(&raw, data).unwrap();
            assert_eq!(restored.strategic(), Some(&campaign));
            checkpoint_sizes.insert(boundary, raw.len());
            let reload_time = save_started.elapsed();
            report_checkpoint(CheckpointReport {
                factions,
                rounds: boundary,
                elapsed: run_started.elapsed(),
                round_times: &round_times,
                npc_phase_times: &npc_phase_times,
                npc_action_times: &npc_action_times,
                peaks: &peaks,
                save_bytes: raw.len(),
                reload_time,
            });

            if replay_at_200 && boundary == 200 && target > boundary {
                let mut resumed = restored.strategic().unwrap().clone();
                let uninterrupted_started = Instant::now();
                run_one_round(&mut campaign, data);
                let uninterrupted_replay = uninterrupted_started.elapsed();
                let resumed_started = Instant::now();
                run_one_round(&mut resumed, data);
                assert_eq!(campaign, resumed, "saved replay diverged after round 200");
                println!(
                    "K18_REPLAY checkpoint=200 uninterrupted_ms={} resumed_ms={}",
                    ms(uninterrupted_replay),
                    ms(resumed_started.elapsed())
                );
                observe(&campaign, &mut peaks);
            }
        }
    }

    assert_eq!(campaign.completed_rounds, target);
    assert!(campaign.diplomacy.ending.is_none());
    assert_eq!(campaign.world.markers.len(), 152);
    assert_eq!(campaign.world.sites.len(), 152);
    assert_eq!(campaign.factions.len(), factions);
    assert!(campaign.history.events.len() <= data.history.detail_max_entries);
    assert!(campaign
        .history
        .person_notables
        .values()
        .all(|rows| rows.len() <= data.history.notable_max_entries));
    assert!(
        campaign.next_ids.person.0 > campaign.people.keys().map(|id| id.0).max().unwrap_or(0),
        "person ID allocator fell behind retained people"
    );
    assert!(
        campaign.next_ids.history.0
            > campaign
                .history
                .events
                .keys()
                .map(|id| id.0)
                .max()
                .unwrap_or(0),
        "history ID allocator fell behind retained records"
    );
    assert!(checkpoint_sizes.values().all(|size| *size > 0));

    RunMetrics {
        factions,
        rounds: campaign.completed_rounds,
        state: campaign,
    }
}

fn begin_player_round(campaign: &mut StrategicCampaign, data: &GameData) {
    assert!(matches!(campaign.phase, CampaignPhase::PlayerTurn));
    renew_continuity_truces(campaign, data);
    apply(campaign, data, Actor::Player, Command::EndTurn).unwrap();
}

fn run_one_round(campaign: &mut StrategicCampaign, data: &GameData) {
    begin_player_round(campaign, data);
    let start_round = campaign.completed_rounds;
    let mut actions = 0;
    while let CampaignPhase::NpcTurn { .. } = campaign.phase {
        engine::advance_npc(campaign, data).unwrap();
        start_player_pending_battle(campaign, data);
        actions += 1;
        assert!(actions <= 195);
    }
    assert_eq!(campaign.completed_rounds, start_round + 1);
    campaign.validate(data).unwrap();
}

fn start_player_pending_battle(campaign: &mut StrategicCampaign, data: &GameData) {
    let Some(pending) = campaign.pending_battle.as_ref() else {
        return;
    };
    if !pending
        .report
        .participant_factions()
        .any(|faction| faction == campaign.player)
    {
        return;
    }
    apply(campaign, data, Actor::Player, Command::StartPendingBattle).unwrap();
}

fn renew_continuity_truces(campaign: &mut StrategicCampaign, data: &GameData) {
    let truce_until = campaign
        .completed_rounds
        .checked_add(data.diplomacy.truce_rounds)
        .expect("continuity truce date remains in range");
    for relation in &campaign.relations {
        if relation.state != kestrum::data::world::DiplomaticState::Peace {
            continue;
        }
        let pair = campaign
            .diplomacy
            .pairs
            .iter_mut()
            .find(|pair| pair.factions == relation.factions)
            .expect("every campaign relation has a diplomacy timer");
        pair.peace_since = Some(campaign.completed_rounds);
        pair.truce_until = Some(truce_until);
        pair.last_offer_round = Some(campaign.completed_rounds);
    }
    campaign.validate(data).unwrap();
}

fn finish_npc_phase(
    faction: Option<kestrum::data::world::FactionId>,
    started: Option<Instant>,
    samples: &mut Vec<Duration>,
) {
    if let (Some(_), Some(started)) = (faction, started) {
        samples.push(started.elapsed());
    }
}

fn observe(campaign: &StrategicCampaign, peaks: &mut Peaks) {
    peaks.people = peaks.people.max(campaign.people.len());
    peaks.alive_people = peaks.alive_people.max(
        campaign
            .people
            .values()
            .filter(|person| person.is_alive())
            .count(),
    );
    peaks.households = peaks.households.max(campaign.households.len());
    peaks.families = peaks.families.max(campaign.families.len());
    peaks.armies = peaks.armies.max(campaign.armies.len());
    peaks.formations = peaks.formations.max(campaign.formations.len());
    peaks.items = peaks.items.max(campaign.legacy_items.len());
    peaks.history_events = peaks.history_events.max(campaign.history.events.len());
    peaks.battles = peaks.battles.max(campaign.battles.len());
    peaks.population = peaks.population.max(
        campaign
            .world
            .population
            .values()
            .map(|value| *value as u64)
            .sum(),
    );
}

fn report_checkpoint(report: CheckpointReport<'_>) {
    println!(
        "K18_METRICS factions={} rounds={} elapsed_ms={} round_ms={} npc_phase_ms={} npc_action_ms={} peaks={:?} save_bytes={} save_roundtrip_ms={}",
        report.factions,
        report.rounds,
        ms(report.elapsed),
        distribution(report.round_times),
        distribution(report.npc_phase_times),
        distribution(report.npc_action_times),
        report.peaks,
        report.save_bytes,
        ms(report.reload_time),
    );
}

fn distribution(values: &[Duration]) -> String {
    if values.is_empty() {
        return "n/a".into();
    }
    let mut values = values.iter().map(|value| ms(*value)).collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    let percentile = |fraction: f64| {
        let index = ((values.len() - 1) as f64 * fraction).ceil() as usize;
        values[index]
    };
    format!(
        "median:{:.3},p95:{:.3},max:{:.3}",
        percentile(0.50),
        percentile(0.95),
        values[values.len() - 1]
    )
}

fn ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}
