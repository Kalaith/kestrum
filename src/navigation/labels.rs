//! Priority and decluttering for Kestrum place labels in logical map pixels.

use super::{MapSelection, MapTarget, HEIGHT, WIDTH};
use macroquad::prelude::{Rect, Vec2};

pub struct MapLabelCandidate {
    pub selection: MapSelection,
    pub center: Vec2,
    pub width: f32,
    pub priority: u8,
    pub minimum_zoom: f32,
}

pub struct PlacedMapLabel {
    pub selection: MapSelection,
    pub bounds: Rect,
}

/// Important places win collisions; names progressively appear as spacing grows.
/// This never changes targets or picking and uses the toolkit camera's zoom.
pub fn place_map_labels(
    mut candidates: Vec<MapLabelCandidate>,
    targets: &[MapTarget],
    reserved: &[Rect],
    zoom: f32,
) -> Vec<PlacedMapLabel> {
    candidates.retain(|candidate| candidate.minimum_zoom <= zoom);
    candidates.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.selection.cmp(&b.selection))
    });
    let mut placed: Vec<PlacedMapLabel> = Vec::new();
    for candidate in candidates {
        let width = candidate.width.clamp(48.0, 230.0);
        let x = (candidate.center.x - width * 0.5).clamp(12.0, WIDTH - width - 12.0);
        let mut options = vec![
            Rect::new(x, candidate.center.y + 44.0, width, 27.0),
            Rect::new(x, candidate.center.y - 54.0, width, 27.0),
        ];
        if candidate.priority >= 80 {
            options.extend(
                [80.0, -99.0, 118.0, 156.0, 194.0]
                    .map(|offset| Rect::new(x, candidate.center.y + offset, width, 27.0)),
            );
            for side in [-1.0, 1.0] {
                let sideways = candidate.center.x + side * (width * 0.5 + 42.0);
                options.push(Rect::new(
                    (sideways - width * 0.5).clamp(12.0, WIDTH - width - 12.0),
                    candidate.center.y - 13.5,
                    width,
                    27.0,
                ));
            }
        }
        if let Some(bounds) = options.into_iter().find(|rect| {
            rect.y >= 92.0
                && rect.bottom() <= HEIGHT - 148.0
                && !reserved.iter().any(|reserved| reserved.overlaps(rect))
                && !placed.iter().any(|label| label.bounds.overlaps(rect))
                && !targets.iter().any(|target| {
                    target.selection != candidate.selection
                        && Rect::new(target.center.x - 20.0, target.center.y - 20.0, 40.0, 40.0)
                            .overlaps(rect)
                })
        }) {
            placed.push(PlacedMapLabel {
                selection: candidate.selection,
                bounds,
            });
        }
    }
    placed
}
