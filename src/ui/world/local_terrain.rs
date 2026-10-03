//! Deterministic fields and tree cover for a developed city's connected region.

use super::*;
use kestrum::{
    data::{
        economy::Habitation,
        world::{Geography, MarkerId, MarkerLocation, Site, SiteId, SiteTag},
    },
    navigation::{HEIGHT, MAP_RECT, WIDTH},
    state::construction::Focus,
};

pub(super) fn draw_background(ctx: &Context<'_>) -> bool {
    if active_city(ctx).is_none() {
        return false;
    }
    draw_rectangle(0.0, 0.0, WIDTH, HEIGHT, Color::new(0.17, 0.25, 0.20, 1.0));
    let zoom = ctx.view.camera.zoom().clamp(0.8, 1.6);
    for (position, width, height, shade) in [
        ([0.24, 0.29], 380.0, 180.0, 0.045),
        ([0.76, 0.34], 460.0, 230.0, 0.035),
        ([0.31, 0.78], 520.0, 240.0, 0.04),
        ([0.82, 0.77], 360.0, 190.0, 0.03),
    ] {
        let at = ctx.view.project_normalized(position);
        draw_ellipse(
            at.x,
            at.y,
            width * zoom,
            height * zoom,
            0.0,
            Color::new(0.55, 0.57, 0.35, shade),
        );
    }
    true
}

pub(super) fn draw(ctx: &Context<'_>) {
    let Some(region) = active_city(ctx) else {
        return;
    };
    let Some(campaign) = ctx.campaign_view else {
        return;
    };
    let world = &campaign.world;
    let zoom = ctx.view.camera.zoom().clamp(0.8, 1.6);
    for id in world.region_sites(region) {
        if !campaign.known_sites.contains(&id) {
            continue;
        }
        let Some(site) = world.site(id) else {
            continue;
        };
        let Some(position) = world.region_site_position(region, id) else {
            continue;
        };
        let at = ctx.view.project_normalized(position);
        if !Rect::new(at.x - 100.0, at.y - 100.0, 200.0, 200.0).overlaps(&MAP_RECT) {
            continue;
        }
        draw_site_countryside(world, site, at, zoom);
    }
}

fn active_city(ctx: &Context<'_>) -> Option<MarkerId> {
    let campaign = ctx.campaign_view?;
    let kestrum::navigation::MapScope::Region(region) = ctx.navigation.scope() else {
        return None;
    };
    if !campaign.world.is_region_available(region)
        || !matches!(
            &campaign.world.marker(region)?.location,
            MarkerLocation::Site { .. }
        )
    {
        return None;
    }
    Some(region)
}

fn draw_site_countryside(
    world: &kestrum::state::world::CampaignWorld,
    site: &Site,
    at: Vec2,
    zoom: f32,
) {
    let development = world.development.get(&site.id);
    let ruined = development.is_some_and(|entry| entry.ruined);
    let damaged = world.structural_damage(site.id);
    let focus = world.focus.get(&site.id).copied();
    let population = world.population.get(&site.id).copied().unwrap_or(0);

    let mut buildings = building_count(site.habitation) + (population / 85).min(3) as usize;
    if damaged > 0 {
        buildings = buildings.saturating_sub(1);
    }
    if ruined {
        buildings = 2;
    }
    for index in 0..buildings.min(12) {
        building(at, zoom, site.id, index, ruined, damaged > 0);
    }

    let mut fields = field_count(site.habitation);
    fields += (population / 70).min(3) as usize;
    if focus == Some(Focus::Growth) {
        fields += 1;
    }
    if damaged > 0 {
        fields = fields.saturating_sub(1);
    }
    if ruined {
        fallow_plot(at, zoom, site.id);
    } else {
        for index in 0..fields.min(8) {
            field_plot(
                at,
                zoom,
                site.id,
                index,
                focus == Some(Focus::Growth),
                damaged,
            );
        }
    }

    let woodland = site.geography == Geography::Forest
        || site.tags.contains(&SiteTag::WoodSource)
        || focus == Some(Focus::Wood);
    let trees = if ruined {
        3
    } else if woodland {
        5 + if site.tags.contains(&SiteTag::WoodSource) {
            2
        } else {
            0
        } + if focus == Some(Focus::Wood) { 2 } else { 0 }
    } else {
        0
    };
    for index in 0..trees.min(9) {
        let offset = cover_offset(site.id, index, 34.0 * zoom, 86.0 * zoom);
        tree(at + offset, zoom, ruined);
    }
    if damaged > 0 && !ruined {
        damage_scar(at, zoom, site.id);
    }
}

fn field_count(habitation: Habitation) -> usize {
    match habitation {
        Habitation::Unsettled | Habitation::Camp | Habitation::Outpost => 0,
        Habitation::Hamlet => 1,
        Habitation::Village => 2,
        Habitation::Town => 3,
        Habitation::City => 4,
        Habitation::MajorCity => 5,
    }
}

fn building_count(habitation: Habitation) -> usize {
    match habitation {
        Habitation::Unsettled => 0,
        Habitation::Camp => 1,
        Habitation::Outpost => 2,
        Habitation::Hamlet => 3,
        Habitation::Village => 4,
        Habitation::Town => 6,
        Habitation::City => 8,
        Habitation::MajorCity => 10,
    }
}

fn building(center: Vec2, zoom: f32, site: SiteId, index: usize, ruined: bool, damaged: bool) {
    let at = center + cover_offset(site, index + 24, 9.0 * zoom, 34.0 * zoom);
    let width = (5.0 + unit(site, index, 23) * 5.0) * zoom;
    let height = (5.0 + unit(site, index, 29) * 7.0) * zoom;
    let body = if ruined {
        Color::new(0.38, 0.34, 0.29, 0.78)
    } else if damaged {
        Color::new(0.48, 0.42, 0.34, 0.82)
    } else {
        Color::new(0.68, 0.59, 0.42, 0.86)
    };
    let roof = if ruined {
        Color::new(0.34, 0.31, 0.27, 0.8)
    } else {
        Color::new(0.45, 0.35, 0.27, 0.84)
    };
    draw_rectangle(
        at.x - width * 0.5 + 1.5 * zoom,
        at.y - height * 0.5 + 2.0 * zoom,
        width,
        height,
        Color::new(0.08, 0.12, 0.10, 0.45),
    );
    draw_rectangle(at.x - width * 0.5, at.y - height * 0.5, width, height, body);
    if ruined {
        draw_line(
            at.x - width * 0.5,
            at.y,
            at.x + width * 0.5,
            at.y - height * 0.3,
            1.5 * zoom,
            roof,
        );
    } else {
        draw_triangle(
            at + vec2(-width * 0.65, -height * 0.4),
            at + vec2(0.0, -height * 0.9),
            at + vec2(width * 0.65, -height * 0.4),
            roof,
        );
    }
}

fn field_plot(center: Vec2, zoom: f32, site: SiteId, index: usize, growth: bool, damage: u32) {
    let at = center + cover_offset(site, index, 28.0 * zoom, 72.0 * zoom);
    let angle = unit(site, index, 3) * std::f32::consts::TAU;
    let width = (21.0 + unit(site, index, 5) * 15.0) * zoom;
    let height = (10.0 + unit(site, index, 7) * 7.0) * zoom;
    let along = vec2(angle.cos(), angle.sin()) * (width * 0.5);
    let across = vec2(-angle.sin(), angle.cos()) * (height * 0.5);
    let corners = [
        at - along - across,
        at + along - across,
        at + along + across,
        at - along + across,
    ];
    let shade = 0.34 + unit(site, index, 11) * 0.08;
    let color = if damage > 0 {
        Color::new(0.39, 0.34, 0.23, shade)
    } else if growth {
        Color::new(0.62, 0.57, 0.34, shade)
    } else {
        Color::new(0.51, 0.49, 0.31, shade)
    };
    draw_triangle(corners[0], corners[1], corners[2], color);
    draw_triangle(corners[0], corners[2], corners[3], color);
    for furrow in [-0.28, 0.0, 0.28] {
        let offset = across * furrow;
        let start = at - along * 0.76 + offset;
        let end = at + along * 0.76 + offset;
        draw_line(
            start.x,
            start.y,
            end.x,
            end.y,
            0.8 * zoom,
            Color::new(0.73, 0.67, 0.42, 0.32),
        );
    }
}

fn fallow_plot(center: Vec2, zoom: f32, site: SiteId) {
    let at = center + cover_offset(site, 0, 28.0 * zoom, 54.0 * zoom);
    draw_ellipse(
        at.x,
        at.y,
        28.0 * zoom,
        14.0 * zoom,
        unit(site, 0, 13) * 0.8,
        Color::new(0.35, 0.31, 0.25, 0.45),
    );
    for index in 0..3 {
        let stump = at + cover_offset(site, index, 5.0 * zoom, 12.0 * zoom);
        draw_line(
            stump.x,
            stump.y - 3.0 * zoom,
            stump.x + 2.5 * zoom,
            stump.y + 3.0 * zoom,
            2.0 * zoom,
            Color::new(0.53, 0.46, 0.35, 0.7),
        );
    }
}

fn damage_scar(center: Vec2, zoom: f32, site: SiteId) {
    let at = center + cover_offset(site, 0, 15.0 * zoom, 23.0 * zoom);
    draw_line(
        at.x - 5.0 * zoom,
        at.y - 2.0 * zoom,
        at.x + 5.0 * zoom,
        at.y + 3.0 * zoom,
        2.0 * zoom,
        Color::new(0.62, 0.42, 0.32, 0.58),
    );
}

fn tree(at: Vec2, zoom: f32, ruined: bool) {
    let canopy = if ruined {
        Color::new(0.35, 0.34, 0.29, 0.58)
    } else {
        Color::new(0.18, 0.31, 0.23, 0.68)
    };
    let trunk = if ruined {
        Color::new(0.52, 0.44, 0.34, 0.72)
    } else {
        Color::new(0.36, 0.34, 0.25, 0.66)
    };
    draw_line(
        at.x,
        at.y - 3.0 * zoom,
        at.x,
        at.y + 5.0 * zoom,
        2.0 * zoom,
        trunk,
    );
    draw_triangle(
        at + vec2(-6.0, 1.0) * zoom,
        at + vec2(0.0, -8.0) * zoom,
        at + vec2(6.0, 1.0) * zoom,
        canopy,
    );
    draw_triangle(
        at + vec2(-4.5, -2.0) * zoom,
        at + vec2(0.0, -10.0) * zoom,
        at + vec2(4.5, -2.0) * zoom,
        canopy,
    );
}

fn cover_offset(site: SiteId, index: usize, low: f32, high: f32) -> Vec2 {
    let angle = unit(site, index, 17) * std::f32::consts::TAU;
    let radius = low + (high - low) * unit(site, index, 19);
    vec2(angle.cos(), angle.sin()) * radius
}

fn unit(site: SiteId, index: usize, salt: u32) -> f32 {
    let mut value = site.0.wrapping_mul(0x9e37_79b9)
        ^ (index as u32).wrapping_mul(0x85eb_ca6b)
        ^ salt.wrapping_mul(0xc2b2_ae35);
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    (value & 0xffff) as f32 / 65535.0
}
