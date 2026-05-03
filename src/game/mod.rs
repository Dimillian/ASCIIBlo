mod combat;
mod input;
mod inventory;
mod menus;
mod progression;
mod spawning;
mod types;

use std::collections::HashSet;

use ::rand::{SeedableRng, rngs::StdRng};
use macroquad::prelude::*;

use crate::{
    content::{Item, NpcKind, merchant_stock, starter_items},
    world::{TILE, World},
};

use input::InputState;
use progression::discipline_next_xp;
pub use types::*;

pub const FIXED_DT: f32 = 1.0 / 60.0;
const PLAYER_RADIUS: f32 = 9.0;
const MONSTER_RADIUS: f32 = 9.0;
const PASSIVE_AGGRO_RADIUS: f32 = 176.0;
const MONSTER_SPAWN_MIN_RADIUS: f32 = TILE * 11.0;
const MONSTER_SPAWN_MAX_RADIUS_TILES: i32 = 36;
const MONSTER_LOCAL_RADIUS: f32 = TILE * 40.0;
const MONSTER_DESPAWN_RADIUS: f32 = TILE * 50.0;
const MONSTER_LOCAL_PACK_REFILL_THRESHOLD: usize = 6;
const MONSTER_LOCAL_PACK_TARGET: usize = 8;
const MONSTER_PACK_MEMBER_MIN_DISTANCE: f32 = TILE * 1.5;
const MONSTER_PACK_MEMBER_MAX_DISTANCE: f32 = TILE * 3.0;
const MONSTER_PACK_SEPARATION: f32 = TILE * 7.0;
const DEFAULT_SPAWN_VISIBILITY_HALF_VIEW: Vec2 = Vec2::new(640.0, 380.0);
const EXPLORATION_RADIUS: i32 = 14;
const MAX_LOG_ENTRIES: usize = 32;

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
    pub meteors: Vec<MeteorStrike>,
    pub skill_xp_toasts: Vec<SkillXpToast>,
    pub notifications: Vec<Notification>,
    pub log: Vec<String>,
    pub log_scroll_offset: usize,
    pub known_tiles: HashSet<IVec2>,
    pub ui_mode: UiMode,
    pub inventory_cursor: usize,
    pub character_cursor: usize,
    pub skill_book_cursor: usize,
    pub skill_book_ability_cursor: usize,
    pub skill_book_focus: SkillBookFocus,
    pub shop_cursor: usize,
    pub shop_tab: ShopTab,
    pub travel_cursor: usize,
    pub merchant_stock: Vec<Item>,
    pub world_map: WorldMapState,
    pub screen_shake: f32,
    pub elapsed: f32,
    pub agility_distance_bank: f32,
    pub preview_hover_world: Option<Vec2>,
    pub preview_hover_screen: Option<Vec2>,
    spawn_visibility_half_view: Vec2,
    next_monster_pack_id: u64,
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
                ability_cooldowns: [0.0; 8],
                bound_abilities: [AbilityKind::Cleave, AbilityKind::Fireball],
                stats: Stats {
                    level: 1,
                    xp: 0,
                    next_xp: 45,
                    strength: 5,
                    agility: 5,
                    vitality: 4,
                    gold: 0,
                    unspent_stat_points: 0,
                },
                inventory: starter_items(),
                equipment: Equipment {
                    weapon: None,
                    armor: None,
                    charm: None,
                },
                disciplines: Disciplines {
                    melee: DisciplineProgress {
                        level: 1,
                        xp: 0,
                        next_xp: discipline_next_xp(1),
                    },
                    magic: DisciplineProgress {
                        level: 1,
                        xp: 0,
                        next_xp: discipline_next_xp(1),
                    },
                    armor: DisciplineProgress {
                        level: 1,
                        xp: 0,
                        next_xp: discipline_next_xp(1),
                    },
                    agility: DisciplineProgress {
                        level: 1,
                        xp: 0,
                        next_xp: discipline_next_xp(1),
                    },
                },
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
            meteors: Vec::new(),
            skill_xp_toasts: Vec::new(),
            notifications: Vec::new(),
            log: vec!["The bell in Ember Town rings. Go make trouble.".into()],
            log_scroll_offset: 0,
            known_tiles: HashSet::new(),
            ui_mode: UiMode::None,
            inventory_cursor: 0,
            character_cursor: 0,
            skill_book_cursor: 0,
            skill_book_ability_cursor: 0,
            skill_book_focus: SkillBookFocus::Disciplines,
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
            agility_distance_bank: 0.0,
            preview_hover_world: None,
            preview_hover_screen: None,
            spawn_visibility_half_view: DEFAULT_SPAWN_VISIBILITY_HALF_VIEW,
            next_monster_pack_id: 0,
            rng: StdRng::seed_from_u64(seed),
            input: InputState::default(),
            quit: false,
        };
        game.reveal_around_tile(World::world_to_tile(game.player.pos), EXPLORATION_RADIUS);
        game.spawn_monster_packs(MONSTER_LOCAL_PACK_TARGET);
        game
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
        for cooldown in &mut self.player.ability_cooldowns {
            *cooldown = (*cooldown - dt).max(0.0);
        }
        self.player.mana =
            (self.player.mana + dt * self.player.mana_regen_rate()).min(self.player.max_mana());

        self.update_player_movement(dt);
        self.reveal_around_tile(World::world_to_tile(self.player.pos), EXPLORATION_RADIUS);
        if self.input.attack_pressed {
            self.basic_attack();
        }
        for slot in 0..self.player.bound_abilities.len() {
            if self.input.ability_slot_pressed[slot] {
                self.cast_ability(self.player.bound_abilities[slot]);
            }
        }
        if self.input.pickup_pressed {
            self.pickup_loot();
        }

        self.update_projectiles(dt);
        self.update_meteors(dt);
        self.update_monsters(dt);
        self.cull_distant_monsters();
        self.replenish_local_monsters();
        self.clear_edge_inputs();
    }

    pub fn set_spawn_visibility_viewport(&mut self, size: Vec2) {
        self.spawn_visibility_half_view = size * 0.5;
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

        for toast in &mut self.skill_xp_toasts {
            toast.ttl -= dt;
        }
        self.skill_xp_toasts.retain(|toast| toast.ttl > 0.0);

        for notification in &mut self.notifications {
            notification.ttl -= dt;
        }
        self.notifications
            .retain(|notification| notification.ttl > 0.0);

        for loot in &mut self.loot {
            loot.bob += dt * 4.0;
        }
        self.screen_shake = (self.screen_shake - dt * 18.0).max(0.0);
    }

    pub fn camera_focus(&self) -> Vec2 {
        self.player.pos + self.player.vel * 0.14
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

    fn update_player_movement(&mut self, dt: f32) {
        let before = self.player.pos;
        let speed = self.player.move_speed();
        let desired = self.input.movement * speed;
        self.player.vel = self.player.vel.lerp(desired, 1.0 - 0.0002_f32.powf(dt));
        if self.input.aim_world.distance_squared(self.player.pos) > 4.0 {
            self.player.facing = (self.input.aim_world - self.player.pos).normalize();
        } else if self.player.vel.length_squared() > 1.0 {
            self.player.facing = self.player.vel.normalize();
        }
        self.move_with_collision(self.player.vel * dt);
        self.award_agility_distance(self.player.pos.distance(before));
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

    pub(super) fn log(&mut self, message: String) {
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
mod tests;
