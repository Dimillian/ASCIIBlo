use macroquad::prelude::*;

use crate::{game::Game, render::with_alpha, stat_display::PublicStat};

use super::widgets::{
    draw_hotkey_hint, draw_modal_backdrop, draw_modal_frame, draw_section_box, wrap_text,
};

const PANEL: Color = Color::new(10.0 / 255.0, 12.0 / 255.0, 16.0 / 255.0, 0.72);
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

    draw_overview(game, vec2(x + 24.0, y + 70.0));

    let left = Rect::new(x + 24.0, y + 108.0, 246.0, 344.0);
    let middle = Rect::new(x + 290.0, y + 108.0, 246.0, 344.0);
    let right = Rect::new(x + 556.0, y + 108.0, 380.0, 344.0);
    draw_section_box(left, "Attributes");
    draw_section_box(middle, "Combat");
    draw_section_box(right, "Details");

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
        StatRow {
            label: "Gold",
            value: game.sim.player.stats.gold.to_string(),
            detail: "Gold buys merchant gear. Death costs 20% of the gold you are carrying.".into(),
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

    let progression_rows = vec![StatRow {
        label: PublicStat::StatPoints.label(),
        value: PublicStat::StatPoints.value(&game.sim.player),
        detail: PublicStat::StatPoints.detail(&game.sim.player),
        cursor_index: None,
    }];

    let hover = game.ui_hover_position();
    let mut focused = draw_rows(&attribute_rows, left, game.ui.character_cursor, hover);
    focused = focused
        .or_else(|| draw_rows(&combat_rows, middle, game.ui.character_cursor, hover))
        .or_else(|| {
            draw_rows(
                &progression_rows,
                Rect::new(right.x + 18.0, right.y + 194.0, right.w - 36.0, 52.0),
                game.ui.character_cursor,
                hover,
            )
        });

    draw_progression_header(game, right);
    draw_detail_panel(
        right,
        focused.unwrap_or_else(|| selected_detail(&attribute_rows, &progression_rows, game)),
    );

    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Up/Down", "select spendable", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("Enter", "spend", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("C", "close", vec2(hint_x, y + h - 44.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 44.0));
}

fn draw_overview(game: &Game, pos: Vec2) {
    draw_text(
        &format!(
            "Level {}   XP {} / {}   Gold {}",
            game.sim.player.stats.level,
            game.sim.player.stats.xp,
            game.sim.player.stats.next_xp,
            game.sim.player.stats.gold
        ),
        pos.x,
        pos.y,
        20.0,
        WHITE,
    );
    draw_text(
        &format!("Stat points {}", game.sim.player.stats.unspent_stat_points),
        pos.x + 378.0,
        pos.y,
        18.0,
        MUTED,
    );
}

fn draw_rows<'a>(
    rows: &'a [StatRow],
    rect: Rect,
    selected_cursor: usize,
    mouse: Vec2,
) -> Option<&'a str> {
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
            hovered = Some(row.detail.as_str());
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

fn selected_detail<'a>(
    attribute_rows: &'a [StatRow],
    progression_rows: &'a [StatRow],
    game: &Game,
) -> &'a str {
    attribute_rows
        .iter()
        .chain(progression_rows.iter())
        .find(|row| row.cursor_index == Some(game.ui.character_cursor))
        .map(|row| row.detail.as_str())
        .unwrap_or("Hover a stat to inspect what it does.")
}

fn draw_progression_header(game: &Game, rect: Rect) {
    let top = rect.y + 18.0;
    draw_text("Progression", rect.x + 18.0, top + 2.0, 18.0, GOLD);
    draw_rectangle(
        rect.x + 18.0,
        top + 18.0,
        rect.w - 36.0,
        18.0,
        with_alpha(WHITE, 0.07),
    );
    draw_rectangle(
        rect.x + 18.0,
        top + 18.0,
        (rect.w - 36.0) * (game.sim.player.stats.xp as f32 / game.sim.player.stats.next_xp as f32),
        18.0,
        with_alpha(GOLD, 0.72),
    );
    draw_text(
        &format!(
            "{} / {} xp",
            game.sim.player.stats.xp, game.sim.player.stats.next_xp
        ),
        rect.x + 28.0,
        top + 32.0,
        15.0,
        WHITE,
    );
    draw_text("Spend", rect.x + 18.0, top + 70.0, 18.0, GOLD);
}

fn draw_detail_panel(rect: Rect, detail: &str) {
    let detail_rect = Rect::new(rect.x + 18.0, rect.y + 100.0, rect.w - 36.0, 84.0);
    draw_rectangle(
        detail_rect.x,
        detail_rect.y,
        detail_rect.w,
        detail_rect.h,
        PANEL,
    );
    for (index, line) in wrap_text(detail, detail_rect.w - 24.0, 17.0, 3)
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
}
