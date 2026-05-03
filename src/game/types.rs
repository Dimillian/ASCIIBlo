use macroquad::prelude::*;

use crate::content::{Item, MonsterKind, MonsterRank, NpcKind};

pub struct Stats {
    pub level: i32,
    pub xp: i32,
    pub next_xp: i32,
    pub strength: i32,
    pub agility: i32,
    pub vitality: i32,
    pub gold: i32,
    pub unspent_stat_points: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisciplineKind {
    Melee,
    Magic,
    Armor,
    Agility,
}

impl DisciplineKind {
    pub const ALL: [Self; 4] = [Self::Melee, Self::Magic, Self::Armor, Self::Agility];

    pub fn name(self) -> &'static str {
        match self {
            Self::Melee => "Melee",
            Self::Magic => "Magic",
            Self::Armor => "Armor",
            Self::Agility => "Agility",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Melee => Color::from_rgba(255, 176, 88, 255),
            Self::Magic => Color::from_rgba(255, 112, 236, 255),
            Self::Armor => Color::from_rgba(128, 214, 255, 255),
            Self::Agility => Color::from_rgba(130, 236, 126, 255),
        }
    }
}

#[derive(Clone, Copy)]
pub struct DisciplineProgress {
    pub level: i32,
    pub xp: i32,
    pub next_xp: i32,
}

pub struct Disciplines {
    pub melee: DisciplineProgress,
    pub magic: DisciplineProgress,
    pub armor: DisciplineProgress,
    pub agility: DisciplineProgress,
}

impl Disciplines {
    pub fn get(&self, kind: DisciplineKind) -> &DisciplineProgress {
        match kind {
            DisciplineKind::Melee => &self.melee,
            DisciplineKind::Magic => &self.magic,
            DisciplineKind::Armor => &self.armor,
            DisciplineKind::Agility => &self.agility,
        }
    }

    pub fn get_mut(&mut self, kind: DisciplineKind) -> &mut DisciplineProgress {
        match kind {
            DisciplineKind::Melee => &mut self.melee,
            DisciplineKind::Magic => &mut self.magic,
            DisciplineKind::Armor => &mut self.armor,
            DisciplineKind::Agility => &mut self.agility,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbilityKind {
    Cleave,
    Rush,
    Whirlwind,
    Execute,
    Fireball,
    Nova,
    IceBolt,
    Meteor,
}

impl AbilityKind {
    pub const ALL: [Self; 8] = [
        Self::Cleave,
        Self::Rush,
        Self::Whirlwind,
        Self::Execute,
        Self::Fireball,
        Self::Nova,
        Self::IceBolt,
        Self::Meteor,
    ];
    pub const MELEE: [Self; 4] = [Self::Cleave, Self::Rush, Self::Whirlwind, Self::Execute];
    pub const MAGIC: [Self; 4] = [Self::Fireball, Self::Nova, Self::IceBolt, Self::Meteor];

    pub fn index(self) -> usize {
        match self {
            Self::Cleave => 0,
            Self::Rush => 1,
            Self::Whirlwind => 2,
            Self::Execute => 3,
            Self::Fireball => 4,
            Self::Nova => 5,
            Self::IceBolt => 6,
            Self::Meteor => 7,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Cleave => "Cleave",
            Self::Rush => "Rush",
            Self::Whirlwind => "Whirlwind",
            Self::Execute => "Execute",
            Self::Fireball => "Fireball",
            Self::Nova => "Nova",
            Self::IceBolt => "Ice Bolt",
            Self::Meteor => "Meteor",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Self::Cleave => "/",
            Self::Rush => ">",
            Self::Whirlwind => "@",
            Self::Execute => "!",
            Self::Fireball => "*",
            Self::Nova => "O",
            Self::IceBolt => "-",
            Self::Meteor => "v",
        }
    }

    pub fn discipline(self) -> DisciplineKind {
        match self {
            Self::Cleave | Self::Rush | Self::Whirlwind | Self::Execute => DisciplineKind::Melee,
            Self::Fireball | Self::Nova | Self::IceBolt | Self::Meteor => DisciplineKind::Magic,
        }
    }

    pub fn unlock_level(self) -> i32 {
        match self {
            Self::Cleave | Self::Fireball => 1,
            Self::Rush | Self::Nova => 2,
            Self::Whirlwind | Self::IceBolt => 4,
            Self::Execute | Self::Meteor => 8,
        }
    }

    pub fn mana_cost(self) -> f32 {
        match self {
            Self::Cleave => 10.0,
            Self::Rush => 8.0,
            Self::Whirlwind => 16.0,
            Self::Execute => 14.0,
            Self::Fireball => 12.0,
            Self::Nova => 14.0,
            Self::IceBolt => 10.0,
            Self::Meteor => 22.0,
        }
    }

    pub fn cooldown(self) -> f32 {
        match self {
            Self::Cleave => 2.2,
            Self::Rush => 1.8,
            Self::Whirlwind => 3.0,
            Self::Execute => 2.6,
            Self::Fireball => 1.2,
            Self::Nova => 3.5,
            Self::IceBolt => 1.0,
            Self::Meteor => 4.0,
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Self::Cleave => "Sweep a broad melee arc in front of you.",
            Self::Rush => "Dash forward and strike enemies reached by the rush.",
            Self::Whirlwind => "Spin through nearby enemies in every direction.",
            Self::Execute => "Deliver a crushing frontal hit that punishes wounded foes.",
            Self::Fireball => "Launch a fireball that explodes on impact.",
            Self::Nova => "Release a close-range burst around yourself.",
            Self::IceBolt => "Fire a fast bolt that chills the first enemy it hits.",
            Self::Meteor => "Call down a delayed blast at the aimed position.",
        }
    }

    pub fn color(self) -> Color {
        self.discipline().color()
    }
}

pub fn abilities_for_discipline(kind: DisciplineKind) -> &'static [AbilityKind] {
    match kind {
        DisciplineKind::Melee => &AbilityKind::MELEE,
        DisciplineKind::Magic => &AbilityKind::MAGIC,
        DisciplineKind::Armor | DisciplineKind::Agility => &[],
    }
}

pub struct Equipment {
    pub weapon: Option<Item>,
    pub armor: Option<Item>,
    pub charm: Option<Item>,
}

impl Equipment {
    pub fn bonus_power(&self) -> i32 {
        self.iter().map(|item| item.power).sum()
    }

    pub fn bonus_armor(&self) -> i32 {
        self.iter().map(|item| item.armor).sum()
    }

    pub fn bonus_vitality(&self) -> i32 {
        self.iter().map(|item| item.vitality).sum()
    }

    pub fn bonus_haste(&self) -> i32 {
        self.iter().map(|item| item.haste).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.weapon
            .iter()
            .chain(self.armor.iter())
            .chain(self.charm.iter())
    }
}

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: Vec2,
    pub hp: f32,
    pub mana: f32,
    pub attack_cd: f32,
    pub ability_cooldowns: [f32; 8],
    pub bound_abilities: [AbilityKind; 2],
    pub stats: Stats,
    pub inventory: Vec<Item>,
    pub equipment: Equipment,
    pub disciplines: Disciplines,
}

impl Player {
    pub fn max_hp(&self) -> f32 {
        64.0 + (self.stats.vitality + self.equipment.bonus_vitality()) as f32 * 7.0
    }

    pub fn max_mana(&self) -> f32 {
        32.0 + self.stats.level as f32 * 4.0
    }

    pub fn power(&self) -> i32 {
        self.stats.strength * 2 + self.equipment.bonus_power()
    }

    pub fn armor(&self) -> i32 {
        self.stats.vitality / 2 + self.equipment.bonus_armor() + self.armor_mastery_bonus()
    }

    pub fn haste(&self) -> i32 {
        self.stats.agility + self.equipment.bonus_haste()
    }

    pub fn crit_chance(&self) -> f32 {
        (0.08 + self.stats.agility as f32 * 0.01).min(0.35)
    }

    pub fn move_speed(&self) -> f32 {
        150.0 + self.haste() as f32 * 4.0 + self.agility_mastery_bonus() as f32
    }

    pub fn attack_interval(&self) -> f32 {
        (0.5 - self.haste() as f32 * 0.018).max(0.16)
    }

    pub fn melee_damage_bonus(&self) -> i32 {
        (self.disciplines.melee.level - 1).max(0) * 2
    }

    pub fn magic_damage_bonus(&self) -> i32 {
        (self.disciplines.magic.level - 1).max(0) * 2
    }

    pub fn magic_regen_bonus(&self) -> f32 {
        (self.disciplines.magic.level - 1).max(0) as f32 * 0.5
    }

    pub fn mana_regen_rate(&self) -> f32 {
        3.0 + self.magic_regen_bonus()
    }

    pub fn armor_mastery_bonus(&self) -> i32 {
        (self.disciplines.armor.level - 1).max(0)
    }

    pub fn agility_mastery_bonus(&self) -> i32 {
        (self.disciplines.agility.level - 1).max(0) * 6
    }

    pub fn is_ability_unlocked(&self, ability: AbilityKind) -> bool {
        self.disciplines.get(ability.discipline()).level >= ability.unlock_level()
    }

    pub fn bound_slot(&self, ability: AbilityKind) -> Option<usize> {
        self.bound_abilities
            .iter()
            .position(|bound| *bound == ability)
    }
}

pub struct Monster {
    pub kind: MonsterKind,
    pub rank: MonsterRank,
    pub pack_id: u64,
    pub pack_center: Vec2,
    pub pos: Vec2,
    pub vel: Vec2,
    pub hit_offset: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    pub level: i32,
    pub attack_cd: f32,
    pub wobble: f32,
    pub hit_flash: f32,
    pub chill_ttl: f32,
}

impl Monster {
    pub fn display_name(&self) -> String {
        format!("{}{}", self.rank.prefix(), self.kind.name())
    }
}

pub struct Loot {
    pub pos: Vec2,
    pub item: Item,
    pub bob: f32,
}

pub struct Npc {
    pub kind: NpcKind,
    pub pos: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopTab {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiMode {
    None,
    Inventory,
    Character,
    SkillBook,
    WorldMap,
    Merchant,
    Trainer,
    Travel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillBookFocus {
    Disciplines,
    Skills,
    Detail,
}

#[derive(Clone, Copy)]
pub struct TravelDestination {
    pub name: &'static str,
    pub pos: IVec2,
    pub min_level: i32,
}

pub struct FloatingText {
    pub pos: Vec2,
    pub text: String,
    pub color: Color,
    pub ttl: f32,
}

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub ttl: f32,
    pub radius: f32,
}

pub struct Pulse {
    pub pos: Vec2,
    pub radius: f32,
    pub ttl: f32,
    pub color: Color,
}

pub struct SlashArc {
    pub pos: Vec2,
    pub direction: Vec2,
    pub radius: f32,
    pub ttl: f32,
    pub color: Color,
}

pub struct Projectile {
    pub ability: AbilityKind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub ttl: f32,
    pub radius: f32,
    pub damage: f32,
    pub aoe_radius: f32,
    pub color: Color,
}

pub struct MeteorStrike {
    pub pos: Vec2,
    pub ttl: f32,
    pub damage: f32,
    pub radius: f32,
}

pub struct WorldMapState {
    pub center_tile: Vec2,
    pub zoom: f32,
}

pub struct SkillXpToast {
    pub kind: DisciplineKind,
    pub amount: i32,
    pub ttl: f32,
}

pub struct Notification {
    pub text: String,
    pub color: Color,
    pub ttl: f32,
}
