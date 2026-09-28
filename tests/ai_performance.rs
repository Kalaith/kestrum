//! Release profiling of the real production decision boundary.
use std::time::{Duration, Instant};

use kestrum::{
    data::{generation::ProductionSetup, rules::Emblem, GameData},
    engine::{self, Actor, Command},
    state::{CampaignPhase, StrategicCampaign},
};

fn measure(mut operation: impl FnMut(), repetitions: u32) -> Duration {
    let start = Instant::now();
    for _ in 0..repetitions {
        operation();
    }
    start.elapsed() / repetitions
}

#[test]
#[ignore = "release profiling; run explicitly with --release --ignored --nocapture"]
fn production_decision_costs() {
    let data = GameData::load().unwrap();
    for factions in [4, 8] {
        let mut campaign = StrategicCampaign::new_production(
            &data,
            &ProductionSetup {
                kingdom_name: "K18 Profile".into(),
                emblem: Emblem::Rose,
                factions,
                seed: 180_018,
            },
        )
        .unwrap();
        engine::apply(&mut campaign, &data, Actor::Player, Command::EndTurn).unwrap();
        let faction = campaign.active_faction();
        let decision = engine::ai::propose(&campaign, &data, faction).unwrap();
        let data_validation = measure(|| data.validate().unwrap(), 20);
        let state_validation = measure(|| campaign.validate(&data).unwrap(), 20);
        let projection = measure(
            || {
                engine::project(&campaign, faction).unwrap();
            },
            20,
        );
        let proposal = measure(
            || {
                assert_eq!(
                    engine::ai::propose(&campaign, &data, faction).unwrap(),
                    decision
                );
            },
            20,
        );
        let application = measure(
            || {
                let mut candidate = campaign.clone();
                engine::apply(
                    &mut candidate,
                    &data,
                    Actor::Npc(faction),
                    decision.command.clone(),
                )
                .unwrap();
            },
            20,
        );
        println!("K18_PROFILE factions={factions} data_validation={data_validation:?} state_validation={state_validation:?} projection={projection:?} proposal={proposal:?} application={application:?} command={:?}", decision.command);
        profile_rounds(campaign, &data, factions);
    }
}

fn profile_rounds(mut campaign: StrategicCampaign, data: &GameData, factions: usize) {
    let mut proposals = Vec::new();
    let mut applications = Vec::new();
    let mut validations = Vec::new();
    let mut slowest = (Duration::ZERO, String::new());
    while campaign.completed_rounds < 40 {
        if campaign.phase == CampaignPhase::PlayerTurn {
            engine::apply(&mut campaign, data, Actor::Player, Command::EndTurn).unwrap();
        }
        let faction = campaign.active_faction();
        let start = Instant::now();
        campaign.validate(data).unwrap();
        validations.push(start.elapsed());
        let start = Instant::now();
        let decision = engine::ai::propose(&campaign, data, faction).unwrap();
        let elapsed = start.elapsed();
        proposals.push(elapsed);
        if elapsed > slowest.0 {
            slowest = (
                elapsed,
                format!("round={} {:?}", campaign.completed_rounds, decision.command),
            );
        }
        let before = campaign.clone();
        let start = Instant::now();
        engine::apply(
            &mut campaign,
            data,
            Actor::Npc(faction),
            decision.command.clone(),
        )
        .unwrap();
        engine::ai::accepted(&mut campaign, &before, data, faction, &decision).unwrap();
        applications.push(start.elapsed());
    }
    println!("K18_PROFILE_40 factions={factions} samples={} proposal={} application={} validation={} slowest_proposal={slowest:?}", proposals.len(), distribution(proposals), distribution(applications), distribution(validations));
}

fn distribution(mut samples: Vec<Duration>) -> String {
    samples.sort_unstable();
    format!(
        "median:{:?},p95:{:?},max:{:?}",
        samples[samples.len() / 2],
        samples[samples.len() * 95 / 100],
        samples.last().unwrap()
    )
}
