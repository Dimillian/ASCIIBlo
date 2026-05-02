use macroquad::prelude::*;

use crate::content::Slot;

use super::{FloatingText, Game, Loot};

impl Game {
    pub(super) fn pickup_loot(&mut self) {
        let Some(index) = self
            .loot
            .iter()
            .position(|loot| loot.pos.distance(self.player.pos) <= 34.0)
        else {
            self.log("Nothing close enough to pocket.".into());
            return;
        };
        if self.player.inventory.len() >= 14 {
            self.log("Pack is full. Equip or drop something first.".into());
            return;
        }
        let loot = self.loot.remove(index);
        self.floating.push(FloatingText {
            pos: self.player.pos,
            text: "LOOT".into(),
            color: loot.item.rarity.color(),
            ttl: 0.9,
        });
        self.log(format!(
            "Picked up {} [{}].",
            loot.item.name,
            loot.item.summary()
        ));
        self.player.inventory.push(loot.item);
    }

    pub(super) fn equip_selected_item(&mut self) {
        if self.player.inventory.is_empty() {
            return;
        }
        let item = self.player.inventory.remove(self.inventory_cursor);
        let slot = match item.slot {
            Slot::Weapon => &mut self.player.equipment.weapon,
            Slot::Armor => &mut self.player.equipment.armor,
            Slot::Charm => &mut self.player.equipment.charm,
        };
        if let Some(previous) = slot.replace(item.clone()) {
            self.player.inventory.push(previous);
        }
        self.player.hp = self.player.hp.min(self.player.max_hp());
        self.log(format!("Equipped {}.", item.name));
        self.inventory_cursor = self
            .inventory_cursor
            .min(self.player.inventory.len().saturating_sub(1));
    }

    pub(super) fn buy_selected_item(&mut self) {
        let Some(item) = self.merchant_stock.get(self.shop_cursor).cloned() else {
            return;
        };
        if self.player.stats.gold < item.value {
            self.log("Not enough gold.".into());
            return;
        }
        if self.player.inventory.len() >= 14 {
            self.log("Pack is full.".into());
            return;
        }
        self.player.stats.gold -= item.value;
        self.player.inventory.push(item.clone());
        self.log(format!("Bought {}.", item.name));
    }

    pub(super) fn sell_selected_item(&mut self) {
        if self.player.inventory.is_empty() {
            return;
        }
        let item = self.player.inventory.remove(self.shop_cursor);
        let payout = (item.value as f32 * 0.6).round() as i32;
        self.player.stats.gold += payout;
        self.log(format!("Sold {} for {} gold.", item.name, payout));
        self.shop_cursor = self
            .shop_cursor
            .min(self.player.inventory.len().saturating_sub(1));
    }

    pub(super) fn drop_selected_item(&mut self) {
        if self.player.inventory.is_empty() {
            return;
        }
        let item = self.player.inventory.remove(self.inventory_cursor);
        self.log(format!("Dropped {}.", item.name));
        self.loot.push(Loot {
            pos: self.player.pos,
            item,
            bob: 0.0,
        });
        self.inventory_cursor = self
            .inventory_cursor
            .min(self.player.inventory.len().saturating_sub(1));
    }
}
