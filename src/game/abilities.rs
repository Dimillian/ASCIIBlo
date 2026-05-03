use macroquad::prelude::*;

use super::{
    AbilityKind, DisciplineKind, Game, MeteorStrike, PLAYER_RADIUS, Projectile, Pulse, SlashArc,
    combat::DamageKind,
};

impl Game {
    pub(super) fn cast_ability(&mut self, ability: AbilityKind) {
        if !self.sim.player.is_ability_unlocked(ability)
            || self.sim.player.ability_cooldowns[ability.index()] > 0.0
            || self.sim.player.mana < ability.mana_cost()
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

    fn cast_rush(&mut self) {
        self.spend_ability_cost(AbilityKind::Rush);
        let direction = self.sim.player.facing;
        let mut travelled = Vec2::ZERO;
        for _ in 0..10 {
            let step = direction * 16.0;
            let next = self.sim.player.pos + travelled + step;
            if self.world.collides_circle(next, PLAYER_RADIUS) {
                break;
            }
            travelled += step;
        }
        self.sim.player.pos += travelled;
        self.fx.pulses.push(Pulse {
            pos: self.sim.player.pos,
            radius: 18.0,
            ttl: 0.25,
            color: Color::from_rgba(112, 180, 255, 255),
        });
        self.spawn_particles(
            self.sim.player.pos,
            10,
            Color::from_rgba(112, 180, 255, 255),
        );

        let hits: Vec<usize> = self
            .sim
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                (monster.pos.distance(self.sim.player.pos) <= 42.0).then_some(index)
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

    fn cast_nova(&mut self) {
        self.spend_ability_cost(AbilityKind::Nova);
        self.fx.pulses.push(Pulse {
            pos: self.sim.player.pos,
            radius: 18.0,
            ttl: 0.42,
            color: Color::from_rgba(255, 112, 236, 255),
        });
        self.spawn_particles(
            self.sim.player.pos,
            18,
            Color::from_rgba(255, 112, 236, 255),
        );
        let hits: Vec<usize> = self
            .sim
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                (monster.pos.distance(self.sim.player.pos) <= 92.0).then_some(index)
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

    fn cast_fireball(&mut self) {
        self.spend_ability_cost(AbilityKind::Fireball);
        let direction = self.sim.player.facing.normalize_or_zero();
        let damage = self.roll_player_damage(DamageKind::MagicSkill) + 8.0;
        self.fx.projectiles.push(Projectile {
            ability: AbilityKind::Fireball,
            pos: self.sim.player.pos + direction * 20.0,
            vel: direction * 320.0,
            ttl: 0.95,
            radius: 7.0,
            damage,
            aoe_radius: 34.0,
            color: Color::from_rgba(255, 132, 64, 255),
        });
        self.spawn_particles(
            self.sim.player.pos + direction * 18.0,
            6,
            Color::from_rgba(255, 132, 64, 255),
        );
    }

    fn cast_cleave(&mut self) {
        self.spend_ability_cost(AbilityKind::Cleave);
        self.fx.pulses.push(Pulse {
            pos: self.sim.player.pos,
            radius: 18.0,
            ttl: 0.3,
            color: Color::from_rgba(255, 176, 88, 255),
        });
        self.fx.slash_arcs.push(SlashArc {
            pos: self.sim.player.pos,
            direction: self.sim.player.facing.normalize_or_zero(),
            radius: 48.0,
            ttl: 0.28,
            color: Color::from_rgba(255, 176, 88, 255),
        });
        self.spawn_particles(self.sim.player.pos, 14, Color::from_rgba(255, 176, 88, 255));

        let direction = self.sim.player.facing.normalize_or_zero();
        let hits: Vec<usize> = self
            .sim
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                let to_monster = monster.pos - self.sim.player.pos;
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

    fn cast_whirlwind(&mut self) {
        self.spend_ability_cost(AbilityKind::Whirlwind);
        self.fx.pulses.push(Pulse {
            pos: self.sim.player.pos,
            radius: 18.0,
            ttl: 0.38,
            color: Color::from_rgba(255, 176, 88, 255),
        });
        self.spawn_particles(self.sim.player.pos, 20, Color::from_rgba(255, 176, 88, 255));
        let hits: Vec<usize> = self
            .sim
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                (monster.pos.distance(self.sim.player.pos) <= 76.0).then_some(index)
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

    fn cast_execute(&mut self) {
        self.spend_ability_cost(AbilityKind::Execute);
        let direction = self.sim.player.facing.normalize_or_zero();
        let target = self
            .sim
            .monsters
            .iter()
            .enumerate()
            .filter_map(|(index, monster)| {
                let to_monster = monster.pos - self.sim.player.pos;
                let distance = to_monster.length();
                let aligned = to_monster.normalize_or_zero().dot(direction);
                (distance <= 64.0 && aligned > 0.45).then_some((index, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index);
        if let Some(index) = target {
            let wounded_bonus =
                if self.sim.monsters[index].hp <= self.sim.monsters[index].max_hp * 0.5 {
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

    fn cast_ice_bolt(&mut self) {
        self.spend_ability_cost(AbilityKind::IceBolt);
        let direction = self.sim.player.facing.normalize_or_zero();
        let damage = self.roll_player_damage(DamageKind::MagicSkill) + 6.0;
        self.fx.projectiles.push(Projectile {
            ability: AbilityKind::IceBolt,
            pos: self.sim.player.pos + direction * 20.0,
            vel: direction * 380.0,
            ttl: 0.8,
            radius: 6.0,
            damage,
            aoe_radius: 0.0,
            color: Color::from_rgba(128, 214, 255, 255),
        });
    }

    fn cast_meteor(&mut self) {
        self.spend_ability_cost(AbilityKind::Meteor);
        let pos = self.runtime.input.aim_world;
        let damage = self.roll_player_damage(DamageKind::MagicSkill) + 14.0;
        self.fx.meteors.push(MeteorStrike {
            pos,
            ttl: 0.72,
            damage,
            radius: 62.0,
        });
        self.log("The air buckles above the target.".into());
    }
}
