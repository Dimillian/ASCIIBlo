use macroquad::prelude::*;

use super::{Game, UiMode, combat_feed_rect};

#[derive(Default)]
pub(super) struct InputState {
    pub(super) movement: Vec2,
    pub(super) aim_world: Vec2,
    pub(super) attack_pressed: bool,
    pub(super) ability_slot_pressed: [bool; 2],
    pub(super) pickup_pressed: bool,
    pub(super) inventory_toggle_pressed: bool,
    pub(super) character_toggle_pressed: bool,
    pub(super) skill_book_toggle_pressed: bool,
    pub(super) world_map_toggle_pressed: bool,
    pub(super) inventory_up_pressed: bool,
    pub(super) inventory_down_pressed: bool,
    pub(super) nav_left_pressed: bool,
    pub(super) nav_right_pressed: bool,
    pub(super) inventory_equip_pressed: bool,
    pub(super) inventory_drop_pressed: bool,
    pub(super) map_recenter_pressed: bool,
    pub(super) map_zoom_delta: f32,
    pub(super) log_scroll_delta: i32,
    pub(super) interact_pressed: bool,
    pub(super) quit_pressed: bool,
}

impl Game {
    pub fn collect_input(&mut self, aim_world: Vec2) {
        let mut movement = Vec2::ZERO;
        if is_key_down(KeyCode::W) {
            movement.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            movement.y += 1.0;
        }
        if is_key_down(KeyCode::A) {
            movement.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            movement.x += 1.0;
        }
        self.runtime.input.movement = movement.normalize_or_zero();
        self.runtime.input.aim_world = aim_world;
        self.runtime.input.attack_pressed |=
            is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space);
        self.runtime.input.ability_slot_pressed[0] |= is_key_pressed(KeyCode::Key1);
        self.runtime.input.ability_slot_pressed[1] |= is_key_pressed(KeyCode::Key2);
        self.runtime.input.pickup_pressed |= is_key_pressed(KeyCode::E);
        self.runtime.input.inventory_toggle_pressed |= is_key_pressed(KeyCode::Tab);
        self.runtime.input.character_toggle_pressed |= is_key_pressed(KeyCode::C);
        self.runtime.input.skill_book_toggle_pressed |= is_key_pressed(KeyCode::B);
        self.runtime.input.world_map_toggle_pressed |= is_key_pressed(KeyCode::M);
        self.runtime.input.inventory_up_pressed |= is_key_pressed(KeyCode::Up);
        self.runtime.input.inventory_down_pressed |= is_key_pressed(KeyCode::Down);
        self.runtime.input.nav_left_pressed |= is_key_pressed(KeyCode::Left);
        self.runtime.input.nav_right_pressed |= is_key_pressed(KeyCode::Right);
        self.runtime.input.inventory_equip_pressed |= is_key_pressed(KeyCode::Enter);
        self.runtime.input.inventory_drop_pressed |= is_key_pressed(KeyCode::Backspace);
        self.runtime.input.map_recenter_pressed |= is_key_pressed(KeyCode::R);
        let wheel_y = mouse_wheel().1;
        if self.ui.mode == UiMode::WorldMap {
            self.runtime.input.map_zoom_delta += wheel_y;
        } else if self.ui.mode == UiMode::None
            && combat_feed_rect().contains(mouse_position().into())
        {
            self.runtime.input.log_scroll_delta += wheel_y.round() as i32;
        }
        self.runtime.input.interact_pressed |= is_key_pressed(KeyCode::F);
        self.runtime.input.quit_pressed |= is_key_pressed(KeyCode::Escape);
        if self.ui.mode == UiMode::WorldMap {
            if is_key_down(KeyCode::Up) {
                self.runtime.input.movement.y -= 1.0;
            }
            if is_key_down(KeyCode::Down) {
                self.runtime.input.movement.y += 1.0;
            }
            if is_key_down(KeyCode::Left) {
                self.runtime.input.movement.x -= 1.0;
            }
            if is_key_down(KeyCode::Right) {
                self.runtime.input.movement.x += 1.0;
            }
            self.runtime.input.movement = self.runtime.input.movement.normalize_or_zero();
        }
    }

    pub(super) fn clear_edge_inputs(&mut self) {
        self.runtime.input.attack_pressed = false;
        self.runtime.input.ability_slot_pressed = [false; 2];
        self.runtime.input.pickup_pressed = false;
        self.runtime.input.inventory_toggle_pressed = false;
        self.runtime.input.character_toggle_pressed = false;
        self.runtime.input.skill_book_toggle_pressed = false;
        self.runtime.input.world_map_toggle_pressed = false;
        self.runtime.input.inventory_up_pressed = false;
        self.runtime.input.inventory_down_pressed = false;
        self.runtime.input.nav_left_pressed = false;
        self.runtime.input.nav_right_pressed = false;
        self.runtime.input.inventory_equip_pressed = false;
        self.runtime.input.inventory_drop_pressed = false;
        self.runtime.input.map_recenter_pressed = false;
        self.runtime.input.map_zoom_delta = 0.0;
        self.runtime.input.log_scroll_delta = 0;
        self.runtime.input.interact_pressed = false;
        self.runtime.input.quit_pressed = false;
    }
}
