//! Independent world extents, bounded cameras and stable information bands.

use super::{MapScope, HEIGHT, MAP_RECT, WIDTH};
use crate::data::MapCameraSettings;
use macroquad::prelude::{vec2, Vec2};
use macroquad_toolkit::camera::{CameraBounds, CameraBoundsPolicy, CameraTransform};
use macroquad_toolkit::input::gestures::TouchGestureFrame;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MapScaleBand {
    Overview,
    #[default]
    Campaign,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapView {
    pub camera: CameraTransform,
    pub(crate) settings: MapCameraSettings,
    extent: Vec2,
    band: MapScaleBand,
    overview_return: Option<(CameraTransform, MapScaleBand)>,
}

impl Default for MapView {
    fn default() -> Self {
        Self::configured(&MapCameraSettings::default(), MapScope::World)
    }
}

impl MapView {
    pub fn configured(settings: &MapCameraSettings, scope: MapScope) -> Self {
        let extent = Vec2::from_array(match scope {
            MapScope::World => settings.world_extent,
            MapScope::Region(_) => settings.region_extent,
        });
        Self {
            camera: CameraTransform::new(extent * 0.5, settings.working_zoom)
                .expect("validated map camera settings"),
            settings: *settings,
            extent,
            band: MapScaleBand::Campaign,
            overview_return: None,
        }
    }

    pub fn extent(&self) -> Vec2 {
        self.extent
    }

    pub fn normalized_world(&self, position: [f32; 2]) -> Vec2 {
        Vec2::from_array(position) * self.extent
    }

    pub fn band(&self) -> MapScaleBand {
        self.band
    }

    pub fn working_zoom(&self) -> f32 {
        self.settings.working_zoom
    }

    pub fn zoom_limits(&self) -> (f32, f32) {
        (
            (WIDTH / self.extent.x).max(HEIGHT / self.extent.y),
            self.settings.maximum_zoom,
        )
    }

    pub fn overview_active(&self) -> bool {
        self.overview_return.is_some()
    }

    pub fn focus(&mut self, position: [f32; 2], zoom: f32) {
        if !position.iter().all(|value| value.is_finite()) || !zoom.is_finite() {
            return;
        }
        self.overview_return = None;
        self.set_focus(position, zoom);
    }

    pub fn focus_group(&mut self, position: [f32; 2]) {
        self.focus(position, self.working_zoom());
    }

    pub fn reset(&mut self) {
        self.overview_return = None;
        self.set_focus([0.5, 0.5], self.working_zoom());
    }

    pub fn pan(&mut self, delta: Vec2) {
        self.camera.pan_screen(delta);
        self.constrain();
    }

    pub fn zoom(&mut self, anchor: Vec2, factor: f32) {
        self.camera
            .zoom_at(MAP_RECT, anchor, factor, self.zoom_limits());
        self.constrain();
        self.update_band();
    }

    pub fn gesture(&mut self, frame: &TouchGestureFrame) {
        self.camera
            .apply_gesture(MAP_RECT, frame, self.zoom_limits());
        self.constrain();
        self.update_band();
    }

    pub fn project(&self, point: Vec2) -> Vec2 {
        self.camera
            .world_to_screen(MAP_RECT, point)
            .unwrap_or(point)
    }

    pub fn project_normalized(&self, position: [f32; 2]) -> Vec2 {
        self.project(self.normalized_world(position))
    }

    pub(crate) fn toggle_overview(&mut self, positions: &[[f32; 2]]) {
        if let Some((camera, band)) = self.overview_return.take() {
            self.camera = camera;
            self.band = band;
            return;
        }
        let previous = (self.camera, self.band);
        self.frame_positions(positions);
        self.overview_return = Some(previous);
    }

    pub(crate) fn frame_positions(&mut self, positions: &[[f32; 2]]) {
        let Some(first) = positions.first() else {
            return;
        };
        let mut minimum = self.normalized_world(*first);
        let mut maximum = minimum;
        for position in positions {
            let point = self.normalized_world(*position);
            minimum = minimum.min(point);
            maximum = maximum.max(point);
        }
        // Reserve the top toolbar and lower accounts before fitting known land.
        let span = (maximum - minimum).max(vec2(1.0, 1.0));
        let zoom = ((WIDTH - 192.0) / span.x)
            .min((HEIGHT - 300.0) / span.y)
            .min(self.settings.overview_enter);
        let center = (minimum + maximum) * 0.5 / self.extent;
        self.set_focus(center.to_array(), zoom);
        self.band = MapScaleBand::Overview;
    }

    fn set_focus(&mut self, position: [f32; 2], zoom: f32) {
        let limits = self.zoom_limits();
        self.camera = CameraTransform::new(
            self.normalized_world(position),
            zoom.clamp(limits.0, limits.1),
        )
        .expect("finite position and positive zoom");
        self.constrain();
        self.update_band();
    }

    fn update_band(&mut self) {
        let zoom = self.camera.zoom();
        self.band = match self.band {
            MapScaleBand::Overview if zoom < self.settings.overview_exit => MapScaleBand::Overview,
            MapScaleBand::Detail if zoom > self.settings.detail_exit => MapScaleBand::Detail,
            _ if zoom <= self.settings.overview_enter => MapScaleBand::Overview,
            _ if zoom >= self.settings.detail_enter => MapScaleBand::Detail,
            _ => MapScaleBand::Campaign,
        };
    }

    fn constrain(&mut self) {
        let half = (vec2(WIDTH, HEIGHT) / (2.0 * self.camera.zoom())).min(self.extent * 0.5);
        self.camera.constrain(
            MAP_RECT,
            CameraBounds::new(half, self.extent - half),
            CameraBoundsPolicy::TargetInside,
        );
    }
}
