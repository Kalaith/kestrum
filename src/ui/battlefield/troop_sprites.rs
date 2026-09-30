//! Small illustrated troop groups use posture and equipment to show battlefield roles.

use kestrum::{data::economy::TroopKind, state::battle::simulation::BattleSide};
use macroquad::prelude::*;

pub(super) struct GroupSprite {
    pub(super) center: Vec2,
    pub(super) kind: Option<TroopKind>,
    pub(super) headcount: u32,
    pub(super) capacity: u32,
    pub(super) side: BattleSide,
    pub(super) facing: f32,
    pub(super) is_braced: bool,
    pub(super) is_routed: bool,
}

pub(super) fn draw_group(sprite: GroupSprite) {
    let GroupSprite {
        center,
        kind,
        headcount,
        capacity,
        side,
        facing,
        is_braced,
        is_routed,
    } = sprite;
    let strength = if is_routed { 0.35 } else { 1.0 };
    let cloth = side_color(side, strength);
    let leather = darken(cloth, 0.42);
    let metal = Color::new(0.72 * strength, 0.72 * strength, 0.65 * strength, strength);
    draw_ellipse(
        center.x,
        center.y + 28.0,
        46.0,
        8.0,
        0.0,
        Color::new(0.18, 0.17, 0.13, 0.32),
    );
    draw_banner(center, side, strength);
    match kind {
        Some(TroopKind::SiegeEngines) => {
            draw_siege_cart(center + vec2(-17.0, 2.0), facing, cloth, leather, metal);
            draw_siege_cart(center + vec2(19.0, 8.0), facing, cloth, leather, metal);
        }
        Some(TroopKind::Riders) => draw_riders(
            center, facing, side, is_braced, strength, headcount, capacity,
        ),
        Some(kind) => {
            let figures = representative_count(kind, headcount, capacity);
            for index in 0..figures {
                let column = index % 5;
                let row = index / 5;
                let x = (column as f32 - 2.0) * 12.0;
                let y = row as f32 * 8.0 + if column % 2 == 0 { 1.0 } else { 0.0 };
                draw_foot_soldier(
                    center + vec2(x, y),
                    kind,
                    facing,
                    cloth,
                    leather,
                    metal,
                    is_braced,
                );
            }
        }
        None => draw_threat_group(center, facing, cloth, leather, metal),
    }
}

fn representative_count(kind: TroopKind, headcount: u32, capacity: u32) -> u32 {
    let maximum = match kind {
        TroopKind::Warriors | TroopKind::Spearmen | TroopKind::Archers => 10,
        TroopKind::Riders => 4,
        TroopKind::Medics => 6,
        TroopKind::SiegeEngines => 2,
    };
    if capacity == 0 || headcount == 0 {
        0
    } else {
        (headcount * maximum).div_ceil(capacity).clamp(1, maximum)
    }
}

fn draw_foot_soldier(
    center: Vec2,
    kind: TroopKind,
    facing: f32,
    cloth: Color,
    leather: Color,
    metal: Color,
    braced: bool,
) {
    let head = center + vec2(0.0, -23.0);
    draw_line(
        center.x - 2.5,
        center.y + 12.0,
        center.x - 4.0,
        center.y + 25.0,
        3.0,
        leather,
    );
    draw_line(
        center.x + 2.5,
        center.y + 12.0,
        center.x + 4.0,
        center.y + 25.0,
        3.0,
        leather,
    );
    draw_rectangle(center.x - 5.0, center.y - 13.0, 10.0, 26.0, cloth);
    draw_line(
        center.x - 4.0,
        center.y - 5.0,
        center.x - facing * 11.0,
        center.y + 5.0,
        2.5,
        leather,
    );
    let hand = vec2(center.x + facing * 7.0, center.y - 7.0);
    draw_circle(head.x, head.y, 4.3, Color::new(0.71, 0.53, 0.38, 1.0));
    draw_rectangle(center.x - 5.0, center.y - 28.0, 10.0, 4.0, metal);
    draw_line(
        center.x - 5.0,
        center.y - 26.0,
        center.x + 5.0,
        center.y - 26.0,
        1.2,
        metal,
    );

    match kind {
        TroopKind::Spearmen => spear(center, hand, facing, braced, metal),
        TroopKind::Archers => bow(center, hand, facing, metal),
        TroopKind::Medics => staff(center, hand, facing, metal),
        _ => sword_and_shield(center, hand, facing, braced, leather, metal),
    }
}

fn spear(center: Vec2, hand: Vec2, facing: f32, braced: bool, metal: Color) {
    let angle = if braced { 0.20 } else { -0.72 };
    let end = hand + vec2(facing * 45.0, if braced { -4.0 } else { -27.0 });
    draw_line(
        hand.x,
        hand.y,
        end.x,
        end.y,
        1.7,
        Color::new(0.36, 0.29, 0.20, 1.0),
    );
    let tip = end + vec2(facing * 8.0, 0.0);
    draw_triangle(
        tip,
        end + vec2(facing * 1.0, -3.0),
        end + vec2(facing * 1.0, 3.0),
        metal,
    );
    let shield_y = center.y - 2.0;
    let shield_x = center.x - facing * 8.0;
    draw_rectangle(
        shield_x - 3.0,
        shield_y - 9.0,
        6.0,
        18.0,
        Color::new(0.45, 0.36, 0.24, 1.0),
    );
    draw_line(
        center.x + facing * 3.0,
        center.y + 1.0,
        center.x + facing * 7.0,
        center.y - 2.0 + angle,
        1.0,
        metal,
    );
}

fn bow(center: Vec2, hand: Vec2, facing: f32, metal: Color) {
    let bow_x = hand.x + facing * 4.0;
    draw_line(
        bow_x,
        center.y - 22.0,
        bow_x + facing * 4.0,
        center.y + 2.0,
        1.8,
        metal,
    );
    draw_line(
        bow_x,
        center.y - 22.0,
        bow_x + facing * 11.0,
        center.y - 10.0,
        1.0,
        metal,
    );
    draw_line(
        bow_x + facing * 11.0,
        center.y - 10.0,
        bow_x + facing * 4.0,
        center.y + 2.0,
        1.0,
        metal,
    );
    draw_line(
        bow_x + facing * 3.0,
        center.y - 10.0,
        bow_x + facing * 29.0,
        center.y - 10.0,
        1.2,
        Color::new(0.85, 0.78, 0.60, 1.0),
    );
    draw_triangle(
        vec2(bow_x + facing * 31.0, center.y - 10.0),
        vec2(bow_x + facing * 26.0, center.y - 12.0),
        vec2(bow_x + facing * 26.0, center.y - 8.0),
        metal,
    );
}

fn staff(center: Vec2, hand: Vec2, facing: f32, metal: Color) {
    draw_line(
        hand.x,
        hand.y,
        hand.x + facing * 6.0,
        center.y - 31.0,
        2.3,
        Color::new(0.38, 0.28, 0.18, 1.0),
    );
    draw_circle(hand.x + facing * 6.0, center.y - 32.0, 4.0, metal);
    draw_line(
        center.x - 3.0,
        center.y - 2.0,
        center.x + 3.0,
        center.y - 2.0,
        2.0,
        metal,
    );
    draw_line(
        center.x,
        center.y - 6.0,
        center.x,
        center.y + 2.0,
        2.0,
        metal,
    );
}

fn sword_and_shield(
    center: Vec2,
    hand: Vec2,
    facing: f32,
    braced: bool,
    leather: Color,
    metal: Color,
) {
    let end = hand + vec2(facing * 25.0, if braced { -4.0 } else { -24.0 });
    draw_line(hand.x, hand.y, end.x, end.y, 2.0, metal);
    draw_line(
        end.x - facing * 2.0,
        end.y - 2.0,
        end.x + facing * 3.0,
        end.y + 2.0,
        1.6,
        metal,
    );
    let shield_x = center.x - facing * 8.0;
    draw_ellipse(shield_x, center.y - 2.0, 5.0, 11.0, 0.0, leather);
    draw_line(
        shield_x - 2.0,
        center.y - 2.0,
        shield_x + 2.0,
        center.y - 2.0,
        1.0,
        metal,
    );
}

fn draw_riders(
    center: Vec2,
    facing: f32,
    side: BattleSide,
    braced: bool,
    strength: f32,
    headcount: u32,
    capacity: u32,
) {
    for index in 0..representative_count(TroopKind::Riders, headcount, capacity) {
        let x = (index as f32 - 1.5) * 18.0;
        let horse = center + vec2(x, 4.0 + (index % 2) as f32 * 5.0);
        let coat = side_color(side, strength);
        draw_ellipse(
            horse.x,
            horse.y + 5.0,
            15.0,
            7.0,
            0.0,
            Color::new(0.30, 0.22, 0.17, strength),
        );
        draw_ellipse(
            horse.x + facing * 13.0,
            horse.y - 1.0,
            5.0,
            4.0,
            0.0,
            Color::new(0.36, 0.28, 0.21, strength),
        );
        for leg in [-8.0, 0.0, 7.0] {
            draw_line(
                horse.x + leg,
                horse.y + 9.0,
                horse.x + leg - 2.0,
                horse.y + 17.0,
                1.8,
                Color::new(0.25, 0.20, 0.16, strength),
            );
        }
        draw_line(
            horse.x,
            horse.y - 2.0,
            horse.x - 3.0,
            horse.y - 19.0,
            2.2,
            coat,
        );
        draw_rectangle(horse.x - 6.0, horse.y - 19.0, 11.0, 15.0, coat);
        draw_circle(
            horse.x,
            horse.y - 25.0,
            4.0,
            Color::new(0.71, 0.53, 0.38, strength),
        );
        let spear_end = horse + vec2(facing * 39.0, if braced { -1.0 } else { -18.0 });
        draw_line(
            horse.x + facing * 4.0,
            horse.y - 11.0,
            spear_end.x,
            spear_end.y,
            1.5,
            Color::new(0.36, 0.29, 0.20, strength),
        );
        draw_triangle(
            spear_end,
            spear_end + vec2(-facing * 5.0, -2.2),
            spear_end + vec2(-facing * 5.0, 2.2),
            Color::new(0.78, 0.77, 0.66, strength),
        );
    }
}

fn draw_siege_cart(center: Vec2, facing: f32, cloth: Color, leather: Color, metal: Color) {
    draw_rectangle(center.x - 15.0, center.y - 1.0, 31.0, 15.0, leather);
    draw_rectangle(center.x - 11.0, center.y - 7.0, 22.0, 7.0, cloth);
    draw_circle(
        center.x - 10.0,
        center.y + 15.0,
        5.0,
        Color::new(0.19, 0.17, 0.14, 1.0),
    );
    draw_circle(
        center.x + 11.0,
        center.y + 15.0,
        5.0,
        Color::new(0.19, 0.17, 0.14, 1.0),
    );
    draw_circle_lines(center.x - 10.0, center.y + 15.0, 3.0, 1.2, metal);
    draw_circle_lines(center.x + 11.0, center.y + 15.0, 3.0, 1.2, metal);
    draw_line(
        center.x - facing * 3.0,
        center.y - 7.0,
        center.x + facing * 22.0,
        center.y - 24.0,
        3.2,
        Color::new(0.35, 0.26, 0.17, 1.0),
    );
    draw_circle(center.x + facing * 22.0, center.y - 24.0, 3.5, metal);
}

fn draw_threat_group(center: Vec2, facing: f32, cloth: Color, leather: Color, metal: Color) {
    for index in 0..6 {
        let x = (index as f32 - 2.5) * 10.0;
        draw_foot_soldier(
            center + vec2(x, (index % 2) as f32 * 5.0),
            TroopKind::Warriors,
            facing,
            cloth,
            leather,
            metal,
            false,
        );
    }
}

fn draw_banner(center: Vec2, side: BattleSide, strength: f32) {
    let x = center.x - 34.0;
    let y = center.y - 30.0;
    let cloth = side_color(side, strength);
    draw_line(
        x,
        y - 9.0,
        x,
        y + 23.0,
        1.8,
        Color::new(0.30, 0.24, 0.18, strength),
    );
    draw_triangle(
        vec2(x, y - 8.0),
        vec2(
            x + if side == BattleSide::Attacker {
                18.0
            } else {
                -18.0
            },
            y - 3.0,
        ),
        vec2(x, y + 12.0),
        cloth,
    );
    draw_circle(
        x + if side == BattleSide::Attacker {
            5.0
        } else {
            -5.0
        },
        y + 1.0,
        2.1,
        Color::new(0.86, 0.75, 0.47, strength),
    );
}

fn side_color(side: BattleSide, alpha: f32) -> Color {
    match side {
        BattleSide::Attacker => Color::new(0.18, 0.34, 0.53, alpha),
        BattleSide::Defender => Color::new(0.57, 0.20, 0.17, alpha),
    }
}

fn darken(color: Color, factor: f32) -> Color {
    Color::new(
        color.r * factor,
        color.g * factor,
        color.b * factor,
        color.a,
    )
}
