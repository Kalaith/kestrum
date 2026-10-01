//! Atlas-only shoreline mask and short overview vocabulary; no route topology.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapPresentation {
    pub land: Vec<Vec<[f32; 2]>>,
    pub water: Vec<Vec<[f32; 2]>>,
    pub text: BTreeMap<String, String>,
}

impl MapPresentation {
    pub fn validate(&self) -> Result<(), String> {
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
