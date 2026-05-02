use macroquad::prelude::*;

use crate::{
    content::NpcKind,
    game::{Game, TRAVEL_DESTINATIONS},
    render::with_alpha,
};

use super::widgets::{draw_hotkey_hint, draw_modal_backdrop, draw_modal_frame, draw_section_box};

pub(crate) fn draw(game: &Game) {
    let w = 560.0;
    let h = 280.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), NpcKind::Wayfinder.name());
    draw_text(
        NpcKind::Wayfinder.greeting(),
        x + 24.0,
        y + 66.0,
        18.0,
        WHITE,
    );
    draw_section_box(
        Rect::new(x + 24.0, y + 88.0, w - 48.0, 146.0),
        "Destinations",
    );
    for (index, destination) in TRAVEL_DESTINATIONS.iter().enumerate() {
        let row_y = y + 122.0 + index as f32 * 26.0;
        if index == game.travel_cursor {
            draw_rectangle(
                x + 36.0,
                row_y - 20.0,
                w - 72.0,
                24.0,
                with_alpha(Color::from_rgba(255, 224, 96, 255), 0.1),
            );
        }
        draw_text(
            &format!(
                "{}  {}  danger {}",
                destination.name,
                game.world.biome_at_tile(destination.pos).name(),
                destination.min_level
            ),
            x + 42.0,
            row_y,
            20.0,
            if index == game.travel_cursor {
                Color::from_rgba(255, 224, 96, 255)
            } else {
                WHITE
            },
        );
    }
    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Up/Down", "select", vec2(hint_x, y + h - 42.0)) + 12.0;
    hint_x += draw_hotkey_hint("Enter", "travel", vec2(hint_x, y + h - 42.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 42.0));
}
