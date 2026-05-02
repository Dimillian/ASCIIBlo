use ::rand::RngExt;
use macroquad::prelude::*;

use crate::{
    content::{monster_damage, monster_xp, roll_item},
    world::World,
};

use super::{
    FloatingText, Game, Loot, MONSTER_RADIUS, Monster, PASSIVE_AGGRO_RADIUS, PLAYER_RADIUS,
    Particle, Projectile, Pulse, SlashArc,
};

impl Game {
    pub(super) fn basic_attack(&mut self) {
        if self.player.attack_cd > 0.0 {
            return;
        }
        let range = 54.0;
        let direction = self.player.facing;
        self.slash_arcs.push(SlashArc {
            pos: self.player.pos,
            direction,
            radius: 34.0,
            ttl: 0.16,
            color: Color::from_rgba(236, 238, 244, 255),
        });
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

    pub(super) fn cast_rush(&mut self) {
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

    pub(super) fn cast_nova(&mut self) {
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

    pub(super) fn cast_fireball(&mut self) {
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

    pub(super) fn cast_cleave(&mut self) {
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

    pub(super) fn update_projectiles(&mut self, dt: f32) {
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

    pub(super) fn update_monsters(&mut self, dt: f32) {
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

    pub(super) fn on_monster_killed(&mut self, monster: Monster) {
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

    pub(super) fn spawn_particles(&mut self, pos: Vec2, count: usize, color: Color) {
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
}
