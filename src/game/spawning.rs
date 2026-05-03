use std::collections::HashSet;

use ::rand::RngExt;
use macroquad::prelude::*;

use crate::{
    content::{
        MonsterKind, MonsterRank, monster_max_hp, roll_melee_monster, roll_monster,
        roll_ranged_monster, scaled_monster_level,
    },
    world::World,
};

use super::{
    Game, MONSTER_DESPAWN_RADIUS, MONSTER_LOCAL_PACK_REFILL_THRESHOLD, MONSTER_LOCAL_PACK_TARGET,
    MONSTER_LOCAL_RADIUS, MONSTER_PACK_MEMBER_MAX_DISTANCE, MONSTER_PACK_MEMBER_MIN_DISTANCE,
    MONSTER_PACK_SEPARATION, MONSTER_SPAWN_MAX_RADIUS_TILES, MONSTER_SPAWN_MIN_RADIUS,
};

const PACK_SIZE_WEIGHTS: [usize; 12] = [2, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 7];
const SPAWN_VISIBILITY_MARGIN: f32 = 24.0;
const MIXED_PACK_PERCENT: u32 = 35;

#[derive(Clone, Copy)]
pub(super) enum AmbientPackComposition {
    Homogeneous(MonsterKind),
    Mixed {
        core_kind: MonsterKind,
        support_kind: MonsterKind,
        support_index: usize,
    },
}

impl AmbientPackComposition {
    pub(super) fn kind_for_member(self, member_index: usize) -> MonsterKind {
        match self {
            Self::Homogeneous(kind) => kind,
            Self::Mixed {
                core_kind,
                support_kind,
                support_index,
            } => {
                if member_index == support_index {
                    support_kind
                } else {
                    core_kind
                }
            }
        }
    }
}

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
            let homogeneous_kind = roll_monster(&mut self.runtime.rng, biome);
            let level = scaled_monster_level(
                self.world.biome_level(pack_center),
                self.sim.player.stats.level,
            );
            let pack_size =
                PACK_SIZE_WEIGHTS[self.runtime.rng.random_range(0..PACK_SIZE_WEIGHTS.len())];
            let composition =
                self.roll_ambient_pack_composition(biome, pack_size, homogeneous_kind);
            let rare_index = self.runtime.rng.random_range(0..pack_size);
            let rare_rank = self.roll_pack_rank();
            let pack_id = self.runtime.next_monster_pack_id;
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
                let kind = composition.kind_for_member(member_index);
                let max_hp = monster_max_hp(kind, level, rank);
                pack.push(super::Monster {
                    kind,
                    rank,
                    quest_id: None,
                    pack_id,
                    pack_center,
                    pos,
                    vel: Vec2::ZERO,
                    hit_offset: Vec2::ZERO,
                    hp: max_hp,
                    max_hp,
                    level,
                    attack_cd: self.runtime.rng.random_range(0.0..kind.attack_cooldown()),
                    engaged: false,
                    wobble: self.runtime.rng.random_range(0.0..10.0),
                    hit_flash: 0.0,
                    chill_ttl: 0.0,
                });
            }

            if pack.len() == pack_size {
                self.runtime.next_monster_pack_id += 1;
                self.sim.monsters.extend(pack);
                return true;
            }
        }
        false
    }

    fn find_pack_center(&mut self) -> Option<Vec2> {
        let player_tile = World::world_to_tile(self.sim.player.pos);
        for _ in 0..128 {
            let tile = player_tile
                + ivec2(
                    self.runtime.rng.random_range(
                        -MONSTER_SPAWN_MAX_RADIUS_TILES..=MONSTER_SPAWN_MAX_RADIUS_TILES,
                    ),
                    self.runtime.rng.random_range(
                        -MONSTER_SPAWN_MAX_RADIUS_TILES..=MONSTER_SPAWN_MAX_RADIUS_TILES,
                    ),
                );
            let pos = World::tile_center(tile);
            if !self.world.tile(tile).walkable
                || self.world.is_safe_zone(tile)
                || pos.distance(self.sim.player.pos) < MONSTER_SPAWN_MIN_RADIUS
                || pos.distance(self.sim.player.pos) > MONSTER_LOCAL_RADIUS
                || pack_center_can_be_seen_with_view(
                    pos,
                    self.sim.player.pos,
                    self.runtime.spawn_visibility_half_view,
                )
                || self
                    .sim
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
            let angle = self.runtime.rng.random_range(0.0..std::f32::consts::TAU);
            let radius = self
                .runtime
                .rng
                .random_range(MONSTER_PACK_MEMBER_MIN_DISTANCE..=MONSTER_PACK_MEMBER_MAX_DISTANCE);
            let pos = pack_center + vec2(angle.cos(), angle.sin()) * radius;
            let tile = World::world_to_tile(pos);
            if !self.world.tile(tile).walkable
                || self.world.is_safe_zone(tile)
                || self
                    .sim
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
        monster_pack_rank_for_roll(self.runtime.rng.random_range(0..100))
    }

    fn roll_ambient_pack_composition(
        &mut self,
        biome: crate::world::Biome,
        pack_size: usize,
        homogeneous_kind: MonsterKind,
    ) -> AmbientPackComposition {
        if should_spawn_mixed_pack(self.runtime.rng.random_range(0..100))
            && let Some((core_kind, support_kind)) =
                roll_melee_monster(&mut self.runtime.rng, biome)
                    .zip(roll_ranged_monster(&mut self.runtime.rng, biome))
        {
            return AmbientPackComposition::Mixed {
                core_kind,
                support_kind,
                support_index: self.runtime.rng.random_range(0..pack_size),
            };
        }
        AmbientPackComposition::Homogeneous(homogeneous_kind)
    }

    pub(super) fn cull_distant_monsters(&mut self) {
        self.sim.monsters.retain(|monster| {
            monster.quest_id.is_some()
                || monster.pack_center.distance(self.sim.player.pos) <= MONSTER_DESPAWN_RADIUS
        });
    }

    pub(super) fn replenish_local_monsters(&mut self) {
        let nearby_pack_ids: HashSet<u64> = self
            .sim
            .monsters
            .iter()
            .filter(|monster| {
                monster.pack_center.distance(self.sim.player.pos) <= MONSTER_LOCAL_RADIUS
            })
            .map(|monster| monster.pack_id)
            .collect();
        if nearby_pack_ids.len() < MONSTER_LOCAL_PACK_REFILL_THRESHOLD {
            self.spawn_monster_packs(MONSTER_LOCAL_PACK_TARGET - nearby_pack_ids.len());
        }
    }
}

pub(super) fn should_spawn_mixed_pack(roll: u32) -> bool {
    roll < MIXED_PACK_PERCENT
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
