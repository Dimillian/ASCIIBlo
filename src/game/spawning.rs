use ::rand::RngExt;
use macroquad::prelude::*;

use crate::{
    content::{monster_max_hp, roll_monster, scaled_monster_level},
    world::World,
};

use super::{
    Game, MONSTER_DESPAWN_RADIUS, MONSTER_LOCAL_RADIUS, MONSTER_SPAWN_MAX_RADIUS_TILES,
    MONSTER_SPAWN_MIN_RADIUS,
};

impl Game {
    pub(super) fn spawn_monsters(&mut self, count: usize) {
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
            self.monsters.push(super::Monster {
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

    pub(super) fn cull_distant_monsters(&mut self) {
        self.monsters
            .retain(|monster| monster.pos.distance(self.player.pos) <= MONSTER_DESPAWN_RADIUS);
    }

    pub(super) fn replenish_local_monsters(&mut self) {
        let nearby_count = self
            .monsters
            .iter()
            .filter(|monster| monster.pos.distance(self.player.pos) <= MONSTER_LOCAL_RADIUS)
            .count();
        if nearby_count < 40 {
            self.spawn_monsters(48 - nearby_count);
        }
    }
}
