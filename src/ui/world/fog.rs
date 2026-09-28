//! Soft atlas cover follows discovered graph locations, including offscreen ones.

use super::*;
use kestrum::navigation::{HEIGHT, WIDTH};

pub fn draw(ctx: &Context<'_>) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let positions: Vec<_> = match ctx.navigation.scope() {
        MapScope::World => campaign
            .world
            .markers
            .iter()
            .map(|marker| marker.position)
            .collect(),
        MapScope::Region(id) => campaign
            .world
            .sites
            .iter()
            .filter(|site| site.marker == id)
            .map(|site| site.position)
            .collect(),
    };
    let centers: Vec<_> = positions
        .into_iter()
        .map(|position| ctx.view.project_normalized(position))
        .collect();
    let radius = if ctx.navigation.scope() == MapScope::World {
        74.0
    } else {
        105.0
    } * ctx.view.camera.zoom();
    // A small screen-space mesh batches the cover into bounded strips, with smoothly
    // interpolated opacity. This atlas is an irregular graph, not a tile grid.
    let mut mesh = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        texture: None,
    };
    let columns = 80_u16;
    let rows = 45_u16;
    for row in 0..rows {
        mesh.vertices.clear();
        mesh.indices.clear();
        for vertex_row in [row, row + 1] {
            for column in 0..=columns {
                let at = vec2(
                    f32::from(column) * WIDTH / f32::from(columns),
                    f32::from(vertex_row) * HEIGHT / f32::from(rows),
                );
                let distance = centers
                    .iter()
                    .map(|center| center.distance(at))
                    .fold(f32::INFINITY, f32::min);
                let opacity = ((distance / radius - 0.65) / 0.35).clamp(0.0, 1.0);
                mesh.vertices.push(Vertex::new(
                    at.x,
                    at.y,
                    0.0,
                    0.0,
                    0.0,
                    Color::new(INK.r, INK.g, INK.b, opacity),
                ));
            }
        }
        for column in 0..columns {
            let a = column;
            let b = a + columns + 1;
            mesh.indices.extend([a, a + 1, b, a + 1, b + 1, b]);
        }
        draw_mesh(&mesh);
    }
}
