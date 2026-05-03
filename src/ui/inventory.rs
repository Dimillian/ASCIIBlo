use macroquad::prelude::*;

use crate::{game::Game, render::with_alpha};

use super::widgets::{
    ITEM_SELECTION, draw_hotkey_hint, draw_item_detail, draw_modal_backdrop, draw_modal_frame,
    draw_section_label, draw_stat_value,
};

pub(crate) fn draw(game: &Game) {
    let w = 960.0;
    let h = 560.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), "Inventory");
    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Up/Down", "move", vec2(hint_x, y + 52.0)) + 12.0;
    hint_x += draw_hotkey_hint("Enter", "equip", vec2(hint_x, y + 52.0)) + 12.0;
    hint_x += draw_hotkey_hint("Backspace", "drop", vec2(hint_x, y + 52.0)) + 12.0;
    draw_hotkey_hint("Tab", "close", vec2(hint_x, y + 52.0));
    let left_x = x + 24.0;
    let left_w = 408.0;
    let right_x = x + 456.0;
    let right_w = 480.0;
    draw_section_label("Backpack", vec2(left_x, y + 102.0));
    draw_rectangle(
        left_x,
        y + 116.0,
        left_w,
        356.0,
        with_alpha(Color::from_rgba(10, 12, 16, 255), 0.72),
    );
    draw_text(
        "Name",
        left_x + 18.0,
        y + 138.0,
        16.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    draw_text(
        "Type",
        left_x + 250.0,
        y + 138.0,
        16.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    draw_text(
        "Lvl",
        left_x + 340.0,
        y + 138.0,
        16.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    let visible_rows: usize = 8;
    let visible_start = game
        .inventory_cursor
        .saturating_sub(visible_rows.saturating_sub(1))
        .min(game.player.inventory.len().saturating_sub(visible_rows));
    for (row, (index, item)) in game
        .player
        .inventory
        .iter()
        .enumerate()
        .skip(visible_start)
        .take(visible_rows)
        .enumerate()
    {
        let row_y = y + 172.0 + row as f32 * 36.0;
        if index == game.inventory_cursor {
            draw_rectangle(
                left_x + 10.0,
                row_y - 24.0,
                left_w - 20.0,
                30.0,
                with_alpha(ITEM_SELECTION, 0.16),
            );
        }
        draw_text(
            &item.name,
            left_x + 18.0,
            row_y,
            19.0,
            if index == game.inventory_cursor {
                ITEM_SELECTION
            } else {
                item.rarity.color()
            },
        );
        draw_text(
            item.slot.label(),
            left_x + 250.0,
            row_y,
            16.0,
            Color::from_rgba(180, 184, 190, 255),
        );
        draw_text(
            &item.item_level.to_string(),
            left_x + 340.0,
            row_y,
            16.0,
            WHITE,
        );
    }
    draw_text(
        &format!(
            "{} / {} items",
            game.player.inventory.len().min(game.inventory_cursor + 1),
            game.player.inventory.len()
        ),
        left_x + left_w - 82.0,
        y + 104.0,
        16.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    if let Some(item) = game.player.inventory.get(game.inventory_cursor) {
        draw_section_label("Selected", vec2(right_x, y + 102.0));
        draw_rectangle(
            right_x,
            y + 116.0,
            right_w,
            216.0,
            with_alpha(Color::from_rgba(10, 12, 16, 255), 0.72),
        );
        draw_item_detail(item, vec2(right_x + 18.0, y + 144.0));
    }
    draw_section_label("Equipment", vec2(right_x, y + 358.0));
    draw_rectangle(
        right_x,
        y + 372.0,
        right_w,
        76.0,
        with_alpha(Color::from_rgba(10, 12, 16, 255), 0.72),
    );
    let gear = [
        ("Weapon", game.player.equipment.weapon.as_ref()),
        ("Armor", game.player.equipment.armor.as_ref()),
        ("Charm", game.player.equipment.charm.as_ref()),
    ];
    for (index, (label, item)) in gear.iter().enumerate() {
        let column_x = right_x + 18.0 + index as f32 * 150.0;
        draw_text(
            label,
            column_x,
            y + 396.0,
            15.0,
            Color::from_rgba(180, 184, 190, 255),
        );
        draw_text(
            item.map(|item| item.name.as_str()).unwrap_or("-"),
            column_x,
            y + 420.0,
            18.0,
            item.map(|item| item.rarity.color()).unwrap_or(WHITE),
        );
    }
    draw_section_label("Derived stats", vec2(right_x, y + 474.0));
    draw_rectangle(
        right_x,
        y + 488.0,
        right_w,
        48.0,
        with_alpha(Color::from_rgba(10, 12, 16, 255), 0.72),
    );
    let derived = [
        ("STR", game.player.stats.strength),
        ("AGI", game.player.stats.agility),
        ("VIT", game.player.stats.vitality),
        ("POW", game.player.power()),
        ("ARM", game.player.armor()),
        ("HST", game.player.haste()),
    ];
    for (index, (label, value)) in derived.iter().enumerate() {
        draw_stat_value(
            label,
            *value,
            vec2(right_x + 18.0 + index as f32 * 74.0, y + 516.0),
        );
    }
}
