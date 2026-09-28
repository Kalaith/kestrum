//! Atlas placement and crossing metadata is versioned independently of campaign rules.
use super::*;

impl WorldLayout {
    pub(super) fn validate_atlas(&self) -> Result<(), String> {
        require(
            LAYOUT_SOURCE,
            "layout_revision",
            self.layout_revision == 2,
            "unsupported atlas revision",
        )?;
        for (marker, position) in &self.legacy_marker_positions {
            require(
                LAYOUT_SOURCE,
                "legacy_marker_positions",
                self.marker(*marker).is_some() && normalized(*position),
                "unknown marker or invalid legacy position",
            )?;
        }
        for (id, path) in &self.atlas_paths {
            let pair = self
                .routes
                .iter()
                .find(|route| route.id == *id)
                .and_then(|route| route.major_connection)
                .and_then(|[from, to]| self.marker(from).zip(self.marker(to)))
                .ok_or_else(|| format!("{LAYOUT_SOURCE}: atlas path needs an external route"))?;
            require(
                LAYOUT_SOURCE,
                "atlas_paths",
                path.waypoints.len() <= 6
                    && path.bridges.len() <= 4
                    && (!path.waypoints.is_empty() || !path.bridges.is_empty())
                    && path
                        .waypoints
                        .iter()
                        .chain(&path.bridges)
                        .all(|point| normalized(*point)),
                "invalid or excessive route geometry",
            )?;
            let points = std::iter::once(pair.0.position)
                .chain(path.waypoints.iter().copied())
                .chain(std::iter::once(pair.1.position))
                .collect::<Vec<_>>();
            for bridge in &path.bridges {
                require(
                    LAYOUT_SOURCE,
                    "atlas_paths.bridges",
                    points
                        .windows(2)
                        .any(|segment| on_segment(*bridge, segment[0], segment[1])),
                    "crossing must lie on its authored route",
                )?;
            }
        }
        Ok(())
    }
}

fn normalized(point: [f32; 2]) -> bool {
    point
        .into_iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
}

fn on_segment(point: [f32; 2], a: [f32; 2], b: [f32; 2]) -> bool {
    let delta = [b[0] - a[0], b[1] - a[1]];
    let length = delta[0] * delta[0] + delta[1] * delta[1];
    if length == 0.0 {
        return false;
    }
    let t =
        (((point[0] - a[0]) * delta[0] + (point[1] - a[1]) * delta[1]) / length).clamp(0.0, 1.0);
    (point[0] - a[0] - t * delta[0]).hypot(point[1] - a[1] - t * delta[1]) < 0.006
}
