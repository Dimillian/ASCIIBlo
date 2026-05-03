use std::collections::HashSet;

use ::rand::RngExt;
use macroquad::prelude::*;

use crate::{
    content::{MonsterRank, monster_max_hp, roll_monster, scaled_monster_level},
    world::World,
};

use super::{
    Game, MONSTER_DESPAWN_RADIUS, MONSTER_LOCAL_PACK_REFILL_THRESHOLD, MONSTER_LOCAL_PACK_TARGET,
    MONSTER_LOCAL_RADIUS, MONSTER_PACK_MEMBER_MAX_DISTANCE, MONSTER_PACK_MEMBER_MIN_DISTANCE,
    MONSTER_PACK_SEPARATION, MONSTER_SPAWN_MAX_RADIUS_TILES, MONSTER_SPAWN_MIN_RADIUS,
};

const PACK_SIZE_WEIGHTS: [usize; 12] = [2, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 7];
const SPAWN_VISIBILITY_MARGIN: f32 = 24.0;

pub(super) fn monster_pack_rank_for_roll(roll: u32) -> MonsterRank {
    match roll {
        0..=1 => MonsterRank::Boss,
        2..=11 => MonsterRank::Elite,
        _ => MonsterRank::Normal,
    }
}

impl Game {
    pub(super) fn spawn_monster_packs(&mut self, count: usize) {
        for _ in 0..count {
            if !self.spawn_monster_pack() {
                break;
            }
        }
    }

    fn spawn_monster_pack(&mut self) -> bool {
        for _ in 0..256 {
            let Some(pack_center) = self.find_pack_center() else {
                continue;
            };
            let biome = self.world.biome_at_world(pack_center);
            let kind = roll_monster(&mut self.rng, biome);
            let level =
                scaled_monster_level(self.world.biome_level(pack_center), self.player.stats.level);
            let pack_size = PACK_SIZE_WEIGHTS[self.rng.random_range(0..PACK_SIZE_WEIGHTS.len())];
            let rare_index = self.rng.random_range(0..pack_size);
            let rare_rank = self.roll_pack_rank();
            let pack_id = self.next_monster_pack_id;
            let mut pack = Vec::with_capacity(pack_size);

            for member_index in 0..pack_size {
                let rank = if member_index == rare_index {
                    rare_rank
                } else {
                    MonsterRank::Normal
                };
                let Some(pos) = self.find_pack_member_pos(pack_center, &pack) else {
                    pack.clear();
                    break;
                };
                let max_hp = monster_max_hp(kind, level, rank);
                pack.push(super::Monster {
                    kind,
                    rank,
                    pack_id,
                    pack_center,
                    pos,
                    vel: Vec2::ZERO,
                    hit_offset: Vec2::ZERO,
                    hp: max_hp,
                    max_hp,
                    level,
                    attack_cd: self.rng.random_range(0.0..kind.attack_cooldown()),
                    wobble: self.rng.random_range(0.0..10.0),
                    hit_flash: 0.0,
                    chill_ttl: 0.0,
                });
            }

            if pack.len() == pack_size {
                self.next_monster_pack_id += 1;
                self.monsters.extend(pack);
                return true;
            }
        }
        false
    }

    fn find_pack_center(&mut self) -> Option<Vec2> {
        let player_tile = World::world_to_tile(self.player.pos);
        for _ in 0..128 {
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
                || pos.distance(self.player.pos) > MONSTER_LOCAL_RADIUS
                || pack_center_can_be_seen_with_view(
                    pos,
                    self.player.pos,
                    self.spawn_visibility_half_view,
                )
                || self
                    .monsters
                    .iter()
                    .any(|monster| monster.pack_center.distance(pos) < MONSTER_PACK_SEPARATION)
            {
                continue;
            }
            return Some(pos);
        }
        None
    }

    fn find_pack_member_pos(&mut self, pack_center: Vec2, pack: &[super::Monster]) -> Option<Vec2> {
        for _ in 0..96 {
            let angle = self.rng.random_range(0.0..std::f32::consts::TAU);
            let radius = self
                .rng
                .random_range(MONSTER_PACK_MEMBER_MIN_DISTANCE..=MONSTER_PACK_MEMBER_MAX_DISTANCE);
            let pos = pack_center + vec2(angle.cos(), angle.sin()) * radius;
            let tile = World::world_to_tile(pos);
            if !self.world.tile(tile).walkable
                || self
                    .monsters
                    .iter()
                    .any(|monster| monster.pos.distance(pos) < 18.0)
                || pack.iter().any(|monster| monster.pos.distance(pos) < 18.0)
            {
                continue;
            }
            return Some(pos);
        }
        None
    }

    fn roll_pack_rank(&mut self) -> MonsterRank {
        monster_pack_rank_for_roll(self.rng.random_range(0..100))
    }

    pub(super) fn cull_distant_monsters(&mut self) {
        self.monsters.retain(|monster| {
            monster.pack_center.distance(self.player.pos) <= MONSTER_DESPAWN_RADIUS
        });
    }

    pub(super) fn replenish_local_monsters(&mut self) {
        let nearby_pack_ids: HashSet<u64> = self
            .monsters
            .iter()
            .filter(|monster| monster.pack_center.distance(self.player.pos) <= MONSTER_LOCAL_RADIUS)
            .map(|monster| monster.pack_id)
            .collect();
        if nearby_pack_ids.len() < MONSTER_LOCAL_PACK_REFILL_THRESHOLD {
            self.spawn_monster_packs(MONSTER_LOCAL_PACK_TARGET - nearby_pack_ids.len());
        }
    }
}

pub(super) fn pack_center_can_be_seen_with_view(
    pack_center: Vec2,
    player_pos: Vec2,
    half_view: Vec2,
) -> bool {
    let delta = pack_center - player_pos;
    let pack_padding = MONSTER_PACK_MEMBER_MAX_DISTANCE + SPAWN_VISIBILITY_MARGIN;
    delta.x.abs() <= half_view.x + pack_padding && delta.y.abs() <= half_view.y + pack_padding
}
