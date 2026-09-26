//! Bounded atlas navigation in logical pixels, shared by mouse and touch.

use macroquad::prelude::{vec2, Rect, Vec2};
use macroquad_toolkit::camera::{CameraBounds, CameraBoundsPolicy, CameraTransform};
use macroquad_toolkit::input::gestures::TouchGestureFrame;

pub const WIDTH: f32 = 1280.0;
pub const HEIGHT: f32 = 720.0;
pub const MAP_RECT: Rect = Rect::new(0.0, 0.0, WIDTH, HEIGHT);
pub const ZOOM_LIMITS: (f32, f32) = (1.0, 3.0);

pub struct MapView {
    pub camera: CameraTransform,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            camera: CameraTransform::new(vec2(WIDTH / 2.0, HEIGHT / 2.0), 1.0)
                .expect("fixed Kestrum map dimensions are valid"),
        }
    }
}

impl MapView {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.camera.pan_screen(delta);
        self.constrain();
    }

    pub fn zoom(&mut self, anchor: Vec2, factor: f32) {
        self.camera.zoom_at(MAP_RECT, anchor, factor, ZOOM_LIMITS);
        self.constrain();
    }

    pub fn gesture(&mut self, frame: &TouchGestureFrame) {
        self.camera.apply_gesture(MAP_RECT, frame, ZOOM_LIMITS);
        self.constrain();
    }

    pub fn project(&self, point: Vec2) -> Vec2 {
        self.camera
            .world_to_screen(MAP_RECT, point)
            .unwrap_or(point)
    }

    fn constrain(&mut self) {
        // Restrict the center by the visible half-extents: no blank edges at any zoom.
        let half = vec2(WIDTH, HEIGHT) / (2.0 * self.camera.zoom());
        self.camera.constrain(
            MAP_RECT,
            CameraBounds::new(half, vec2(WIDTH, HEIGHT) - half),
            CameraBoundsPolicy::TargetInside,
        );
    }
}
