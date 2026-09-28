//! Guide persistence and command receipts without fabricating simulation history.

use kestrum::{
    data::{world::SiteId, GameData},
    engine::{Command, MoveOrder},
    state::{
        military::ArmyId,
        tutorial::{TutorialProgress, TutorialStep, TUTORIAL_STEPS},
        Campaign, GameState, Overlay, StrategicCampaign,
    },
};

fn fixture() -> (GameData, GameState) {
    let mut data = GameData::load().unwrap();
    data.threats.initial.clear();
    let mut state = GameState::default();
    state.new_game(&data).unwrap();
    (data, state)
}

fn guide(state: &GameState) -> &TutorialProgress {
    &state
        .campaign
        .as_ref()
        .unwrap()
        .strategic()
        .unwrap()
        .tutorial
}

fn movement(destination: u32) -> Command {
    Command::Move(MoveOrder {
        armies: vec![ArmyId(1)],
        path: vec![SiteId(1), SiteId(destination)],
    })
}

#[test]
fn new_campaigns_start_guidance_and_older_saves_opt_in_without_other_changes() {
    let (data, state) = fixture();
    assert_eq!(guide(&state).current(), Some(TutorialStep::Headquarters));
    let original = state.campaign.unwrap().strategic().unwrap().clone();
    let mut old = serde_json::to_value(&original).unwrap();
    old.as_object_mut().unwrap().remove("tutorial");
    let mut loaded: StrategicCampaign = serde_json::from_value(old).unwrap();
    loaded.validate(&data).unwrap();
    assert_eq!(loaded.tutorial.current(), None);
    loaded.tutorial.reopen();
    assert_eq!(loaded, original);
}

#[test]
fn out_of_order_completion_is_idempotent_and_dismissal_does_not_complete_actions() {
    let mut progress = TutorialProgress::new();
    progress.record(TutorialStep::Career);
    progress.record(TutorialStep::Career);
    progress.record(TutorialStep::Headquarters);
    assert_eq!(progress.current(), Some(TutorialStep::Movement));
    progress.dismiss();
    assert_eq!(progress.current(), None);
    assert!(!progress.completed(TutorialStep::Movement));
    let mut loaded: TutorialProgress =
        serde_json::from_str(&serde_json::to_string(&progress).unwrap()).unwrap();
    loaded.reopen();
    assert_eq!(loaded.current(), Some(TutorialStep::Movement));
    for step in TUTORIAL_STEPS {
        loaded.record(step);
    }
    assert!(loaded.is_complete());
    assert_eq!(loaded.current(), None);
    loaded.reopen();
    assert_eq!(loaded.current(), Some(TutorialStep::Headquarters));
    assert!(!loaded.is_complete());
}

#[test]
fn invalid_and_obstructed_orders_never_advance_the_guide() {
    let (data, mut state) = fixture();
    let untouched = state.campaign.clone();
    assert!(state.command(&data, movement(999_999)).is_err());
    assert_eq!(state.campaign, untouched);
    state.overlay = Overlay::Help;
    assert!(state.command(&data, movement(5)).is_err());
    assert!(state.end_turn(&data).is_err());
    assert_eq!(state.campaign, untouched);
    assert!(!guide(&state).completed(TutorialStep::Movement));
    assert!(!guide(&state).completed(TutorialStep::FirstTurn));
}

#[test]
fn actual_movement_and_end_turn_are_saved_and_do_not_mark_other_lessons() {
    let (data, mut state) = fixture();
    state.overlay = Overlay::MoveReview;
    let result = state.command(&data, movement(5)).unwrap();
    assert_eq!(result.movement.unwrap().path, [SiteId(1), SiteId(5)]);
    assert!(guide(&state).completed(TutorialStep::Movement));
    state.overlay = Overlay::None;
    state.end_turn(&data).unwrap();
    assert!(guide(&state).completed(TutorialStep::FirstTurn));
    assert!(!guide(&state).completed(TutorialStep::Headquarters));
    assert!(!guide(&state).completed(TutorialStep::Records));
    let saved: Campaign =
        serde_json::from_str(&serde_json::to_string(state.campaign.as_ref().unwrap()).unwrap())
            .unwrap();
    let mut resumed = GameState::default();
    resumed.load_campaign(saved, &data).unwrap();
    assert_eq!(guide(&resumed), guide(&state));
    let before = resumed.campaign.clone();
    assert!(resumed.end_turn(&data).is_err());
    assert_eq!(resumed.campaign, before);
}

#[test]
fn guide_progress_is_local_to_the_saved_campaign() {
    let (data, mut state) = fixture();
    state.command(&data, movement(5)).unwrap();
    let previous = state.campaign.clone().unwrap();
    state.new_game(&data).unwrap();
    assert!(!guide(&state).completed(TutorialStep::Movement));
    state.load_campaign(previous, &data).unwrap();
    assert!(guide(&state).completed(TutorialStep::Movement));
}
