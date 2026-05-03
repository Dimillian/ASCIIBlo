use macroquad::prelude::*;

use crate::{
    game::{AbilityKind, Game},
    render::with_alpha,
    world::World,
};

use super::widgets::{
    draw_alert_icon, draw_hotkey_badge, draw_hotkey_hint, hotkey_badge_width, wrap_text,
};

pub(crate) fn draw(game: &Game) {
    draw_rectangle(0.0, 0.0, screen_width(), 82.0, with_alpha(BLACK, 0.58));
    draw_text(
        "ASCIIBlo",
        22.0,
        34.0,
        30.0,
        Color::from_rgba(255, 224, 96, 255),
    );
    draw_text(
        &format!(
            "HP {}/{}",
            game.player.hp.round() as i32,
            game.player.max_hp().round() as i32
        ),
        178.0,
        28.0,
        22.0,
        Color::from_rgba(130, 236, 126, 255),
    );
    draw_bar(
        vec2(178.0, 40.0),
        170.0,
        game.player.hp / game.player.max_hp(),
        Color::from_rgba(130, 236, 126, 255),
    );
    draw_text(
        &format!(
            "MP {}/{}",
            game.player.mana.round() as i32,
            game.player.max_mana().round() as i32
        ),
        370.0,
        28.0,
        22.0,
        Color::from_rgba(112, 180, 255, 255),
    );
    draw_bar(
        vec2(370.0, 40.0),
        150.0,
        game.player.mana / game.player.max_mana(),
        Color::from_rgba(112, 180, 255, 255),
    );
    draw_text(
        &format!(
            "LV {}  XP {}/{}  Gold {}",
            game.player.stats.level,
            game.player.stats.xp,
            game.player.stats.next_xp,
            game.player.stats.gold
        ),
        548.0,
        28.0,
        22.0,
        WHITE,
    );
    draw_text(
        &format!("FPS {}", get_fps()),
        790.0,
        28.0,
        18.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    draw_bar(
        vec2(548.0, 40.0),
        220.0,
        game.player.stats.xp as f32 / game.player.stats.next_xp as f32,
        Color::from_rgba(255, 224, 96, 255),
    );

    let controls_x = screen_width() - 310.0;
    draw_text(
        &format!(
            "{}  level {}",
            game.world.region_name(game.player.pos),
            game.world.biome_level(game.player.pos)
        ),
        controls_x,
        26.0,
        20.0,
        WHITE,
    );
    let bar_y = screen_height() - 58.0;
    draw_rectangle(0.0, bar_y, screen_width(), 58.0, with_alpha(BLACK, 0.62));
    draw_line(
        0.0,
        bar_y,
        screen_width(),
        bar_y,
        2.0,
        Color::from_rgba(255, 224, 96, 180),
    );
    let mut bottom_x = 22.0;
    for (index, ability) in game.player.bound_abilities.iter().copied().enumerate() {
        let cooldown = game.player.ability_cooldowns[ability.index()];
        bottom_x += draw_ability_hint(
            &(index + 1).to_string(),
            ability,
            cooldown,
            vec2(bottom_x, bar_y + 17.0),
        ) + 14.0;
    }
    bottom_x += draw_hotkey_hint("E", "loot", vec2(bottom_x, bar_y + 17.0)) + 14.0;
    bottom_x += draw_hotkey_hint("F", "talk", vec2(bottom_x, bar_y + 17.0)) + 14.0;
    bottom_x += draw_hotkey_hint("Tab", "inventory", vec2(bottom_x, bar_y + 17.0)) + 14.0;
    bottom_x += draw_alerting_hotkey_hint(
        "C",
        "character",
        vec2(bottom_x, bar_y + 17.0),
        game.player.stats.unspent_stat_points > 0,
    ) + 14.0;
    bottom_x += draw_hotkey_hint("B", "mastery", vec2(bottom_x, bar_y + 17.0)) + 14.0;
    draw_hotkey_hint("M", "map", vec2(bottom_x, bar_y + 17.0));
    draw_text(
        &format!(
            "POW {}  ARM {}  HST {}",
            game.player.power(),
            game.player.armor(),
            game.player.haste()
        ),
        screen_width() - 238.0,
        bar_y + 35.0,
        18.0,
        WHITE,
    );
    draw_log(game);
    draw_skill_feedback(game);
    draw_minimap(game);
    draw_hovered_monster_tooltip(game);
    draw_nearest_loot_tooltip(game);
}

fn draw_alerting_hotkey_hint(label: &str, text: &str, pos: Vec2, alert: bool) -> f32 {
    let width = draw_hotkey_hint(label, text, pos);
    if alert {
        draw_alert_icon(vec2(pos.x + hotkey_badge_width(label) - 2.0, pos.y - 2.0));
    }
    width
}

fn draw_ability_hint(label: &str, ability: AbilityKind, cooldown: f32, pos: Vec2) -> f32 {
    let badge_w = if cooldown > 0.0 {
        draw_cooldown_badge(label, ability, cooldown, pos)
    } else {
        draw_hotkey_badge(label, pos)
    };
    let text = ability.name().to_lowercase();
    draw_text(
        &text,
        pos.x + badge_w + 8.0,
        pos.y + 18.0,
        17.0,
        if cooldown > 0.0 {
            Color::from_rgba(160, 164, 172, 255)
        } else {
            Color::from_rgba(210, 214, 220, 255)
        },
    );
    badge_w + 8.0 + measure_text(&text, None, 17, 1.0).width
}

fn draw_cooldown_badge(label: &str, ability: AbilityKind, cooldown: f32, pos: Vec2) -> f32 {
    let badge_w = hotkey_badge_width(label);
    draw_rectangle(
        pos.x.round(),
        pos.y.round(),
        badge_w,
        24.0,
        with_alpha(Color::from_rgba(24, 26, 32, 255), 0.96),
    );
    if cooldown > 0.0 {
        let fill_ratio = (cooldown / ability.cooldown()).clamp(0.0, 1.0);
        let fill_h = 24.0 * fill_ratio;
        draw_rectangle(
            pos.x.round(),
            pos.y.round() + 24.0 - fill_h,
            badge_w,
            fill_h,
            with_alpha(ability.color(), 0.28),
        );
        let text = format!("{:.1}", cooldown);
        let dims = measure_text(&text, None, 13, 1.0);
        draw_text(
            &text,
            pos.x + ((badge_w - dims.width) * 0.5).round(),
            pos.y + 17.0,
            13.0,
            WHITE,
        );
    }
    draw_rectangle_lines(
        pos.x.round() + 0.5,
        pos.y.round() + 0.5,
        badge_w - 1.0,
        23.0,
        1.0,
        with_alpha(ability.color(), 0.9),
    );
    badge_w
}

fn draw_skill_feedback(game: &Game) {
    let panel_w = 260.0;
    let x = screen_width() - panel_w - 18.0;
    let mut y = 300.0;

    for notification in game.notifications.iter().rev().take(3) {
        draw_feedback_row(x, y, panel_w, &notification.text, notification.color, 1.0);
        y += 34.0;
    }
    for toast in game.skill_xp_toasts.iter().rev().take(3) {
        draw_feedback_row(
            x,
            y,
            panel_w,
            &format!("+{} {} XP", toast.amount, toast.kind.name()),
            toast.kind.color(),
            if toast.ttl > 0.65 {
                1.0
            } else {
                (toast.ttl / 0.65).clamp(0.3, 1.0)
            },
        );
        y += 34.0;
    }
}

fn draw_feedback_row(x: f32, y: f32, w: f32, text: &str, color: Color, alpha: f32) {
    draw_rectangle(
        x,
        y,
        w,
        26.0,
        with_alpha(Color::from_rgba(12, 14, 18, 255), 0.86 * alpha),
    );
    draw_rectangle(x, y, 4.0, 26.0, with_alpha(color, alpha));
    draw_text(text, x + 12.0, y + 19.0, 17.0, with_alpha(WHITE, alpha));
}

fn draw_log(game: &Game) {
    let panel_x = 18.0;
    let panel_w = 420.0;
    let title_h = 34.0;
    let line_h = 17.0;
    let row_gap = 4.0;
    let rows: Vec<_> = game
        .log
        .iter()
        .rev()
        .skip(game.log_scroll_offset)
        .take(6)
        .map(|line| {
            let wrapped = wrap_text(line, panel_w - 68.0, 16.0, 2);
            let row_h = wrapped.len() as f32 * line_h + row_gap;
            (line, wrapped, row_h)
        })
        .collect();
    let rows_h: f32 = rows.iter().map(|(_, _, row_h)| *row_h).sum();
    let panel_h = title_h + rows_h + 12.0;
    let panel_y = screen_height() - 70.0 - panel_h;
    draw_rectangle(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        with_alpha(Color::from_rgba(12, 14, 18, 255), 0.84),
    );
    draw_text(
        "Combat feed",
        panel_x + 16.0,
        panel_y + 23.0,
        19.0,
        Color::from_rgba(255, 224, 96, 255),
    );
    if game.log_scroll_offset > 0 {
        draw_text(
            &format!("+{}", game.log_scroll_offset),
            panel_x + panel_w - 36.0,
            panel_y + 23.0,
            16.0,
            Color::from_rgba(180, 184, 190, 255),
        );
    }
    let mut cursor_y = panel_y + title_h + 10.0;
    for (index, (line, wrapped, row_h)) in rows.iter().enumerate() {
        let latest_visible = game.log_scroll_offset == 0 && index == 0;
        let color = log_color(line, latest_visible);
        if latest_visible {
            draw_rectangle(
                panel_x + 12.0,
                cursor_y - 14.0,
                panel_w - 24.0,
                *row_h,
                with_alpha(color, 0.08),
            );
        }
        draw_rectangle(
            panel_x + 16.0,
            cursor_y - 12.0,
            3.0,
            wrapped.len() as f32 * line_h + 2.0,
            color,
        );
        for (line_index, text) in wrapped.iter().enumerate() {
            draw_text(
                text,
                panel_x + 28.0,
                cursor_y + line_index as f32 * line_h,
                16.0,
                if line_index == 0 {
                    color
                } else {
                    with_alpha(color, 0.88)
                },
            );
        }
        cursor_y += *row_h;
    }
}

fn draw_minimap(game: &Game) {
    let panel_size = vec2(220.0, 176.0);
    let map_size = vec2(196.0, 132.0);
    let x = screen_width() - panel_size.x - 18.0;
    let y = 108.0;
    draw_rectangle(
        x,
        y,
        panel_size.x,
        panel_size.y,
        with_alpha(Color::from_rgba(12, 14, 18, 255), 0.92),
    );
    draw_rectangle_lines(
        x,
        y,
        panel_size.x,
        panel_size.y,
        2.0,
        with_alpha(Color::from_rgba(255, 224, 96, 255), 0.78),
    );
    draw_rectangle_lines(
        x + 5.0,
        y + 5.0,
        panel_size.x - 10.0,
        panel_size.y - 10.0,
        1.0,
        with_alpha(Color::from_rgba(255, 255, 255, 255), 0.12),
    );
    let half_w = 24;
    let half_h = 16;
    let map_x = x + 12.0;
    let map_y = y + 32.0;
    let scale_x = map_size.x / (half_w * 2) as f32;
    let scale_y = map_size.y / (half_h * 2) as f32;
    let player_tile = World::world_to_tile(game.player.pos);
    draw_text(
        &format!(
            "{}  L{}",
            game.world.region_name(game.player.pos),
            game.world.biome_level(game.player.pos)
        ),
        x + 12.0,
        y + 22.0,
        17.0,
        WHITE,
    );
    draw_text(
        &format!("({}, {})", player_tile.x, player_tile.y),
        x + panel_size.x - 74.0,
        y + 22.0,
        16.0,
        Color::from_rgba(180, 184, 190, 255),
    );
    for sample_y in -half_h..half_h {
        for sample_x in -half_w..half_w {
            let tile_pos = player_tile + ivec2(sample_x, sample_y);
            let tile = game.world.tile(tile_pos);
            let (bg, _) = tile.colors();
            draw_rectangle(
                map_x + (sample_x + half_w) as f32 * scale_x,
                map_y + (sample_y + half_h) as f32 * scale_y,
                scale_x + 1.0,
                scale_y + 1.0,
                bg,
            );
        }
    }
    for npc in &game.npcs {
        let npc_tile = World::world_to_tile(npc.pos);
        draw_circle(
            map_x + (npc_tile.x - player_tile.x + half_w) as f32 * scale_x,
            map_y + (npc_tile.y - player_tile.y + half_h) as f32 * scale_y,
            3.0,
            npc.kind.color(),
        );
    }
    draw_circle(
        map_x + half_w as f32 * scale_x,
        map_y + half_h as f32 * scale_y,
        4.0,
        WHITE,
    );
}

fn draw_hovered_monster_tooltip(game: &Game) {
    if game.ui_mode != crate::game::UiMode::None {
        return;
    }
    let Some(monster) = game.hovered_monster() else {
        return;
    };
    let w = 360.0;
    let h = 62.0;
    let x = (screen_width() - w) * 0.5;
    let y = 90.0;
    let color = monster.rank.accent_color();
    draw_rectangle(
        x,
        y,
        w,
        h,
        with_alpha(Color::from_rgba(12, 14, 18, 255), 0.9),
    );
    draw_rectangle(x, y, 4.0, h, with_alpha(color, 0.92));
    draw_text(&monster.display_name(), x + 16.0, y + 23.0, 20.0, color);
    draw_text(
        &format!("Lv {}", monster.level),
        x + w - 66.0,
        y + 23.0,
        18.0,
        WHITE,
    );
    draw_text(
        &format!(
            "{}/{} HP",
            monster.hp.ceil() as i32,
            monster.max_hp.ceil() as i32
        ),
        x + 16.0,
        y + 45.0,
        16.0,
        WHITE,
    );
    draw_bar(
        vec2(x + 104.0, y + 38.0),
        w - 188.0,
        monster.hp / monster.max_hp,
        color,
    );
    draw_text(
        &format!(
            "ATK {}",
            crate::content::monster_damage(monster.kind, monster.level, monster.rank).round()
        ),
        x + w - 74.0,
        y + 47.0,
        16.0,
        Color::from_rgba(210, 214, 220, 255),
    );
}

fn draw_nearest_loot_tooltip(game: &Game) {
    let Some(loot) = game
        .loot
        .iter()
        .filter(|loot| loot.pos.distance(game.player.pos) <= 42.0)
        .min_by(|a, b| {
            a.pos
                .distance(game.player.pos)
                .total_cmp(&b.pos.distance(game.player.pos))
        })
    else {
        return;
    };
    let name_lines = wrap_text(&loot.item.name, 310.0, 20.0, 2);
    let summary = loot.item.summary();
    let content_h = name_lines.len() as f32 * 22.0 + if summary.is_empty() { 0.0 } else { 22.0 };
    let w = 390.0;
    let h = 54.0 + content_h;
    let x = (screen_width() - w) * 0.5;
    let y = screen_height() - 58.0 - h - 12.0;
    draw_rectangle(
        x,
        y,
        w,
        h,
        with_alpha(Color::from_rgba(12, 14, 18, 255), 0.88),
    );
    let badge_w = draw_hotkey_badge("E", vec2(x + 16.0, y + 16.0));
    draw_text(
        "Pick up",
        x + 16.0 + badge_w + 10.0,
        y + 34.0,
        18.0,
        Color::from_rgba(210, 214, 220, 255),
    );
    let mut cursor_y = y + 60.0;
    for line in name_lines {
        draw_text(&line, x + 16.0, cursor_y, 20.0, loot.item.rarity.color());
        cursor_y += 22.0;
    }
    if !summary.is_empty() {
        draw_text(&summary, x + 16.0, cursor_y, 18.0, WHITE);
    }
}

fn log_color(line: &str, latest: bool) -> Color {
    if line.contains("xp") || line.contains("level") {
        Color::from_rgba(130, 236, 126, 255)
    } else if line.contains("Bought")
        || line.contains("Sold")
        || line.contains("drops")
        || line.contains("Equipped")
    {
        Color::from_rgba(128, 214, 255, 255)
    } else if line.contains("hit") || line.contains("bites") {
        Color::from_rgba(255, 180, 120, 255)
    } else if line.contains(':') || line.contains("sends") || line.contains("wake") {
        Color::from_rgba(196, 156, 255, 255)
    } else if latest {
        WHITE
    } else {
        Color::from_rgba(180, 184, 190, 255)
    }
}

fn draw_bar(pos: Vec2, width: f32, ratio: f32, color: Color) {
    draw_rectangle(pos.x, pos.y, width, 8.0, with_alpha(BLACK, 0.65));
    draw_rectangle(pos.x, pos.y, width * ratio.clamp(0.0, 1.0), 8.0, color);
}
