use macroquad::prelude::*;

use super::{AbilityKind, DisciplineKind, FloatingText, Game, Notification, SkillXpToast};

pub(super) enum GameplayEvent {
    DisciplineXpGained {
        kind: DisciplineKind,
        amount: i32,
    },
    DisciplineLeveled {
        kind: DisciplineKind,
        level: i32,
    },
    AbilityUnlocked {
        kind: DisciplineKind,
        ability: AbilityKind,
    },
    MonsterHit {
        pos: Vec2,
        damage: f32,
        flashy: bool,
        display_name: String,
    },
    PlayerHit {
        pos: Vec2,
        damage: f32,
        attacker_name: String,
    },
    MonsterKilled {
        pos: Vec2,
        color: Color,
        display_name: String,
        xp: i32,
    },
    PlayerLeveled {
        pos: Vec2,
        level: i32,
    },
    LootPickedUp {
        pos: Vec2,
        color: Color,
        name: String,
        summary: String,
    },
    LootDropped {
        display_name: String,
        item_name: String,
    },
}

impl Game {
    pub(super) fn emit(&mut self, event: GameplayEvent) {
        match event {
            GameplayEvent::DisciplineXpGained { kind, amount } => {
                self.push_skill_xp_toast(kind, amount);
            }
            GameplayEvent::DisciplineLeveled { kind, level } => {
                self.fx.notifications.push(Notification {
                    text: format!("{} reaches level {}", kind.name(), level),
                    color: kind.color(),
                    ttl: 2.2,
                });
                self.log(format!("{} mastery reaches level {}.", kind.name(), level));
            }
            GameplayEvent::AbilityUnlocked { kind, ability } => {
                self.fx.notifications.push(Notification {
                    text: format!("Unlocked {}", ability.name()),
                    color: kind.color(),
                    ttl: 2.6,
                });
                self.log(format!("{} unlocks {}.", kind.name(), ability.name()));
            }
            GameplayEvent::MonsterHit {
                pos,
                damage,
                flashy,
                display_name,
            } => {
                self.fx.floating.push(FloatingText {
                    pos,
                    text: format!("-{}", damage.round() as i32),
                    color: if flashy {
                        Color::from_rgba(255, 112, 236, 255)
                    } else {
                        WHITE
                    },
                    ttl: 0.84,
                });
                self.spawn_particles(
                    pos,
                    if flashy { 12 } else { 6 },
                    if flashy {
                        Color::from_rgba(255, 224, 96, 255)
                    } else {
                        Color::from_rgba(255, 180, 120, 255)
                    },
                );
                self.fx.screen_shake = self.fx.screen_shake.max(if flashy { 7.0 } else { 4.0 });
                self.log(format!(
                    "You hit {} for {}.",
                    display_name,
                    damage.round() as i32
                ));
            }
            GameplayEvent::PlayerHit {
                pos,
                damage,
                attacker_name,
            } => {
                self.fx.floating.push(FloatingText {
                    pos,
                    text: format!("-{}", damage.round() as i32),
                    color: Color::from_rgba(255, 100, 100, 255),
                    ttl: 0.85,
                });
                self.fx.screen_shake = self.fx.screen_shake.max(8.0);
                self.log(format!(
                    "{} bites for {}.",
                    attacker_name,
                    damage.round() as i32
                ));
            }
            GameplayEvent::MonsterKilled {
                pos,
                color,
                display_name,
                xp,
            } => {
                self.fx.pulses.push(super::Pulse {
                    pos,
                    radius: 10.0,
                    ttl: 0.24,
                    color,
                });
                self.fx.floating.push(FloatingText {
                    pos,
                    text: format!("+{} xp", xp),
                    color: Color::from_rgba(122, 236, 126, 255),
                    ttl: 1.05,
                });
                self.spawn_particles(pos, 18, color);
                self.log(format!("{display_name} pops. +{xp} xp."));
            }
            GameplayEvent::PlayerLeveled { pos, level } => {
                self.fx.floating.push(FloatingText {
                    pos,
                    text: format!("LEVEL {level}"),
                    color: Color::from_rgba(255, 224, 96, 255),
                    ttl: 1.35,
                });
                self.spawn_particles(pos, 30, Color::from_rgba(255, 224, 96, 255));
            }
            GameplayEvent::LootPickedUp {
                pos,
                color,
                name,
                summary,
            } => {
                self.fx.floating.push(FloatingText {
                    pos,
                    text: "LOOT".into(),
                    color,
                    ttl: 0.9,
                });
                self.log(format!("Picked up {name} [{summary}]."));
            }
            GameplayEvent::LootDropped {
                display_name,
                item_name,
            } => {
                self.log(format!("{display_name} drops {item_name}."));
            }
        }
    }

    fn push_skill_xp_toast(&mut self, kind: DisciplineKind, amount: i32) {
        if let Some(toast) = self
            .fx
            .skill_xp_toasts
            .iter_mut()
            .find(|toast| toast.kind == kind)
        {
            toast.amount += amount;
            toast.ttl = super::progression::SKILL_XP_TOAST_TTL;
            return;
        }
        self.fx.skill_xp_toasts.push(SkillXpToast {
            kind,
            amount,
            ttl: super::progression::SKILL_XP_TOAST_TTL,
        });
    }
}
