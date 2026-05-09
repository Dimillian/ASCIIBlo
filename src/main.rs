mod content;
mod game;
mod launch;
mod preview;
mod render;
mod stat_display;
mod ui;
mod world;

use game::{FIXED_DT, Game, render_balance_report};
use launch::LaunchOptions;
use macroquad::prelude::*;
use preview::{PreviewRequest, PreviewRunner, PreviewTick};
use render::Renderer;

fn window_conf() -> Conf {
    Conf {
        window_title: "ASCIIBlo".into(),
        window_width: 1280,
        window_height: 760,
        high_dpi: true,
        sample_count: 1,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let launch = LaunchOptions::from_env();
    if launch.balance_report {
        println!("{}", render_balance_report(launch.seed));
        return;
    }
    println!("World seed: {}", launch.seed);
    let mut game = Game::new(launch.seed);
    let preview_request = launch.preview_request;
    match &preview_request {
        PreviewRequest::Single { path, .. } => {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
        }
        PreviewRequest::All { dir } => {
            let _ = std::fs::create_dir_all(dir);
        }
        PreviewRequest::None => {}
    }
    let mut preview_runner = PreviewRunner::from_request(&preview_request);
    match &preview_request {
        PreviewRequest::Single { mode, .. } => mode.configure(&mut game),
        PreviewRequest::All { .. } => {
            if let Some(runner) = &preview_runner {
                runner.current_mode().configure(&mut game);
            }
        }
        PreviewRequest::None => {}
    }
    let mut renderer = Renderer::new();
    let mut accumulator = 0.0;
    let mut rendered_frames = 0;

    loop {
        let frame_dt = get_frame_time().min(0.1);
        accumulator += frame_dt;

        game.set_spawn_visibility_viewport(vec2(screen_width(), screen_height()));
        game.collect_input(renderer.screen_to_world(mouse_position().into()));
        while accumulator >= FIXED_DT {
            game.fixed_update(FIXED_DT);
            accumulator -= FIXED_DT;
        }
        game.frame_update(frame_dt);

        renderer.sync_camera(game.camera_focus(), frame_dt);
        renderer.draw(&game);
        rendered_frames += 1;

        if rendered_frames >= 3 {
            match &preview_request {
                PreviewRequest::Single { path, .. } => {
                    get_screen_data().export_png(&path.to_string_lossy());
                    break;
                }
                PreviewRequest::None | PreviewRequest::All { .. } => {}
            }
        }

        if let Some(runner) = &mut preview_runner {
            match runner.tick() {
                PreviewTick::Continue => {}
                PreviewTick::CaptureAndAdvance(path, next_mode) => {
                    get_screen_data().export_png(&path.to_string_lossy());
                    next_mode.configure(&mut game);
                }
                PreviewTick::CaptureAndFinish(path) => {
                    get_screen_data().export_png(&path.to_string_lossy());
                    break;
                }
            }
        }

        if game.quit_requested() {
            break;
        }
        next_frame().await;
    }
}
