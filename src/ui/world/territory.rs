//! Restrained political ink, clipped to atlas land and the known map footprint.

use super::*;
use kestrum::navigation::{HEIGHT, MAP_RECT, WIDTH};
use macroquad_toolkit::colors::with_alpha;

#[derive(Clone, Copy)]
struct Claim {
    at: Vec2,
    owner: Option<FactionId>,
    known: bool,
}

#[derive(Clone, Copy)]
struct Border {
    start: Vec2,
    end: Vec2,
    normal: Vec2,
    first_owner: Option<FactionId>,
    second_owner: Option<FactionId>,
}

const BORDER_STEP: f32 = 8.0;
const BORDER_SHADOW_WIDTH: f32 = 6.5;
const BORDER_RAIL_OFFSET: f32 = 1.6;
const BORDER_RAIL_WIDTH: f32 = 2.0;
const BORDER_SHADOW_ALPHA: f32 = 0.94;
const MAX_BORDER_QUADS: usize = 512;

pub(super) fn draw(ctx: &Context<'_>, exploration: Option<&MapExploration>) {
    let claims = claims(ctx);
    if claims.is_empty() {
        return;
    }
    // The atlas has no administrative polygons. Its existing known locations
    // partition the painted land; cells are display ink, never movement tiles.
    let step = BORDER_STEP;
    let columns = (WIDTH / step) as usize;
    let rows = (HEIGHT / step) as usize;
    let mut cells = vec![None; columns * rows];
    let mut mesh = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        texture: None,
    };
    for row in 0..rows {
        mesh.vertices.clear();
        mesh.indices.clear();
        for column in 0..columns {
            let at = vec2(column as f32 * step, row as f32 * step);
            let point = ctx
                .view
                .camera
                .screen_to_world(MAP_RECT, at + Vec2::splat(step * 0.5))
                .unwrap_or(at);
            if !known_land(ctx, exploration, point, step / ctx.view.camera.zoom()) {
                continue;
            }
            let nearest = claims.iter().min_by(|a, b| {
                a.at.distance_squared(point)
                    .total_cmp(&b.at.distance_squared(point))
            });
            let Some(claim) = nearest.filter(|claim| claim.known) else {
                continue;
            };
            cells[row * columns + column] = Some(claim.owner);
            let Some(owner) = claim.owner else {
                continue;
            };
            let color = with_alpha(faction_color(ctx, Some(owner)), 0.19);
            let index = mesh.vertices.len() as u16;
            for offset in [
                Vec2::ZERO,
                vec2(step, 0.0),
                Vec2::splat(step),
                vec2(0.0, step),
            ] {
                mesh.vertices.push(Vertex::new(
                    at.x + offset.x,
                    at.y + offset.y,
                    0.0,
                    0.0,
                    0.0,
                    color,
                ));
            }
            mesh.indices
                .extend([index, index + 1, index + 2, index, index + 2, index + 3]);
        }
        draw_mesh(&mesh);
    }
    let borders = borders(&cells, columns, rows, step);
    draw_border_shadow(&borders);
    draw_border_rails(ctx, &borders, true);
    draw_border_rails(ctx, &borders, false);
}

fn borders(
    cells: &[Option<Option<FactionId>>],
    columns: usize,
    rows: usize,
    step: f32,
) -> Vec<Border> {
    let mut borders = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            let Some(owner) = cells[row * columns + column] else {
                continue;
            };
            let x = column as f32 * step;
            let y = row as f32 * step;
            if column + 1 < columns {
                let right = cells[row * columns + column + 1].filter(|other| *other != owner);
                if let Some(other) = right {
                    borders.push(Border {
                        start: vec2(x + step, y),
                        end: vec2(x + step, y + step),
                        normal: vec2(1.0, 0.0),
                        first_owner: owner,
                        second_owner: other,
                    });
                }
            }
            if row + 1 < rows {
                if let Some(other) =
                    cells[(row + 1) * columns + column].filter(|other| *other != owner)
                {
                    borders.push(Border {
                        start: vec2(x, y + step),
                        end: vec2(x + step, y + step),
                        normal: vec2(0.0, 1.0),
                        first_owner: owner,
                        second_owner: other,
                    });
                }
            }
        }
    }
    borders
}

fn draw_border_shadow(borders: &[Border]) {
    for chunk in borders.chunks(MAX_BORDER_QUADS) {
        let mut mesh = empty_mesh();
        for border in chunk {
            append_stroke(
                &mut mesh,
                border.start,
                border.end,
                BORDER_SHADOW_WIDTH,
                with_alpha(INK, BORDER_SHADOW_ALPHA),
            );
        }
        draw_mesh(&mesh);
    }
}

fn draw_border_rails(ctx: &Context<'_>, borders: &[Border], first_side: bool) {
    for chunk in borders.chunks(MAX_BORDER_QUADS) {
        let mut mesh = empty_mesh();
        for border in chunk {
            let (owner, direction) = if first_side {
                (border.first_owner, -1.0)
            } else {
                (border.second_owner, 1.0)
            };
            let offset = border.normal * (direction * BORDER_RAIL_OFFSET);
            append_stroke(
                &mut mesh,
                border.start + offset,
                border.end + offset,
                BORDER_RAIL_WIDTH,
                faction_color(ctx, owner),
            );
        }
        draw_mesh(&mesh);
    }
}

fn empty_mesh() -> Mesh {
    Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
        texture: None,
    }
}

fn append_stroke(mesh: &mut Mesh, start: Vec2, end: Vec2, width: f32, color: Color) {
    let direction = (end - start).normalize();
    let half_width = vec2(-direction.y, direction.x) * (width * 0.5);
    let index = mesh.vertices.len() as u16;
    for point in [
        start + half_width,
        end + half_width,
        end - half_width,
        start - half_width,
    ] {
        mesh.vertices
            .push(Vertex::new(point.x, point.y, 0.0, 0.0, 0.0, color));
    }
    mesh.indices
        .extend([index, index + 1, index + 2, index, index + 2, index + 3]);
}

fn known_land(
    ctx: &Context<'_>,
    exploration: Option<&MapExploration>,
    point: Vec2,
    step: f32,
) -> bool {
    let margin = step * 0.5;
    [
        vec2(-margin, -margin),
        vec2(margin, -margin),
        vec2(margin, margin),
        vec2(-margin, margin),
    ]
    .into_iter()
    .all(|offset| {
        let sample = point + offset;
        exploration.is_none_or(|area| area.opacity(sample) < 0.04)
            && (ctx.navigation.scope() != MapScope::World
                || ctx.data.map.is_atlas_land([
                    sample.x / ctx.view.extent().x,
                    sample.y / ctx.view.extent().y,
                ]))
    })
}

fn claims(ctx: &Context<'_>) -> Vec<Claim> {
    let (Some(view), Some(overview)) = (ctx.campaign_view, ctx.overview) else {
        return Vec::new();
    };
    match ctx.navigation.scope() {
        MapScope::World => view
            .world
            .markers
            .iter()
            .filter_map(|marker| {
                let summary = overview.markers.get(&marker.id)?;
                Some(Claim {
                    at: ctx.view.normalized_world(marker.position),
                    owner: summary.political_owner,
                    known: summary.political_known,
                })
            })
            .collect(),
        MapScope::Region(region) => view
            .world
            .sites
            .iter()
            .filter(|site| site.marker == region)
            .filter_map(|site| {
                let summary = overview.sites.get(&site.id)?;
                Some(Claim {
                    at: ctx.view.normalized_world(site.position),
                    owner: summary.controller,
                    known: view.known_sites.contains(&site.id),
                })
            })
            .collect(),
    }
}
