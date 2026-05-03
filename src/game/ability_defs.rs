use macroquad::prelude::*;

use super::{AbilityKind, DisciplineKind};

pub struct AbilityDef {
    pub name: &'static str,
    pub glyph: &'static str,
    pub discipline: DisciplineKind,
    pub unlock_level: i32,
    pub mana_cost: f32,
    pub cooldown: f32,
    pub summary: &'static str,
}

pub(super) const ABILITY_DEFS: [AbilityDef; 8] = [
    AbilityDef {
        name: "Cleave",
        glyph: "/",
        discipline: DisciplineKind::Melee,
        unlock_level: 1,
        mana_cost: 10.0,
        cooldown: 2.2,
        summary: "Sweep a broad melee arc in front of you.",
    },
    AbilityDef {
        name: "Rush",
        glyph: ">",
        discipline: DisciplineKind::Melee,
        unlock_level: 2,
        mana_cost: 8.0,
        cooldown: 1.8,
        summary: "Dash forward and strike enemies reached by the rush.",
    },
    AbilityDef {
        name: "Whirlwind",
        glyph: "@",
        discipline: DisciplineKind::Melee,
        unlock_level: 4,
        mana_cost: 16.0,
        cooldown: 3.0,
        summary: "Spin through nearby enemies in every direction.",
    },
    AbilityDef {
        name: "Execute",
        glyph: "!",
        discipline: DisciplineKind::Melee,
        unlock_level: 8,
        mana_cost: 14.0,
        cooldown: 2.6,
        summary: "Deliver a crushing frontal hit that punishes wounded foes.",
    },
    AbilityDef {
        name: "Fireball",
        glyph: "*",
        discipline: DisciplineKind::Magic,
        unlock_level: 1,
        mana_cost: 12.0,
        cooldown: 1.2,
        summary: "Launch a fireball that explodes on impact.",
    },
    AbilityDef {
        name: "Nova",
        glyph: "O",
        discipline: DisciplineKind::Magic,
        unlock_level: 2,
        mana_cost: 14.0,
        cooldown: 3.5,
        summary: "Release a close-range burst around yourself.",
    },
    AbilityDef {
        name: "Ice Bolt",
        glyph: "-",
        discipline: DisciplineKind::Magic,
        unlock_level: 4,
        mana_cost: 10.0,
        cooldown: 1.0,
        summary: "Fire a fast bolt that chills the first enemy it hits.",
    },
    AbilityDef {
        name: "Meteor",
        glyph: "v",
        discipline: DisciplineKind::Magic,
        unlock_level: 8,
        mana_cost: 22.0,
        cooldown: 4.0,
        summary: "Call down a delayed blast at the aimed position.",
    },
];

pub(super) fn ability_def(kind: AbilityKind) -> &'static AbilityDef {
    &ABILITY_DEFS[kind.index()]
}

pub(super) fn ability_color(kind: AbilityKind) -> Color {
    ability_def(kind).discipline.color()
}
