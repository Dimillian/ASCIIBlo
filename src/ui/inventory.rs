use macroquad::prelude::*;

use super::widgets::{
    CHROME_DIM, ITEM_SELECTION, draw_hotkey_hint, draw_interior_card, draw_item_detail,
    draw_modal_backdrop, draw_modal_frame, draw_section_box,
};
use crate::{
    content::{Item, Slot},
    game::{BACKPACK_HEIGHT, BACKPACK_WIDTH, Game, InventoryFocus},
    render::with_alpha,
    stat_display::item_summary,
};

const CELL: f32 = 30.0;
const CELL_GAP: f32 = 3.0;
const MUTED: Color = Color::new(180.0 / 255.0, 184.0 / 255.0, 190.0 / 255.0, 1.0);
const GOLD: Color = Color::new(255.0 / 255.0, 224.0 / 255.0, 96.0 / 255.0, 1.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InventoryHit {
    Backpack(usize),
    Equipment(usize),
}

pub(crate) fn draw(game: &Game) {
    let layout = layout();
    draw_modal_backdrop();
    draw_modal_frame(layout.modal, "Inventory");

    draw_section_box(layout.loadout, "Loadout");
    draw_section_box(layout.backpack, "Backpack");
    draw_section_box(layout.inspect, "Inspect");

    let pointer = game.ui_hover_position();
    let hover = if game
        .ui
        .inventory_hover_suppressed_at
        .is_some_and(|suppressed_at| suppressed_at.distance(pointer) <= 1.0)
    {
        vec2(-1_000.0, -1_000.0)
    } else {
        pointer
    };
    let hovered_equipment = draw_loadout(game, layout.loadout, hover);
    let hovered_backpack = draw_backpack(game, layout.backpack, hover);
    let inspected = hovered_equipment
        .or(hovered_backpack)
        .or_else(|| focused_item(game));

    draw_inspector(game, layout.inspect, inspected);
    draw_footer_hints(game, layout.modal);
}

pub(crate) fn hit_test(game: &Game, pos: Vec2) -> Option<InventoryHit> {
    let layout = layout();
    equipment_index_at(layout.loadout, pos)
        .map(InventoryHit::Equipment)
        .or_else(|| backpack_index_at(game, layout.backpack, pos).map(InventoryHit::Backpack))
}

fn draw_backpack<'a>(game: &'a Game, rect: Rect, hover: Vec2) -> Option<&'a Item> {
    let (grid_x, grid_y) = grid_origin(rect);
    draw_text(
        &format!(
            "{} items   {} / {} cells",
            game.sim.player.inventory.len(),
            occupied_cells(game),
            BACKPACK_WIDTH * BACKPACK_HEIGHT
        ),
        rect.x + 18.0,
        rect.y + 20.0,
        15.0,
        MUTED,
    );
    for y in 0..BACKPACK_HEIGHT {
        for x in 0..BACKPACK_WIDTH {
            let cell = cell_rect(grid_x, grid_y, x, y, 1, 1);
            draw_rectangle(cell.x, cell.y, cell.w, cell.h, with_alpha(WHITE, 0.035));
            draw_rectangle_lines(cell.x, cell.y, cell.w, cell.h, 1.0, with_alpha(WHITE, 0.08));
        }
    }

    let mut hovered = None;
    for (index, entry) in game.sim.player.inventory.entries().iter().enumerate() {
        let item_rect = entry_rect(rect, entry);
        let is_hovered = item_rect.contains(hover);
        let is_selected = game.ui.inventory_selection_active
            && game.ui.inventory_focus == InventoryFocus::Backpack
            && index == game.ui.inventory_backpack_cursor;
        if is_hovered {
            hovered = Some(&entry.item);
        }
        draw_rectangle(
            item_rect.x,
            item_rect.y,
            item_rect.w,
            item_rect.h,
            if is_selected || is_hovered {
                with_alpha(entry.item.rarity.color(), 0.18)
            } else {
                with_alpha(BLACK, 0.12)
            },
        );
        draw_rectangle_lines(
            item_rect.x + 0.5,
            item_rect.y + 0.5,
            item_rect.w - 1.0,
            item_rect.h - 1.0,
            if is_selected || is_hovered { 2.0 } else { 1.0 },
            if is_selected || is_hovered {
                ITEM_SELECTION
            } else {
                with_alpha(CHROME_DIM, 0.42)
            },
        );
        draw_text(
            slot_glyph(entry.item.slot),
            item_rect.x + 10.0,
            item_rect.y + 23.0,
            20.0,
            entry.item.rarity.color(),
        );
        if entry.item.footprint().width > 1 {
            draw_text(
                &entry.item.base_name,
                item_rect.x + 10.0,
                item_rect.y + item_rect.h - 12.0,
                15.0,
                WHITE,
            );
        }
    }
    hovered
}

fn draw_loadout<'a>(game: &'a Game, rect: Rect, hover: Vec2) -> Option<&'a Item> {
    let slots = equipment_slots(rect, game);
    let mut hovered = None;
    for (index, (slot, slot_rect, item)) in slots.iter().enumerate() {
        let is_hovered = slot_rect.contains(hover);
        let is_selected = game.ui.inventory_selection_active
            && game.ui.inventory_focus == InventoryFocus::Equipment
            && index == game.ui.inventory_equipment_cursor;
        if is_hovered {
            hovered = *item;
        }

        draw_interior_card(
            *slot_rect,
            if is_selected || is_hovered {
                ITEM_SELECTION
            } else if let Some(item) = item {
                item.rarity.color()
            } else {
                CHROME_DIM
            },
            is_selected || is_hovered,
        );
        draw_text(
            slot.label(),
            slot_rect.x + 12.0,
            slot_rect.y + 19.0,
            15.0,
            MUTED,
        );
        if let Some(item) = item {
            draw_text(
                &item.name,
                slot_rect.x + 12.0,
                slot_rect.y + 42.0,
                18.0,
                item.rarity.color(),
            );
            draw_text(
                &item_summary(item),
                slot_rect.x + 12.0,
                slot_rect.y + 64.0,
                15.0,
                WHITE,
            );
        } else {
            draw_text(
                "Empty slot",
                slot_rect.x + 12.0,
                slot_rect.y + 47.0,
                17.0,
                MUTED,
            );
        }
    }

    draw_text("Current sheet", rect.x + 18.0, rect.y + 286.0, 17.0, GOLD);
    draw_sheet_stat(
        rect.x + 18.0,
        rect.y + 318.0,
        "POW",
        game.sim.player.power(),
    );
    draw_sheet_stat(
        rect.x + 104.0,
        rect.y + 318.0,
        "ARM",
        game.sim.player.armor(),
    );
    draw_sheet_stat(
        rect.x + 18.0,
        rect.y + 352.0,
        "Life",
        game.sim.player.max_hp().round() as i32,
    );
    draw_sheet_stat(
        rect.x + 104.0,
        rect.y + 352.0,
        "Speed",
        game.sim.player.move_speed_rating().round() as i32,
    );
    hovered
}

fn draw_sheet_stat(x: f32, y: f32, label: &str, value: i32) {
    draw_text(label, x, y, 15.0, MUTED);
    draw_text(&value.to_string(), x + 46.0, y, 18.0, WHITE);
}

fn draw_inspector(game: &Game, rect: Rect, item: Option<&Item>) {
    if let Some(item) = item {
        draw_item_detail(
            &game.sim.player,
            item,
            vec2(rect.x + 18.0, rect.y + 22.0),
            rect.w - 36.0,
        );
    } else {
        draw_text(
            "No item selected",
            rect.x + 18.0,
            rect.y + 42.0,
            20.0,
            MUTED,
        );
    }
}

fn draw_footer_hints(game: &Game, rect: Rect) {
    let mut hint_x = rect.x + 24.0;
    let hint_y = rect.y + rect.h - 40.0;
    hint_x += draw_hotkey_hint("Left/Right", "focus", vec2(hint_x, hint_y)) + 12.0;
    hint_x += draw_hotkey_hint("Up/Down", "select", vec2(hint_x, hint_y)) + 12.0;
    match game.ui.inventory_focus {
        InventoryFocus::Backpack => {
            hint_x += draw_hotkey_hint("Enter", "equip", vec2(hint_x, hint_y)) + 12.0;
            hint_x += draw_hotkey_hint("Backspace", "drop", vec2(hint_x, hint_y)) + 12.0;
        }
        InventoryFocus::Equipment => {
            hint_x += draw_hotkey_hint("Enter", "unequip", vec2(hint_x, hint_y)) + 12.0;
        }
    }
    draw_hotkey_hint("Tab", "close", vec2(hint_x, hint_y));
}

fn focused_item(game: &Game) -> Option<&Item> {
    if !game.ui.inventory_selection_active {
        return None;
    }
    match game.ui.inventory_focus {
        InventoryFocus::Backpack => game
            .sim
            .player
            .inventory
            .item(game.ui.inventory_backpack_cursor),
        InventoryFocus::Equipment => match game.ui.inventory_equipment_cursor {
            0 => game.sim.player.equipment.weapon.as_ref(),
            1 => game.sim.player.equipment.armor.as_ref(),
            _ => game.sim.player.equipment.charm.as_ref(),
        },
    }
}

fn backpack_index_at(game: &Game, rect: Rect, pos: Vec2) -> Option<usize> {
    game.sim
        .player
        .inventory
        .entries()
        .iter()
        .enumerate()
        .find_map(|(index, entry)| entry_rect(rect, entry).contains(pos).then_some(index))
}

fn equipment_index_at(rect: Rect, pos: Vec2) -> Option<usize> {
    equipment_slot_rects(rect)
        .iter()
        .enumerate()
        .find_map(|(index, slot_rect)| slot_rect.contains(pos).then_some(index))
}

fn equipment_slots<'a>(rect: Rect, game: &'a Game) -> [(Slot, Rect, Option<&'a Item>); 3] {
    let rects = equipment_slot_rects(rect);
    [
        (
            Slot::Weapon,
            rects[0],
            game.sim.player.equipment.weapon.as_ref(),
        ),
        (
            Slot::Armor,
            rects[1],
            game.sim.player.equipment.armor.as_ref(),
        ),
        (
            Slot::Charm,
            rects[2],
            game.sim.player.equipment.charm.as_ref(),
        ),
    ]
}

fn equipment_slot_rects(rect: Rect) -> [Rect; 3] {
    [
        Rect::new(rect.x + 18.0, rect.y + 24.0, rect.w - 36.0, 68.0),
        Rect::new(rect.x + 18.0, rect.y + 106.0, rect.w - 36.0, 68.0),
        Rect::new(rect.x + 18.0, rect.y + 188.0, rect.w - 36.0, 68.0),
    ]
}

fn occupied_cells(game: &Game) -> usize {
    game.sim
        .player
        .inventory
        .entries()
        .iter()
        .map(|entry| {
            let footprint = entry.item.footprint();
            footprint.width * footprint.height
        })
        .sum()
}

fn entry_rect(rect: Rect, entry: &crate::game::BackpackEntry) -> Rect {
    let (grid_x, grid_y) = grid_origin(rect);
    let footprint = entry.item.footprint();
    cell_rect(
        grid_x,
        grid_y,
        entry.x,
        entry.y,
        footprint.width,
        footprint.height,
    )
}

fn grid_origin(rect: Rect) -> (f32, f32) {
    (rect.x + 18.0, rect.y + 42.0)
}

fn cell_rect(grid_x: f32, grid_y: f32, x: usize, y: usize, width: usize, height: usize) -> Rect {
    Rect::new(
        grid_x + x as f32 * (CELL + CELL_GAP),
        grid_y + y as f32 * (CELL + CELL_GAP),
        width as f32 * CELL + width.saturating_sub(1) as f32 * CELL_GAP,
        height as f32 * CELL + height.saturating_sub(1) as f32 * CELL_GAP,
    )
}

fn slot_glyph(slot: Slot) -> &'static str {
    match slot {
        Slot::Weapon => "/",
        Slot::Armor => "#",
        Slot::Charm => "o",
    }
}

fn layout() -> InventoryLayout {
    let w = 960.0;
    let h = 560.0;
    let x = (screen_width() - w) * 0.5;
    let y = (screen_height() - h) * 0.5;
    InventoryLayout {
        modal: Rect::new(x, y, w, h),
        loadout: Rect::new(x + 24.0, y + 88.0, 226.0, 412.0),
        backpack: Rect::new(x + 268.0, y + 88.0, 350.0, 412.0),
        inspect: Rect::new(x + 636.0, y + 88.0, 300.0, 412.0),
    }
}

struct InventoryLayout {
    modal: Rect,
    loadout: Rect,
    backpack: Rect,
    inspect: Rect,
}
