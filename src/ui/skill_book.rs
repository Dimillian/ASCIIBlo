use macroquad::prelude::*;

use crate::{game::Game, render::with_alpha};

use super::widgets::{
    draw_hotkey_hint, draw_modal_backdrop, draw_modal_frame, draw_section_box, wrap_text,
};

const GOLD: Color = Color::new(255.0 / 255.0, 224.0 / 255.0, 96.0 / 255.0, 1.0);
const MUTED: Color = Color::new(180.0 / 255.0, 184.0 / 255.0, 190.0 / 255.0, 1.0);

struct SkillRow {
    hotkey: &'static str,
    name: &'static str,
    rank: i32,
    cost: &'static str,
    cooldown: &'static str,
    summary: &'static str,
    rank_detail: &'static str,
}

pub(crate) fn draw(game: &Game) {
    let w = 900.0;
    let h = 520.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), "Skill Book");

    draw_text(
        &format!(
            "Unspent skill points {}",
            game.player.stats.unspent_skill_points
        ),
        x + 24.0,
        y + 70.0,
        20.0,
        WHITE,
    );

    let list_rect = Rect::new(x + 24.0, y + 108.0, 356.0, 324.0);
    let detail_rect = Rect::new(x + 404.0, y + 108.0, 472.0, 324.0);
    draw_section_box(list_rect, "Known skills");
    draw_section_box(detail_rect, "Skill detail");

    let skills = [
        SkillRow {
            hotkey: "1",
            name: "Rush",
            rank: game.player.rush_rank,
            cost: "8 mana",
            cooldown: "1.8 sec",
            summary: "Dash forward and strike enemies reached by the rush.",
            rank_detail: "Each rank adds 2 damage.",
        },
        SkillRow {
            hotkey: "2",
            name: "Nova",
            rank: game.player.nova_rank,
            cost: "14 mana",
            cooldown: "3.5 sec",
            summary: "Release a close-range burst around yourself.",
            rank_detail: "Each rank adds 2 damage.",
        },
        SkillRow {
            hotkey: "3",
            name: "Fireball",
            rank: game.player.fireball_rank,
            cost: "12 mana",
            cooldown: "1.2 sec",
            summary: "Launch a fireball that explodes on impact.",
            rank_detail: "Each rank adds 2 damage and widens the blast.",
        },
        SkillRow {
            hotkey: "4",
            name: "Cleave",
            rank: game.player.cleave_rank,
            cost: "10 mana",
            cooldown: "2.2 sec",
            summary: "Sweep a broad melee arc in front of you.",
            rank_detail: "Each rank adds 2 damage.",
        },
    ];

    let mouse = game.ui_hover_position();
    let mut focused_index = game.skill_book_cursor;
    for (index, skill) in skills.iter().enumerate() {
        let row = Rect::new(
            list_rect.x + 14.0,
            list_rect.y + 18.0 + index as f32 * 68.0,
            list_rect.w - 28.0,
            54.0,
        );
        let hovered = row.contains(mouse);
        if hovered {
            focused_index = index;
        }
        if hovered || index == game.skill_book_cursor {
            draw_rectangle(
                row.x,
                row.y,
                row.w,
                row.h,
                with_alpha(GOLD, if hovered { 0.16 } else { 0.1 }),
            );
        }
        draw_text(skill.hotkey, row.x + 12.0, row.y + 23.0, 18.0, MUTED);
        draw_text(
            skill.name,
            row.x + 42.0,
            row.y + 23.0,
            20.0,
            if index == game.skill_book_cursor {
                GOLD
            } else {
                WHITE
            },
        );
        draw_text(
            &format!("Rank {}", skill.rank),
            row.x + 42.0,
            row.y + 45.0,
            16.0,
            MUTED,
        );
        let metadata = format!("{}   {}", skill.cost, skill.cooldown);
        let dims = measure_text(&metadata, None, 16, 1.0);
        draw_text(
            &metadata,
            row.x + row.w - dims.width - 12.0,
            row.y + 32.0,
            16.0,
            MUTED,
        );
    }

    draw_skill_detail(&skills[focused_index], detail_rect);

    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Up/Down", "select", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("Enter", "spend point", vec2(hint_x, y + h - 44.0)) + 12.0;
    hint_x += draw_hotkey_hint("B", "close", vec2(hint_x, y + h - 44.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 44.0));
}

fn draw_skill_detail(skill: &SkillRow, rect: Rect) {
    draw_text(skill.name, rect.x + 18.0, rect.y + 34.0, 26.0, GOLD);
    draw_text(
        &format!("Rank {}", skill.rank),
        rect.x + rect.w - 92.0,
        rect.y + 34.0,
        18.0,
        WHITE,
    );
    draw_text("Cost", rect.x + 18.0, rect.y + 74.0, 16.0, MUTED);
    draw_text(skill.cost, rect.x + 18.0, rect.y + 98.0, 20.0, WHITE);
    draw_text("Cooldown", rect.x + 146.0, rect.y + 74.0, 16.0, MUTED);
    draw_text(skill.cooldown, rect.x + 146.0, rect.y + 98.0, 20.0, WHITE);
    draw_text("Effect", rect.x + 18.0, rect.y + 138.0, 16.0, MUTED);
    for (index, line) in wrap_text(skill.summary, rect.w - 36.0, 20.0, 3)
        .iter()
        .enumerate()
    {
        draw_text(
            line,
            rect.x + 18.0,
            rect.y + 166.0 + index as f32 * 24.0,
            20.0,
            WHITE,
        );
    }
    draw_text("Rank bonus", rect.x + 18.0, rect.y + 248.0, 16.0, MUTED);
    draw_text(
        skill.rank_detail,
        rect.x + 18.0,
        rect.y + 274.0,
        20.0,
        WHITE,
    );
}
