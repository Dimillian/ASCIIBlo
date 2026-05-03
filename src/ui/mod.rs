mod character;
mod hud;
pub(crate) mod inventory;
mod shop;
mod skill_book;
mod trainer;
mod travel;
mod widgets;
mod world_map;

use crate::game::Game;
use macroquad::prelude::Vec2;

pub(crate) fn draw_hud(game: &Game) {
    hud::draw(game);
}

pub(crate) fn draw_inventory(game: &Game) {
    inventory::draw(game);
}

pub(crate) fn inventory_hit_test(game: &Game, pos: Vec2) -> Option<inventory::InventoryHit> {
    inventory::hit_test(game, pos)
}

pub(crate) fn draw_character(game: &Game) {
    character::draw(game);
}

pub(crate) fn draw_skill_book(game: &Game) {
    skill_book::draw(game);
}

pub(crate) fn draw_shop(game: &Game) {
    shop::draw(game);
}

pub(crate) fn draw_trainer(game: &Game) {
    trainer::draw(game);
}

pub(crate) fn draw_travel(game: &Game) {
    travel::draw(game);
}

pub(crate) fn draw_world_map(game: &Game) {
    world_map::draw(game);
}

pub(crate) fn draw_world_hotkey_hint(label: &str, text: &str, pos: Vec2) {
    widgets::draw_hotkey_hint(label, text, pos);
}
