//! Title, chronology, and save boundary regressions for the empty campaign.

use kestrum::state::{Campaign, GameState, Overlay, Screen, SAVE_VERSION};

#[test]
fn new_campaign_starts_from_the_title_and_resets_chronology() {
    let mut state = GameState::default();
    assert_eq!(state.screen, Screen::Title);
    assert!(state.campaign.is_none());
    state.new_game();
    assert_eq!(state.screen, Screen::Campaign);
    assert_eq!(state.overlay, Overlay::None);
    assert_eq!(state.campaign.as_ref().unwrap().turn, 1);
    state.end_turn();
    state.new_game();
    assert_eq!(state.campaign.unwrap().turn, 1);
}

#[test]
fn seasons_advance_only_from_unobstructed_campaign_play() {
    let mut state = GameState::default();
    assert!(!state.end_turn());
    state.new_game();
    for overlay in [
        Overlay::Menu,
        Overlay::Settings,
        Overlay::Help,
        Overlay::ConfirmNew,
    ] {
        state.overlay = overlay;
        assert!(!state.end_turn());
    }
    state.overlay = Overlay::None;
    for (season, year) in [(0, 1), (1, 1), (2, 1), (3, 1), (0, 2)] {
        let campaign = state.campaign.as_ref().unwrap();
        assert_eq!((campaign.season_index(), campaign.year(1)), (season, year));
        assert!(state.end_turn());
    }
}

#[test]
fn closing_nested_menus_preserves_the_campaign() {
    let mut state = GameState::default();
    state.new_game();
    state.end_turn();
    state.overlay = Overlay::Help;
    state.back();
    assert_eq!(state.overlay, Overlay::Menu);
    state.back();
    assert_eq!(state.overlay, Overlay::None);
    state.main_menu();
    assert_eq!(state.screen, Screen::Title);
    assert_eq!(state.campaign.unwrap().turn, 2);
}

#[test]
fn saved_campaign_round_trips_and_invalid_loads_preserve_current_play() {
    let original = Campaign {
        version: SAVE_VERSION,
        turn: 37,
    };
    let encoded = serde_json::to_string(&original).unwrap();
    let restored: Campaign = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored, original);
    let mut state = GameState::default();
    state.load_campaign(restored).unwrap();
    for invalid in [
        Campaign {
            version: 90,
            turn: 2,
        },
        Campaign {
            version: SAVE_VERSION,
            turn: 0,
        },
    ] {
        assert!(state.load_campaign(invalid).is_err());
        assert_eq!(state.campaign.as_ref().unwrap(), &original);
    }
}

#[test]
fn turn_counter_does_not_wrap_to_an_invalid_save() {
    let mut campaign = Campaign {
        version: SAVE_VERSION,
        turn: u32::MAX,
    };
    campaign.end_turn();
    assert!(campaign.validate().is_ok());
    assert_eq!(campaign.turn, u32::MAX);
}
