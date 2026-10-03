//! Kestrum's native and WebGL entry point.

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod game;
mod profiling;
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
        capture::prepare_capture_surface("KESTRUM")
            .await
            .expect("hidden capture framebuffer must match its requested dimensions");
        for config in configs {
            game.begin_capture_scene(&config.scene);
            let mut rendered = 0;
            capture::run_capture_once(&config, |dt| {
                game.frame(dt);
                rendered += 1;
                if rendered == config.frames {
                    println!(
                        "KESTRUM_CAPTURE_METRICS scene={} {}",
                        config.scene,
                        game.profile_context()
                    );
                }
            })
            .await;
        }
        return;
    }

    let mut profile = profiling::FrameProfile::new();
    let benchmark = profiling::benchmark_frames() > 0;
    if benchmark {
        if let Err(error) = game.begin_profile() {
            error!("K18_PROFILE_FAILED {error}");
            return;
        }
    }
    let mut frame = 0_u64;
    while game.running {
        if is_key_pressed(KeyCode::F3) {
            profile.toggle(&game.profile_context());
        }
        let started = get_time();
        if benchmark && frame > 120 && frame.is_multiple_of(12) {
            game.profile_order();
        }
        let input = is_mouse_button_released(MouseButton::Left)
            || touches()
                .iter()
                .any(|touch| touch.phase == TouchPhase::Ended);
        let dt = get_frame_time().min(0.1);
        game.frame(dt);
        let cpu = get_time() - started;
        next_frame().await;
        if profile.enabled()
            && profile.record(cpu, get_time() - started, input, &game.profile_context())
        {
            break;
        }
        frame += 1;
    }
}
