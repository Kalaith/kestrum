use super::*;

#[test]
fn embedded_data_loads() {
    let data = GameData::load().unwrap();

    assert!(!data.config.game_name.is_empty());
    assert!(data.actions.contains("gather"));
    assert!(data.config.world_width > 0);
    assert!(data.config.world_height > 0);
}
