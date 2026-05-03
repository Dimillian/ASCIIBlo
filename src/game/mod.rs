mod abilities;
mod ability_defs;
mod combat;
mod events;
mod input;
mod inventory;
mod menus;
mod progression;
mod quests;
mod spawning;
mod state;
mod types;

use ::rand::{SeedableRng, rngs::StdRng};
use macroquad::prelude::*;

use crate::{
    content::{NpcKind, merchant_stock, starter_items},
    world::{SettlementTier, TILE, TOWN_RADIUS, World},
};

use input::InputState;
use progression::discipline_next_xp;
use state::{FxState, RuntimeState, SimulationState, UiState};
pub use types::*;

pub const FIXED_DT: f32 = 1.0 / 60.0;
const PLAYER_RADIUS: f32 = 9.0;
const MONSTER_RADIUS: f32 = 9.0;
const PASSIVE_AGGRO_RADIUS: f32 = 176.0;
const MONSTER_DISENGAGE_RADIUS: f32 = TILE * 20.0;
const MONSTER_SPAWN_MIN_RADIUS: f32 = TILE * 11.0;
const MONSTER_SPAWN_MAX_RADIUS_TILES: i32 = 36;
const MONSTER_LOCAL_RADIUS: f32 = TILE * 40.0;
const MONSTER_DESPAWN_RADIUS: f32 = TILE * 50.0;
const MONSTER_LOCAL_PACK_REFILL_THRESHOLD: usize = 6;
const MONSTER_LOCAL_PACK_TARGET: usize = 8;
const MONSTER_PACK_MEMBER_MIN_DISTANCE: f32 = TILE * 1.5;
const MONSTER_PACK_MEMBER_MAX_DISTANCE: f32 = TILE * 3.0;
const MONSTER_PACK_ALERT_RADIUS: f32 = MONSTER_PACK_MEMBER_MAX_DISTANCE * 2.0;
const MONSTER_PACK_SEPARATION: f32 = TILE * 7.0;
const DEFAULT_SPAWN_VISIBILITY_HALF_VIEW: Vec2 = Vec2::new(640.0, 380.0);
const EXPLORATION_RADIUS: i32 = 14;
const MAX_LOG_ENTRIES: usize = 32;

pub struct Game {
    pub world: World,
    pub sim: SimulationState,
    pub ui: UiState,
    pub fx: FxState,
    pub runtime: RuntimeState,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let world = World::new(seed);
        let spawn = World::tile_center(ivec2(0, 0));
        let player = Player {
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
            inventory: Backpack::from_items(starter_items()),
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
        };
        let mut game = Self {
            world,
            sim: SimulationState::new(player, merchant_stock()),
            ui: UiState::new(),
            fx: FxState::new(),
            runtime: RuntimeState::new(StdRng::seed_from_u64(seed)),
        };
        game.reveal_around_tile(
            World::world_to_tile(game.sim.player.pos),
            EXPLORATION_RADIUS,
        );
        game.sync_local_npcs();
        game.spawn_monster_packs(MONSTER_LOCAL_PACK_TARGET);
        game
    }

    pub fn fixed_update(&mut self, dt: f32) {
        self.runtime.elapsed += dt;
        self.update_log_scroll();
        if self.runtime.input.quit_pressed && self.ui.mode == UiMode::None {
            self.runtime.quit = true;
        }
        if self.runtime.input.inventory_toggle_pressed {
            self.ui.mode = if self.ui.mode == UiMode::Inventory {
                UiMode::None
            } else {
                UiMode::Inventory
            };
            self.ui.inventory_backpack_cursor = self
                .ui
                .inventory_backpack_cursor
                .min(self.sim.player.inventory.len().saturating_sub(1));
        }
        if self.runtime.input.character_toggle_pressed {
            self.ui.mode = if self.ui.mode == UiMode::Character {
                UiMode::None
            } else {
                UiMode::Character
            };
        }
        if self.runtime.input.skill_book_toggle_pressed {
            self.ui.mode = if self.ui.mode == UiMode::SkillBook {
                UiMode::None
            } else {
                UiMode::SkillBook
            };
        }
        if self.runtime.input.world_map_toggle_pressed {
            self.ui.mode = if self.ui.mode == UiMode::WorldMap {
                UiMode::None
            } else {
                self.center_world_map_on_player();
                UiMode::WorldMap
            };
        }
        if self.runtime.input.quit_pressed && self.ui.mode != UiMode::None {
            self.ui.mode = UiMode::None;
        }
        match self.ui.mode {
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

        if self.runtime.input.interact_pressed {
            self.interact_with_nearby_world_entity();
        }
        if self.ui.mode != UiMode::None {
            self.clear_edge_inputs();
            return;
        }

        self.sim.player.attack_cd = (self.sim.player.attack_cd - dt).max(0.0);
        for cooldown in &mut self.sim.player.ability_cooldowns {
            *cooldown = (*cooldown - dt).max(0.0);
        }
        self.sim.player.mana = (self.sim.player.mana + dt * self.sim.player.mana_regen_rate())
            .min(self.sim.player.max_mana());

        self.update_player_movement(dt);
        self.reveal_around_tile(
            World::world_to_tile(self.sim.player.pos),
            EXPLORATION_RADIUS,
        );
        self.sync_local_npcs();
        if self.runtime.input.attack_pressed {
            self.basic_attack();
        }
        for slot in 0..self.sim.player.bound_abilities.len() {
            if self.runtime.input.ability_slot_pressed[slot] {
                self.cast_ability(self.sim.player.bound_abilities[slot]);
            }
        }
        if self.runtime.input.pickup_pressed {
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
        self.runtime.spawn_visibility_half_view = size * 0.5;
    }

    pub fn frame_update(&mut self, dt: f32) {
        for text in &mut self.fx.floating {
            text.ttl -= dt;
            text.pos.y -= dt * 24.0;
        }
        self.fx.floating.retain(|text| text.ttl > 0.0);

        for particle in &mut self.fx.particles {
            particle.ttl -= dt;
            particle.pos += particle.vel * dt;
            particle.vel *= 0.94_f32.powf(dt * 60.0);
        }
        self.fx.particles.retain(|particle| particle.ttl > 0.0);

        for pulse in &mut self.fx.pulses {
            pulse.ttl -= dt;
            pulse.radius += dt * 140.0;
        }
        self.fx.pulses.retain(|pulse| pulse.ttl > 0.0);

        for slash in &mut self.fx.slash_arcs {
            slash.ttl -= dt;
            slash.radius += dt * 36.0;
        }
        self.fx.slash_arcs.retain(|slash| slash.ttl > 0.0);

        for toast in &mut self.fx.skill_xp_toasts {
            toast.ttl -= dt;
        }
        self.fx.skill_xp_toasts.retain(|toast| toast.ttl > 0.0);

        for notification in &mut self.fx.notifications {
            notification.ttl -= dt;
        }
        self.fx
            .notifications
            .retain(|notification| notification.ttl > 0.0);

        for loot in &mut self.sim.loot {
            loot.bob += dt * 4.0;
        }
        self.fx.screen_shake = (self.fx.screen_shake - dt * 18.0).max(0.0);
    }

    pub fn camera_focus(&self) -> Vec2 {
        self.sim.player.pos + self.sim.player.vel * 0.14
    }

    pub fn hovered_monster(&self) -> Option<&Monster> {
        let hover_world = self
            .runtime
            .preview_hover_world
            .unwrap_or(self.runtime.input.aim_world);
        self.sim
            .monsters
            .iter()
            .filter(|monster| monster.pos.distance(hover_world) <= 18.0)
            .min_by(|a, b| {
                a.pos
                    .distance(hover_world)
                    .total_cmp(&b.pos.distance(hover_world))
            })
    }

    pub fn quit_requested(&self) -> bool {
        self.runtime.quit
    }

    pub fn ui_hover_position(&self) -> Vec2 {
        self.runtime
            .preview_hover_screen
            .unwrap_or(mouse_position().into())
    }

    pub(crate) fn reveal_around_tile(&mut self, center: IVec2, radius: i32) {
        for y in center.y - radius..=center.y + radius {
            for x in center.x - radius..=center.x + radius {
                let tile = ivec2(x, y);
                if tile.distance_squared(center) <= radius * radius {
                    self.sim.known_tiles.insert(tile);
                }
            }
        }
        self.discover_settlements_near(center, radius + TOWN_RADIUS + 2);
    }

    fn discover_settlements_near(&mut self, center: IVec2, radius: i32) {
        for site in self.world.settlements_near_tile(center, radius) {
            if !self
                .sim
                .discovered_settlements
                .iter()
                .any(|known| known.site.id == site.id)
            {
                self.sim
                    .discovered_settlements
                    .push(DiscoveredSettlement { site });
                if site.tier == SettlementTier::Town {
                    self.sim.travel_destinations.push(TravelDestination {
                        name: site.name(),
                        pos: site.center,
                        min_level: self.world.biome_level_at_tile(site.center),
                    });
                    self.sim
                        .travel_destinations
                        .sort_by_key(|destination| destination.min_level);
                }
            }
        }
    }

    fn sync_local_npcs(&mut self) {
        let player_tile = World::world_to_tile(self.sim.player.pos);
        let mut npcs = Vec::new();
        for site in self.world.settlements_near_tile(player_tile, 42) {
            if site.tier == SettlementTier::Town || site.is_origin() {
                npcs.push(Npc {
                    kind: NpcKind::Merchant,
                    name: NpcKind::Merchant.name().into(),
                    quest_id: None,
                    pos: World::tile_center(site.center + ivec2(-5, 0)),
                });
                npcs.push(Npc {
                    kind: NpcKind::Trainer,
                    name: NpcKind::Trainer.name().into(),
                    quest_id: None,
                    pos: World::tile_center(site.center + ivec2(0, -5)),
                });
                npcs.push(Npc {
                    kind: NpcKind::Wayfinder,
                    name: NpcKind::Wayfinder.name().into(),
                    quest_id: None,
                    pos: World::tile_center(site.center + ivec2(5, 0)),
                });
            } else {
                npcs.push(Npc {
                    kind: NpcKind::Merchant,
                    name: NpcKind::Merchant.name().into(),
                    quest_id: None,
                    pos: World::tile_center(site.center + ivec2(-4, 0)),
                });
            }
        }
        if let Some(quest_id) = self.sim.active_quest.as_ref().map(|quest| quest.id) {
            npcs.extend(
                self.sim
                    .npcs
                    .iter()
                    .filter(|npc| npc.quest_id == Some(quest_id))
                    .cloned(),
            );
        }
        self.sim.npcs = npcs;
    }

    fn update_player_movement(&mut self, dt: f32) {
        let before = self.sim.player.pos;
        let speed = self.sim.player.move_speed();
        let desired = self.runtime.input.movement * speed;
        self.sim.player.vel = self.sim.player.vel.lerp(desired, 1.0 - 0.0002_f32.powf(dt));
        if self
            .runtime
            .input
            .aim_world
            .distance_squared(self.sim.player.pos)
            > 4.0
        {
            self.sim.player.facing =
                (self.runtime.input.aim_world - self.sim.player.pos).normalize();
        } else if self.sim.player.vel.length_squared() > 1.0 {
            self.sim.player.facing = self.sim.player.vel.normalize();
        }
        self.move_with_collision(self.sim.player.vel * dt);
        self.award_agility_distance(self.sim.player.pos.distance(before));
    }

    fn move_with_collision(&mut self, delta: Vec2) {
        let next_x = self.sim.player.pos + vec2(delta.x, 0.0);
        if !self.world.collides_circle(next_x, PLAYER_RADIUS) {
            self.sim.player.pos.x = next_x.x;
        }
        let next_y = self.sim.player.pos + vec2(0.0, delta.y);
        if !self.world.collides_circle(next_y, PLAYER_RADIUS) {
            self.sim.player.pos.y = next_y.y;
        }
    }

    pub(super) fn log(&mut self, message: String) {
        self.fx.log.push(message);
        if self.fx.log_scroll_offset > 0 {
            self.fx.log_scroll_offset += 1;
        }
        if self.fx.log.len() > MAX_LOG_ENTRIES {
            self.fx.log.remove(0);
            self.fx.log_scroll_offset = self
                .fx
                .log_scroll_offset
                .min(self.fx.log.len().saturating_sub(6));
        }
    }
}

pub fn combat_feed_rect() -> Rect {
    Rect::new(18.0, screen_height() - 320.0, 420.0, 250.0)
}

#[cfg(test)]
mod tests;
