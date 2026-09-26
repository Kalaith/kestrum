//! Kestrum's native and WebGL entry point.

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod game;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    capture::capture_window_conf(
        "KESTRUM",
        "Kestrum",
        kestrum::navigation::WIDTH as i32,
        kestrum::navigation::HEIGHT as i32,
    )
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = match Game::new().await {
        Ok(game) => game,
        Err(error) => {
            eprintln!("Kestrum could not start: {error}");
            loop {
                clear_background(BLACK);
                draw_text(
                    "Kestrum could not load its atlas. Please reload the game.",
                    30.0,
                    60.0,
                    25.0,
                    WHITE,
                );
                for (index, line) in
                    macroquad_toolkit::ui::wrap_text(&error, screen_width() - 60.0, 20.0)
                        .iter()
                        .enumerate()
                {
                    draw_text(line, 30.0, 100.0 + index as f32 * 26.0, 20.0, WHITE);
                }
                next_frame().await;
            }
        }
    };

    // Deterministic scenes isolate verification from local saves and preferences.
    if let Some(configs) = capture::CaptureConfig::all_from_env("KESTRUM") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |dt| {
                game.frame(dt);
            })
            .await;
        }
        return;
    }

    while game.running {
        let dt = get_frame_time().min(0.1);
        game.frame(dt);
        next_frame().await;
    }
}
