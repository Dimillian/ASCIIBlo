use ::rand::RngExt;
use macroquad::prelude::*;

use crate::{
    content::{monster_damage, monster_xp, roll_item},
    world::World,
};

use super::{
    AbilityKind, DisciplineKind, FloatingText, Game, Loot, MONSTER_RADIUS, MeteorStrike, Monster,
    PASSIVE_AGGRO_RADIUS, PLAYER_RADIUS, Particle, Projectile, Pulse, SlashArc,
};

#[derive(Clone, Copy)]
enum DamageKind {
    Basic,
    PhysicalSkill,
    MagicSkill,
}

impl Game {
    pub(super) fn cast_ability(&mut self, ability: AbilityKind) {
        if !self.player.is_ability_unlocked(ability)
            || self.player.ability_cooldowns[ability.index()] > 0.0
            || self.player.mana < ability.mana_cost()
        {
            return;
        }

        match ability {
            AbilityKind::Cleave => self.cast_cleave(),
            AbilityKind::Rush => self.cast_rush(),
            AbilityKind::Whirlwind => self.cast_whirlwind(),
            AbilityKind::Execute => self.cast_execute(),
            AbilityKind::Fireball => self.cast_fireball(),
            AbilityKind::Nova => self.cast_nova(),
            AbilityKind::IceBolt => self.cast_ice_bolt(),
            AbilityKind::Meteor => self.cast_meteor(),
        }
    }

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
            let damage = self.roll_player_damage(DamageKind::Basic);
            self.hit_monster(index, damage, false);
            self.award_discipline_xp(DisciplineKind::Melee, 2);
        } else {
            self.log("You carve only air.".into());
        }
        self.player.attack_cd = self.player.attack_interval();
    }

    pub(super) fn cast_rush(&mut self) {
        self.spend_ability_cost(AbilityKind::Rush);
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
        let landed = !hits.is_empty();
        for index in hits.into_iter().rev() {
            let damage = self.roll_player_damage(DamageKind::PhysicalSkill) + 6.0;
            self.hit_monster(index, damage, true);
        }
        if landed {
            self.award_discipline_xp(DisciplineKind::Melee, 4);
        }
    }

    pub(super) fn cast_nova(&mut self) {
        self.spend_ability_cost(AbilityKind::Nova);
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
        let landed = !hits.is_empty();
        for index in hits.into_iter().rev() {
            let damage = self.roll_player_damage(DamageKind::MagicSkill) + 3.0;
            self.hit_monster(index, damage, true);
        }
        if landed {
            self.award_discipline_xp(DisciplineKind::Magic, 4);
        }
    }

    pub(super) fn cast_fireball(&mut self) {
        self.spend_ability_cost(AbilityKind::Fireball);
        let direction = self.player.facing.normalize_or_zero();
        let damage = self.roll_player_damage(DamageKind::MagicSkill) + 8.0;
        self.projectiles.push(Projectile {
            ability: AbilityKind::Fireball,
            pos: self.player.pos + direction * 20.0,
            vel: direction * 320.0,
            ttl: 0.95,
            radius: 7.0,
            damage,
            aoe_radius: 34.0,
            color: Color::from_rgba(255, 132, 64, 255),
        });
        self.spawn_particles(
            self.player.pos + direction * 18.0,
            6,
            Color::from_rgba(255, 132, 64, 255),
        );
    }

    pub(super) fn cast_cleave(&mut self) {
        self.spend_ability_cost(AbilityKind::Cleave);
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
        let landed = !hits.is_empty();
        for index in hits.into_iter().rev() {
            let damage = self.roll_player_damage(DamageKind::PhysicalSkill) + 5.0;
            self.hit_monster(index, damage, true);
        }
        if landed {
            self.award_discipline_xp(DisciplineKind::Melee, 4);
        }
    }

    pub(super) fn cast_whirlwind(&mut self) {
        self.spend_ability_cost(AbilityKind::Whirlwind);
        self.pulses.push(Pulse {
            pos: self.player.pos,
            radius: 18.0,
            ttl: 0.38,
            color: Color::from_rgba(255, 176, 88, 255),
        });
        self.spawn_particles(self.player.pos, 20, Color::from_rgba(255, 176, 88, 255));
        let hits: Vec<usize> = self
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                (monster.pos.distance(self.player.pos) <= 76.0).then_some(index)
            })
            .collect();
        if hits.is_empty() {
            self.log("Whirlwind spins up only dust.".into());
        }
        let landed = !hits.is_empty();
        for index in hits.into_iter().rev() {
            let damage = self.roll_player_damage(DamageKind::PhysicalSkill) + 7.0;
            self.hit_monster(index, damage, true);
        }
        if landed {
            self.award_discipline_xp(DisciplineKind::Melee, 5);
        }
    }

    pub(super) fn cast_execute(&mut self) {
        self.spend_ability_cost(AbilityKind::Execute);
        let direction = self.player.facing.normalize_or_zero();
        let target = self
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                let to_monster = monster.pos - self.player.pos;
                let distance = to_monster.length();
                let aligned = to_monster.normalize_or_zero().dot(direction);
                (distance <= 64.0 && aligned > 0.45).then_some((index, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index);
        if let Some(index) = target {
            let wounded_bonus = if self.monsters[index].hp <= self.monsters[index].max_hp * 0.5 {
                12.0
            } else {
                0.0
            };
            let damage = self.roll_player_damage(DamageKind::PhysicalSkill) + 10.0 + wounded_bonus;
            self.hit_monster(index, damage, true);
            self.award_discipline_xp(DisciplineKind::Melee, 5);
        } else {
            self.log("Execute finds no opening.".into());
        }
    }

    pub(super) fn cast_ice_bolt(&mut self) {
        self.spend_ability_cost(AbilityKind::IceBolt);
        let direction = self.player.facing.normalize_or_zero();
        let damage = self.roll_player_damage(DamageKind::MagicSkill) + 6.0;
        self.projectiles.push(Projectile {
            ability: AbilityKind::IceBolt,
            pos: self.player.pos + direction * 20.0,
            vel: direction * 380.0,
            ttl: 0.8,
            radius: 6.0,
            damage,
            aoe_radius: 0.0,
            color: Color::from_rgba(128, 214, 255, 255),
        });
    }

    pub(super) fn cast_meteor(&mut self) {
        self.spend_ability_cost(AbilityKind::Meteor);
        let pos = self.input.aim_world;
        let damage = self.roll_player_damage(DamageKind::MagicSkill) + 14.0;
        self.meteors.push(MeteorStrike {
            pos,
            ttl: 0.72,
            damage,
            radius: 62.0,
        });
        self.log("The air buckles above the target.".into());
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
            match projectile.ability {
                AbilityKind::Fireball => self.detonate_fireball(projectile),
                AbilityKind::IceBolt => self.impact_ice_bolt(projectile),
                _ => {}
            }
        }
    }

    pub(super) fn update_meteors(&mut self, dt: f32) {
        let mut pending = Vec::with_capacity(self.meteors.len());
        let mut impacts = Vec::new();
        for mut meteor in self.meteors.drain(..) {
            meteor.ttl -= dt;
            if meteor.ttl <= 0.0 {
                impacts.push(meteor);
            } else {
                pending.push(meteor);
            }
        }
        self.meteors = pending;
        for meteor in impacts {
            self.impact_meteor(meteor);
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
            .monsters
            .iter()
            .enumerate()
            .find(|(_, monster)| monster.pos.distance(projectile.pos) <= projectile.radius + 12.0)
            .map(|(index, _)| index);
        if let Some(index) = target {
            self.monsters[index].chill_ttl = self.monsters[index].chill_ttl.max(1.8);
            self.hit_monster(index, projectile.damage, true);
            self.award_discipline_xp(DisciplineKind::Magic, 4);
        } else {
            self.log("Ice Bolt cracks against the ground.".into());
        }
    }

    fn impact_meteor(&mut self, meteor: MeteorStrike) {
        self.pulses.push(Pulse {
            pos: meteor.pos,
            radius: 20.0,
            ttl: 0.48,
            color: Color::from_rgba(255, 132, 64, 255),
        });
        self.spawn_particles(meteor.pos, 28, Color::from_rgba(255, 132, 64, 255));
        let hits: Vec<usize> = self
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
        let player_pos = self.player.pos;
        let mut attacks = Vec::new();
        for index in 0..self.monsters.len() {
            let monster = &mut self.monsters[index];
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
            let raw = monster_damage(
                self.monsters[index].kind,
                self.monsters[index].level,
                self.monsters[index].rank,
            ) + self.rng.random_range(-2.0..=3.0);
            let damage = (raw - self.player.armor() as f32).max(1.0);
            if damage < raw {
                self.award_discipline_xp(DisciplineKind::Armor, 2);
            }
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
                self.monsters[index].display_name(),
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

    fn roll_player_damage(&mut self, kind: DamageKind) -> f32 {
        let crit = self.rng.random_bool(self.player.crit_chance() as f64);
        let base = self.player.power() as f32 + self.rng.random_range(3.0..=8.0);
        let discipline_bonus = match kind {
            DamageKind::Basic | DamageKind::PhysicalSkill => self.player.melee_damage_bonus(),
            DamageKind::MagicSkill => self.player.magic_damage_bonus(),
        } as f32;
        let skill_bonus = match kind {
            DamageKind::Basic => 0.0,
            DamageKind::PhysicalSkill | DamageKind::MagicSkill => 4.0,
        };
        if crit {
            self.floating.push(FloatingText {
                pos: self.player.pos + vec2(0.0, -18.0),
                text: "CRIT!".into(),
                color: Color::from_rgba(255, 224, 96, 255),
                ttl: 0.72,
            });
            (base + discipline_bonus + skill_bonus) * 2.0
        } else {
            base + discipline_bonus + skill_bonus
        }
    }

    fn spend_ability_cost(&mut self, ability: AbilityKind) {
        self.player.mana -= ability.mana_cost();
        self.player.ability_cooldowns[ability.index()] = ability.cooldown();
    }

    fn hit_monster(&mut self, index: usize, damage: f32, flashy: bool) {
        if index >= self.monsters.len() {
            return;
        }
        let monster_pos = self.monsters[index].pos;
        let monster_name = self.monsters[index].display_name();
        self.monsters[index].hp -= damage;
        self.monsters[index].hit_flash = 0.12;
        self.monsters[index].hit_offset =
            (monster_pos - self.player.pos).normalize_or_zero() * if flashy { 10.0 } else { 6.0 };
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
        let xp = monster_xp(monster.kind, monster.level, monster.rank);
        let (gold_min, gold_max) = monster.rank.gold_roll_bounds();
        self.player.stats.xp += xp;
        self.player.stats.gold += self.rng.random_range(gold_min..=gold_max);
        self.pulses.push(Pulse {
            pos: monster.pos,
            radius: 10.0,
            ttl: 0.24,
            color: monster.kind.color(),
        });
        self.floating.push(FloatingText {
            pos: monster.pos,
            text: format!("+{} xp", xp),
            color: Color::from_rgba(122, 236, 126, 255),
            ttl: 1.05,
        });
        self.spawn_particles(monster.pos, 18, monster.kind.color());
        self.log(format!("{} pops. +{} xp.", monster.display_name(), xp));
        if self.rng.random_bool(monster.rank.drop_chance()) {
            let item = roll_item(&mut self.rng, monster.level);
            self.log(format!("{} drops {}.", monster.display_name(), item.name));
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
