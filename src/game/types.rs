use macroquad::prelude::*;

use crate::{
    content::{Item, MonsterKind, MonsterRank, NpcKind},
    world::{SettlementSite, SettlementTier},
};

use super::ability_defs::{ability_color, ability_def};

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
        ability_def(self).name
    }

    pub fn glyph(self) -> &'static str {
        ability_def(self).glyph
    }

    pub fn discipline(self) -> DisciplineKind {
        ability_def(self).discipline
    }

    pub fn unlock_level(self) -> i32 {
        ability_def(self).unlock_level
    }

    pub fn mana_cost(self) -> f32 {
        ability_def(self).mana_cost
    }

    pub fn cooldown(self) -> f32 {
        ability_def(self).cooldown
    }

    pub fn summary(self) -> &'static str {
        ability_def(self).summary
    }

    pub fn color(self) -> Color {
        ability_color(self)
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

pub const BACKPACK_WIDTH: usize = 9;
pub const BACKPACK_HEIGHT: usize = 8;

#[derive(Clone)]
pub struct BackpackEntry {
    pub item: Item,
    pub x: usize,
    pub y: usize,
}

#[derive(Default)]
pub struct Backpack {
    entries: Vec<BackpackEntry>,
}

impl Backpack {
    pub fn from_items(items: Vec<Item>) -> Self {
        let mut backpack = Self::default();
        for item in items {
            backpack
                .insert_first_fit(item)
                .expect("starter items must fit in the backpack");
        }
        backpack
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.entries.iter().map(|entry| &entry.item)
    }

    pub fn entries(&self) -> &[BackpackEntry] {
        &self.entries
    }

    pub fn entry(&self, index: usize) -> Option<&BackpackEntry> {
        self.entries.get(index)
    }

    pub fn item(&self, index: usize) -> Option<&Item> {
        self.entry(index).map(|entry| &entry.item)
    }

    pub fn entry_index_at(&self, x: usize, y: usize) -> Option<usize> {
        self.entries.iter().position(|entry| {
            let footprint = entry.item.footprint();
            x >= entry.x
                && x < entry.x + footprint.width
                && y >= entry.y
                && y < entry.y + footprint.height
        })
    }

    pub fn can_fit(&self, item: &Item) -> bool {
        self.find_first_fit(item).is_some()
    }

    pub fn insert_first_fit(&mut self, item: Item) -> Result<usize, Item> {
        let Some((x, y)) = self.find_first_fit(&item) else {
            return Err(item);
        };
        self.entries.push(BackpackEntry { item, x, y });
        self.sort_entries();
        Ok(self
            .entries
            .iter()
            .position(|entry| entry.x == x && entry.y == y)
            .expect("new backpack entry should exist"))
    }

    pub fn remove(&mut self, index: usize) -> Option<BackpackEntry> {
        if index >= self.entries.len() {
            return None;
        }
        Some(self.entries.remove(index))
    }

    pub fn restore(&mut self, entry: BackpackEntry) {
        self.entries.push(entry);
        self.sort_entries();
    }

    fn find_first_fit(&self, item: &Item) -> Option<(usize, usize)> {
        let footprint = item.footprint();
        if footprint.width > BACKPACK_WIDTH || footprint.height > BACKPACK_HEIGHT {
            return None;
        }
        for y in 0..=BACKPACK_HEIGHT - footprint.height {
            for x in 0..=BACKPACK_WIDTH - footprint.width {
                if self.fits_at(item, x, y) {
                    return Some((x, y));
                }
            }
        }
        None
    }

    fn fits_at(&self, item: &Item, x: usize, y: usize) -> bool {
        let footprint = item.footprint();
        (x..x + footprint.width).all(|cell_x| {
            (y..y + footprint.height).all(|cell_y| self.entry_index_at(cell_x, cell_y).is_none())
        })
    }

    fn sort_entries(&mut self) {
        self.entries.sort_by_key(|entry| (entry.y, entry.x));
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
    pub inventory: Backpack,
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

    pub fn move_speed_rating(&self) -> f32 {
        self.move_speed() - 70.0
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

#[derive(Clone)]
pub struct Monster {
    pub kind: MonsterKind,
    pub rank: MonsterRank,
    pub quest_id: Option<u64>,
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

#[derive(Clone)]
pub struct Npc {
    pub kind: NpcKind,
    pub name: String,
    pub quest_id: Option<u64>,
    pub pos: Vec2,
}

pub struct QuestItem {
    pub quest_id: u64,
    pub pos: Vec2,
    pub name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestKind {
    KillPack,
    BountyBoss,
    MeetNpc,
    RecoverItems,
}

impl QuestKind {
    pub const ALL: [Self; 4] = [
        Self::KillPack,
        Self::BountyBoss,
        Self::MeetNpc,
        Self::RecoverItems,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestStage {
    Active,
    ReadyToTurnIn,
}

#[derive(Clone)]
pub struct QuestReward {
    pub gold: i32,
    pub xp: i32,
    pub item_chance: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QuestSignature {
    KillPack {
        tile_x: i32,
        tile_y: i32,
        kind: MonsterKind,
    },
    BountyBoss {
        tile_x: i32,
        tile_y: i32,
        kind: MonsterKind,
    },
    MeetNpc {
        town_id: u64,
    },
    RecoverItems {
        landmark_id: u64,
    },
}

#[derive(Clone)]
pub struct Quest {
    pub id: u64,
    pub kind: QuestKind,
    pub signature: QuestSignature,
    pub stage: QuestStage,
    pub giver: SettlementSite,
    pub title: String,
    pub objective: String,
    pub target_pos: Vec2,
    pub progress: usize,
    pub goal: usize,
    pub reward: QuestReward,
}

impl Quest {
    pub fn progress_text(&self) -> String {
        match self.kind {
            QuestKind::MeetNpc => format!("{}/1 met", self.progress),
            QuestKind::KillPack | QuestKind::BountyBoss => {
                format!("{}/{} slain", self.progress, self.goal)
            }
            QuestKind::RecoverItems => format!("{}/{} recovered", self.progress, self.goal),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.stage == QuestStage::ReadyToTurnIn
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopTab {
    Buy,
    Sell,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryFocus {
    Backpack,
    Equipment,
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

#[derive(Clone)]
pub struct TravelDestination {
    pub name: String,
    pub pos: IVec2,
    pub min_level: i32,
}

#[derive(Clone, Copy)]
pub struct DiscoveredSettlement {
    pub site: SettlementSite,
}

impl DiscoveredSettlement {
    pub fn tier(self) -> SettlementTier {
        self.site.tier
    }
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
