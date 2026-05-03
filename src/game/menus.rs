use macroquad::prelude::*;

use crate::{
    content::NpcKind,
    world::{LandmarkKind, World},
};

use super::{EXPLORATION_RADIUS, Game, InventoryFocus, ShopTab, UiMode};

impl Game {
    pub(super) fn update_inventory_controls(&mut self) {
        if self.runtime.input.attack_pressed {
            match crate::ui::inventory_hit_test(self, self.ui_hover_position()) {
                Some(crate::ui::inventory::InventoryHit::Backpack(index)) => {
                    self.ui.inventory_focus = InventoryFocus::Backpack;
                    self.ui.inventory_backpack_cursor = index;
                    self.equip_selected_item();
                    return;
                }
                Some(crate::ui::inventory::InventoryHit::Equipment(index)) => {
                    self.ui.inventory_focus = InventoryFocus::Equipment;
                    self.ui.inventory_equipment_cursor = index;
                    self.unequip_selected_item();
                    return;
                }
                None => {}
            }
        }
        if self.runtime.input.nav_left_pressed {
            self.ui.inventory_focus = InventoryFocus::Backpack;
        }
        if self.runtime.input.nav_right_pressed {
            self.ui.inventory_focus = InventoryFocus::Equipment;
        }
        match self.ui.inventory_focus {
            InventoryFocus::Backpack => {
                if self.runtime.input.inventory_up_pressed {
                    self.ui.inventory_backpack_cursor =
                        self.ui.inventory_backpack_cursor.saturating_sub(1);
                }
                if self.runtime.input.inventory_down_pressed
                    && !self.sim.player.inventory.is_empty()
                {
                    self.ui.inventory_backpack_cursor = (self.ui.inventory_backpack_cursor + 1)
                        .min(self.sim.player.inventory.len() - 1);
                }
            }
            InventoryFocus::Equipment => {
                if self.runtime.input.inventory_up_pressed {
                    self.ui.inventory_equipment_cursor =
                        self.ui.inventory_equipment_cursor.saturating_sub(1);
                }
                if self.runtime.input.inventory_down_pressed {
                    self.ui.inventory_equipment_cursor =
                        (self.ui.inventory_equipment_cursor + 1).min(2);
                }
            }
        }
        if self.runtime.input.inventory_equip_pressed {
            match self.ui.inventory_focus {
                InventoryFocus::Backpack => self.equip_selected_item(),
                InventoryFocus::Equipment => self.unequip_selected_item(),
            }
        }
        if self.runtime.input.inventory_drop_pressed
            && self.ui.inventory_focus == InventoryFocus::Backpack
        {
            self.drop_selected_item();
        }
    }

    pub(super) fn update_character_controls(&mut self) {
        if self.runtime.input.inventory_up_pressed {
            self.ui.character_cursor = self.ui.character_cursor.saturating_sub(1);
        }
        if self.runtime.input.inventory_down_pressed {
            self.ui.character_cursor = (self.ui.character_cursor + 1).min(2);
        }
        if self.runtime.input.inventory_equip_pressed {
            match self.ui.character_cursor {
                0 if self.sim.player.stats.unspent_stat_points > 0 => {
                    self.sim.player.stats.strength += 1;
                    self.sim.player.stats.unspent_stat_points -= 1;
                    self.log("Strength rises.".into());
                }
                1 if self.sim.player.stats.unspent_stat_points > 0 => {
                    self.sim.player.stats.agility += 1;
                    self.sim.player.stats.unspent_stat_points -= 1;
                    self.log("Agility sharpens.".into());
                }
                2 if self.sim.player.stats.unspent_stat_points > 0 => {
                    self.sim.player.stats.vitality += 1;
                    self.sim.player.stats.unspent_stat_points -= 1;
                    self.sim.player.hp = self.sim.player.max_hp();
                    self.log("Vitality deepens.".into());
                }
                _ => {}
            }
        }
    }

    pub(super) fn update_skill_book_controls(&mut self) {
        if self.runtime.input.nav_left_pressed {
            self.ui.skill_book_focus = match self.ui.skill_book_focus {
                super::SkillBookFocus::Disciplines => super::SkillBookFocus::Disciplines,
                super::SkillBookFocus::Skills => super::SkillBookFocus::Disciplines,
                super::SkillBookFocus::Detail => super::SkillBookFocus::Skills,
            };
        }
        if self.runtime.input.nav_right_pressed {
            self.ui.skill_book_focus = match self.ui.skill_book_focus {
                super::SkillBookFocus::Disciplines => super::SkillBookFocus::Skills,
                super::SkillBookFocus::Skills => super::SkillBookFocus::Detail,
                super::SkillBookFocus::Detail => super::SkillBookFocus::Detail,
            };
        }
        let discipline = super::DisciplineKind::ALL[self.ui.skill_book_cursor];
        let abilities = super::abilities_for_discipline(discipline);
        match self.ui.skill_book_focus {
            super::SkillBookFocus::Disciplines => {
                let previous = self.ui.skill_book_cursor;
                if self.runtime.input.inventory_up_pressed {
                    self.ui.skill_book_cursor = self.ui.skill_book_cursor.saturating_sub(1);
                }
                if self.runtime.input.inventory_down_pressed {
                    self.ui.skill_book_cursor =
                        (self.ui.skill_book_cursor + 1).min(super::DisciplineKind::ALL.len() - 1);
                }
                if previous != self.ui.skill_book_cursor {
                    let discipline = super::DisciplineKind::ALL[self.ui.skill_book_cursor];
                    self.ui.skill_book_ability_cursor = self.preferred_skill_index(discipline);
                }
            }
            super::SkillBookFocus::Skills if !abilities.is_empty() => {
                if self.runtime.input.inventory_up_pressed {
                    self.ui.skill_book_ability_cursor =
                        self.ui.skill_book_ability_cursor.saturating_sub(1);
                }
                if self.runtime.input.inventory_down_pressed {
                    self.ui.skill_book_ability_cursor =
                        (self.ui.skill_book_ability_cursor + 1).min(abilities.len() - 1);
                }
            }
            _ => {}
        }
        if !abilities.is_empty() {
            let selected = abilities[self.ui.skill_book_ability_cursor];
            for slot in 0..self.runtime.input.ability_slot_pressed.len() {
                if self.runtime.input.ability_slot_pressed[slot]
                    && self.sim.player.is_ability_unlocked(selected)
                {
                    self.bind_ability(slot, selected);
                }
            }
        }
    }

    fn preferred_skill_index(&self, discipline: super::DisciplineKind) -> usize {
        let abilities = super::abilities_for_discipline(discipline);
        abilities
            .iter()
            .position(|ability| {
                self.sim.player.is_ability_unlocked(*ability)
                    && self.sim.player.bound_slot(*ability).is_none()
            })
            .or_else(|| {
                abilities
                    .iter()
                    .position(|ability| self.sim.player.is_ability_unlocked(*ability))
            })
            .unwrap_or(0)
    }

    pub(super) fn bind_ability(&mut self, slot: usize, ability: super::AbilityKind) {
        if !self.sim.player.is_ability_unlocked(ability) {
            return;
        }
        if let Some(existing_slot) = self.sim.player.bound_slot(ability) {
            if existing_slot == slot {
                return;
            }
            self.sim.player.bound_abilities.swap(slot, existing_slot);
        } else {
            self.sim.player.bound_abilities[slot] = ability;
        }
        self.log(format!("{} bound to {}.", ability.name(), slot + 1));
    }

    pub(super) fn update_world_map_controls(&mut self, dt: f32) {
        let pan_speed_tiles = 420.0 / self.ui.world_map.zoom;
        self.ui.world_map.center_tile += self.runtime.input.movement * pan_speed_tiles * dt;
        if self.runtime.input.map_zoom_delta != 0.0 {
            self.ui.world_map.zoom = (self.ui.world_map.zoom
                * 1.18_f32.powf(self.runtime.input.map_zoom_delta))
            .clamp(3.5, 22.0);
        }
        if self.runtime.input.map_recenter_pressed {
            self.center_world_map_on_player();
        }
    }

    pub(super) fn update_shop_controls(&mut self) {
        if self.runtime.input.nav_left_pressed || self.runtime.input.nav_right_pressed {
            self.ui.shop_tab = match self.ui.shop_tab {
                ShopTab::Buy => ShopTab::Sell,
                ShopTab::Sell => ShopTab::Buy,
            };
            self.ui.shop_cursor = 0;
        }
        let len = match self.ui.shop_tab {
            ShopTab::Buy => self.sim.merchant_stock.len(),
            ShopTab::Sell => self.sim.player.inventory.len(),
        };
        if self.runtime.input.inventory_up_pressed {
            self.ui.shop_cursor = self.ui.shop_cursor.saturating_sub(1);
        }
        if self.runtime.input.inventory_down_pressed && len > 0 {
            self.ui.shop_cursor = (self.ui.shop_cursor + 1).min(len - 1);
        }
        if self.runtime.input.inventory_equip_pressed {
            match self.ui.shop_tab {
                ShopTab::Buy => self.buy_selected_item(),
                ShopTab::Sell => self.sell_selected_item(),
            }
        }
    }

    pub(super) fn update_trainer_controls(&mut self) {
        if self.runtime.input.inventory_equip_pressed {
            self.ui.mode = UiMode::SkillBook;
        }
    }

    pub(super) fn update_travel_controls(&mut self) {
        if self.runtime.input.inventory_up_pressed {
            self.ui.travel_cursor = self.ui.travel_cursor.saturating_sub(1);
        }
        if self.runtime.input.inventory_down_pressed && !self.sim.travel_destinations.is_empty() {
            self.ui.travel_cursor =
                (self.ui.travel_cursor + 1).min(self.sim.travel_destinations.len() - 1);
        }
        if self.runtime.input.inventory_equip_pressed && !self.sim.travel_destinations.is_empty() {
            let destination = self.sim.travel_destinations[self.ui.travel_cursor].clone();
            self.sim.player.pos = World::tile_center(destination.pos);
            self.sim.player.vel = Vec2::ZERO;
            self.reveal_around_tile(destination.pos, EXPLORATION_RADIUS);
            self.sync_local_npcs();
            self.ui.mode = UiMode::None;
            self.log(format!("Rill sends you toward {}.", destination.name));
        }
    }

    pub(super) fn center_world_map_on_player(&mut self) {
        let tile = World::world_to_tile(self.sim.player.pos);
        self.ui.world_map.center_tile = vec2(tile.x as f32, tile.y as f32);
    }

    pub(super) fn update_log_scroll(&mut self) {
        if self.runtime.input.log_scroll_delta == 0 {
            return;
        }
        let max_offset = self.fx.log.len().saturating_sub(6);
        if self.runtime.input.log_scroll_delta > 0 {
            self.fx.log_scroll_offset = self
                .fx
                .log_scroll_offset
                .saturating_add(self.runtime.input.log_scroll_delta as usize)
                .min(max_offset);
        } else {
            self.fx.log_scroll_offset = self
                .fx
                .log_scroll_offset
                .saturating_sub(self.runtime.input.log_scroll_delta.unsigned_abs() as usize);
        }
    }

    pub(super) fn interact_with_nearby_world_entity(&mut self) {
        if self.interact_with_nearby_quest_board() {
            return;
        }
        if self.interact_with_nearby_npc() {
            return;
        }
        self.interact_with_nearby_landmark();
    }

    pub(super) fn interact_with_nearby_npc(&mut self) -> bool {
        let Some(npc) = self
            .sim
            .npcs
            .iter()
            .filter(|npc| npc.pos.distance(self.sim.player.pos) <= 42.0)
            .min_by(|a, b| {
                a.pos
                    .distance(self.sim.player.pos)
                    .total_cmp(&b.pos.distance(self.sim.player.pos))
            })
            .cloned()
        else {
            return false;
        };
        if self.interact_with_quest_contact(&npc) {
            return true;
        }
        self.log(format!("{}: {}", npc.name, npc.kind.greeting()));
        self.ui.mode = match npc.kind {
            NpcKind::Merchant => UiMode::Merchant,
            NpcKind::Trainer => UiMode::Trainer,
            NpcKind::Wayfinder => UiMode::Travel,
            NpcKind::QuestContact => UiMode::None,
        };
        true
    }

    fn interact_with_nearby_landmark(&mut self) {
        let Some(landmark) = self.world.landmark_at_world(self.sim.player.pos) else {
            return;
        };
        if self.sim.used_landmarks.contains(&landmark.id) {
            self.log(format!(
                "The {} is quiet now.",
                landmark.kind.name().to_lowercase()
            ));
            return;
        }
        match landmark.kind {
            LandmarkKind::Shrine => {
                self.sim.player.mana = self.sim.player.max_mana();
                self.log("The shrine answers. Mana restored.".into());
                self.sim.used_landmarks.insert(landmark.id);
            }
            LandmarkKind::Well => {
                self.sim.player.hp = self.sim.player.max_hp();
                self.log("Cool water steadies you. Life restored.".into());
                self.sim.used_landmarks.insert(landmark.id);
            }
            LandmarkKind::Camp => self.log("The camp is cold, but recently used.".into()),
            LandmarkKind::Graveyard => self.log("Names fade from the stones.".into()),
            LandmarkKind::StandingStones => self.log("The stones hum with old weather.".into()),
            LandmarkKind::Cart => self.log("The cart has already been picked clean.".into()),
        }
    }
}
