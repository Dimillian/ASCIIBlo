use macroquad::prelude::*;

use crate::{
    content::NpcKind,
    game::{DisciplineKind, Game},
};

use super::widgets::{draw_hotkey_hint, draw_modal_backdrop, draw_modal_frame, draw_section_box};

pub(crate) fn draw(game: &Game) {
    let w = 620.0;
    let h = 260.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), NpcKind::Trainer.name());
    draw_section_box(Rect::new(x + 24.0, y + 82.0, w - 48.0, 120.0), "Guidance");
    draw_text(
        NpcKind::Trainer.greeting(),
        x + 42.0,
        y + 112.0,
        19.0,
        WHITE,
    );
    let mut cursor_x = x + 42.0;
    for kind in DisciplineKind::ALL {
        let progress = game.player.disciplines.get(kind);
        let label = format!("{} {}", kind.name(), progress.level);
        draw_text(&label, cursor_x, y + 146.0, 18.0, kind.color());
        cursor_x += measure_text(&label, None, 18, 1.0).width + 24.0;
    }
    draw_text(
        "Use what you want to master. Bind any two unlocked skills in the mastery screen.",
        x + 42.0,
        y + 178.0,
        18.0,
        WHITE,
    );
    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Enter", "open mastery", vec2(hint_x, y + h - 46.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 46.0));
}
