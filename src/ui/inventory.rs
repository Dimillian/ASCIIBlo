use macroquad::prelude::*;

use super::widgets::{
    ITEM_SELECTION, draw_item_detail, draw_modal_backdrop, draw_modal_frame, draw_section_box,
};
use crate::{
    content::{Item, Slot},
    game::{BACKPACK_HEIGHT, BACKPACK_WIDTH, Game, InventoryFocus},
    render::with_alpha,
    stat_display::item_summary,
};

const CELL: f32 = 30.0;
const CELL_GAP: f32 = 3.0;
const PANEL: Color = Color::new(10.0 / 255.0, 12.0 / 255.0, 16.0 / 255.0, 0.72);
const MUTED: Color = Color::new(180.0 / 255.0, 184.0 / 255.0, 190.0 / 255.0, 1.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InventoryHit {
    Backpack(usize),
    Equipment(usize),
}

pub(crate) fn draw(game: &Game) {
    let layout = layout();
    draw_modal_backdrop();
    draw_modal_frame(layout.modal, "Inventory");

    draw_section_box(layout.backpack, "Backpack");
    draw_section_box(layout.equipment, "Equipped");
    draw_section_box(layout.inspect, "Inspect");

    let hover = game.ui_hover_position();
    let hovered_backpack = draw_backpack(game, layout.backpack, hover);
    let hovered_equipment = draw_equipment(game, layout.equipment, hover);
    let inspected = hovered_backpack
        .or(hovered_equipment)
        .or_else(|| focused_item(game));

    if let Some(item) = inspected {
        draw_item_detail(
            &game.sim.player,
            item,
            vec2(layout.inspect.x + 18.0, layout.inspect.y + 30.0),
            layout.inspect.w - 36.0,
        );
    } else {
        draw_text(
            "No item selected",
            layout.inspect.x + 18.0,
            layout.inspect.y + 42.0,
            20.0,
            MUTED,
        );
    }
}

pub(crate) fn hit_test(game: &Game, pos: Vec2) -> Option<InventoryHit> {
    let layout = layout();
    backpack_index_at(game, layout.backpack, pos)
        .map(InventoryHit::Backpack)
        .or_else(|| equipment_index_at(layout.equipment, pos).map(InventoryHit::Equipment))
}

fn draw_backpack<'a>(game: &'a Game, rect: Rect, hover: Vec2) -> Option<&'a Item> {
    let (grid_x, grid_y) = grid_origin(rect);
    draw_text(
        &format!("{} items", game.sim.player.inventory.len()),
        rect.x + rect.w - 74.0,
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
        let is_selected = game.ui.inventory_focus == InventoryFocus::Backpack
            && index == game.ui.inventory_backpack_cursor;
        if is_hovered {
            hovered = Some(&entry.item);
        }
        draw_rectangle(
            item_rect.x,
            item_rect.y,
            item_rect.w,
            item_rect.h,
            with_alpha(entry.item.rarity.color(), 0.12),
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
                with_alpha(entry.item.rarity.color(), 0.82)
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

fn draw_equipment<'a>(game: &'a Game, rect: Rect, hover: Vec2) -> Option<&'a Item> {
    let slots = equipment_slots(rect, game);
    let mut hovered = None;
    for (index, (slot, slot_rect, item)) in slots.iter().enumerate() {
        let is_hovered = slot_rect.contains(hover);
        let is_selected = game.ui.inventory_focus == InventoryFocus::Equipment
            && index == game.ui.inventory_equipment_cursor;
        if is_hovered {
            hovered = *item;
        }

        draw_rectangle(slot_rect.x, slot_rect.y, slot_rect.w, slot_rect.h, PANEL);
        draw_rectangle_lines(
            slot_rect.x + 0.5,
            slot_rect.y + 0.5,
            slot_rect.w - 1.0,
            slot_rect.h - 1.0,
            if is_selected || is_hovered { 2.0 } else { 1.0 },
            if is_selected || is_hovered {
                ITEM_SELECTION
            } else if let Some(item) = item {
                with_alpha(item.rarity.color(), 0.72)
            } else {
                with_alpha(WHITE, 0.1)
            },
        );
        draw_text(
            slot.label(),
            slot_rect.x + 14.0,
            slot_rect.y + 21.0,
            15.0,
            MUTED,
        );
        draw_text(
            item.map(|item| item.name.as_str()).unwrap_or("-"),
            slot_rect.x + 14.0,
            slot_rect.y + 48.0,
            19.0,
            item.map(|item| item.rarity.color()).unwrap_or(WHITE),
        );
        draw_text(
            item.map(item_summary)
                .filter(|summary| !summary.is_empty())
                .as_deref()
                .unwrap_or("Empty slot"),
            slot_rect.x + 14.0,
            slot_rect.y + 76.0,
            16.0,
            item.map(|_| WHITE).unwrap_or(MUTED),
        );
    }
    hovered
}

fn focused_item(game: &Game) -> Option<&Item> {
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
        Rect::new(rect.x + 18.0, rect.y + 24.0, rect.w - 36.0, 104.0),
        Rect::new(rect.x + 18.0, rect.y + 146.0, rect.w - 36.0, 104.0),
        Rect::new(rect.x + 18.0, rect.y + 268.0, rect.w - 36.0, 104.0),
    ]
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
    (rect.x + 18.0, rect.y + 24.0)
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
        backpack: Rect::new(x + 24.0, y + 88.0, 330.0, 428.0),
        equipment: Rect::new(x + 374.0, y + 88.0, 238.0, 428.0),
        inspect: Rect::new(x + 632.0, y + 88.0, 304.0, 428.0),
    }
}

struct InventoryLayout {
    modal: Rect,
    backpack: Rect,
    equipment: Rect,
    inspect: Rect,
}
