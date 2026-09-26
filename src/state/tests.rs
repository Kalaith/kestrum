use super::*;

fn test_config() -> GameConfig {
    GameConfig {
        game_name: "game_template".to_owned(),
        display_name: "Game Template".to_owned(),
        save_slot: "autosave".to_owned(),
        version: "1.0.0".to_owned(),
        starting_points: 10,
        starting_energy: 5.0,
        max_energy: 10.0,
        energy_per_second: 1.0,
        world_width: 8,
        world_height: 8,
    }
}

#[test]
fn action_spends_energy_and_rewards_points() {
    let config = test_config();
    let action = ActionDef {
        id: "test".to_owned(),
        name: "Test".to_owned(),
        description: "Test action".to_owned(),
        energy_cost: 3.0,
        points_reward: 7,
    };
    let mut session = GameSession::new(&config);

    assert!(session.apply_action(&action));
    assert_eq!(session.player.points, 17);
    assert_eq!(session.player.turn, 2);
    assert!((session.player.energy - 2.0).abs() < f32::EPSILON);
}

#[test]
fn legacy_save_migrates_to_current_shape() {
    let config = test_config();
    let value = serde_json::json!({
        "points": 42,
        "energy": 99.0,
        "turn": 3
    });

    let migrated = migrate_save_value(Some("0.1.0".to_owned()), value, &config).unwrap();

    assert_eq!(migrated.version, "1.0.0");
    assert_eq!(migrated.player.points, 42);
    assert_eq!(migrated.player.energy, 10.0);
    assert_eq!(migrated.player.turn, 3);
}
