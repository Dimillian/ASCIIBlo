use macroquad::prelude::*;

use crate::{
    game::{AbilityKind, DisciplineKind, Game, SkillBookFocus, abilities_for_discipline},
    render::with_alpha,
};

use super::widgets::{
    draw_hotkey_badge, draw_hotkey_hint, draw_modal_backdrop, draw_modal_frame, draw_section_box,
    wrap_text,
};

const GOLD: Color = Color::new(255.0 / 255.0, 224.0 / 255.0, 96.0 / 255.0, 1.0);
const MUTED: Color = Color::new(180.0 / 255.0, 184.0 / 255.0, 190.0 / 255.0, 1.0);
const PANEL_FILL: Color = Color::new(10.0 / 255.0, 12.0 / 255.0, 16.0 / 255.0, 1.0);

pub(crate) fn draw(game: &Game) {
    let w = 980.0;
    let h = 600.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    draw_modal_backdrop();
    draw_modal_frame(Rect::new(x, y, w, h), "Mastery");

    draw_text(
        "Choose what you practice and which two skills you bring into combat.",
        x + 24.0,
        y + 76.0,
        20.0,
        WHITE,
    );

    let loadout_rect = Rect::new(x + 24.0, y + 100.0, w - 48.0, 62.0);
    let disciplines_rect = Rect::new(x + 24.0, y + 194.0, 252.0, 314.0);
    let skills_rect = Rect::new(x + 296.0, y + 194.0, 290.0, 314.0);
    let detail_rect = Rect::new(x + 606.0, y + 194.0, 350.0, 314.0);

    draw_loadout_strip(game, loadout_rect);
    draw_panel(
        disciplines_rect,
        "Disciplines",
        game.skill_book_focus == SkillBookFocus::Disciplines,
    );
    draw_panel(
        skills_rect,
        if abilities_for_discipline(DisciplineKind::ALL[game.skill_book_cursor]).is_empty() {
            "Milestones"
        } else {
            "Skills"
        },
        game.skill_book_focus == SkillBookFocus::Skills,
    );
    draw_panel(
        detail_rect,
        "Detail",
        game.skill_book_focus == SkillBookFocus::Detail,
    );

    draw_disciplines(game, disciplines_rect);
    let discipline = DisciplineKind::ALL[game.skill_book_cursor];
    let selected = draw_skills(game, discipline, skills_rect);
    draw_detail(game, discipline, selected, detail_rect);

    let mut hint_x = x + 24.0;
    hint_x += draw_hotkey_hint("Left/Right", "panel", vec2(hint_x, y + h - 42.0)) + 12.0;
    hint_x += draw_hotkey_hint("Up/Down", "move", vec2(hint_x, y + h - 42.0)) + 12.0;
    hint_x += draw_hotkey_hint("1/2", "bind", vec2(hint_x, y + h - 42.0)) + 12.0;
    hint_x += draw_hotkey_hint("B", "close", vec2(hint_x, y + h - 42.0)) + 12.0;
    draw_hotkey_hint("Esc", "close", vec2(hint_x, y + h - 42.0));
}

fn draw_panel(rect: Rect, title: &str, focused: bool) {
    draw_section_box(rect, title);
    if focused {
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, with_alpha(GOLD, 0.82));
    }
}

fn draw_loadout_strip(game: &Game, rect: Rect) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, with_alpha(PANEL_FILL, 0.82));
    draw_text("Loadout", rect.x + 16.0, rect.y + 24.0, 18.0, GOLD);
    draw_text(
        "Current battle loadout",
        rect.x + 16.0,
        rect.y + 46.0,
        15.0,
        MUTED,
    );

    let slot_w = 244.0;
    for (index, ability) in game.player.bound_abilities.iter().copied().enumerate() {
        let slot = Rect::new(
            rect.x + 320.0 + index as f32 * 262.0,
            rect.y + 10.0,
            slot_w,
            42.0,
        );
        draw_rectangle(
            slot.x,
            slot.y,
            slot.w,
            slot.h,
            with_alpha(ability.color(), 0.10),
        );
        draw_rectangle_lines(
            slot.x,
            slot.y,
            slot.w,
            slot.h,
            1.0,
            with_alpha(ability.color(), 0.7),
        );
        draw_hotkey_badge(&(index + 1).to_string(), vec2(slot.x + 10.0, slot.y + 9.0));
        draw_text(
            ability.glyph(),
            slot.x + 56.0,
            slot.y + 28.0,
            20.0,
            ability.color(),
        );
        draw_text(ability.name(), slot.x + 82.0, slot.y + 28.0, 19.0, WHITE);
    }
}

fn draw_disciplines(game: &Game, rect: Rect) {
    for (index, kind) in DisciplineKind::ALL.iter().copied().enumerate() {
        let row = Rect::new(
            rect.x + 12.0,
            rect.y + 16.0 + index as f32 * 72.0,
            rect.w - 24.0,
            58.0,
        );
        let selected = index == game.skill_book_cursor;
        if selected {
            draw_rectangle(
                row.x,
                row.y,
                row.w,
                row.h,
                with_alpha(
                    GOLD,
                    if game.skill_book_focus == SkillBookFocus::Disciplines {
                        0.18
                    } else {
                        0.10
                    },
                ),
            );
        }
        let progress = game.player.disciplines.get(kind);
        draw_text(
            kind.name(),
            row.x + 10.0,
            row.y + 20.0,
            18.0,
            if selected { GOLD } else { WHITE },
        );
        draw_text(
            &format!("L{}", progress.level),
            row.x + row.w - 34.0,
            row.y + 20.0,
            16.0,
            WHITE,
        );
        draw_progress_bar(
            vec2(row.x + 10.0, row.y + 31.0),
            row.w - 20.0,
            progress.xp as f32 / progress.next_xp as f32,
            kind.color(),
        );
        draw_text(
            &format!("{} / {}", progress.xp, progress.next_xp),
            row.x + 10.0,
            row.y + 50.0,
            15.0,
            MUTED,
        );
        draw_text(
            &discipline_next_label(kind, progress.level),
            row.x + 76.0,
            row.y + 50.0,
            15.0,
            MUTED,
        );
    }
}

fn draw_skills(game: &Game, discipline: DisciplineKind, rect: Rect) -> Option<AbilityKind> {
    let abilities = abilities_for_discipline(discipline);
    if abilities.is_empty() {
        draw_passive_milestones(game, discipline, rect);
        return None;
    }

    let selected = abilities[game.skill_book_ability_cursor.min(abilities.len() - 1)];
    for (index, ability) in abilities.iter().copied().enumerate() {
        let row = Rect::new(
            rect.x + 12.0,
            rect.y + 16.0 + index as f32 * 72.0,
            rect.w - 24.0,
            58.0,
        );
        let unlocked = game.player.is_ability_unlocked(ability);
        let is_selected = index == game.skill_book_ability_cursor;
        if is_selected {
            draw_rectangle(
                row.x,
                row.y,
                row.w,
                row.h,
                with_alpha(
                    GOLD,
                    if game.skill_book_focus == SkillBookFocus::Skills {
                        0.22
                    } else {
                        0.12
                    },
                ),
            );
        }
        draw_rectangle(
            row.x + 10.0,
            row.y + 10.0,
            26.0,
            26.0,
            with_alpha(ability.color(), if unlocked { 0.18 } else { 0.08 }),
        );
        draw_text(
            ability.glyph(),
            row.x + 18.0,
            row.y + 29.0,
            18.0,
            if unlocked { ability.color() } else { MUTED },
        );
        draw_text(
            ability.name(),
            row.x + 48.0,
            row.y + 22.0,
            18.0,
            if unlocked { WHITE } else { MUTED },
        );
        draw_text(
            if unlocked {
                "Unlocked".into()
            } else {
                format!("Level {}", ability.unlock_level())
            }
            .as_str(),
            row.x + 48.0,
            row.y + 44.0,
            15.0,
            MUTED,
        );
        if let Some(slot) = game.player.bound_slot(ability) {
            draw_badge(
                &format!("Bound {}", slot + 1),
                row.x + row.w - 76.0,
                row.y + 12.0,
                ability.color(),
            );
        }
    }
    Some(selected)
}

fn draw_passive_milestones(game: &Game, discipline: DisciplineKind, rect: Rect) {
    let progress = game.player.disciplines.get(discipline);
    draw_text(
        "Passive discipline",
        rect.x + 16.0,
        rect.y + 28.0,
        18.0,
        WHITE,
    );
    for offset in 1..=4 {
        let level = progress.level + offset;
        let row = Rect::new(
            rect.x + 12.0,
            rect.y + 42.0 + offset as f32 * 52.0,
            rect.w - 24.0,
            40.0,
        );
        draw_rectangle(row.x, row.y, row.w, row.h, with_alpha(WHITE, 0.03));
        draw_text(
            &format!("Level {}", level),
            row.x + 12.0,
            row.y + 25.0,
            17.0,
            WHITE,
        );
        draw_text(
            &passive_bonus_at_level(discipline, level),
            row.x + 90.0,
            row.y + 25.0,
            16.0,
            MUTED,
        );
    }
}

fn draw_detail(game: &Game, discipline: DisciplineKind, selected: Option<AbilityKind>, rect: Rect) {
    let progress = game.player.disciplines.get(discipline);
    draw_text(discipline.name(), rect.x + 16.0, rect.y + 30.0, 24.0, GOLD);
    draw_text(
        &format!("Level {}", progress.level),
        rect.x + rect.w - 92.0,
        rect.y + 30.0,
        17.0,
        WHITE,
    );
    draw_text("Current bonus", rect.x + 16.0, rect.y + 62.0, 15.0, MUTED);
    for (index, line) in wrap_text(&current_bonus(game, discipline), rect.w - 32.0, 17.0, 2)
        .iter()
        .enumerate()
    {
        draw_text(
            line,
            rect.x + 16.0,
            rect.y + 84.0 + index as f32 * 20.0,
            17.0,
            WHITE,
        );
    }
    draw_text("Next reward", rect.x + 16.0, rect.y + 130.0, 15.0, MUTED);
    draw_text(
        &next_reward(discipline, progress.level),
        rect.x + 16.0,
        rect.y + 152.0,
        17.0,
        WHITE,
    );

    let Some(ability) = selected else {
        draw_text(
            "No active skills",
            rect.x + 16.0,
            rect.y + 198.0,
            22.0,
            discipline.color(),
        );
        for (index, line) in wrap_text(
            "This discipline improves passively as it levels.",
            rect.w - 32.0,
            17.0,
            2,
        )
        .iter()
        .enumerate()
        {
            draw_text(
                line,
                rect.x + 16.0,
                rect.y + 224.0 + index as f32 * 20.0,
                17.0,
                WHITE,
            );
        }
        return;
    };

    draw_text(
        ability.name(),
        rect.x + 16.0,
        rect.y + 198.0,
        22.0,
        ability.color(),
    );
    draw_text(
        &format!("{} mana", ability.mana_cost() as i32),
        rect.x + 16.0,
        rect.y + 224.0,
        16.0,
        WHITE,
    );
    draw_text(
        &format!("{:.1} sec", ability.cooldown()),
        rect.x + 110.0,
        rect.y + 224.0,
        16.0,
        WHITE,
    );
    for (index, line) in wrap_text(ability.summary(), rect.w - 32.0, 17.0, 2)
        .iter()
        .enumerate()
    {
        draw_text(
            line,
            rect.x + 16.0,
            rect.y + 250.0 + index as f32 * 20.0,
            17.0,
            if game.player.is_ability_unlocked(ability) {
                WHITE
            } else {
                MUTED
            },
        );
    }
    draw_binding_prompts(game, ability, rect);
}

fn draw_binding_prompts(game: &Game, ability: AbilityKind, rect: Rect) {
    if !game.player.is_ability_unlocked(ability) {
        draw_text(
            &format!("Unlocks at level {}", ability.unlock_level()),
            rect.x + 16.0,
            rect.y + rect.h - 16.0,
            16.0,
            MUTED,
        );
        return;
    }
    for slot in 0..game.player.bound_abilities.len() {
        let line_y = rect.y + rect.h - 36.0 + slot as f32 * 20.0;
        draw_text(
            &binding_prompt(game, ability, slot),
            rect.x + 16.0,
            line_y,
            16.0,
            if slot == 0 { GOLD } else { WHITE },
        );
    }
}

fn binding_prompt(game: &Game, ability: AbilityKind, slot: usize) -> String {
    if game.player.bound_abilities[slot] == ability {
        return format!("{} already bound to {}", ability.name(), slot + 1);
    }
    if game.player.bound_slot(ability).is_some() {
        return format!(
            "Press {} to swap with {}",
            slot + 1,
            game.player.bound_abilities[slot].name()
        );
    }
    format!(
        "Press {} to replace {}",
        slot + 1,
        game.player.bound_abilities[slot].name()
    )
}

fn draw_badge(text: &str, x: f32, y: f32, color: Color) {
    let dims = measure_text(text, None, 14, 1.0);
    let width = dims.width + 14.0;
    draw_rectangle(x, y, width, 22.0, with_alpha(color, 0.12));
    draw_rectangle_lines(x, y, width, 22.0, 1.0, with_alpha(color, 0.7));
    draw_text(text, x + 7.0, y + 15.0, 14.0, color);
}

fn current_bonus(game: &Game, kind: DisciplineKind) -> String {
    match kind {
        DisciplineKind::Melee => format!(
            "+{} physical damage on basic attacks and melee skills.",
            game.player.melee_damage_bonus()
        ),
        DisciplineKind::Magic => format!(
            "+{} spell damage. +{:.1} mana/sec regeneration.",
            game.player.magic_damage_bonus(),
            game.player.magic_regen_bonus()
        ),
        DisciplineKind::Armor => format!(
            "+{} effective armor from mastery.",
            game.player.armor_mastery_bonus()
        ),
        DisciplineKind::Agility => format!(
            "+{} movement speed from mastery.",
            game.player.agility_mastery_bonus()
        ),
    }
}

fn next_reward(kind: DisciplineKind, level: i32) -> String {
    abilities_for_discipline(kind)
        .iter()
        .find(|ability| ability.unlock_level() > level)
        .map(|ability| {
            format!(
                "Level {} unlocks {}.",
                ability.unlock_level(),
                ability.name()
            )
        })
        .unwrap_or_else(|| match kind {
            DisciplineKind::Armor | DisciplineKind::Agility => {
                format!(
                    "Level {} grants {}.",
                    level + 1,
                    passive_bonus_at_level(kind, level + 1)
                )
            }
            _ => "All active skills unlocked.".into(),
        })
}

fn discipline_next_label(kind: DisciplineKind, level: i32) -> String {
    abilities_for_discipline(kind)
        .iter()
        .find(|ability| ability.unlock_level() > level)
        .map(|ability| format!("Next {} L{}", ability.name(), ability.unlock_level()))
        .unwrap_or_else(|| match kind {
            DisciplineKind::Armor | DisciplineKind::Agility => {
                format!("Next {}", passive_bonus_at_level(kind, level + 1))
            }
            _ => "All skills unlocked".into(),
        })
}

fn passive_bonus_at_level(kind: DisciplineKind, level: i32) -> String {
    match kind {
        DisciplineKind::Armor => format!("+{} armor", (level - 1).max(0)),
        DisciplineKind::Agility => format!("+{} speed", (level - 1).max(0) * 6),
        _ => String::new(),
    }
}

fn draw_progress_bar(pos: Vec2, width: f32, ratio: f32, color: Color) {
    draw_rectangle(pos.x, pos.y, width, 8.0, with_alpha(BLACK, 0.55));
    draw_rectangle(pos.x, pos.y, width * ratio.clamp(0.0, 1.0), 8.0, color);
}
