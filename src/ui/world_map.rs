use macroquad::prelude::*;

use crate::{game::Game, render::with_alpha, world::World};

use super::widgets::{draw_hotkey_hint, draw_modal_backdrop, draw_modal_frame};

pub(crate) fn draw(game: &Game) {
    let frame = Rect::new(24.0, 24.0, screen_width() - 48.0, screen_height() - 48.0);
    let map_rect = Rect::new(
        frame.x + 24.0,
        frame.y + 82.0,
        frame.w - 48.0,
        frame.h - 150.0,
    );
    draw_modal_backdrop();
    draw_modal_frame(frame, "World Map");
    draw_text(
        &format!(
            "Known tiles {}   Center ({}, {})   Zoom {:.1}x",
            game.known_tiles.len(),
            game.world_map.center_tile.x.round() as i32,
            game.world_map.center_tile.y.round() as i32,
            game.world_map.zoom
        ),
        frame.x + 24.0,
        frame.y + 66.0,
        18.0,
        WHITE,
    );
    draw_map_panel(game, map_rect);

    let mut hint_x = frame.x + 24.0;
    let hint_y = frame.y + frame.h - 42.0;
    hint_x += draw_hotkey_hint("WASD", "pan", vec2(hint_x, hint_y)) + 12.0;
    hint_x += draw_hotkey_hint("Wheel", "zoom", vec2(hint_x, hint_y)) + 12.0;
    hint_x += draw_hotkey_hint("R", "recenter", vec2(hint_x, hint_y)) + 12.0;
    hint_x += draw_hotkey_hint("M", "close", vec2(hint_x, hint_y)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, hint_y));
}

fn draw_map_panel(game: &Game, rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        with_alpha(Color::from_rgba(8, 10, 14, 255), 0.96),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        with_alpha(Color::from_rgba(255, 224, 96, 255), 0.72),
    );

    let zoom = game.world_map.zoom;
    let center = game.world_map.center_tile;
    let half_w = rect.w * 0.5 / zoom;
    let half_h = rect.h * 0.5 / zoom;
    let min_x = (center.x - half_w).floor() as i32 - 1;
    let max_x = (center.x + half_w).ceil() as i32 + 1;
    let min_y = (center.y - half_h).floor() as i32 - 1;
    let max_y = (center.y + half_h).ceil() as i32 + 1;

    draw_grid(rect, center, zoom, min_x, max_x, min_y, max_y);

    for tile in &game.known_tiles {
        if tile.x < min_x || tile.x > max_x || tile.y < min_y || tile.y > max_y {
            continue;
        }
        let screen = tile_to_screen(*tile, rect, center, zoom);
        let (bg, _) = game.world.tile(*tile).colors();
        draw_rectangle(
            screen.x - zoom * 0.5,
            screen.y - zoom * 0.5,
            zoom + 0.5,
            zoom + 0.5,
            bg,
        );
    }

    for settlement in &game.discovered_settlements {
        let destination = settlement.site;
        if !game.known_tiles.contains(&destination.center) {
            continue;
        }
        let screen = tile_to_screen(destination.center, rect, center, zoom);
        if rect.contains(screen) {
            draw_circle(
                screen.x,
                screen.y,
                (zoom * 0.42).max(3.0),
                if settlement.tier() == crate::world::SettlementTier::Town {
                    Color::from_rgba(255, 224, 96, 255)
                } else {
                    Color::from_rgba(180, 184, 190, 255)
                },
            );
        }
    }

    let player_tile = World::world_to_tile(game.player.pos);
    let player_screen = tile_to_screen(player_tile, rect, center, zoom);
    if rect.contains(player_screen) {
        draw_circle(
            player_screen.x,
            player_screen.y,
            (zoom * 0.7).max(5.0),
            with_alpha(WHITE, 0.16),
        );
        draw_circle(
            player_screen.x,
            player_screen.y,
            (zoom * 0.35).max(3.0),
            WHITE,
        );
    }
}

fn draw_grid(rect: Rect, center: Vec2, zoom: f32, min_x: i32, max_x: i32, min_y: i32, max_y: i32) {
    let grid_color = with_alpha(Color::from_rgba(255, 255, 255, 255), 0.035);
    for x in min_x..=max_x {
        let screen_x = tile_to_screen(ivec2(x, 0), rect, center, zoom).x;
        draw_line(screen_x, rect.y, screen_x, rect.y + rect.h, 1.0, grid_color);
    }
    for y in min_y..=max_y {
        let screen_y = tile_to_screen(ivec2(0, y), rect, center, zoom).y;
        draw_line(rect.x, screen_y, rect.x + rect.w, screen_y, 1.0, grid_color);
    }
}

fn tile_to_screen(tile: IVec2, rect: Rect, center: Vec2, zoom: f32) -> Vec2 {
    vec2(
        rect.x + rect.w * 0.5 + (tile.x as f32 - center.x) * zoom,
        rect.y + rect.h * 0.5 + (tile.y as f32 - center.y) * zoom,
    )
}
