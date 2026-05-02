use macroquad::prelude::*;

use crate::{game::Game, render::with_alpha};

use super::widgets::{draw_hotkey_hint, draw_modal_backdrop, draw_modal_frame, draw_section_box};

pub(crate) fn draw(game: &Game) {
    let w = 560.0;
    let h = 360.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), "Character");
    draw_text(
        &format!(
            "Stat points {}   Skill points {}",
            game.player.stats.unspent_stat_points, game.player.stats.unspent_skill_points
        ),
        x + 24.0,
        y + 66.0,
        20.0,
        WHITE,
    );
    draw_section_box(Rect::new(x + 24.0, y + 88.0, 246.0, 198.0), "Attributes");
    draw_section_box(Rect::new(x + 290.0, y + 88.0, 246.0, 198.0), "Skills");
    let rows = [
        format!("Strength {}", game.player.stats.strength),
        format!("Agility {}", game.player.stats.agility),
        format!("Vitality {}", game.player.stats.vitality),
        format!("Rush rank {}", game.player.rush_rank),
        format!("Nova rank {}", game.player.nova_rank),
    ];
    for (index, row) in rows.iter().enumerate() {
        let (row_x, local_index) = if index < 3 {
            (x + 42.0, index)
        } else {
            (x + 308.0, index - 3)
        };
        let row_y = y + 124.0 + local_index as f32 * 42.0;
        if index == game.character_cursor {
            draw_rectangle(
                row_x - 10.0,
                row_y - 24.0,
                210.0,
                30.0,
                with_alpha(Color::from_rgba(255, 224, 96, 255), 0.1),
            );
        }
        draw_text(
            row,
            row_x,
            row_y,
            22.0,
            if index == game.character_cursor {
                Color::from_rgba(255, 224, 96, 255)
            } else {
                WHITE
            },
        );
    }
    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Up/Down", "select", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("Enter", "spend", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("C", "close", vec2(hint_x, y + h - 44.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 44.0));
}
