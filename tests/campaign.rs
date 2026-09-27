//! Title, explicit shell compatibility, chronology and invalid-load regressions.

use kestrum::{
    data::GameData,
    engine::RuleError,
    state::{Campaign, GameState, Overlay, Screen, ShellCampaign, SAVE_VERSION},
};
use macroquad_toolkit::{
    persistence::{decode_slot_with_migration, encode_slot},
    rng::SeededRng,
};

fn round_trip(campaign: &Campaign) -> Campaign {
    let encoded = serde_json::to_string(campaign).unwrap();
    let decoded: Campaign = serde_json::from_str(&encoded).unwrap();
    assert_eq!(&decoded, campaign);
    let version = if campaign.strategic().is_some() {
        "2"
    } else {
        "1"
    };
    let envelope = encode_slot("round_trip", campaign, version).unwrap();
    let restored: Campaign = decode_slot_with_migration(&envelope, version, |_, _| {
        Err("A same-version payload must not request migration".into())
    })
    .unwrap();
    assert_eq!(&restored, campaign);
    restored
}

#[test]
fn new_campaign_starts_from_the_title_and_resets_chronology() {
    let data = GameData::load().unwrap();
    let mut state = GameState::default();
    assert_eq!(state.screen, Screen::Title);
    assert!(state.campaign.is_none());
    state.new_game(&data).unwrap();
    assert_eq!(state.screen, Screen::Campaign);
    assert_eq!(state.overlay, Overlay::None);
    assert_eq!(state.campaign.as_ref().unwrap().display_turn(), 1);
    state.end_turn(&data).unwrap();
    state.new_game(&data).unwrap();
    assert_eq!(
        state
            .campaign
            .unwrap()
            .strategic()
            .unwrap()
            .accepted_sequence,
        0
    );
}

#[test]
fn seasons_advance_only_from_unobstructed_campaign_play() {
    let data = GameData::load().unwrap();
    let mut state = GameState::default();
    assert!(state.end_turn(&data).is_err());
    state.new_game(&data).unwrap();
    let untouched = state.campaign.clone();
    for overlay in [
        Overlay::Menu,
        Overlay::Settings,
        Overlay::Help,
        Overlay::ConfirmNew,
        Overlay::Credits,
    ] {
        state.overlay = overlay;
        assert_eq!(state.end_turn(&data), Err(RuleError::PlayObstructed));
        assert_eq!(state.campaign, untouched);
    }
    state.overlay = Overlay::None;
    for (season, year) in [(0, 1), (1, 1), (2, 1), (3, 1), (0, 2)] {
        let campaign = state.campaign.as_ref().unwrap();
        assert_eq!((campaign.season_index(), campaign.year(1)), (season, year));
        state.end_turn(&data).unwrap();
        let mut commands = 0;
        while matches!(
            state.campaign.as_ref().unwrap().strategic().unwrap().phase,
            kestrum::state::CampaignPhase::NpcTurn { .. }
        ) {
            state.advance_npc(&data).unwrap();
            commands += 1;
            assert!(commands <= 195);
        }
    }
}

#[test]
fn closing_nested_menus_preserves_the_campaign() {
    let data = GameData::load().unwrap();
    let mut state = GameState::default();
    state.new_game(&data).unwrap();
    state.end_turn(&data).unwrap();
    let campaign = state.campaign.clone();
    state.overlay = Overlay::Help;
    state.back();
    assert_eq!(state.overlay, Overlay::Menu);
    state.back();
    assert_eq!(state.overlay, Overlay::None);
    state.main_menu();
    assert_eq!(state.screen, Screen::Title);
    assert_eq!(state.campaign, campaign);
    assert!(state.advance_npc(&data).is_err());
}

#[test]
fn saved_campaign_round_trips_and_invalid_loads_preserve_current_play() {
    let data = GameData::load().unwrap();
    let original: Campaign = serde_json::from_str(r#"{"version":1,"turn":37}"#).unwrap();
    assert!(matches!(original, Campaign::Shell(_)));
    let restored = round_trip(&original);
    let mut state = GameState::default();
    state.load_campaign(restored, &data).unwrap();
    for invalid in [
        Campaign::Shell(ShellCampaign {
            version: 90,
            turn: 2,
        }),
        Campaign::Shell(ShellCampaign {
            version: SAVE_VERSION,
            turn: 0,
        }),
    ] {
        assert!(state.load_campaign(invalid, &data).is_err());
        assert_eq!(state.campaign.as_ref().unwrap(), &original);
    }
    let mut candidates = Vec::new();
    state.new_game(&data).unwrap();
    let strategic = state
        .campaign
        .as_ref()
        .unwrap()
        .strategic()
        .unwrap()
        .clone();
    let wrapped = Campaign::Strategic(Box::new(strategic.clone()));
    let restored_strategic = round_trip(&wrapped);
    state.load_campaign(restored_strategic, &data).unwrap();
    let mut full_width = strategic.clone();
    full_width.campaign_id.0 = u64::MAX;
    full_width.seed = u64::MAX;
    full_width.rng.generation = SeededRng::from_state(u64::MAX);
    round_trip(&Campaign::Strategic(Box::new(full_width)))
        .validate(&data)
        .unwrap();
    for malformed in [
        r#"{}"#,
        r#"{"version":90,"turn":37}"#,
        r#"{"version":1,"turn":37,"unexpected":true}"#,
        r#"{"version":2,"turn":37}"#,
    ] {
        assert!(serde_json::from_str::<Campaign>(malformed).is_err());
    }
    for variant in 0..7 {
        let mut invalid = strategic.clone();
        match variant {
            0 => invalid.version += 1,
            1 => invalid.content_version += 1,
            2 => invalid.next_ids.site.0 = 1,
            3 => {
                invalid
                    .factions
                    .get_mut(&invalid.player)
                    .unwrap()
                    .resources
                    .gold = -1
            }
            4 => invalid.world.routes[0].to.0 = u32::MAX,
            5 => invalid.acted.insert(invalid.player).then_some(()).unwrap(),
            _ => {
                invalid.world.routes[0].road.improved = false;
                invalid.world.routes[0].road.damage = 1;
            }
        }
        candidates.push(Campaign::Strategic(Box::new(invalid)));
    }
    let valid = state.campaign.clone();
    state.overlay = Overlay::Menu;
    for invalid in candidates {
        assert!(state.load_campaign(invalid, &data).is_err());
        assert_eq!(state.campaign, valid);
        assert_eq!(state.overlay, Overlay::Menu);
    }
}

#[test]
fn legacy_chronology_is_read_only_and_cannot_wrap() {
    let data = GameData::load().unwrap();
    let original = Campaign::Shell(ShellCampaign {
        version: SAVE_VERSION,
        turn: u32::MAX,
    });
    let mut state = GameState::default();
    state.load_campaign(original.clone(), &data).unwrap();
    assert_eq!(state.end_turn(&data), Err(RuleError::LegacyReadOnly));
    assert_eq!(state.campaign.as_ref(), Some(&original));
    assert!(original.validate(&data).is_ok());
    assert_eq!(original.display_turn(), u32::MAX);
}
