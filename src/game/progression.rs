use super::{AbilityKind, DisciplineKind, Game, Notification, SkillXpToast};

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

        self.push_skill_xp_toast(kind, amount);
        self.player.disciplines.get_mut(kind).xp += amount;

        loop {
            let Some(level) = ({
                let progress = self.player.disciplines.get_mut(kind);
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

            self.notifications.push(Notification {
                text: format!("{} reaches level {}", kind.name(), level),
                color: kind.color(),
                ttl: 2.2,
            });
            self.log(format!("{} mastery reaches level {}.", kind.name(), level));

            for ability in unlocked_abilities(kind, level) {
                self.notifications.push(Notification {
                    text: format!("Unlocked {}", ability.name()),
                    color: kind.color(),
                    ttl: 2.6,
                });
                self.log(format!("{} unlocks {}.", kind.name(), ability.name()));
            }
        }
    }

    pub(super) fn award_agility_distance(&mut self, distance: f32) {
        if distance <= 0.0 {
            return;
        }
        self.agility_distance_bank += distance;
        while self.agility_distance_bank >= AGILITY_XP_DISTANCE {
            self.agility_distance_bank -= AGILITY_XP_DISTANCE;
            self.award_discipline_xp(DisciplineKind::Agility, 1);
        }
    }

    fn push_skill_xp_toast(&mut self, kind: DisciplineKind, amount: i32) {
        if let Some(toast) = self
            .skill_xp_toasts
            .iter_mut()
            .find(|toast| toast.kind == kind)
        {
            toast.amount += amount;
            toast.ttl = SKILL_XP_TOAST_TTL;
            return;
        }
        self.skill_xp_toasts.push(SkillXpToast {
            kind,
            amount,
            ttl: SKILL_XP_TOAST_TTL,
        });
    }
}

pub(super) fn unlocked_abilities(kind: DisciplineKind, level: i32) -> Vec<AbilityKind> {
    AbilityKind::ALL
        .into_iter()
        .filter(|ability| ability.discipline() == kind && ability.unlock_level() == level)
        .collect()
}
