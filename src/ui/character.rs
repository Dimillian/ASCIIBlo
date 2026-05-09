use macroquad::prelude::*;

use crate::{game::Game, render::with_alpha, stat_display::PublicStat};

use super::widgets::{
    CHROME_GOLD, draw_hotkey_hint, draw_interior_card, draw_modal_backdrop, draw_modal_frame,
    draw_section_box, wrap_text,
};

const MUTED: Color = Color::new(180.0 / 255.0, 184.0 / 255.0, 190.0 / 255.0, 1.0);
const GOLD: Color = Color::new(255.0 / 255.0, 224.0 / 255.0, 96.0 / 255.0, 1.0);

struct StatRow {
    label: &'static str,
    value: String,
    detail: String,
    cursor_index: Option<usize>,
}

pub(crate) fn draw(game: &Game) {
    let w = 960.0;
    let h = 560.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), "Character");

    draw_overview(game, Rect::new(x + 24.0, y + 64.0, w - 48.0, 34.0));

    let left = Rect::new(x + 24.0, y + 122.0, 246.0, 330.0);
    let middle = Rect::new(x + 290.0, y + 122.0, 246.0, 330.0);
    let right = Rect::new(x + 556.0, y + 122.0, 380.0, 330.0);
    draw_section_box(left, "Attributes");
    draw_section_box(middle, "Combat");
    draw_section_box(right, "Inspector");

    let attribute_rows = vec![
        StatRow {
            label: PublicStat::Strength.label(),
            value: PublicStat::Strength.value(&game.sim.player),
            detail: PublicStat::Strength.detail(&game.sim.player),
            cursor_index: Some(0),
        },
        StatRow {
            label: PublicStat::Agility.label(),
            value: PublicStat::Agility.value(&game.sim.player),
            detail: PublicStat::Agility.detail(&game.sim.player),
            cursor_index: Some(1),
        },
        StatRow {
            label: PublicStat::Vitality.label(),
            value: PublicStat::Vitality.value(&game.sim.player),
            detail: PublicStat::Vitality.detail(&game.sim.player),
            cursor_index: Some(2),
        },
        StatRow {
            label: PublicStat::Life.label(),
            value: PublicStat::Life.value(&game.sim.player),
            detail: PublicStat::Life.detail(&game.sim.player),
            cursor_index: None,
        },
        StatRow {
            label: PublicStat::Mana.label(),
            value: PublicStat::Mana.value(&game.sim.player),
            detail: PublicStat::Mana.detail(&game.sim.player),
            cursor_index: None,
        },
    ];

    let combat_rows = vec![
        StatRow {
            label: PublicStat::Power.label(),
            value: PublicStat::Power.value(&game.sim.player),
            detail: PublicStat::Power.detail(&game.sim.player),
            cursor_index: None,
        },
        StatRow {
            label: PublicStat::Armor.label(),
            value: PublicStat::Armor.value(&game.sim.player),
            detail: PublicStat::Armor.detail(&game.sim.player),
            cursor_index: None,
        },
        StatRow {
            label: PublicStat::CritChance.label(),
            value: PublicStat::CritChance.value(&game.sim.player),
            detail: PublicStat::CritChance.detail(&game.sim.player),
            cursor_index: None,
        },
        StatRow {
            label: PublicStat::AttackDelay.label(),
            value: PublicStat::AttackDelay.value(&game.sim.player),
            detail: PublicStat::AttackDelay.detail(&game.sim.player),
            cursor_index: None,
        },
        StatRow {
            label: PublicStat::MoveSpeed.label(),
            value: PublicStat::MoveSpeed.value(&game.sim.player),
            detail: PublicStat::MoveSpeed.detail(&game.sim.player),
            cursor_index: None,
        },
    ];

    let hover = game.ui_hover_position();
    let hovered_attribute = draw_rows(&attribute_rows, left, game.ui.character_cursor, hover);
    let hovered_combat = draw_rows(&combat_rows, middle, game.ui.character_cursor, hover);
    let focused = hovered_attribute
        .or(hovered_combat)
        .unwrap_or_else(|| selected_row(&attribute_rows, game));

    draw_inspector(game, right, focused);

    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Up/Down", "select spendable", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("Enter", "spend", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("C", "close", vec2(hint_x, y + h - 44.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 44.0));
}

fn draw_overview(game: &Game, rect: Rect) {
    let xp_ratio = game.sim.player.stats.xp as f32 / game.sim.player.stats.next_xp as f32;
    draw_text(
        &format!("Level {}", game.sim.player.stats.level),
        rect.x,
        rect.y + 22.0,
        20.0,
        WHITE,
    );

    let xp_rect = Rect::new(rect.x + 108.0, rect.y + 5.0, 324.0, 20.0);
    draw_rectangle(
        xp_rect.x,
        xp_rect.y,
        xp_rect.w,
        xp_rect.h,
        with_alpha(WHITE, 0.07),
    );
    draw_rectangle(
        xp_rect.x,
        xp_rect.y,
        xp_rect.w * xp_ratio,
        xp_rect.h,
        with_alpha(GOLD, 0.72),
    );
    draw_text(
        &format!(
            "{} / {} xp",
            game.sim.player.stats.xp, game.sim.player.stats.next_xp
        ),
        xp_rect.x + 10.0,
        xp_rect.y + 15.0,
        15.0,
        WHITE,
    );

    draw_text(
        &format!("Gold {}", game.sim.player.stats.gold),
        rect.x + 468.0,
        rect.y + 22.0,
        19.0,
        WHITE,
    );
    draw_text(
        &format!("Stat points {}", game.sim.player.stats.unspent_stat_points),
        rect.x + rect.w - 160.0,
        rect.y + 22.0,
        19.0,
        if game.sim.player.stats.unspent_stat_points > 0 {
            GOLD
        } else {
            MUTED
        },
    );
}

fn draw_rows<'a>(
    rows: &'a [StatRow],
    rect: Rect,
    selected_cursor: usize,
    mouse: Vec2,
) -> Option<&'a StatRow> {
    let mut hovered = None;

    for (index, row) in rows.iter().enumerate() {
        let row_rect = Rect::new(
            rect.x + 14.0,
            rect.y + 16.0 + index as f32 * 34.0,
            rect.w - 28.0,
            28.0,
        );
        let is_hovered = row_rect.contains(mouse);
        let is_selected = row.cursor_index == Some(selected_cursor);
        if is_hovered {
            hovered = Some(row);
        }
        if is_hovered || is_selected {
            draw_rectangle(
                row_rect.x,
                row_rect.y,
                row_rect.w,
                row_rect.h,
                with_alpha(GOLD, if is_hovered { 0.16 } else { 0.1 }),
            );
        }
        draw_text(
            row.label,
            row_rect.x + 8.0,
            row_rect.y + 20.0,
            18.0,
            if is_selected { GOLD } else { WHITE },
        );
        let dims = measure_text(&row.value, None, 18, 1.0);
        draw_text(
            &row.value,
            row_rect.x + row_rect.w - dims.width - 8.0,
            row_rect.y + 20.0,
            18.0,
            if is_hovered { GOLD } else { WHITE },
        );
    }

    hovered
}

fn selected_row<'a>(attribute_rows: &'a [StatRow], game: &Game) -> &'a StatRow {
    attribute_rows
        .iter()
        .find(|row| row.cursor_index == Some(game.ui.character_cursor))
        .unwrap_or(&attribute_rows[0])
}

fn draw_inspector(game: &Game, rect: Rect, row: &StatRow) {
    draw_text(row.label, rect.x + 18.0, rect.y + 34.0, 24.0, WHITE);
    let value_dims = measure_text(&row.value, None, 24, 1.0);
    draw_text(
        &row.value,
        rect.x + rect.w - value_dims.width - 18.0,
        rect.y + 34.0,
        24.0,
        GOLD,
    );

    let detail_rect = Rect::new(rect.x + 18.0, rect.y + 54.0, rect.w - 36.0, 110.0);
    draw_interior_card(detail_rect, CHROME_GOLD, false);
    for (index, line) in wrap_text(&row.detail, detail_rect.w - 24.0, 17.0, 4)
        .iter()
        .enumerate()
    {
        draw_text(
            line,
            detail_rect.x + 12.0,
            detail_rect.y + 24.0 + index as f32 * 19.0,
            17.0,
            WHITE,
        );
    }

    let spending_text = if row.cursor_index.is_some() {
        if game.sim.player.stats.unspent_stat_points > 0 {
            "Press Enter to invest 1 point into the selected attribute."
        } else {
            "Earn a level to gain more points for Strength, Agility, or Vitality."
        }
    } else {
        "Only Strength, Agility, and Vitality can be raised directly."
    };
    draw_text("Spendable", rect.x + 18.0, rect.y + 204.0, 18.0, GOLD);
    for (index, line) in wrap_text(spending_text, rect.w - 36.0, 17.0, 2)
        .iter()
        .enumerate()
    {
        draw_text(
            line,
            rect.x + 18.0,
            rect.y + 232.0 + index as f32 * 19.0,
            17.0,
            MUTED,
        );
    }
}
