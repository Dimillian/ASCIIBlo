use super::{AbilityKind, DisciplineKind, Game, events::GameplayEvent};

const AGILITY_XP_DISTANCE: f32 = 144.0;
pub(super) const SKILL_XP_TOAST_TTL: f32 = 2.4;

pub(super) fn discipline_next_xp(level: i32) -> i32 {
    let tier = (level - 1).max(0);
    24 + tier * tier * 16
}

impl Game {
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
