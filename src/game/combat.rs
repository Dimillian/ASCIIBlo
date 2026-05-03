use ::rand::RngExt;
use macroquad::prelude::*;

use crate::{
    content::{monster_damage, monster_xp, roll_item},
    world::World,
};

use super::{
    AbilityKind, DisciplineKind, FloatingText, Game, Loot, MONSTER_RADIUS, MeteorStrike, Monster,
    PASSIVE_AGGRO_RADIUS, Particle, Projectile, Pulse, SlashArc, events::GameplayEvent,
};

#[derive(Clone, Copy)]
pub(super) enum DamageKind {
    Basic,
    PhysicalSkill,
    MagicSkill,
}

impl Game {
    pub(super) fn basic_attack(&mut self) {
        if self.sim.player.attack_cd > 0.0 {
            return;
        }
        let range = 54.0;
        let direction = self.sim.player.facing;
        self.fx.slash_arcs.push(SlashArc {
            pos: self.sim.player.pos,
            direction,
            radius: 34.0,
            ttl: 0.16,
            color: Color::from_rgba(236, 238, 244, 255),
        });
        let target = self
            .sim
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                let to_monster = monster.pos - self.sim.player.pos;
                let distance = to_monster.length();
                let aligned = to_monster.normalize_or_zero().dot(direction);
                (distance <= range && aligned > 0.45).then_some((index, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index);
        if let Some(index) = target {
            let damage = self.roll_player_damage(DamageKind::Basic);
            self.hit_monster(index, damage, false);
            self.award_discipline_xp(DisciplineKind::Melee, 2);
        } else {
            self.log("You carve only air.".into());
        }
        self.sim.player.attack_cd = self.sim.player.attack_interval();
    }

    pub(super) fn update_projectiles(&mut self, dt: f32) {
        let mut active = Vec::with_capacity(self.fx.projectiles.len());
        let mut impacts = Vec::new();
        for mut projectile in self.fx.projectiles.drain(..) {
            projectile.ttl -= dt;
            projectile.pos += projectile.vel * dt;
            let hits_monster =
                self.sim.monsters.iter().any(|monster| {
                    monster.pos.distance(projectile.pos) <= projectile.radius + 12.0
                });
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
        self.fx.projectiles = active;
        for projectile in impacts {
            match projectile.ability {
                AbilityKind::Fireball => self.detonate_fireball(projectile),
                AbilityKind::IceBolt => self.impact_ice_bolt(projectile),
                _ => {}
            }
        }
    }

    pub(super) fn update_meteors(&mut self, dt: f32) {
        let mut pending = Vec::with_capacity(self.fx.meteors.len());
        let mut impacts = Vec::new();
        for mut meteor in self.fx.meteors.drain(..) {
            meteor.ttl -= dt;
            if meteor.ttl <= 0.0 {
                impacts.push(meteor);
            } else {
                pending.push(meteor);
            }
        }
        self.fx.meteors = pending;
        for meteor in impacts {
            self.impact_meteor(meteor);
        }
    }

    fn detonate_fireball(&mut self, projectile: Projectile) {
        self.fx.pulses.push(Pulse {
            pos: projectile.pos,
            radius: 14.0,
            ttl: 0.36,
            color: projectile.color,
        });
        self.spawn_particles(projectile.pos, 20, projectile.color);
        let hits: Vec<usize> = self
            .sim
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
        let landed = !hits.is_empty();
        for index in hits.into_iter().rev() {
            self.hit_monster(index, projectile.damage, true);
        }
        if landed {
            self.award_discipline_xp(DisciplineKind::Magic, 4);
        }
    }

    fn impact_ice_bolt(&mut self, projectile: Projectile) {
        let target = self
            .sim
            .monsters
            .iter()
            .enumerate()
            .find(|(_, monster)| monster.pos.distance(projectile.pos) <= projectile.radius + 12.0)
            .map(|(index, _)| index);
        if let Some(index) = target {
            self.sim.monsters[index].chill_ttl = self.sim.monsters[index].chill_ttl.max(1.8);
            self.hit_monster(index, projectile.damage, true);
            self.award_discipline_xp(DisciplineKind::Magic, 4);
        } else {
            self.log("Ice Bolt cracks against the ground.".into());
        }
    }

    fn impact_meteor(&mut self, meteor: MeteorStrike) {
        self.fx.pulses.push(Pulse {
            pos: meteor.pos,
            radius: 20.0,
            ttl: 0.48,
            color: Color::from_rgba(255, 132, 64, 255),
        });
        self.spawn_particles(meteor.pos, 28, Color::from_rgba(255, 132, 64, 255));
        let hits: Vec<usize> = self
            .sim
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                (monster.pos.distance(meteor.pos) <= meteor.radius).then_some(index)
            })
            .collect();
        if hits.is_empty() {
            self.log("Meteor hammers empty ground.".into());
        }
        let landed = !hits.is_empty();
        for index in hits.into_iter().rev() {
            self.hit_monster(index, meteor.damage, true);
        }
        if landed {
            self.award_discipline_xp(DisciplineKind::Magic, 6);
        }
    }

    pub(super) fn update_monsters(&mut self, dt: f32) {
        let player_pos = self.sim.player.pos;
        let mut attacks = Vec::new();
        for index in 0..self.sim.monsters.len() {
            let monster = &mut self.sim.monsters[index];
            monster.attack_cd = (monster.attack_cd - dt).max(0.0);
            monster.wobble += dt * 5.0;
            monster.hit_flash = (monster.hit_flash - dt).max(0.0);
            monster.chill_ttl = (monster.chill_ttl - dt).max(0.0);
            monster.hit_offset *= 0.0003_f32.powf(dt);
            let to_player = player_pos - monster.pos;
            let distance = to_player.length();
            if distance < 26.0 && monster.attack_cd <= 0.0 {
                attacks.push(index);
                monster.attack_cd = monster.kind.attack_cooldown();
                continue;
            }
            if distance < PASSIVE_AGGRO_RADIUS {
                let chill_factor = if monster.chill_ttl > 0.0 { 0.55 } else { 1.0 };
                monster.vel =
                    to_player.normalize_or_zero() * monster.kind.move_speed() * chill_factor;
            } else {
                monster.vel *= 0.88;
            }
        }

        for index in 0..self.sim.monsters.len() {
            let delta = self.sim.monsters[index].vel * dt;
            let next_x = self.sim.monsters[index].pos + vec2(delta.x, 0.0);
            if !self.world.collides_circle(next_x, MONSTER_RADIUS)
                && self.world.biome_level(next_x) > 0
            {
                self.sim.monsters[index].pos.x = next_x.x;
            }
            let next_y = self.sim.monsters[index].pos + vec2(0.0, delta.y);
            if !self.world.collides_circle(next_y, MONSTER_RADIUS)
                && self.world.biome_level(next_y) > 0
            {
                self.sim.monsters[index].pos.y = next_y.y;
            }
        }

        for index in attacks {
            if index >= self.sim.monsters.len() {
                continue;
            }
            let raw = monster_damage(
                self.sim.monsters[index].kind,
                self.sim.monsters[index].level,
                self.sim.monsters[index].rank,
            ) + self.runtime.rng.random_range(-2.0..=3.0);
            let damage = (raw - self.sim.player.armor() as f32).max(1.0);
            if damage < raw {
                self.award_discipline_xp(DisciplineKind::Armor, 2);
            }
            self.sim.player.hp -= damage;
            self.emit(GameplayEvent::PlayerHit {
                pos: self.sim.player.pos,
                damage,
                attacker_name: self.sim.monsters[index].display_name(),
            });
            if self.sim.player.hp <= 0.0 {
                self.sim.player.hp = self.sim.player.max_hp();
                self.sim.player.pos = World::tile_center(ivec2(0, 0));
                self.sim.player.stats.gold = (self.sim.player.stats.gold as f32 * 0.8) as i32;
                self.log("You wake at the town well, lighter in coin and pride.".into());
            }
        }
    }

    pub(super) fn roll_player_damage(&mut self, kind: DamageKind) -> f32 {
        let crit = self
            .runtime
            .rng
            .random_bool(self.sim.player.crit_chance() as f64);
        let base = self.sim.player.power() as f32 + self.runtime.rng.random_range(3.0..=8.0);
        let discipline_bonus = match kind {
            DamageKind::Basic | DamageKind::PhysicalSkill => self.sim.player.melee_damage_bonus(),
            DamageKind::MagicSkill => self.sim.player.magic_damage_bonus(),
        } as f32;
        let skill_bonus = match kind {
            DamageKind::Basic => 0.0,
            DamageKind::PhysicalSkill | DamageKind::MagicSkill => 4.0,
        };
        if crit {
            self.fx.floating.push(FloatingText {
                pos: self.sim.player.pos + vec2(0.0, -18.0),
                text: "CRIT!".into(),
                color: Color::from_rgba(255, 224, 96, 255),
                ttl: 0.72,
            });
            (base + discipline_bonus + skill_bonus) * 2.0
        } else {
            base + discipline_bonus + skill_bonus
        }
    }

    pub(super) fn spend_ability_cost(&mut self, ability: AbilityKind) {
        self.sim.player.mana -= ability.mana_cost();
        self.sim.player.ability_cooldowns[ability.index()] = ability.cooldown();
    }

    pub(super) fn hit_monster(&mut self, index: usize, damage: f32, flashy: bool) {
        if index >= self.sim.monsters.len() {
            return;
        }
        let monster_pos = self.sim.monsters[index].pos;
        let monster_name = self.sim.monsters[index].display_name();
        self.sim.monsters[index].hp -= damage;
        self.sim.monsters[index].hit_flash = 0.12;
        self.sim.monsters[index].hit_offset = (monster_pos - self.sim.player.pos)
            .normalize_or_zero()
            * if flashy { 10.0 } else { 6.0 };
        self.emit(GameplayEvent::MonsterHit {
            pos: monster_pos,
            damage,
            flashy,
            display_name: monster_name,
        });
        if self.sim.monsters[index].hp <= 0.0 {
            let monster = self.sim.monsters.remove(index);
            self.on_monster_killed(monster);
        }
    }

    pub(super) fn on_monster_killed(&mut self, monster: Monster) {
        self.on_quest_monster_killed(&monster);
        let xp = monster_xp(monster.kind, monster.level, monster.rank);
        let (gold_min, gold_max) = monster.rank.gold_roll_bounds();
        self.grant_player_xp(xp);
        self.sim.player.stats.gold += self.runtime.rng.random_range(gold_min..=gold_max);
        self.emit(GameplayEvent::MonsterKilled {
            pos: monster.pos,
            color: monster.kind.color(),
            display_name: monster.display_name(),
            xp,
        });
        if self.runtime.rng.random_bool(monster.rank.drop_chance()) {
            let item = roll_item(&mut self.runtime.rng, monster.level);
            self.emit(GameplayEvent::LootDropped {
                display_name: monster.display_name(),
                item_name: item.name.clone(),
            });
            self.sim.loot.push(Loot {
                pos: monster.pos,
                item,
                bob: self.runtime.rng.random_range(0.0..10.0),
            });
        }
    }

    pub(super) fn spawn_particles(&mut self, pos: Vec2, count: usize, color: Color) {
        for _ in 0..count {
            let angle = self.runtime.rng.random_range(0.0..std::f32::consts::TAU);
            let speed = self.runtime.rng.random_range(28.0..=130.0);
            self.fx.particles.push(Particle {
                pos,
                vel: vec2(angle.cos(), angle.sin()) * speed,
                color,
                ttl: self.runtime.rng.random_range(0.22..=0.62),
                radius: self.runtime.rng.random_range(1.5..=4.0),
            });
        }
    }
}
