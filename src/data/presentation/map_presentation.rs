//! Atlas-only shoreline mask and short overview vocabulary; no route topology.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapPresentation {
    pub camera: MapCameraSettings,
    pub land: Vec<Vec<[f32; 2]>>,
    pub water: Vec<Vec<[f32; 2]>>,
    pub text: BTreeMap<String, String>,
}

impl MapPresentation {
    pub fn validate(&self) -> Result<(), String> {
        self.camera.validate()?;
        if self.land.is_empty()
            || self.land.iter().chain(&self.water).any(|polygon| {
                polygon.len() < 3
                    || polygon
                        .iter()
                        .flatten()
                        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            })
        {
            return Err("map_presentation.json: invalid normalized atlas land mask".into());
        }
        for key in [
            "capital",
            "region",
            "idle",
            "queued",
            "siege",
            "troops",
            "armies",
            "unsupplied",
            "occupied",
            "contested",
            "unknown_claim",
            "mixed",
            "neutral",
            "legend",
            "work",
            "damage",
            "wood",
            "stone",
        ] {
            if self.text.get(key).is_none_or(|text| text.trim().is_empty()) {
                return Err(format!("map_presentation.json: missing text {key}"));
            }
        }
        Ok(())
    }

    /// A conservative outline of the existing painted atlas. This affects ink
    /// only: it cannot create land, change connectivity, or grant discovery.
    pub fn is_atlas_land(&self, point: [f32; 2]) -> bool {
        self.land.iter().any(|polygon| contains(polygon, point))
            && !self.water.iter().any(|polygon| contains(polygon, point))
    }

    pub fn text<'a>(&'a self, key: &'a str) -> &'a str {
        self.text.get(key).map(String::as_str).unwrap_or(key)
    }
}

/// Presentation distances are independent of route costs and saved geography.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapCameraSettings {
    pub world_extent: [f32; 2],
    pub region_extent: [f32; 2],
    pub working_zoom: f32,
    pub maximum_zoom: f32,
    pub overview_enter: f32,
    pub overview_exit: f32,
    pub detail_enter: f32,
    pub detail_exit: f32,
    pub army_group_distance: f32,
}

impl Default for MapCameraSettings {
    fn default() -> Self {
        Self {
            world_extent: [5280.0, 2970.0],
            region_extent: [3360.0, 1890.0],
            working_zoom: 1.0,
            maximum_zoom: 2.4,
            overview_enter: 0.68,
            overview_exit: 0.78,
            detail_enter: 1.45,
            detail_exit: 1.30,
            army_group_distance: 88.0,
        }
    }
}

impl MapCameraSettings {
    fn validate(&self) -> Result<(), String> {
        let values = [
            self.working_zoom,
            self.maximum_zoom,
            self.overview_enter,
            self.overview_exit,
            self.detail_enter,
            self.detail_exit,
            self.army_group_distance,
        ];
        if values
            .iter()
            .any(|value| !value.is_finite() || *value <= 0.0)
            || [self.world_extent, self.region_extent]
                .iter()
                .any(|extent| {
                    extent.iter().any(|value| !value.is_finite())
                        || extent[0] < 1920.0
                        || extent[1] < 1080.0
                        || (extent[0] / extent[1] - 16.0 / 9.0).abs() > 0.001
                        || 1920.0 / extent[0] > self.overview_enter
                        || 1080.0 / extent[1] > self.overview_enter
                })
            || !(self.overview_enter < self.overview_exit
                && self.overview_exit < self.working_zoom
                && self.working_zoom < self.detail_exit
                && self.detail_exit < self.detail_enter
                && self.detail_enter < self.maximum_zoom)
            || !(48.0..=160.0).contains(&self.army_group_distance)
        {
            return Err(
                "map_presentation.json: invalid map extent, zoom bands or grouping distance".into(),
            );
        }
        Ok(())
    }
}

fn contains(polygon: &[[f32; 2]], [x, y]: [f32; 2]) -> bool {
    let mut inside = false;
    let mut previous = polygon[polygon.len() - 1];
    for &current in polygon {
        if (current[1] > y) != (previous[1] > y)
            && x < (previous[0] - current[0]) * (y - current[1]) / (previous[1] - current[1])
                + current[0]
        {
            inside = !inside;
        }
        previous = current;
    }
    inside
}
