//! A shared hillside, distant ruins and a worn lane hold both sides of the fight.

use macroquad::prelude::*;

pub(super) fn draw() {
    sky();
    mountains();
    ruins();
    hillside();
    grass_and_stone();
    army_pennants();
}

fn sky() {
    draw_rectangle(0.0, 0.0, 1280.0, 275.0, Color::new(0.60, 0.69, 0.72, 1.0));
    draw_circle(628.0, 120.0, 48.0, Color::new(0.93, 0.79, 0.56, 0.84));
    draw_circle(628.0, 120.0, 63.0, Color::new(0.93, 0.79, 0.56, 0.12));
    cloud(210.0, 88.0, 1.0);
    cloud(1035.0, 98.0, 0.8);
}

fn cloud(x: f32, y: f32, scale: f32) {
    let pale = Color::new(0.88, 0.84, 0.73, 0.30);
    draw_ellipse(x, y, 74.0 * scale, 13.0 * scale, 0.0, pale);
    draw_ellipse(
        x + 18.0 * scale,
        y - 6.0 * scale,
        34.0 * scale,
        15.0 * scale,
        0.0,
        pale,
    );
    draw_ellipse(
        x - 21.0 * scale,
        y - 3.0 * scale,
        29.0 * scale,
        12.0 * scale,
        0.0,
        pale,
    );
}

fn mountains() {
    let far = Color::new(0.38, 0.49, 0.53, 1.0);
    let near = Color::new(0.48, 0.55, 0.54, 1.0);
    for (left, peak, right) in [
        (0.0, 165.0, 302.0),
        (126.0, 196.0, 474.0),
        (342.0, 173.0, 676.0),
        (589.0, 178.0, 896.0),
        (826.0, 194.0, 1110.0),
        (1011.0, 158.0, 1280.0),
    ] {
        draw_triangle(
            vec2(left, 264.0),
            vec2(peak, 118.0),
            vec2(right, 264.0),
            far,
        );
        draw_triangle(
            vec2(peak, 118.0),
            vec2((peak + right) * 0.5, 264.0),
            vec2(right, 264.0),
            near,
        );
        draw_triangle(
            vec2(peak, 118.0),
            vec2(peak - 31.0, 164.0),
            vec2(peak + 20.0, 153.0),
            Color::new(0.82, 0.82, 0.72, 0.88),
        );
    }
    draw_rectangle(0.0, 254.0, 1280.0, 88.0, Color::new(0.55, 0.57, 0.50, 1.0));
}

fn ruins() {
    let stone = Color::new(0.31, 0.34, 0.33, 1.0);
    let lit = Color::new(0.49, 0.48, 0.41, 1.0);
    draw_rectangle(566.0, 175.0, 23.0, 91.0, stone);
    draw_rectangle(557.0, 165.0, 41.0, 17.0, lit);
    draw_rectangle(596.0, 194.0, 22.0, 73.0, stone);
    draw_rectangle(591.0, 187.0, 32.0, 12.0, lit);
    draw_rectangle(651.0, 185.0, 26.0, 83.0, stone);
    draw_rectangle(643.0, 173.0, 42.0, 18.0, lit);
    for x in [562.0, 578.0, 602.0, 612.0, 648.0, 669.0] {
        draw_rectangle(x, 204.0, 5.0, 21.0, Color::new(0.14, 0.18, 0.19, 1.0));
    }
    draw_line(589.0, 235.0, 622.0, 216.0, 3.0, stone);
    draw_line(622.0, 216.0, 650.0, 230.0, 3.0, stone);
}

fn hillside() {
    draw_triangle(
        vec2(0.0, 278.0),
        vec2(379.0, 226.0),
        vec2(730.0, 288.0),
        Color::new(0.45, 0.43, 0.34, 1.0),
    );
    draw_triangle(
        vec2(542.0, 281.0),
        vec2(948.0, 220.0),
        vec2(1280.0, 277.0),
        Color::new(0.39, 0.42, 0.35, 1.0),
    );
    draw_rectangle(0.0, 276.0, 1280.0, 444.0, Color::new(0.55, 0.43, 0.29, 1.0));
    draw_triangle(
        vec2(480.0, 276.0),
        vec2(654.0, 397.0),
        vec2(823.0, 276.0),
        Color::new(0.68, 0.54, 0.35, 0.82),
    );
    draw_triangle(
        vec2(515.0, 276.0),
        vec2(654.0, 372.0),
        vec2(790.0, 276.0),
        Color::new(0.73, 0.60, 0.39, 0.60),
    );
    draw_line(
        654.0,
        374.0,
        654.0,
        650.0,
        2.0,
        Color::new(0.42, 0.34, 0.25, 0.45),
    );
    draw_line(
        635.0,
        385.0,
        581.0,
        625.0,
        2.0,
        Color::new(0.71, 0.58, 0.39, 0.40),
    );
    draw_line(
        674.0,
        385.0,
        734.0,
        625.0,
        2.0,
        Color::new(0.71, 0.58, 0.39, 0.40),
    );
}

fn grass_and_stone() {
    let grass = Color::new(0.28, 0.32, 0.25, 0.70);
    let straw = Color::new(0.82, 0.69, 0.44, 0.58);
    for index in 0..46 {
        let x = (index * 83 % 1270) as f32 + 4.0;
        let y = 289.0 + (index * 47 % 410) as f32;
        let length = 5.0 + (index % 5) as f32;
        draw_line(x, y, x - 3.0, y - length, 1.2, grass);
        if index % 3 == 0 {
            draw_line(x + 2.0, y, x + 5.0, y - length * 0.8, 1.0, straw);
        }
        if index % 7 == 0 {
            draw_ellipse(
                x + 12.0,
                y + 8.0,
                6.0,
                3.0,
                0.0,
                Color::new(0.28, 0.29, 0.26, 0.62),
            );
        }
    }
    for (x, y, size) in [
        (60.0, 618.0, 22.0),
        (1220.0, 600.0, 26.0),
        (638.0, 585.0, 13.0),
    ] {
        draw_triangle(
            vec2(x - size, y + size * 0.5),
            vec2(x - 4.0, y - size * 0.6),
            vec2(x + size, y + size * 0.5),
            Color::new(0.29, 0.30, 0.27, 0.88),
        );
        draw_line(x - 4.0, y - size * 0.6, x + 3.0, y + size * 0.5, 1.2, straw);
    }
}

fn army_pennants() {
    banner(
        22.0,
        111.0,
        1.0,
        Color::new(0.13, 0.25, 0.38, 1.0),
        Color::new(0.85, 0.72, 0.43, 1.0),
    );
    banner(
        1258.0,
        111.0,
        -1.0,
        Color::new(0.42, 0.15, 0.15, 1.0),
        Color::new(0.88, 0.77, 0.58, 1.0),
    );
}

fn banner(x: f32, y: f32, facing: f32, cloth: Color, trim: Color) {
    draw_line(
        x,
        y - 43.0,
        x,
        y + 23.0,
        3.0,
        Color::new(0.25, 0.20, 0.15, 1.0),
    );
    let tip = x + facing * 42.0;
    draw_triangle(
        vec2(x, y - 39.0),
        vec2(tip, y - 31.0),
        vec2(x, y + 3.0),
        cloth,
    );
    draw_line(x, y - 38.0, tip - facing * 2.0, y - 31.0, 1.5, trim);
    draw_circle(x + facing * 14.0, y - 18.0, 5.0, trim);
}
