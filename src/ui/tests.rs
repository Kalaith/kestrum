use super::*;

#[test]
fn map_selection_round_trips_after_pan_and_zoom() {
    let data = GameData::load().unwrap();
    let session = GameSession::new(&data.config);
    let ui = VirtualUi::from_screen_size(LOGICAL_WIDTH, LOGICAL_HEIGHT, 1920.0, 1080.0);
    let viewport = world_grid_rect();
    let mut camera = CameraTransform::new(vec2(40.0, -30.0), 1.0).unwrap();
    camera.pan_screen(vec2(25.0, 15.0));
    camera.zoom_at(viewport, viewport.center(), 1.4, (0.75, 1.75));
    let ctx = UiContext {
        data: &data,
        session: &session,
        save_exists: false,
        save_slots: &[],
        loaded_assets: 0,
        camera,
        paused: false,
        show_stats: false,
        dt: 0.0,
        ui: &ui,
    };
    let tile = TilePos::new(6, 4);
    let point = GridView::new(&ctx, viewport).tile_rect(tile).center();
    assert_eq!(selected_tile_at(&ctx, point), Some(tile));
    assert_eq!(selected_tile_at(&ctx, vec2(900.0, 300.0)), None);
    assert_eq!(selected_tile_at(&ctx, vec2(100.0, 570.0)), None);
}
