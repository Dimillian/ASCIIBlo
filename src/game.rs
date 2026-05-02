use std::collections::HashSet;

use ::rand::{RngExt, SeedableRng, rngs::StdRng};
use macroquad::prelude::*;

use crate::{
    content::{
        Item, MonsterKind, NpcKind, Slot, merchant_stock, monster_damage, monster_max_hp,
        monster_xp, roll_item, roll_monster, scaled_monster_level, starter_items,
    },
    world::{TILE, World},
};

pub const FIXED_DT: f32 = 1.0 / 60.0;
const PLAYER_RADIUS: f32 = 9.0;
const MONSTER_RADIUS: f32 = 9.0;
const PASSIVE_AGGRO_RADIUS: f32 = 176.0;
const MONSTER_SPAWN_MIN_RADIUS: f32 = TILE * 11.0;
const MONSTER_SPAWN_MAX_RADIUS_TILES: i32 = 22;
const MONSTER_LOCAL_RADIUS: f32 = TILE * 32.0;
const MONSTER_DESPAWN_RADIUS: f32 = TILE * 42.0;
const EXPLORATION_RADIUS: i32 = 14;
const MAX_LOG_ENTRIES: usize = 32;

#[derive(Default)]
struct InputState {
    movement: Vec2,
    aim_world: Vec2,
    attack_pressed: bool,
    rush_pressed: bool,
    nova_pressed: bool,
    fireball_pressed: bool,
    cleave_pressed: bool,
    pickup_pressed: bool,
    inventory_toggle_pressed: bool,
    character_toggle_pressed: bool,
    skill_book_toggle_pressed: bool,
    world_map_toggle_pressed: bool,
    inventory_up_pressed: bool,
    inventory_down_pressed: bool,
    nav_left_pressed: bool,
    nav_right_pressed: bool,
    inventory_equip_pressed: bool,
    inventory_drop_pressed: bool,
    map_recenter_pressed: bool,
    map_zoom_delta: f32,
    log_scroll_delta: i32,
    interact_pressed: bool,
    quit_pressed: bool,
}

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
    pub hp: f32,
    pub max_hp: f32,
    pub level: i32,
    pub attack_cd: f32,
    pub wobble: f32,
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

pub struct Game {
    pub world: World,
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub npcs: Vec<Npc>,
    pub loot: Vec<Loot>,
    pub floating: Vec<FloatingText>,
    pub particles: Vec<Particle>,
    pub pulses: Vec<Pulse>,
    pub slash_arcs: Vec<SlashArc>,
    pub projectiles: Vec<Projectile>,
    pub log: Vec<String>,
    pub log_scroll_offset: usize,
    pub known_tiles: HashSet<IVec2>,
    pub ui_mode: UiMode,
    pub inventory_cursor: usize,
    pub character_cursor: usize,
    pub skill_book_cursor: usize,
    pub shop_cursor: usize,
    pub shop_tab: ShopTab,
    pub travel_cursor: usize,
    pub merchant_stock: Vec<Item>,
    pub world_map: WorldMapState,
    pub screen_shake: f32,
    pub elapsed: f32,
    pub preview_hover_world: Option<Vec2>,
    pub preview_hover_screen: Option<Vec2>,
    rng: StdRng,
    input: InputState,
    quit: bool,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let world = World::new(seed);
        let spawn = World::tile_center(ivec2(0, 0));
        let mut game = Self {
            world,
            player: Player {
                pos: spawn,
                vel: Vec2::ZERO,
                facing: Vec2::Y,
                hp: 92.0,
                mana: 36.0,
                attack_cd: 0.0,
                rush_cd: 0.0,
                nova_cd: 0.0,
                fireball_cd: 0.0,
                cleave_cd: 0.0,
                stats: Stats {
                    level: 1,
                    xp: 0,
                    next_xp: 45,
                    strength: 5,
                    agility: 5,
                    vitality: 4,
                    gold: 0,
                    unspent_stat_points: 0,
                    unspent_skill_points: 0,
                },
                inventory: starter_items(),
                equipment: Equipment {
                    weapon: None,
                    armor: None,
                    charm: None,
                },
                rush_rank: 1,
                nova_rank: 1,
                fireball_rank: 1,
                cleave_rank: 1,
            },
            monsters: Vec::new(),
            npcs: vec![
                Npc {
                    kind: NpcKind::Merchant,
                    pos: World::tile_center(ivec2(-5, 0)),
                },
                Npc {
                    kind: NpcKind::Trainer,
                    pos: World::tile_center(ivec2(0, -5)),
                },
                Npc {
                    kind: NpcKind::Wayfinder,
                    pos: World::tile_center(ivec2(5, 0)),
                },
            ],
            loot: Vec::new(),
            floating: Vec::new(),
            particles: Vec::new(),
            pulses: Vec::new(),
            slash_arcs: Vec::new(),
            projectiles: Vec::new(),
            log: vec!["The bell in Ember Town rings. Go make trouble.".into()],
            log_scroll_offset: 0,
            known_tiles: HashSet::new(),
            ui_mode: UiMode::None,
            inventory_cursor: 0,
            character_cursor: 0,
            skill_book_cursor: 0,
            shop_cursor: 0,
            shop_tab: ShopTab::Buy,
            travel_cursor: 0,
            merchant_stock: merchant_stock(),
            world_map: WorldMapState {
                center_tile: Vec2::ZERO,
                zoom: 8.0,
            },
            screen_shake: 0.0,
            elapsed: 0.0,
            preview_hover_world: None,
            preview_hover_screen: None,
            rng: StdRng::seed_from_u64(seed),
            input: InputState::default(),
            quit: false,
        };
        game.reveal_around_tile(World::world_to_tile(game.player.pos), EXPLORATION_RADIUS);
        game.spawn_monsters(52);
        game
    }

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

    pub fn fixed_update(&mut self, dt: f32) {
        self.elapsed += dt;
        self.update_log_scroll();
        if self.input.quit_pressed && self.ui_mode == UiMode::None {
            self.quit = true;
        }
        if self.input.inventory_toggle_pressed {
            self.ui_mode = if self.ui_mode == UiMode::Inventory {
                UiMode::None
            } else {
                UiMode::Inventory
            };
            self.inventory_cursor = self
                .inventory_cursor
                .min(self.player.inventory.len().saturating_sub(1));
        }
        if self.input.character_toggle_pressed {
            self.ui_mode = if self.ui_mode == UiMode::Character {
                UiMode::None
            } else {
                UiMode::Character
            };
        }
        if self.input.skill_book_toggle_pressed {
            self.ui_mode = if self.ui_mode == UiMode::SkillBook {
                UiMode::None
            } else {
                UiMode::SkillBook
            };
        }
        if self.input.world_map_toggle_pressed {
            self.ui_mode = if self.ui_mode == UiMode::WorldMap {
                UiMode::None
            } else {
                self.center_world_map_on_player();
                UiMode::WorldMap
            };
        }
        if self.input.quit_pressed && self.ui_mode != UiMode::None {
            self.ui_mode = UiMode::None;
        }
        match self.ui_mode {
            UiMode::Inventory => {
                self.update_inventory_controls();
                self.clear_edge_inputs();
                return;
            }
            UiMode::Character => {
                self.update_character_controls();
                self.clear_edge_inputs();
                return;
            }
            UiMode::SkillBook => {
                self.update_skill_book_controls();
                self.clear_edge_inputs();
                return;
            }
            UiMode::WorldMap => {
                self.update_world_map_controls(dt);
                self.clear_edge_inputs();
                return;
            }
            UiMode::Merchant => {
                self.update_shop_controls();
                self.clear_edge_inputs();
                return;
            }
            UiMode::Trainer => {
                self.update_trainer_controls();
                self.clear_edge_inputs();
                return;
            }
            UiMode::Travel => {
                self.update_travel_controls();
                self.clear_edge_inputs();
                return;
            }
            UiMode::None => {}
        }

        if self.input.interact_pressed {
            self.interact_with_nearby_npc();
        }
        if self.ui_mode != UiMode::None {
            self.clear_edge_inputs();
            return;
        }

        self.player.attack_cd = (self.player.attack_cd - dt).max(0.0);
        self.player.rush_cd = (self.player.rush_cd - dt).max(0.0);
        self.player.nova_cd = (self.player.nova_cd - dt).max(0.0);
        self.player.fireball_cd = (self.player.fireball_cd - dt).max(0.0);
        self.player.cleave_cd = (self.player.cleave_cd - dt).max(0.0);
        self.player.mana = (self.player.mana + dt * 4.0).min(self.player.max_mana());

        self.update_player_movement(dt);
        self.reveal_around_tile(World::world_to_tile(self.player.pos), EXPLORATION_RADIUS);
        if self.input.attack_pressed {
            self.basic_attack();
        }
        if self.input.rush_pressed {
            self.cast_rush();
        }
        if self.input.nova_pressed {
            self.cast_nova();
        }
        if self.input.fireball_pressed {
            self.cast_fireball();
        }
        if self.input.cleave_pressed {
            self.cast_cleave();
        }
        if self.input.pickup_pressed {
            self.pickup_loot();
        }

        self.update_projectiles(dt);
        self.update_monsters(dt);
        self.cull_distant_monsters();
        self.replenish_local_monsters();
        self.clear_edge_inputs();
    }

    pub fn frame_update(&mut self, dt: f32) {
        for text in &mut self.floating {
            text.ttl -= dt;
            text.pos.y -= dt * 24.0;
        }
        self.floating.retain(|text| text.ttl > 0.0);

        for particle in &mut self.particles {
            particle.ttl -= dt;
            particle.pos += particle.vel * dt;
            particle.vel *= 0.94_f32.powf(dt * 60.0);
        }
        self.particles.retain(|particle| particle.ttl > 0.0);

        for pulse in &mut self.pulses {
            pulse.ttl -= dt;
            pulse.radius += dt * 140.0;
        }
        self.pulses.retain(|pulse| pulse.ttl > 0.0);

        for slash in &mut self.slash_arcs {
            slash.ttl -= dt;
            slash.radius += dt * 36.0;
        }
        self.slash_arcs.retain(|slash| slash.ttl > 0.0);

        for loot in &mut self.loot {
            loot.bob += dt * 4.0;
        }
        self.screen_shake = (self.screen_shake - dt * 18.0).max(0.0);
    }

    pub fn camera_focus(&self) -> Vec2 {
        self.player.pos
    }

    pub fn hovered_monster(&self) -> Option<&Monster> {
        let hover_world = self.preview_hover_world.unwrap_or(self.input.aim_world);
        self.monsters
            .iter()
            .filter(|monster| monster.pos.distance(hover_world) <= 18.0)
            .min_by(|a, b| {
                a.pos
                    .distance(hover_world)
                    .total_cmp(&b.pos.distance(hover_world))
            })
    }

    pub fn quit_requested(&self) -> bool {
        self.quit
    }

    pub fn ui_hover_position(&self) -> Vec2 {
        self.preview_hover_screen.unwrap_or(mouse_position().into())
    }

    fn clear_edge_inputs(&mut self) {
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

    fn update_inventory_controls(&mut self) {
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

    fn update_character_controls(&mut self) {
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

    fn update_skill_book_controls(&mut self) {
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

    fn update_world_map_controls(&mut self, dt: f32) {
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

    fn update_shop_controls(&mut self) {
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

    fn update_trainer_controls(&mut self) {
        if self.input.inventory_equip_pressed {
            self.ui_mode = UiMode::SkillBook;
        }
    }

    fn update_travel_controls(&mut self) {
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

    fn center_world_map_on_player(&mut self) {
        let tile = World::world_to_tile(self.player.pos);
        self.world_map.center_tile = vec2(tile.x as f32, tile.y as f32);
    }

    fn update_log_scroll(&mut self) {
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

    pub(crate) fn reveal_around_tile(&mut self, center: IVec2, radius: i32) {
        for y in center.y - radius..=center.y + radius {
            for x in center.x - radius..=center.x + radius {
                let tile = ivec2(x, y);
                if tile.distance_squared(center) <= radius * radius {
                    self.known_tiles.insert(tile);
                }
            }
        }
    }

    fn interact_with_nearby_npc(&mut self) {
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

    fn update_player_movement(&mut self, dt: f32) {
        let speed = self.player.move_speed();
        let desired = self.input.movement * speed;
        self.player.vel = self.player.vel.lerp(desired, 1.0 - 0.0002_f32.powf(dt));
        if self.input.aim_world.distance_squared(self.player.pos) > 4.0 {
            self.player.facing = (self.input.aim_world - self.player.pos).normalize();
        } else if self.player.vel.length_squared() > 1.0 {
            self.player.facing = self.player.vel.normalize();
        }
        self.move_with_collision(self.player.vel * dt);
    }

    fn move_with_collision(&mut self, delta: Vec2) {
        let next_x = self.player.pos + vec2(delta.x, 0.0);
        if !self.world.collides_circle(next_x, PLAYER_RADIUS) {
            self.player.pos.x = next_x.x;
        }
        let next_y = self.player.pos + vec2(0.0, delta.y);
        if !self.world.collides_circle(next_y, PLAYER_RADIUS) {
            self.player.pos.y = next_y.y;
        }
    }

    fn basic_attack(&mut self) {
        if self.player.attack_cd > 0.0 {
            return;
        }
        let range = 54.0;
        let direction = self.player.facing;
        let target = self
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                let to_monster = monster.pos - self.player.pos;
                let distance = to_monster.length();
                let aligned = to_monster.normalize_or_zero().dot(direction);
                (distance <= range && aligned > 0.45).then_some((index, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index);
        if let Some(index) = target {
            let damage = self.roll_player_damage(false);
            self.hit_monster(index, damage, false);
        } else {
            self.log("You carve only air.".into());
        }
        self.player.attack_cd = self.player.attack_interval();
    }

    fn cast_rush(&mut self) {
        if self.player.rush_cd > 0.0 || self.player.mana < 8.0 {
            return;
        }
        self.player.mana -= 8.0;
        self.player.rush_cd = 1.8;
        let direction = self.player.facing;
        let mut travelled = Vec2::ZERO;
        for _ in 0..10 {
            let step = direction * 16.0;
            let next = self.player.pos + travelled + step;
            if self.world.collides_circle(next, PLAYER_RADIUS) {
                break;
            }
            travelled += step;
        }
        self.player.pos += travelled;
        self.pulses.push(Pulse {
            pos: self.player.pos,
            radius: 18.0,
            ttl: 0.25,
            color: Color::from_rgba(112, 180, 255, 255),
        });
        self.spawn_particles(self.player.pos, 10, Color::from_rgba(112, 180, 255, 255));

        let hits: Vec<usize> = self
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                (monster.pos.distance(self.player.pos) <= 42.0).then_some(index)
            })
            .collect();
        if hits.is_empty() {
            self.log("Rush snaps the grass flat.".into());
        }
        for index in hits.into_iter().rev() {
            let damage = self.roll_player_damage(true) + 6.0 + self.player.rush_rank as f32 * 2.0;
            self.hit_monster(index, damage, true);
        }
    }

    fn cast_nova(&mut self) {
        if self.player.nova_cd > 0.0 || self.player.mana < 14.0 {
            return;
        }
        self.player.mana -= 14.0;
        self.player.nova_cd = 3.5;
        self.pulses.push(Pulse {
            pos: self.player.pos,
            radius: 18.0,
            ttl: 0.42,
            color: Color::from_rgba(255, 112, 236, 255),
        });
        self.spawn_particles(self.player.pos, 18, Color::from_rgba(255, 112, 236, 255));
        let hits: Vec<usize> = self
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                (monster.pos.distance(self.player.pos) <= 92.0).then_some(index)
            })
            .collect();
        if hits.is_empty() {
            self.log("Nova blooms with nobody close enough to regret it.".into());
        }
        for index in hits.into_iter().rev() {
            let damage = self.roll_player_damage(true) + 3.0 + self.player.nova_rank as f32 * 2.0;
            self.hit_monster(index, damage, true);
        }
    }

    fn cast_fireball(&mut self) {
        if self.player.fireball_cd > 0.0 || self.player.mana < 12.0 {
            return;
        }
        self.player.mana -= 12.0;
        self.player.fireball_cd = 1.2;
        let direction = self.player.facing.normalize_or_zero();
        let damage = self.roll_player_damage(true) + 8.0 + self.player.fireball_rank as f32 * 2.0;
        self.projectiles.push(Projectile {
            pos: self.player.pos + direction * 20.0,
            vel: direction * 320.0,
            ttl: 0.95,
            radius: 7.0,
            damage,
            aoe_radius: 34.0 + self.player.fireball_rank as f32 * 3.0,
            color: Color::from_rgba(255, 132, 64, 255),
        });
        self.spawn_particles(
            self.player.pos + direction * 18.0,
            6,
            Color::from_rgba(255, 132, 64, 255),
        );
    }

    fn cast_cleave(&mut self) {
        if self.player.cleave_cd > 0.0 || self.player.mana < 10.0 {
            return;
        }
        self.player.mana -= 10.0;
        self.player.cleave_cd = 2.2;
        self.pulses.push(Pulse {
            pos: self.player.pos,
            radius: 18.0,
            ttl: 0.3,
            color: Color::from_rgba(255, 176, 88, 255),
        });
        self.slash_arcs.push(SlashArc {
            pos: self.player.pos,
            direction: self.player.facing.normalize_or_zero(),
            radius: 48.0,
            ttl: 0.28,
            color: Color::from_rgba(255, 176, 88, 255),
        });
        self.spawn_particles(self.player.pos, 14, Color::from_rgba(255, 176, 88, 255));

        let direction = self.player.facing.normalize_or_zero();
        let hits: Vec<usize> = self
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                let to_monster = monster.pos - self.player.pos;
                let distance = to_monster.length();
                let aligned = to_monster.normalize_or_zero().dot(direction);
                (distance <= 68.0 && aligned >= 0.0).then_some(index)
            })
            .collect();
        if hits.is_empty() {
            self.log("Cleave whistles through empty air.".into());
        }
        for index in hits.into_iter().rev() {
            let damage = self.roll_player_damage(true) + 5.0 + self.player.cleave_rank as f32 * 2.0;
            self.hit_monster(index, damage, true);
        }
    }

    fn update_projectiles(&mut self, dt: f32) {
        let mut active = Vec::with_capacity(self.projectiles.len());
        let mut impacts = Vec::new();
        for mut projectile in self.projectiles.drain(..) {
            projectile.ttl -= dt;
            projectile.pos += projectile.vel * dt;
            let hits_monster = self
                .monsters
                .iter()
                .any(|monster| monster.pos.distance(projectile.pos) <= projectile.radius + 12.0);
            if projectile.ttl <= 0.0
                || hits_monster
                || self
                    .world
                    .collides_circle(projectile.pos, projectile.radius)
            {
                impacts.push(projectile);
            } else {
                active.push(projectile);
            }
        }
        self.projectiles = active;
        for projectile in impacts {
            self.detonate_fireball(projectile);
        }
    }

    fn detonate_fireball(&mut self, projectile: Projectile) {
        self.pulses.push(Pulse {
            pos: projectile.pos,
            radius: 14.0,
            ttl: 0.36,
            color: projectile.color,
        });
        self.spawn_particles(projectile.pos, 20, projectile.color);
        let hits: Vec<usize> = self
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                (monster.pos.distance(projectile.pos) <= projectile.aoe_radius).then_some(index)
            })
            .collect();
        if hits.is_empty() {
            self.log("Fireball blooms against the ground.".into());
        }
        for index in hits.into_iter().rev() {
            self.hit_monster(index, projectile.damage, true);
        }
    }

    fn update_monsters(&mut self, dt: f32) {
        let player_pos = self.player.pos;
        let mut attacks = Vec::new();
        for index in 0..self.monsters.len() {
            let monster = &mut self.monsters[index];
            monster.attack_cd = (monster.attack_cd - dt).max(0.0);
            monster.wobble += dt * 5.0;
            let to_player = player_pos - monster.pos;
            let distance = to_player.length();
            if distance < 26.0 && monster.attack_cd <= 0.0 {
                attacks.push(index);
                monster.attack_cd = monster.kind.attack_cooldown();
                continue;
            }
            if distance < PASSIVE_AGGRO_RADIUS {
                monster.vel = to_player.normalize_or_zero() * monster.kind.move_speed();
            } else {
                monster.vel *= 0.88;
            }
        }

        for index in 0..self.monsters.len() {
            let delta = self.monsters[index].vel * dt;
            let next_x = self.monsters[index].pos + vec2(delta.x, 0.0);
            if !self.world.collides_circle(next_x, MONSTER_RADIUS)
                && self.world.biome_level(next_x) > 0
            {
                self.monsters[index].pos.x = next_x.x;
            }
            let next_y = self.monsters[index].pos + vec2(0.0, delta.y);
            if !self.world.collides_circle(next_y, MONSTER_RADIUS)
                && self.world.biome_level(next_y) > 0
            {
                self.monsters[index].pos.y = next_y.y;
            }
        }

        for index in attacks {
            if index >= self.monsters.len() {
                continue;
            }
            let raw = monster_damage(self.monsters[index].kind, self.monsters[index].level)
                + self.rng.random_range(-2.0..=3.0);
            let damage = (raw - self.player.armor() as f32).max(1.0);
            self.player.hp -= damage;
            self.floating.push(FloatingText {
                pos: self.player.pos,
                text: format!("-{}", damage.round() as i32),
                color: Color::from_rgba(255, 100, 100, 255),
                ttl: 0.85,
            });
            self.screen_shake = self.screen_shake.max(8.0);
            self.log(format!(
                "{} bites for {}.",
                self.monsters[index].kind.name(),
                damage.round() as i32
            ));
            if self.player.hp <= 0.0 {
                self.player.hp = self.player.max_hp();
                self.player.pos = World::tile_center(ivec2(0, 0));
                self.player.stats.gold = (self.player.stats.gold as f32 * 0.8) as i32;
                self.log("You wake at the town well, lighter in coin and pride.".into());
            }
        }
    }

    fn roll_player_damage(&mut self, skill: bool) -> f32 {
        let crit = self.rng.random_bool(self.player.crit_chance() as f64);
        let base = self.player.power() as f32 + self.rng.random_range(3.0..=8.0);
        let skill_bonus = if skill { 4.0 } else { 0.0 };
        if crit {
            self.floating.push(FloatingText {
                pos: self.player.pos + vec2(0.0, -18.0),
                text: "CRIT!".into(),
                color: Color::from_rgba(255, 224, 96, 255),
                ttl: 0.72,
            });
            (base + skill_bonus) * 2.0
        } else {
            base + skill_bonus
        }
    }

    fn hit_monster(&mut self, index: usize, damage: f32, flashy: bool) {
        if index >= self.monsters.len() {
            return;
        }
        let monster_pos = self.monsters[index].pos;
        let monster_name = self.monsters[index].kind.name();
        self.monsters[index].hp -= damage;
        self.floating.push(FloatingText {
            pos: monster_pos,
            text: format!("-{}", damage.round() as i32),
            color: if flashy {
                Color::from_rgba(255, 112, 236, 255)
            } else {
                WHITE
            },
            ttl: 0.84,
        });
        self.spawn_particles(
            monster_pos,
            if flashy { 12 } else { 6 },
            if flashy {
                Color::from_rgba(255, 224, 96, 255)
            } else {
                Color::from_rgba(255, 180, 120, 255)
            },
        );
        self.screen_shake = self.screen_shake.max(if flashy { 7.0 } else { 4.0 });
        self.log(format!(
            "You hit {} for {}.",
            monster_name,
            damage.round() as i32
        ));
        if self.monsters[index].hp <= 0.0 {
            let monster = self.monsters.remove(index);
            self.on_monster_killed(monster);
        }
    }

    fn on_monster_killed(&mut self, monster: Monster) {
        let xp = monster_xp(monster.kind, monster.level);
        self.player.stats.xp += xp;
        self.player.stats.gold += self.rng.random_range(1..=7);
        self.floating.push(FloatingText {
            pos: monster.pos,
            text: format!("+{} xp", xp),
            color: Color::from_rgba(122, 236, 126, 255),
            ttl: 1.05,
        });
        self.spawn_particles(monster.pos, 18, monster.kind.color());
        self.log(format!("{} pops. +{} xp.", monster.kind.name(), xp));
        if self.rng.random_bool(0.54) {
            let item = roll_item(&mut self.rng, monster.level);
            self.log(format!("{} drops {}.", monster.kind.name(), item.name));
            self.loot.push(Loot {
                pos: monster.pos,
                item,
                bob: self.rng.random_range(0.0..10.0),
            });
        }
        while self.player.stats.xp >= self.player.stats.next_xp {
            self.player.stats.xp -= self.player.stats.next_xp;
            self.player.stats.level += 1;
            self.player.stats.next_xp = (self.player.stats.next_xp as f32 * 1.35) as i32;
            self.player.stats.strength += 1;
            self.player.stats.agility += 1;
            self.player.stats.vitality += 1;
            self.player.stats.unspent_stat_points += 3;
            self.player.stats.unspent_skill_points += 1;
            self.player.hp = self.player.max_hp();
            self.player.mana = self.player.max_mana();
            self.pulses.push(Pulse {
                pos: self.player.pos,
                radius: 22.0,
                ttl: 0.9,
                color: Color::from_rgba(255, 224, 96, 255),
            });
            self.spawn_particles(self.player.pos, 30, Color::from_rgba(255, 224, 96, 255));
            self.floating.push(FloatingText {
                pos: self.player.pos,
                text: format!("LEVEL {}", self.player.stats.level),
                color: Color::from_rgba(255, 224, 96, 255),
                ttl: 1.35,
            });
            self.log(format!(
                "Level {}! Everything hums louder.",
                self.player.stats.level
            ));
        }
    }

    fn spawn_particles(&mut self, pos: Vec2, count: usize, color: Color) {
        for _ in 0..count {
            let angle = self.rng.random_range(0.0..std::f32::consts::TAU);
            let speed = self.rng.random_range(28.0..=130.0);
            self.particles.push(Particle {
                pos,
                vel: vec2(angle.cos(), angle.sin()) * speed,
                color,
                ttl: self.rng.random_range(0.22..=0.62),
                radius: self.rng.random_range(1.5..=4.0),
            });
        }
    }

    fn spawn_monsters(&mut self, count: usize) {
        for _ in 0..count {
            self.spawn_monster();
        }
    }

    fn spawn_monster(&mut self) {
        loop {
            let player_tile = World::world_to_tile(self.player.pos);
            let tile = player_tile
                + ivec2(
                    self.rng.random_range(
                        -MONSTER_SPAWN_MAX_RADIUS_TILES..=MONSTER_SPAWN_MAX_RADIUS_TILES,
                    ),
                    self.rng.random_range(
                        -MONSTER_SPAWN_MAX_RADIUS_TILES..=MONSTER_SPAWN_MAX_RADIUS_TILES,
                    ),
                );
            let pos = World::tile_center(tile);
            if !self.world.tile(tile).walkable
                || pos.distance(self.player.pos) < MONSTER_SPAWN_MIN_RADIUS
                || self
                    .monsters
                    .iter()
                    .any(|monster| monster.pos.distance(pos) < 18.0)
            {
                continue;
            }
            let biome = self.world.biome_at_world(pos);
            let kind = roll_monster(&mut self.rng, biome);
            let level = scaled_monster_level(self.world.biome_level(pos), self.player.stats.level);
            let max_hp = monster_max_hp(kind, level);
            self.monsters.push(Monster {
                kind,
                pos,
                vel: Vec2::ZERO,
                hp: max_hp,
                max_hp,
                level,
                attack_cd: self.rng.random_range(0.0..kind.attack_cooldown()),
                wobble: self.rng.random_range(0.0..10.0),
            });
            break;
        }
    }

    fn cull_distant_monsters(&mut self) {
        self.monsters
            .retain(|monster| monster.pos.distance(self.player.pos) <= MONSTER_DESPAWN_RADIUS);
    }

    fn replenish_local_monsters(&mut self) {
        let nearby_count = self
            .monsters
            .iter()
            .filter(|monster| monster.pos.distance(self.player.pos) <= MONSTER_LOCAL_RADIUS)
            .count();
        if nearby_count < 40 {
            self.spawn_monsters(48 - nearby_count);
        }
    }

    fn pickup_loot(&mut self) {
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

    fn equip_selected_item(&mut self) {
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

    fn buy_selected_item(&mut self) {
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

    fn sell_selected_item(&mut self) {
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

    fn drop_selected_item(&mut self) {
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

    fn log(&mut self, message: String) {
        self.log.push(message);
        if self.log_scroll_offset > 0 {
            self.log_scroll_offset += 1;
        }
        if self.log.len() > MAX_LOG_ENTRIES {
            self.log.remove(0);
            self.log_scroll_offset = self.log_scroll_offset.min(self.log.len().saturating_sub(6));
        }
    }
}

pub fn combat_feed_rect() -> Rect {
    Rect::new(18.0, screen_height() - 320.0, 420.0, 250.0)
}

pub const TRAVEL_DESTINATIONS: [TravelDestination; 5] = [
    TravelDestination {
        name: "Ember Town",
        pos: IVec2::new(0, 0),
        min_level: 0,
    },
    TravelDestination {
        name: "North Road",
        pos: IVec2::new(0, -18),
        min_level: 1,
    },
    TravelDestination {
        name: "East March",
        pos: IVec2::new(30, 0),
        min_level: 2,
    },
    TravelDestination {
        name: "South Reach",
        pos: IVec2::new(0, 48),
        min_level: 3,
    },
    TravelDestination {
        name: "West Verge",
        pos: IVec2::new(-72, 0),
        min_level: 4,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leveling_grants_spendable_points() {
        let mut game = Game::new(1);
        game.player.stats.xp = game.player.stats.next_xp - monster_xp(MonsterKind::Imp, 1);
        game.on_monster_killed(Monster {
            kind: MonsterKind::Imp,
            pos: game.player.pos + vec2(40.0, 0.0),
            vel: Vec2::ZERO,
            hp: 0.0,
            max_hp: MonsterKind::Imp.max_hp(),
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
        });

        assert_eq!(game.player.stats.level, 2);
        assert_eq!(game.player.stats.unspent_stat_points, 3);
        assert_eq!(game.player.stats.unspent_skill_points, 1);
    }

    #[test]
    fn travel_destinations_reach_progressively_harder_biomes() {
        let world = World::new(1);
        let levels: Vec<i32> = TRAVEL_DESTINATIONS
            .iter()
            .map(|destination| world.biome_level(World::tile_center(destination.pos)))
            .collect();

        assert_eq!(levels, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn new_game_starts_in_town() {
        let game = Game::new(1);
        assert_eq!(game.world.biome_level(game.player.pos), 0);
        assert!(!game.known_tiles.is_empty());
    }

    #[test]
    fn walking_far_from_town_repopulates_monsters_around_the_player() {
        let mut game = Game::new(1);
        game.player.pos = World::tile_center(ivec2(220, 0));

        game.fixed_update(FIXED_DT);

        assert_eq!(
            game.monsters
                .iter()
                .filter(|monster| monster.pos.distance(game.player.pos) <= MONSTER_LOCAL_RADIUS)
                .count(),
            48
        );
        assert!(
            game.monsters
                .iter()
                .all(|monster| monster.pos.distance(game.player.pos) <= MONSTER_DESPAWN_RADIUS)
        );
        assert!(
            game.monsters
                .iter()
                .all(|monster| monster.level >= game.world.biome_level(game.player.pos) - 2)
        );
    }

    #[test]
    fn world_map_tracks_discovery_and_supports_navigation() {
        let mut game = Game::new(1);
        let known_at_start = game.known_tiles.len();
        game.player.pos = World::tile_center(ivec2(32, 0));
        game.fixed_update(FIXED_DT);
        assert!(game.known_tiles.len() > known_at_start);

        game.input.world_map_toggle_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.ui_mode, UiMode::WorldMap);
        assert_eq!(game.world_map.center_tile, vec2(32.0, 0.0));

        game.input.movement = Vec2::X;
        game.input.map_zoom_delta = 1.0;
        let zoom_before = game.world_map.zoom;
        game.fixed_update(FIXED_DT);
        assert!(game.world_map.center_tile.x > 32.0);
        assert!(game.world_map.zoom > zoom_before);

        game.input.map_recenter_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.world_map.center_tile, vec2(32.0, 0.0));
    }

    #[test]
    fn hovered_monster_prefers_the_enemy_under_the_cursor() {
        let mut game = Game::new(1);
        game.monsters = vec![
            Monster {
                kind: MonsterKind::Imp,
                pos: game.player.pos + vec2(20.0, 0.0),
                vel: Vec2::ZERO,
                hp: 12.0,
                max_hp: 24.0,
                level: 1,
                attack_cd: 0.0,
                wobble: 0.0,
            },
            Monster {
                kind: MonsterKind::Brute,
                pos: game.player.pos + vec2(24.0, 0.0),
                vel: Vec2::ZERO,
                hp: 48.0,
                max_hp: 62.0,
                level: 2,
                attack_cd: 0.0,
                wobble: 0.0,
            },
        ];
        game.input.aim_world = game.player.pos + vec2(24.0, 0.0);

        assert_eq!(
            game.hovered_monster().map(|monster| monster.kind),
            Some(MonsterKind::Brute)
        );
    }

    #[test]
    fn combat_feed_scrolls_through_older_entries() {
        let mut game = Game::new(1);
        game.log = (0..10).map(|index| format!("Entry {}", index)).collect();
        game.input.log_scroll_delta = 2;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.log_scroll_offset, 2);

        game.input.log_scroll_delta = -1;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.log_scroll_offset, 1);

        game.input.log_scroll_delta = 99;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.log_scroll_offset, 4);
    }

    #[test]
    fn equipping_moves_item_into_matching_slot() {
        let mut game = Game::new(1);
        game.inventory_cursor = 0;
        game.equip_selected_item();

        assert!(game.player.equipment.weapon.is_some());
        assert_eq!(game.player.inventory.len(), 1);
    }

    #[test]
    fn shop_buy_and_sell_round_trip_updates_gold_and_inventory() {
        let mut game = Game::new(1);
        game.player.stats.gold = 100;
        game.shop_cursor = 0;
        let starting_inventory = game.player.inventory.len();

        game.buy_selected_item();
        assert_eq!(game.player.inventory.len(), starting_inventory + 1);
        assert_eq!(game.player.stats.gold, 76);

        game.shop_tab = ShopTab::Sell;
        game.shop_cursor = game.player.inventory.len() - 1;
        game.sell_selected_item();
        assert_eq!(game.player.inventory.len(), starting_inventory);
        assert_eq!(game.player.stats.gold, 90);
    }

    #[test]
    fn gameplay_smoke_flow_reaches_combat_loot_shop_and_travel() {
        let mut game = Game::new(1);
        game.monsters.clear();
        game.player.facing = Vec2::X;
        game.player.stats.xp = game.player.stats.next_xp - monster_xp(MonsterKind::Imp, 1);
        game.monsters.push(Monster {
            kind: MonsterKind::Imp,
            pos: game.player.pos + vec2(32.0, 0.0),
            vel: Vec2::ZERO,
            hp: 1.0,
            max_hp: 1.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
        });

        game.basic_attack();
        assert!(game.monsters.is_empty());
        assert!(game.floating.iter().any(|text| text.text.starts_with('-')));
        assert!(game.floating.iter().any(|text| text.text.contains("xp")));
        assert_eq!(game.player.stats.level, 2);

        game.loot.push(Loot {
            pos: game.player.pos,
            item: roll_item(&mut game.rng, 1),
            bob: 0.0,
        });
        let inventory_before_loot = game.player.inventory.len();
        game.pickup_loot();
        assert_eq!(game.player.inventory.len(), inventory_before_loot + 1);

        game.inventory_cursor = 0;
        game.equip_selected_item();
        assert!(game.player.equipment.weapon.is_some());

        game.player.pos = game
            .npcs
            .iter()
            .find(|npc| npc.kind == NpcKind::Merchant)
            .unwrap()
            .pos;
        game.interact_with_nearby_npc();
        assert_eq!(game.ui_mode, UiMode::Merchant);

        game.player.stats.gold = 100;
        game.buy_selected_item();
        assert!(game.player.inventory.len() >= 2);

        game.ui_mode = UiMode::None;
        game.player.pos = game
            .npcs
            .iter()
            .find(|npc| npc.kind == NpcKind::Wayfinder)
            .unwrap()
            .pos;
        game.interact_with_nearby_npc();
        assert_eq!(game.ui_mode, UiMode::Travel);
        game.travel_cursor = 4;
        game.input.inventory_equip_pressed = true;
        game.update_travel_controls();
        assert_eq!(game.world.biome_level(game.player.pos), 4);
    }

    #[test]
    fn fireball_explodes_and_hits_nearby_monsters() {
        let mut game = Game::new(7);
        game.monsters = vec![
            Monster {
                kind: MonsterKind::Imp,
                pos: game.player.pos + vec2(34.0, 0.0),
                vel: Vec2::ZERO,
                hp: 80.0,
                max_hp: 80.0,
                level: 1,
                attack_cd: 0.0,
                wobble: 0.0,
            },
            Monster {
                kind: MonsterKind::Slime,
                pos: game.player.pos + vec2(48.0, 10.0),
                vel: Vec2::ZERO,
                hp: 80.0,
                max_hp: 80.0,
                level: 1,
                attack_cd: 0.0,
                wobble: 0.0,
            },
            Monster {
                kind: MonsterKind::Brute,
                pos: game.player.pos + vec2(120.0, 0.0),
                vel: Vec2::ZERO,
                hp: 80.0,
                max_hp: 80.0,
                level: 1,
                attack_cd: 0.0,
                wobble: 0.0,
            },
        ];
        game.player.facing = Vec2::X;

        game.cast_fireball();
        game.update_projectiles(FIXED_DT);

        assert!(game.projectiles.is_empty());
        assert!(game.monsters[0].hp < 80.0);
        assert!(game.monsters[1].hp < 80.0);
        assert_eq!(game.monsters[2].hp, 80.0);
    }

    #[test]
    fn cleave_hits_front_arc_without_hitting_behind() {
        let mut game = Game::new(8);
        game.monsters = vec![
            Monster {
                kind: MonsterKind::Imp,
                pos: game.player.pos + vec2(36.0, 0.0),
                vel: Vec2::ZERO,
                hp: 80.0,
                max_hp: 80.0,
                level: 1,
                attack_cd: 0.0,
                wobble: 0.0,
            },
            Monster {
                kind: MonsterKind::Slime,
                pos: game.player.pos + vec2(-36.0, 0.0),
                vel: Vec2::ZERO,
                hp: 80.0,
                max_hp: 80.0,
                level: 1,
                attack_cd: 0.0,
                wobble: 0.0,
            },
        ];
        game.player.facing = Vec2::X;

        game.cast_cleave();

        assert!(game.monsters[0].hp < 80.0);
        assert_eq!(game.monsters[1].hp, 80.0);
    }

    #[test]
    fn skill_book_spends_points_on_selected_skill() {
        let mut game = Game::new(9);
        game.player.stats.unspent_skill_points = 1;
        game.skill_book_cursor = 2;
        let starting_rank = game.player.fireball_rank;
        game.input.inventory_equip_pressed = true;

        game.update_skill_book_controls();

        assert_eq!(game.player.fireball_rank, starting_rank + 1);
        assert_eq!(game.player.stats.unspent_skill_points, 0);
    }

    #[test]
    fn every_ui_window_supports_basic_navigation() {
        let mut game = Game::new(3);

        game.input.inventory_toggle_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.ui_mode, UiMode::Inventory);
        game.input.inventory_down_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.inventory_cursor, 1);

        game.ui_mode = UiMode::None;
        game.input = InputState::default();
        game.input.character_toggle_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.ui_mode, UiMode::Character);
        game.input.inventory_down_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.character_cursor, 1);

        game.ui_mode = UiMode::None;
        game.input = InputState::default();
        game.input.skill_book_toggle_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.ui_mode, UiMode::SkillBook);
        game.input.inventory_down_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.skill_book_cursor, 1);

        game.ui_mode = UiMode::None;
        game.input = InputState::default();
        game.input.world_map_toggle_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.ui_mode, UiMode::WorldMap);

        game.ui_mode = UiMode::None;
        game.player.pos = game
            .npcs
            .iter()
            .find(|npc| npc.kind == NpcKind::Merchant)
            .unwrap()
            .pos;
        game.interact_with_nearby_npc();
        assert_eq!(game.ui_mode, UiMode::Merchant);
        game.input = InputState::default();
        game.input.nav_right_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.shop_tab, ShopTab::Sell);

        game.ui_mode = UiMode::None;
        game.player.pos = game
            .npcs
            .iter()
            .find(|npc| npc.kind == NpcKind::Trainer)
            .unwrap()
            .pos;
        game.interact_with_nearby_npc();
        assert_eq!(game.ui_mode, UiMode::Trainer);
        game.input = InputState::default();
        game.input.inventory_equip_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.ui_mode, UiMode::SkillBook);

        game.ui_mode = UiMode::None;
        game.player.pos = game
            .npcs
            .iter()
            .find(|npc| npc.kind == NpcKind::Wayfinder)
            .unwrap()
            .pos;
        game.interact_with_nearby_npc();
        assert_eq!(game.ui_mode, UiMode::Travel);
        game.input = InputState::default();
        game.input.inventory_down_pressed = true;
        game.fixed_update(FIXED_DT);
        assert_eq!(game.travel_cursor, 1);
    }
}
