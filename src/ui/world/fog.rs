//! Soft atlas cover follows the boundary between known and unknown locations.

use super::*;
use kestrum::navigation::{MapExploration, HEIGHT, MAP_RECT, WIDTH};

pub fn exploration(ctx: &Context<'_>) -> Option<MapExploration> {
    let campaign = ctx.state.campaign.as_ref()?.strategic()?;
    Some(MapExploration::new(
        &campaign.world,
        &ctx.campaign_view?.world,
        ctx.navigation.scope(),
        ctx.view,
    ))
}

pub fn draw(ctx: &Context<'_>, exploration: &MapExploration) {
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
                let point = ctx.view.camera.screen_to_world(MAP_RECT, at).unwrap_or(at);
                let opacity = exploration.opacity(point);
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
