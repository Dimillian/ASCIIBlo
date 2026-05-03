use macroquad::prelude::*;

use crate::{content::Item, render::with_alpha};

pub(crate) const ITEM_SELECTION: Color = Color::new(128.0 / 255.0, 214.0 / 255.0, 1.0, 1.0);

pub(crate) fn draw_modal_backdrop() {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        with_alpha(BLACK, 0.55),
    );
}

pub(crate) fn draw_modal_frame(rect: Rect, title: &str) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(18, 20, 26, 245),
    );
    draw_rectangle(rect.x, rect.y, rect.w, 48.0, with_alpha(BLACK, 0.22));
    draw_line(
        rect.x,
        rect.y + 48.0,
        rect.x + rect.w,
        rect.y + 48.0,
        2.0,
        Color::from_rgba(255, 224, 96, 180),
    );
    draw_text(
        title,
        rect.x + 24.0,
        rect.y + 34.0,
        28.0,
        Color::from_rgba(255, 224, 96, 255),
    );
}

pub(crate) fn draw_section_box(rect: Rect, title: &str) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        with_alpha(Color::from_rgba(10, 12, 16, 255), 0.72),
    );
    draw_section_label(title, vec2(rect.x, rect.y - 10.0));
}

pub(crate) fn draw_label_inline(label: &str, value: &str, pos: Vec2, value_offset: f32) {
    draw_text(
        label,
        pos.x,
        pos.y,
        15.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    draw_text(value, pos.x + value_offset, pos.y, 18.0, WHITE);
}

pub(crate) fn draw_stat_chip(label: &str, value: i32, pos: Vec2) {
    draw_rectangle(pos.x, pos.y - 18.0, 62.0, 28.0, with_alpha(WHITE, 0.06));
    draw_text(
        &format!("{} {}", label, value),
        pos.x + 8.0,
        pos.y,
        16.0,
        if value > 0 {
            WHITE
        } else {
            Color::from_rgba(120, 124, 132, 255)
        },
    );
}

pub(crate) fn draw_stat_value(label: &str, value: i32, pos: Vec2) {
    draw_text(
        label,
        pos.x,
        pos.y - 16.0,
        14.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    draw_text(&value.to_string(), pos.x, pos.y + 6.0, 18.0, WHITE);
}

pub(crate) fn draw_section_label(label: &str, pos: Vec2) {
    draw_text(
        label,
        pos.x,
        pos.y,
        19.0,
        Color::from_rgba(255, 224, 96, 255),
    );
}

pub(crate) fn hotkey_badge_width(label: &str) -> f32 {
    (measure_text(label, None, 15, 1.0).width.max(14.0) + 16.0).ceil()
}

pub(crate) fn draw_hotkey_badge(label: &str, pos: Vec2) -> f32 {
    let width = hotkey_badge_width(label);
    let rect = Rect::new(pos.x.round(), pos.y.round(), width, 24.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        with_alpha(Color::from_rgba(24, 26, 32, 255), 0.96),
    );
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.0,
        with_alpha(Color::from_rgba(255, 224, 96, 255), 0.82),
    );
    let dims = measure_text(label, None, 15, 1.0);
    let text_x = rect.x + ((rect.w - dims.width) * 0.5).round();
    let text_y = rect.y + ((rect.h + dims.height) * 0.5).round() - 1.0;
    draw_text(label, text_x, text_y, 15.0, WHITE);
    width
}

pub(crate) fn draw_hotkey_hint(label: &str, text: &str, pos: Vec2) -> f32 {
    let badge_w = draw_hotkey_badge(label, pos);
    draw_text(
        text,
        pos.x + badge_w + 8.0,
        pos.y + 18.0,
        17.0,
        Color::from_rgba(210, 214, 220, 255),
    );
    badge_w + 8.0 + measure_text(text, None, 17, 1.0).width
}

pub(crate) fn draw_alert_icon(pos: Vec2) {
    draw_circle(pos.x, pos.y, 8.0, Color::from_rgba(255, 96, 96, 255));
    draw_circle(pos.x, pos.y, 5.5, Color::from_rgba(24, 26, 32, 255));
    draw_text("!", pos.x - 2.5, pos.y + 4.5, 13.0, WHITE);
}

pub(crate) fn draw_item_detail(item: &Item, pos: Vec2) {
    draw_text(&item.name, pos.x, pos.y, 22.0, item.rarity.color());
    draw_label_inline(
        "Quality",
        item.rarity.label(),
        vec2(pos.x, pos.y + 34.0),
        68.0,
    );
    draw_label_inline(
        "Slot",
        item.slot.label(),
        vec2(pos.x + 190.0, pos.y + 34.0),
        42.0,
    );
    draw_label_inline("Base", &item.base_name, vec2(pos.x, pos.y + 60.0), 68.0);
    draw_label_inline(
        "Item lvl",
        &item.item_level.to_string(),
        vec2(pos.x + 190.0, pos.y + 60.0),
        70.0,
    );

    draw_text(
        "Affixes",
        pos.x,
        pos.y + 88.0,
        16.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    let affixes = if item.affixes.is_empty() {
        "No affixes".into()
    } else {
        item.affixes.join(", ")
    };
    for (index, line) in wrap_text(&affixes, 292.0, 18.0, 2).iter().enumerate() {
        draw_text(
            line,
            pos.x,
            pos.y + 110.0 + index as f32 * 20.0,
            18.0,
            item.rarity.color(),
        );
    }
    draw_text(
        "Stats",
        pos.x,
        pos.y + 148.0,
        16.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    let stats = [
        ("POW", item.power),
        ("ARM", item.armor),
        ("VIT", item.vitality),
        ("HST", item.haste),
    ];
    for (index, (label, value)) in stats.iter().enumerate() {
        draw_stat_chip(
            label,
            *value,
            vec2(pos.x + index as f32 * 78.0, pos.y + 174.0),
        );
    }
    draw_text(
        &format!("Value {}", item.value),
        pos.x + 332.0,
        pos.y + 176.0,
        18.0,
        WHITE,
    );
}

pub(crate) fn wrap_text(text: &str, max_width: f32, size: f32, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current, word)
        };
        if measure_text(&candidate, None, size as u16, 1.0).width <= max_width {
            current = candidate;
            continue;
        }
        if !current.is_empty() {
            lines.push(current);
            current = word.to_string();
        } else {
            current = clip_text(word, max_width, size);
        }
        if lines.len() == max_lines {
            break;
        }
    }

    if !current.is_empty() && lines.len() < max_lines {
        lines.push(current);
    }
    if lines.is_empty() {
        return vec![String::new()];
    }
    if lines.len() == max_lines {
        let original_words = text.split_whitespace().count();
        let shown_words = lines
            .iter()
            .flat_map(|line| line.split_whitespace())
            .count();
        if shown_words < original_words {
            let last = lines.pop().unwrap_or_default();
            lines.push(clip_text(&format!("{}...", last), max_width, size));
        }
    }
    lines
}

fn clip_text(text: &str, max_width: f32, size: f32) -> String {
    if measure_text(text, None, size as u16, 1.0).width <= max_width {
        return text.to_string();
    }
    let mut clipped = text.to_string();
    while !clipped.is_empty()
        && measure_text(&format!("{}...", clipped), None, size as u16, 1.0).width > max_width
    {
        clipped.pop();
    }
    format!("{}...", clipped)
}
