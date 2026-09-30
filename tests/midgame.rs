//! The native review save is a progressed, resumable production campaign.

#[path = "../examples/prepare_midgame/campaign.rs"]
mod review_campaign;

use kestrum::{
    data::GameData,
    engine::{apply, Actor, Command},
    state::{Campaign, CampaignPhase, StrategicCampaign},
};
use std::sync::OnceLock;

fn fixture() -> (GameData, StrategicCampaign) {
    static CAMPAIGN: OnceLock<StrategicCampaign> = OnceLock::new();
    let data = GameData::load().unwrap();
    let campaign = CAMPAIGN
        .get_or_init(|| review_campaign::generate(&data).unwrap())
        .clone();
    (data, campaign)
}

#[test]
fn review_save_has_real_progress_and_a_living_player_army() {
    let (data, campaign) = fixture();
    campaign.validate(&data).unwrap();
    assert_eq!(campaign.completed_rounds, 60);
    assert_eq!(campaign.phase, CampaignPhase::PlayerTurn);
    assert!(campaign.diplomacy.ending.is_none());
    assert!(!campaign.battles.is_empty());
    assert!(campaign
        .battle_templates
        .iter()
        .any(|template| template.name == "Homeward Line"));
    assert!(campaign
        .formations
        .values()
        .any(|formation| formation.faction == campaign.player && formation.headcount > 0));
}

#[test]
fn progressed_save_roundtrips_with_an_immutable_battle_history() {
    let (data, campaign) = fixture();
    let snapshot = Campaign::Strategic(Box::new(campaign.clone()));
    let raw = serde_json::to_string(&snapshot).unwrap();
    let restored: Campaign = serde_json::from_str(&raw).unwrap();
    assert_eq!(restored, snapshot);
    let mut restored = restored.strategic().unwrap().clone();
    let history = restored.battles.clone();
    assert!(apply(
        &mut restored,
        &data,
        Actor::Player,
        Command::StartPendingBattle
    )
    .is_err());
    assert_eq!(restored, campaign);
    apply(&mut restored, &data, Actor::Player, Command::EndTurn).unwrap();
    assert_eq!(restored.battles, history);
    restored.validate(&data).unwrap();
}
