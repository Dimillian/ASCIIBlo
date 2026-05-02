use macroquad::prelude::*;

use crate::{content::NpcKind, world::World};

use super::{EXPLORATION_RADIUS, Game, ShopTab, TRAVEL_DESTINATIONS, UiMode};

impl Game {
    pub(super) fn update_inventory_controls(&mut self) {
        if self.input.inventory_up_pressed {
            self.inventory_cursor = self.inventory_cursor.saturating_sub(1);
        }
        if self.input.inventory_down_pressed && !self.player.inventory.is_empty() {
            self.inventory_cursor =
                (self.inventory_cursor + 1).min(self.player.inventory.len() - 1);
        }
        if self.input.inventory_equip_pressed {
            self.equip_selected_item();
        }
        if self.input.inventory_drop_pressed {
            self.drop_selected_item();
        }
    }

    pub(super) fn update_character_controls(&mut self) {
        if self.input.inventory_up_pressed {
            self.character_cursor = self.character_cursor.saturating_sub(1);
        }
        if self.input.inventory_down_pressed {
            self.character_cursor = (self.character_cursor + 1).min(2);
        }
        if self.input.inventory_equip_pressed {
            match self.character_cursor {
                0 if self.player.stats.unspent_stat_points > 0 => {
                    self.player.stats.strength += 1;
                    self.player.stats.unspent_stat_points -= 1;
                    self.log("Strength rises.".into());
                }
                1 if self.player.stats.unspent_stat_points > 0 => {
                    self.player.stats.agility += 1;
                    self.player.stats.unspent_stat_points -= 1;
                    self.log("Agility sharpens.".into());
                }
                2 if self.player.stats.unspent_stat_points > 0 => {
                    self.player.stats.vitality += 1;
                    self.player.stats.unspent_stat_points -= 1;
                    self.player.hp = self.player.max_hp();
                    self.log("Vitality deepens.".into());
                }
                _ => {}
            }
        }
    }

    pub(super) fn update_skill_book_controls(&mut self) {
        if self.input.inventory_up_pressed {
            self.skill_book_cursor = self.skill_book_cursor.saturating_sub(1);
        }
        if self.input.inventory_down_pressed {
            self.skill_book_cursor = (self.skill_book_cursor + 1).min(3);
        }
        if self.input.inventory_equip_pressed && self.player.stats.unspent_skill_points > 0 {
            match self.skill_book_cursor {
                0 => {
                    self.player.rush_rank += 1;
                    self.log(format!("Rush reaches rank {}.", self.player.rush_rank));
                }
                1 => {
                    self.player.nova_rank += 1;
                    self.log(format!("Nova reaches rank {}.", self.player.nova_rank));
                }
                2 => {
                    self.player.fireball_rank += 1;
                    self.log(format!(
                        "Fireball reaches rank {}.",
                        self.player.fireball_rank
                    ));
                }
                3 => {
                    self.player.cleave_rank += 1;
                    self.log(format!("Cleave reaches rank {}.", self.player.cleave_rank));
                }
                _ => {}
            }
            self.player.stats.unspent_skill_points -= 1;
        }
    }

    pub(super) fn update_world_map_controls(&mut self, dt: f32) {
        let pan_speed_tiles = 420.0 / self.world_map.zoom;
        self.world_map.center_tile += self.input.movement * pan_speed_tiles * dt;
        if self.input.map_zoom_delta != 0.0 {
            self.world_map.zoom =
                (self.world_map.zoom * 1.18_f32.powf(self.input.map_zoom_delta)).clamp(3.5, 22.0);
        }
        if self.input.map_recenter_pressed {
            self.center_world_map_on_player();
        }
    }

    pub(super) fn update_shop_controls(&mut self) {
        if self.input.nav_left_pressed || self.input.nav_right_pressed {
            self.shop_tab = match self.shop_tab {
                ShopTab::Buy => ShopTab::Sell,
                ShopTab::Sell => ShopTab::Buy,
            };
            self.shop_cursor = 0;
        }
        let len = match self.shop_tab {
            ShopTab::Buy => self.merchant_stock.len(),
            ShopTab::Sell => self.player.inventory.len(),
        };
        if self.input.inventory_up_pressed {
            self.shop_cursor = self.shop_cursor.saturating_sub(1);
        }
        if self.input.inventory_down_pressed && len > 0 {
            self.shop_cursor = (self.shop_cursor + 1).min(len - 1);
        }
        if self.input.inventory_equip_pressed {
            match self.shop_tab {
                ShopTab::Buy => self.buy_selected_item(),
                ShopTab::Sell => self.sell_selected_item(),
            }
        }
    }

    pub(super) fn update_trainer_controls(&mut self) {
        if self.input.inventory_equip_pressed {
            self.ui_mode = UiMode::SkillBook;
        }
    }

    pub(super) fn update_travel_controls(&mut self) {
        if self.input.inventory_up_pressed {
            self.travel_cursor = self.travel_cursor.saturating_sub(1);
        }
        if self.input.inventory_down_pressed {
            self.travel_cursor = (self.travel_cursor + 1).min(TRAVEL_DESTINATIONS.len() - 1);
        }
        if self.input.inventory_equip_pressed {
            let destination = TRAVEL_DESTINATIONS[self.travel_cursor];
            self.player.pos = World::tile_center(destination.pos);
            self.player.vel = Vec2::ZERO;
            self.reveal_around_tile(destination.pos, EXPLORATION_RADIUS);
            self.ui_mode = UiMode::None;
            self.log(format!("Rill sends you toward {}.", destination.name));
        }
    }

    pub(super) fn center_world_map_on_player(&mut self) {
        let tile = World::world_to_tile(self.player.pos);
        self.world_map.center_tile = vec2(tile.x as f32, tile.y as f32);
    }

    pub(super) fn update_log_scroll(&mut self) {
        if self.input.log_scroll_delta == 0 {
            return;
        }
        let max_offset = self.log.len().saturating_sub(6);
        if self.input.log_scroll_delta > 0 {
            self.log_scroll_offset = self
                .log_scroll_offset
                .saturating_add(self.input.log_scroll_delta as usize)
                .min(max_offset);
        } else {
            self.log_scroll_offset = self
                .log_scroll_offset
                .saturating_sub(self.input.log_scroll_delta.unsigned_abs() as usize);
        }
    }

    pub(super) fn interact_with_nearby_npc(&mut self) {
        let Some(kind) = self
            .npcs
            .iter()
            .find(|npc| npc.pos.distance(self.player.pos) <= 42.0)
            .map(|npc| npc.kind)
        else {
            return;
        };
        self.log(format!("{}: {}", kind.name(), kind.greeting()));
        self.ui_mode = match kind {
            NpcKind::Merchant => UiMode::Merchant,
            NpcKind::Trainer => UiMode::Trainer,
            NpcKind::Wayfinder => UiMode::Travel,
        };
    }
}
