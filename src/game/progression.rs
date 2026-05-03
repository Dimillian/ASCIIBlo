use macroquad::prelude::Color;

use super::{AbilityKind, DisciplineKind, Game, Pulse, events::GameplayEvent};

const AGILITY_XP_DISTANCE: f32 = 144.0;
pub(super) const SKILL_XP_TOAST_TTL: f32 = 2.4;

pub(super) fn discipline_next_xp(level: i32) -> i32 {
    let tier = (level - 1).max(0);
    24 + tier * tier * 16
}

impl Game {
    pub(super) fn grant_player_xp(&mut self, amount: i32) {
        if amount <= 0 {
            return;
        }
        self.sim.player.stats.xp += amount;
        while self.sim.player.stats.xp >= self.sim.player.stats.next_xp {
            self.sim.player.stats.xp -= self.sim.player.stats.next_xp;
            self.sim.player.stats.level += 1;
            self.sim.player.stats.next_xp = (self.sim.player.stats.next_xp as f32 * 1.35) as i32;
            self.sim.player.stats.strength += 1;
            self.sim.player.stats.agility += 1;
            self.sim.player.stats.vitality += 1;
            self.sim.player.stats.unspent_stat_points += 3;
            self.sim.player.hp = self.sim.player.max_hp();
            self.sim.player.mana = self.sim.player.max_mana();
            self.fx.pulses.push(Pulse {
                pos: self.sim.player.pos,
                radius: 22.0,
                ttl: 0.9,
                color: Color::from_rgba(255, 224, 96, 255),
            });
            self.emit(GameplayEvent::PlayerLeveled {
                pos: self.sim.player.pos,
                level: self.sim.player.stats.level,
            });
            self.log(format!(
                "Level {}! Everything hums louder.",
                self.sim.player.stats.level
            ));
        }
    }

    pub(super) fn award_discipline_xp(&mut self, kind: DisciplineKind, amount: i32) {
        if amount <= 0 {
            return;
        }

        self.emit(GameplayEvent::DisciplineXpGained { kind, amount });
        self.sim.player.disciplines.get_mut(kind).xp += amount;

        loop {
            let Some(level) = ({
                let progress = self.sim.player.disciplines.get_mut(kind);
                if progress.xp < progress.next_xp {
                    None
                } else {
                    progress.xp -= progress.next_xp;
                    progress.level += 1;
                    progress.next_xp = discipline_next_xp(progress.level);
                    Some(progress.level)
                }
            }) else {
                break;
            };

            self.emit(GameplayEvent::DisciplineLeveled { kind, level });

            for ability in unlocked_abilities(kind, level) {
                self.emit(GameplayEvent::AbilityUnlocked { kind, ability });
            }
        }
    }

    pub(super) fn award_agility_distance(&mut self, distance: f32) {
        if distance <= 0.0 {
            return;
        }
        self.runtime.agility_distance_bank += distance;
        while self.runtime.agility_distance_bank >= AGILITY_XP_DISTANCE {
            self.runtime.agility_distance_bank -= AGILITY_XP_DISTANCE;
            self.award_discipline_xp(DisciplineKind::Agility, 1);
        }
    }
}

pub(super) fn unlocked_abilities(kind: DisciplineKind, level: i32) -> Vec<AbilityKind> {
    AbilityKind::ALL
        .into_iter()
        .filter(|ability| ability.discipline() == kind && ability.unlock_level() == level)
        .collect()
}
