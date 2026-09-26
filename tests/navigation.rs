//! Atlas bounds and input coordinates remain consistent across pan and zoom.

use kestrum::navigation::{MapView, HEIGHT, MAP_RECT, WIDTH};
use macroquad::prelude::vec2;
use macroquad_toolkit::input::gestures::TouchGestureFrame;

#[test]
fn full_map_is_visible_and_cannot_be_dragged_offscreen() {
    let mut view = MapView::default();
    view.pan(vec2(9000.0, -8000.0));
    assert_eq!(view.project(vec2(0.0, 0.0)), vec2(0.0, 0.0));
    assert_eq!(view.project(vec2(WIDTH, HEIGHT)), vec2(WIDTH, HEIGHT));
}

#[test]
fn zoom_anchor_stays_on_the_same_geography() {
    let mut view = MapView::default();
    let anchor = vec2(800.0, 400.0);
    let world = view.camera.screen_to_world(MAP_RECT, anchor).unwrap();
    view.zoom(anchor, 2.0);
    assert!(view.project(world).distance(anchor) < 0.001);
}

#[test]
fn extreme_pans_keep_all_viewport_edges_covered() {
    let mut view = MapView::default();
    view.zoom(vec2(640.0, 360.0), 2.0);
    for delta in [vec2(-9999.0, 9999.0), vec2(9999.0, -9999.0)] {
        view.pan(delta);
        let top_left = view.project(vec2(0.0, 0.0));
        let bottom_right = view.project(vec2(WIDTH, HEIGHT));
        assert!(top_left.x <= 0.001 && top_left.y <= 0.001);
        assert!(bottom_right.x >= WIDTH && bottom_right.y >= HEIGHT);
    }
}

#[test]
fn pinch_is_bounded_and_recenter_restores_the_whole_atlas() {
    let mut view = MapView::default();
    view.gesture(&TouchGestureFrame {
        claimed: true,
        scale: 50.0,
        center: vec2(640.0, 360.0),
        ..Default::default()
    });
    assert_eq!(view.camera.zoom(), 3.0);
    view.reset();
    assert_eq!(view.camera.zoom(), 1.0);
    view.zoom(vec2(640.0, 360.0), 0.0001);
    assert_eq!(view.camera.zoom(), 1.0);
}

#[test]
fn unclaimed_touch_does_not_move_the_map() {
    let mut view = MapView::default();
    let before = view.camera;
    view.gesture(&TouchGestureFrame {
        pan: vec2(100.0, 50.0),
        scale: 2.0,
        ..Default::default()
    });
    assert_eq!(view.camera, before);
}
