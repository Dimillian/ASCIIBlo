use macroquad::prelude::*;

use crate::content::{Item, MonsterKind, NpcKind};

pub struct Stats {
    pub level: i32,
    pub xp: i32,
    pub next_xp: i32,
    pub strength: i32,
    pub agility: i32,
    pub vitality: i32,
    pub gold: i32,
    pub unspent_stat_points: i32,
    pub unspent_skill_points: i32,
}

pub struct Equipment {
    pub weapon: Option<Item>,
    pub armor: Option<Item>,
    pub charm: Option<Item>,
}

impl Equipment {
    pub fn bonus_power(&self) -> i32 {
        self.iter().map(|item| item.power).sum()
    }

    pub fn bonus_armor(&self) -> i32 {
        self.iter().map(|item| item.armor).sum()
    }

    pub fn bonus_vitality(&self) -> i32 {
        self.iter().map(|item| item.vitality).sum()
    }

    pub fn bonus_haste(&self) -> i32 {
        self.iter().map(|item| item.haste).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.weapon
            .iter()
            .chain(self.armor.iter())
            .chain(self.charm.iter())
    }
}

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: Vec2,
    pub hp: f32,
    pub mana: f32,
    pub attack_cd: f32,
    pub rush_cd: f32,
    pub nova_cd: f32,
    pub fireball_cd: f32,
    pub cleave_cd: f32,
    pub stats: Stats,
    pub inventory: Vec<Item>,
    pub equipment: Equipment,
    pub rush_rank: i32,
    pub nova_rank: i32,
    pub fireball_rank: i32,
    pub cleave_rank: i32,
}

impl Player {
    pub fn max_hp(&self) -> f32 {
        64.0 + (self.stats.vitality + self.equipment.bonus_vitality()) as f32 * 7.0
    }

    pub fn max_mana(&self) -> f32 {
        32.0 + self.stats.level as f32 * 4.0
    }

    pub fn power(&self) -> i32 {
        self.stats.strength * 2 + self.equipment.bonus_power()
    }

    pub fn armor(&self) -> i32 {
        self.stats.vitality / 2 + self.equipment.bonus_armor()
    }

    pub fn haste(&self) -> i32 {
        self.stats.agility + self.equipment.bonus_haste()
    }

    pub fn crit_chance(&self) -> f32 {
        (0.08 + self.stats.agility as f32 * 0.01).min(0.35)
    }

    pub fn move_speed(&self) -> f32 {
        150.0 + self.haste() as f32 * 4.0
    }

    pub fn attack_interval(&self) -> f32 {
        (0.5 - self.haste() as f32 * 0.018).max(0.16)
    }
}

pub struct Monster {
    pub kind: MonsterKind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub hit_offset: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    pub level: i32,
    pub attack_cd: f32,
    pub wobble: f32,
    pub hit_flash: f32,
}

pub struct Loot {
    pub pos: Vec2,
    pub item: Item,
    pub bob: f32,
}

pub struct Npc {
    pub kind: NpcKind,
    pub pos: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopTab {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMode {
    None,
    Inventory,
    Character,
    SkillBook,
    WorldMap,
    Merchant,
    Trainer,
    Travel,
}

#[derive(Clone, Copy)]
pub struct TravelDestination {
    pub name: &'static str,
    pub pos: IVec2,
    pub min_level: i32,
}

pub struct FloatingText {
    pub pos: Vec2,
    pub text: String,
    pub color: Color,
    pub ttl: f32,
}

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub ttl: f32,
    pub radius: f32,
}

pub struct Pulse {
    pub pos: Vec2,
    pub radius: f32,
    pub ttl: f32,
    pub color: Color,
}

pub struct SlashArc {
    pub pos: Vec2,
    pub direction: Vec2,
    pub radius: f32,
    pub ttl: f32,
    pub color: Color,
}

pub struct Projectile {
    pub pos: Vec2,
    pub vel: Vec2,
    pub ttl: f32,
    pub radius: f32,
    pub damage: f32,
    pub aoe_radius: f32,
    pub color: Color,
}

pub struct WorldMapState {
    pub center_tile: Vec2,
    pub zoom: f32,
}
