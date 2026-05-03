use std::collections::HashSet;

use ::rand::rngs::StdRng;
use macroquad::prelude::*;

use crate::content::Item;

use super::{
    DEFAULT_SPAWN_VISIBILITY_HALF_VIEW, DiscoveredSettlement, FloatingText, InputState, Loot,
    MeteorStrike, Monster, Notification, Npc, Particle, Projectile, Pulse, Quest, QuestItem,
    QuestSignature, ShopTab, SkillBookFocus, SkillXpToast, SlashArc, TravelDestination, UiMode,
    WorldMapState,
};

pub struct SimulationState {
    pub player: super::Player,
    pub monsters: Vec<Monster>,
    pub npcs: Vec<Npc>,
    pub active_quest: Option<Quest>,
    pub quest_items: Vec<QuestItem>,
    pub completed_quest_signatures: HashSet<QuestSignature>,
    pub discovered_settlements: Vec<DiscoveredSettlement>,
    pub travel_destinations: Vec<TravelDestination>,
    pub used_landmarks: HashSet<u64>,
    pub loot: Vec<Loot>,
    pub known_tiles: HashSet<IVec2>,
    pub merchant_stock: Vec<Item>,
}

pub struct UiState {
    pub mode: UiMode,
    pub inventory_focus: super::InventoryFocus,
    pub inventory_backpack_cursor: usize,
    pub inventory_equipment_cursor: usize,
    pub character_cursor: usize,
    pub skill_book_cursor: usize,
    pub skill_book_ability_cursor: usize,
    pub skill_book_focus: SkillBookFocus,
    pub shop_cursor: usize,
    pub shop_tab: ShopTab,
    pub travel_cursor: usize,
    pub world_map: WorldMapState,
}

pub struct FxState {
    pub floating: Vec<FloatingText>,
    pub particles: Vec<Particle>,
    pub pulses: Vec<Pulse>,
    pub slash_arcs: Vec<SlashArc>,
    pub projectiles: Vec<Projectile>,
    pub meteors: Vec<MeteorStrike>,
    pub skill_xp_toasts: Vec<SkillXpToast>,
    pub notifications: Vec<Notification>,
    pub log: Vec<String>,
    pub log_scroll_offset: usize,
    pub screen_shake: f32,
}

pub struct RuntimeState {
    pub elapsed: f32,
    pub(super) agility_distance_bank: f32,
    pub preview_hover_world: Option<Vec2>,
    pub preview_hover_screen: Option<Vec2>,
    pub(super) spawn_visibility_half_view: Vec2,
    pub(super) next_monster_pack_id: u64,
    pub(super) next_quest_id: u64,
    pub(super) rng: StdRng,
    pub(super) input: InputState,
    pub(super) quit: bool,
}

impl RuntimeState {
    pub(super) fn new(rng: StdRng) -> Self {
        Self {
            elapsed: 0.0,
            agility_distance_bank: 0.0,
            preview_hover_world: None,
            preview_hover_screen: None,
            spawn_visibility_half_view: DEFAULT_SPAWN_VISIBILITY_HALF_VIEW,
            next_monster_pack_id: 0,
            next_quest_id: 1,
            rng,
            input: InputState::default(),
            quit: false,
        }
    }
}

impl SimulationState {
    pub(super) fn new(player: super::Player, merchant_stock: Vec<Item>) -> Self {
        Self {
            player,
            monsters: Vec::new(),
            npcs: Vec::new(),
            active_quest: None,
            quest_items: Vec::new(),
            completed_quest_signatures: HashSet::new(),
            discovered_settlements: Vec::new(),
            travel_destinations: Vec::new(),
            used_landmarks: HashSet::new(),
            loot: Vec::new(),
            known_tiles: HashSet::new(),
            merchant_stock,
        }
    }
}

impl UiState {
    pub(super) fn new() -> Self {
        Self {
            mode: UiMode::None,
            inventory_focus: super::InventoryFocus::Backpack,
            inventory_backpack_cursor: 0,
            inventory_equipment_cursor: 0,
            character_cursor: 0,
            skill_book_cursor: 0,
            skill_book_ability_cursor: 0,
            skill_book_focus: SkillBookFocus::Disciplines,
            shop_cursor: 0,
            shop_tab: ShopTab::Buy,
            travel_cursor: 0,
            world_map: WorldMapState {
                center_tile: Vec2::ZERO,
                zoom: 8.0,
            },
        }
    }
}

impl FxState {
    pub(super) fn new() -> Self {
        Self {
            floating: Vec::new(),
            particles: Vec::new(),
            pulses: Vec::new(),
            slash_arcs: Vec::new(),
            projectiles: Vec::new(),
            meteors: Vec::new(),
            skill_xp_toasts: Vec::new(),
            notifications: Vec::new(),
            log: vec!["The bell in Ember Town rings. Go make trouble.".into()],
            log_scroll_offset: 0,
            screen_shake: 0.0,
        }
    }
}
