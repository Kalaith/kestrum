//! Authored country geography drawn below discovery fog and place markers.

use super::*;
use kestrum::{data::world::TerrainFeatureKind, navigation::MAP_RECT};

pub(super) fn draw(ctx: &Context<'_>) {
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let world = &campaign.world;
    if world.layout_revision < 5 {
        return;
    }
    let scope = ctx.navigation.scope();
    let local = match scope {
        MapScope::World => None,
        MapScope::Region(region) if world.is_region_available(region) => Some(region),
        MapScope::Region(_) => return,
    };
    let local_scale = local
        .and_then(|region| world.region_geometry_scale(region))
        .unwrap_or(1.0);
    for feature in &world.terrain_features {
        if local.is_some_and(|region| world.region_site_position(region, feature.anchor).is_none())
        {
            continue;
        }
        let points = feature
            .points
            .iter()
            .filter_map(|point| {
                let normalized = match local {
                    Some(region) => world.region_position(region, *point)?,
                    None => *point,
                };
                Some(ctx.view.project_normalized(normalized))
            })
            .collect::<Vec<_>>();
        if points.len() < 2 {
            continue;
        }
        let width =
            (feature.width * ctx.view.extent().x * ctx.view.camera.zoom() * local_scale).max(5.0);
        draw_feature(feature.kind, &points, width, feature.key.as_bytes());
    }
}

fn draw_feature(kind: TerrainFeatureKind, points: &[Vec2], width: f32, key: &[u8]) {
    match kind {
        TerrainFeatureKind::River => river(points, width),
        TerrainFeatureKind::Forest => forest(points, width, key),
        TerrainFeatureKind::Marsh => marsh(points, width, key),
        TerrainFeatureKind::Ridge => ridge(points, width, key),
        TerrainFeatureKind::Coast => coast(points, width),
        TerrainFeatureKind::Fields => fields(points, width),
    }
}

fn river(points: &[Vec2], width: f32) {
    terrain_ribbon(points, width + 8.0, Color::new(0.11, 0.20, 0.20, 0.72));
    terrain_ribbon(points, width, Color::new(0.25, 0.48, 0.57, 0.88));
    terrain_ribbon(
        points,
        (width * 0.17).max(2.0),
        Color::new(0.70, 0.82, 0.78, 0.72),
    );
}

fn forest(points: &[Vec2], width: f32, key: &[u8]) {
    terrain_ribbon(points, width + 10.0, Color::new(0.08, 0.16, 0.12, 0.36));
    terrain_ribbon(points, width, Color::new(0.16, 0.29, 0.20, 0.50));
    for (index, (center, direction, normal)) in framed_samples(points, (width * 0.34).max(14.0))
        .into_iter()
        .enumerate()
    {
        for row in 0..2_u32 {
            let seed = stable_noise(key, 1, index as u32 * 2 + row);
            let side = if row == 0 { -1.0 } else { 1.0 };
            let lateral = width * (0.19 + unit(seed) * 0.17);
            let along = width * (unit(seed.rotate_left(7)) - 0.5) * 0.12;
            let at = center + normal * side * lateral + direction * along;
            let radius = (width * (0.045 + unit(seed.rotate_left(9)) * 0.025)).clamp(3.5, 7.0);
            conifer(at, radius);
        }
    }
}

fn conifer(center: Vec2, radius: f32) {
    let lower_left = center + vec2(-radius * 0.72, radius * 0.60);
    let lower_right = center + vec2(radius * 0.72, radius * 0.60);
    let top = center + vec2(0.0, -radius);
    draw_triangle(
        lower_left + vec2(1.0, 1.0),
        lower_right + vec2(1.0, 1.0),
        top + vec2(1.0, 1.0),
        INK,
    );
    draw_triangle(
        lower_left,
        lower_right,
        top,
        Color::new(0.14, 0.27, 0.18, 0.98),
    );
    draw_triangle(
        center + vec2(-radius * 0.43, radius * 0.12),
        center + vec2(radius * 0.43, radius * 0.12),
        center + vec2(0.0, -radius * 0.58),
        Color::new(0.34, 0.45, 0.27, 0.94),
    );
    draw_line(
        center.x,
        center.y + radius * 0.36,
        center.x,
        center.y + radius * 0.78,
        1.4,
        Color::new(0.44, 0.34, 0.22, 0.92),
    );
}

fn marsh(points: &[Vec2], width: f32, key: &[u8]) {
    terrain_ribbon(points, width + 7.0, Color::new(0.12, 0.19, 0.15, 0.42));
    terrain_ribbon(points, width, Color::new(0.36, 0.39, 0.24, 0.60));
    for (index, point) in sample(points, (width * 0.42).max(14.0))
        .into_iter()
        .enumerate()
    {
        let seed = stable_noise(key, 3, index as u32);
        let tilt = (unit(seed) - 0.5) * width * 0.22;
        for offset in [-4.0_f32, 0.0, 4.0] {
            let start = point + vec2(offset, width * 0.10);
            let end = start + vec2(tilt + offset * 0.16, -width * 0.18);
            draw_line(
                start.x,
                start.y,
                end.x,
                end.y,
                1.6,
                Color::new(0.72, 0.69, 0.43, 0.78),
            );
        }
    }
}

fn ridge(points: &[Vec2], width: f32, key: &[u8]) {
    terrain_ribbon(points, width + 8.0, Color::new(0.13, 0.17, 0.16, 0.34));
    terrain_ribbon(points, width, Color::new(0.39, 0.40, 0.34, 0.42));
    for (index, (center, direction, normal)) in framed_samples(points, (width * 0.48).max(20.0))
        .into_iter()
        .enumerate()
    {
        let seed = stable_noise(key, 4, index as u32);
        let side = if seed & 1 == 0 { -1.0 } else { 1.0 };
        let span = (width * (0.15 + unit(seed.rotate_left(6)) * 0.07)).clamp(7.0, 20.0);
        let rise = (width * (0.17 + unit(seed.rotate_left(11)) * 0.10)).clamp(8.0, 21.0);
        let left = center - direction * span;
        let right = center + direction * span;
        let peak = center + normal * side * rise;
        let shadow = normal * side * 1.5;
        let dark = Color::new(0.15, 0.19, 0.17, 0.84);
        draw_line(
            (left + shadow).x,
            (left + shadow).y,
            (peak + shadow).x,
            (peak + shadow).y,
            3.0,
            dark,
        );
        draw_line(
            (peak + shadow).x,
            (peak + shadow).y,
            (right + shadow).x,
            (right + shadow).y,
            3.0,
            dark,
        );
        draw_line(
            left.x,
            left.y,
            peak.x,
            peak.y,
            1.8,
            Color::new(0.78, 0.72, 0.56, 0.88),
        );
        draw_line(
            peak.x,
            peak.y,
            right.x,
            right.y,
            2.6,
            Color::new(0.25, 0.30, 0.27, 0.92),
        );
    }
}

fn coast(points: &[Vec2], width: f32) {
    terrain_ribbon(points, width + 9.0, Color::new(0.15, 0.22, 0.23, 0.40));
    terrain_ribbon(points, width, Color::new(0.74, 0.70, 0.52, 0.66));
    terrain_ribbon(
        points,
        (width * 0.28).max(2.0),
        Color::new(0.48, 0.62, 0.59, 0.76),
    );
}

fn fields(points: &[Vec2], width: f32) {
    terrain_ribbon(points, width + 8.0, Color::new(0.18, 0.21, 0.15, 0.28));
    terrain_ribbon(points, width, Color::new(0.58, 0.53, 0.32, 0.24));
    let spacing = (width * 0.27).max(30.0);
    for (index, (center, direction, normal)) in
        framed_samples(points, spacing).into_iter().enumerate()
    {
        for parcel in 0..2_u32 {
            let seed = stable_noise(b"fields", index as u32, parcel);
            let side = if (index as u32 + parcel) & 1 == 0 {
                -1.0
            } else {
                1.0
            };
            let lateral = width * (0.12 + unit(seed) * 0.30);
            let along_shift = width * (unit(seed.rotate_left(8)) - 0.5) * 0.12;
            let angle = (unit(seed.rotate_left(15)) - 0.5) * 0.72;
            let (sin, cos) = angle.sin_cos();
            let axis = direction * cos + normal * sin;
            let cross = normal * cos - direction * sin;
            let at = center + normal * side * lateral + direction * along_shift;
            let half_length = (width * (0.10 + unit(seed.rotate_left(19)) * 0.06)).clamp(9.0, 24.0);
            let half_width =
                (width * (0.045 + unit(seed.rotate_left(24)) * 0.025)).clamp(4.0, 10.0);
            field_parcel(at, axis, cross, half_length, half_width, seed);
        }
    }
}

fn field_parcel(
    center: Vec2,
    axis: Vec2,
    cross: Vec2,
    half_length: f32,
    half_width: f32,
    seed: u32,
) {
    let corners = [
        center - axis * half_length - cross * half_width,
        center + axis * half_length - cross * half_width,
        center + axis * half_length + cross * half_width,
        center - axis * half_length + cross * half_width,
    ];
    let shade = if seed & 1 == 0 { 0.0 } else { 0.04 };
    let fill = Color::new(0.65 + shade, 0.58 + shade, 0.35, 0.72);
    draw_triangle(corners[0], corners[1], corners[2], fill);
    draw_triangle(corners[0], corners[2], corners[3], fill);
    for edge in 0..4 {
        let a = corners[edge];
        let b = corners[(edge + 1) % 4];
        draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(0.31, 0.32, 0.23, 0.76));
    }
    for offset in [-0.28, 0.28] {
        let start = center - axis * half_length * 0.64 + cross * half_width * offset;
        let end = center + axis * half_length * 0.64 + cross * half_width * offset;
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            1.0,
            Color::new(0.86, 0.75, 0.47, 0.74),
        );
    }
}

fn terrain_ribbon(points: &[Vec2], width: f32, color: Color) {
    let mut path = Vec::with_capacity(points.len());
    for point in points {
        if path
            .last()
            .is_none_or(|last: &Vec2| last.distance_squared(*point) > 0.0001)
        {
            path.push(*point);
        }
    }
    if path.len() < 2 {
        return;
    }

    let half_width = width * 0.5;
    let directions = path
        .windows(2)
        .map(|segment| (segment[1] - segment[0]).normalize())
        .collect::<Vec<_>>();
    let mut left = Vec::with_capacity(path.len());
    let mut right = Vec::with_capacity(path.len());
    for (index, point) in path.iter().enumerate() {
        let before = directions[index.saturating_sub(1)];
        let after = directions[index.min(directions.len() - 1)];
        let before_normal = vec2(-before.y, before.x);
        let after_normal = vec2(-after.y, after.x);
        let offset = if index == 0 {
            after_normal * half_width
        } else if index == path.len() - 1 {
            before_normal * half_width
        } else {
            let combined = before_normal + after_normal;
            if combined.length_squared() <= 0.0001 {
                after_normal * half_width
            } else {
                let miter = combined.normalize();
                let denominator = miter.dot(after_normal);
                let length = if denominator.abs() > 0.0001 {
                    (half_width / denominator).clamp(-2.0 * half_width, 2.0 * half_width)
                } else {
                    2.0 * half_width
                };
                miter * length
            }
        };
        left.push(*point + offset);
        right.push(*point - offset);
    }

    for index in 0..path.len() - 1 {
        let corners = [left[index], right[index], left[index + 1], right[index + 1]];
        let min = corners
            .iter()
            .copied()
            .reduce(Vec2::min)
            .expect("ribbon segment has corners");
        let max = corners
            .iter()
            .copied()
            .reduce(Vec2::max)
            .expect("ribbon segment has corners");
        if !Rect::new(min.x, min.y, max.x - min.x, max.y - min.y).overlaps(&MAP_RECT) {
            continue;
        }
        draw_triangle(left[index], right[index], left[index + 1], color);
        draw_triangle(left[index + 1], right[index], right[index + 1], color);
    }
}

fn framed_samples(points: &[Vec2], spacing: f32) -> Vec<(Vec2, Vec2, Vec2)> {
    let mut samples = Vec::new();
    for segment in points.windows(2) {
        let delta = segment[1] - segment[0];
        let length = delta.length();
        if length <= 0.0 {
            continue;
        }
        let count = (length / spacing).ceil().max(1.0) as usize;
        let direction = delta / length;
        let normal = vec2(-direction.y, direction.x);
        for index in 0..count {
            let t = (index as f32 + 0.5) / count as f32;
            samples.push((segment[0].lerp(segment[1], t), direction, normal));
        }
    }
    samples
}

fn sample(points: &[Vec2], spacing: f32) -> Vec<Vec2> {
    let mut samples = Vec::new();
    for segment in points.windows(2) {
        let length = segment[0].distance(segment[1]);
        let count = (length / spacing).ceil().max(1.0) as usize;
        for index in 0..count {
            samples.push(segment[0].lerp(segment[1], (index as f32 + 0.5) / count as f32));
        }
    }
    samples
}

fn stable_noise(key: &[u8], segment: u32, index: u32) -> u32 {
    let mut value = key.iter().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    });
    value ^= segment.wrapping_mul(0x9e37_79b9) ^ index.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^ (value >> 15)
}

fn unit(value: u32) -> f32 {
    (value & 0xffff) as f32 / 65535.0
}
