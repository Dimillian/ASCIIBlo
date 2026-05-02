use macroquad::prelude::*;

use crate::{content::NpcKind, game::Game};

use super::widgets::{draw_hotkey_hint, draw_modal_backdrop, draw_modal_frame, draw_section_box};

pub(crate) fn draw(_game: &Game) {
    let w = 520.0;
    let h = 190.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), NpcKind::Trainer.name());
    draw_section_box(Rect::new(x + 24.0, y + 82.0, w - 48.0, 54.0), "Guidance");
    draw_text(
        NpcKind::Trainer.greeting(),
        x + 42.0,
        y + 116.0,
        19.0,
        WHITE,
    );
    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Enter", "open hero", vec2(hint_x, y + h - 46.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 46.0));
}
