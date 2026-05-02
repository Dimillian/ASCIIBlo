use macroquad::prelude::*;

use super::{Game, UiMode, combat_feed_rect};

#[derive(Default)]
pub(super) struct InputState {
    pub(super) movement: Vec2,
    pub(super) aim_world: Vec2,
    pub(super) attack_pressed: bool,
    pub(super) rush_pressed: bool,
    pub(super) nova_pressed: bool,
    pub(super) fireball_pressed: bool,
    pub(super) cleave_pressed: bool,
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
        self.input.movement = movement.normalize_or_zero();
        self.input.aim_world = aim_world;
        self.input.attack_pressed |=
            is_mouse_button_pressed(MouseButton::Left) || is_key_pressed(KeyCode::Space);
        self.input.rush_pressed |= is_key_pressed(KeyCode::Key1);
        self.input.nova_pressed |= is_key_pressed(KeyCode::Key2);
        self.input.fireball_pressed |= is_key_pressed(KeyCode::Key3);
        self.input.cleave_pressed |= is_key_pressed(KeyCode::Key4);
        self.input.pickup_pressed |= is_key_pressed(KeyCode::E);
        self.input.inventory_toggle_pressed |= is_key_pressed(KeyCode::Tab);
        self.input.character_toggle_pressed |= is_key_pressed(KeyCode::C);
        self.input.skill_book_toggle_pressed |= is_key_pressed(KeyCode::B);
        self.input.world_map_toggle_pressed |= is_key_pressed(KeyCode::M);
        self.input.inventory_up_pressed |= is_key_pressed(KeyCode::Up);
        self.input.inventory_down_pressed |= is_key_pressed(KeyCode::Down);
        self.input.nav_left_pressed |= is_key_pressed(KeyCode::Left);
        self.input.nav_right_pressed |= is_key_pressed(KeyCode::Right);
        self.input.inventory_equip_pressed |= is_key_pressed(KeyCode::Enter);
        self.input.inventory_drop_pressed |= is_key_pressed(KeyCode::Backspace);
        self.input.map_recenter_pressed |= is_key_pressed(KeyCode::R);
        let wheel_y = mouse_wheel().1;
        if self.ui_mode == UiMode::WorldMap {
            self.input.map_zoom_delta += wheel_y;
        } else if self.ui_mode == UiMode::None
            && combat_feed_rect().contains(mouse_position().into())
        {
            self.input.log_scroll_delta += wheel_y.round() as i32;
        }
        self.input.interact_pressed |= is_key_pressed(KeyCode::F);
        self.input.quit_pressed |= is_key_pressed(KeyCode::Escape);
        if self.ui_mode == UiMode::WorldMap {
            if is_key_down(KeyCode::Up) {
                self.input.movement.y -= 1.0;
            }
            if is_key_down(KeyCode::Down) {
                self.input.movement.y += 1.0;
            }
            if is_key_down(KeyCode::Left) {
                self.input.movement.x -= 1.0;
            }
            if is_key_down(KeyCode::Right) {
                self.input.movement.x += 1.0;
            }
            self.input.movement = self.input.movement.normalize_or_zero();
        }
    }

    pub(super) fn clear_edge_inputs(&mut self) {
        self.input.attack_pressed = false;
        self.input.rush_pressed = false;
        self.input.nova_pressed = false;
        self.input.fireball_pressed = false;
        self.input.cleave_pressed = false;
        self.input.pickup_pressed = false;
        self.input.inventory_toggle_pressed = false;
        self.input.character_toggle_pressed = false;
        self.input.skill_book_toggle_pressed = false;
        self.input.world_map_toggle_pressed = false;
        self.input.inventory_up_pressed = false;
        self.input.inventory_down_pressed = false;
        self.input.nav_left_pressed = false;
        self.input.nav_right_pressed = false;
        self.input.inventory_equip_pressed = false;
        self.input.inventory_drop_pressed = false;
        self.input.map_recenter_pressed = false;
        self.input.map_zoom_delta = 0.0;
        self.input.log_scroll_delta = 0;
        self.input.interact_pressed = false;
        self.input.quit_pressed = false;
    }
}
