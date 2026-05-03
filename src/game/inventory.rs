use macroquad::prelude::*;

use crate::{content::Slot, stat_display::item_summary};

use super::{Game, Loot, events::GameplayEvent};

impl Game {
    pub(super) fn pickup_loot(&mut self) {
        if self.pickup_nearby_quest_item() {
            return;
        }
        let Some(index) = self
            .sim
            .loot
            .iter()
            .position(|loot| loot.pos.distance(self.sim.player.pos) <= 34.0)
        else {
            self.log("Nothing close enough to pocket.".into());
            return;
        };
        if !self
            .sim
            .player
            .inventory
            .can_fit(&self.sim.loot[index].item)
        {
            self.log("Pack is full. Equip or drop something first.".into());
            return;
        }
        let loot = self.sim.loot.remove(index);
        self.emit(GameplayEvent::LootPickedUp {
            pos: self.sim.player.pos,
            color: loot.item.rarity.color(),
            name: loot.item.name.clone(),
            summary: item_summary(&loot.item),
        });
        self.sim
            .player
            .inventory
            .insert_first_fit(loot.item)
            .expect("fitting loot should insert");
    }

    pub(super) fn equip_selected_item(&mut self) {
        if self.sim.player.inventory.is_empty() {
            return;
        }
        let Some(entry) = self
            .sim
            .player
            .inventory
            .remove(self.ui.inventory_backpack_cursor)
        else {
            return;
        };
        let item = entry.item.clone();
        let slot = match item.slot {
            Slot::Weapon => &mut self.sim.player.equipment.weapon,
            Slot::Armor => &mut self.sim.player.equipment.armor,
            Slot::Charm => &mut self.sim.player.equipment.charm,
        };
        if let Some(previous) = slot.take() {
            match self.sim.player.inventory.insert_first_fit(previous) {
                Ok(_) => {}
                Err(previous) => {
                    self.sim.player.inventory.restore(entry);
                    *slot = Some(previous);
                    self.log("Pack is full. Cannot swap gear.".into());
                    return;
                }
            }
        }
        *slot = Some(item.clone());
        self.sim.player.hp = self.sim.player.hp.min(self.sim.player.max_hp());
        self.log(format!("Equipped {}.", item.name));
        self.ui.inventory_backpack_cursor = self
            .ui
            .inventory_backpack_cursor
            .min(self.sim.player.inventory.len().saturating_sub(1));
    }

    pub(super) fn unequip_selected_item(&mut self) {
        let Some(item) = self.selected_equipment_item().cloned() else {
            return;
        };
        if !self.sim.player.inventory.can_fit(&item) {
            self.log("Pack is full. Cannot unequip gear.".into());
            return;
        }
        let slot = self.selected_equipment_slot_mut();
        let item = slot.take().expect("slot was checked above");
        self.sim
            .player
            .inventory
            .insert_first_fit(item.clone())
            .expect("fitting gear should insert");
        self.sim.player.hp = self.sim.player.hp.min(self.sim.player.max_hp());
        self.log(format!("Unequipped {}.", item.name));
    }

    pub(super) fn buy_selected_item(&mut self) {
        let Some(item) = self.sim.merchant_stock.get(self.ui.shop_cursor).cloned() else {
            return;
        };
        if self.sim.player.stats.gold < item.value {
            self.log("Not enough gold.".into());
            return;
        }
        if !self.sim.player.inventory.can_fit(&item) {
            self.log("Pack is full.".into());
            return;
        }
        self.sim.player.stats.gold -= item.value;
        self.sim
            .player
            .inventory
            .insert_first_fit(item.clone())
            .expect("fitting merchant item should insert");
        self.log(format!("Bought {}.", item.name));
    }

    pub(super) fn sell_selected_item(&mut self) {
        if self.sim.player.inventory.is_empty() {
            return;
        }
        let Some(entry) = self.sim.player.inventory.remove(self.ui.shop_cursor) else {
            return;
        };
        let item = entry.item;
        let payout = (item.value as f32 * 0.6).round() as i32;
        self.sim.player.stats.gold += payout;
        self.log(format!("Sold {} for {} gold.", item.name, payout));
        self.ui.shop_cursor = self
            .ui
            .shop_cursor
            .min(self.sim.player.inventory.len().saturating_sub(1));
    }

    pub(super) fn drop_selected_item(&mut self) {
        if self.sim.player.inventory.is_empty() {
            return;
        }
        let Some(entry) = self
            .sim
            .player
            .inventory
            .remove(self.ui.inventory_backpack_cursor)
        else {
            return;
        };
        let item = entry.item;
        self.log(format!("Dropped {}.", item.name));
        self.sim.loot.push(Loot {
            pos: self.sim.player.pos,
            item,
            bob: 0.0,
        });
        self.ui.inventory_backpack_cursor = self
            .ui
            .inventory_backpack_cursor
            .min(self.sim.player.inventory.len().saturating_sub(1));
    }

    fn selected_equipment_slot_mut(&mut self) -> &mut Option<crate::content::Item> {
        match self.ui.inventory_equipment_cursor {
            0 => &mut self.sim.player.equipment.weapon,
            1 => &mut self.sim.player.equipment.armor,
            _ => &mut self.sim.player.equipment.charm,
        }
    }

    fn selected_equipment_item(&self) -> Option<&crate::content::Item> {
        match self.ui.inventory_equipment_cursor {
            0 => self.sim.player.equipment.weapon.as_ref(),
            1 => self.sim.player.equipment.armor.as_ref(),
            _ => self.sim.player.equipment.charm.as_ref(),
        }
    }
}
