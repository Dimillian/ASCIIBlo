use macroquad::prelude::Color;
use rand::{RngExt, rngs::StdRng};

use crate::world::Biome;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Weapon,
    Armor,
    Charm,
}

impl Slot {
    pub fn label(self) -> &'static str {
        match self {
            Slot::Weapon => "Weapon",
            Slot::Armor => "Armor",
            Slot::Charm => "Charm",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

impl Rarity {
    pub fn color(self) -> Color {
        match self {
            Rarity::Normal => Color::from_rgba(230, 230, 224, 255),
            Rarity::Magic => Color::from_rgba(96, 224, 255, 255),
            Rarity::Rare => Color::from_rgba(255, 224, 96, 255),
            Rarity::Unique => Color::from_rgba(220, 184, 112, 255),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Rarity::Normal => "Normal",
            Rarity::Magic => "Magic",
            Rarity::Rare => "Rare",
            Rarity::Unique => "Unique",
        }
    }
}

#[derive(Clone)]
pub struct Item {
    pub name: String,
    pub base_name: String,
    pub slot: Slot,
    pub rarity: Rarity,
    pub item_level: i32,
    pub affixes: Vec<String>,
    pub power: i32,
    pub armor: i32,
    pub vitality: i32,
    pub haste: i32,
    pub value: i32,
}

impl Item {
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if self.power != 0 {
            parts.push(format!("+{} POW", self.power));
        }
        if self.armor != 0 {
            parts.push(format!("+{} ARM", self.armor));
        }
        if self.vitality != 0 {
            parts.push(format!("+{} VIT", self.vitality));
        }
        if self.haste != 0 {
            parts.push(format!("+{} HST", self.haste));
        }
        parts.join(" ")
    }
}

#[derive(Clone, Copy)]
struct BaseItem {
    name: &'static str,
    slot: Slot,
    power: i32,
    armor: i32,
    vitality: i32,
    haste: i32,
    value: i32,
}

#[derive(Clone, Copy)]
struct Affix {
    name: &'static str,
    power: i32,
    armor: i32,
    vitality: i32,
    haste: i32,
    value: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonsterKind {
    Imp,
    Slime,
    Brute,
    Wisp,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NpcKind {
    Merchant,
    Trainer,
    Wayfinder,
}

impl NpcKind {
    pub fn glyph(self) -> char {
        match self {
            NpcKind::Merchant => '$',
            NpcKind::Trainer => '!',
            NpcKind::Wayfinder => '?',
        }
    }

    pub fn color(self) -> Color {
        match self {
            NpcKind::Merchant => Color::from_rgba(255, 214, 108, 255),
            NpcKind::Trainer => Color::from_rgba(255, 132, 120, 255),
            NpcKind::Wayfinder => Color::from_rgba(128, 214, 255, 255),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            NpcKind::Merchant => "Mira the Merchant",
            NpcKind::Trainer => "Oren the Trainer",
            NpcKind::Wayfinder => "Rill the Wayfinder",
        }
    }

    pub fn greeting(self) -> &'static str {
        match self {
            NpcKind::Merchant => "Steel, charms, and honest prices. Mostly honest.",
            NpcKind::Trainer => "Practice is what turns motion into mastery.",
            NpcKind::Wayfinder => "Roads remember where your boots have been.",
        }
    }
}

impl MonsterKind {
    pub fn glyph(self) -> char {
        match self {
            MonsterKind::Imp => 'i',
            MonsterKind::Slime => 's',
            MonsterKind::Brute => 'B',
            MonsterKind::Wisp => 'w',
        }
    }

    pub fn color(self) -> Color {
        match self {
            MonsterKind::Imp => Color::from_rgba(255, 92, 92, 255),
            MonsterKind::Slime => Color::from_rgba(116, 232, 96, 255),
            MonsterKind::Brute => Color::from_rgba(210, 70, 70, 255),
            MonsterKind::Wisp => Color::from_rgba(112, 226, 255, 255),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            MonsterKind::Imp => "Imp",
            MonsterKind::Slime => "Slime",
            MonsterKind::Brute => "Brute",
            MonsterKind::Wisp => "Wisp",
        }
    }

    pub fn max_hp(self) -> f32 {
        match self {
            MonsterKind::Imp => 24.0,
            MonsterKind::Slime => 34.0,
            MonsterKind::Brute => 62.0,
            MonsterKind::Wisp => 22.0,
        }
    }

    pub fn move_speed(self) -> f32 {
        match self {
            MonsterKind::Imp => 86.0,
            MonsterKind::Slime => 62.0,
            MonsterKind::Brute => 52.0,
            MonsterKind::Wisp => 104.0,
        }
    }

    pub fn damage(self) -> f32 {
        match self {
            MonsterKind::Imp => 8.0,
            MonsterKind::Slime => 6.0,
            MonsterKind::Brute => 16.0,
            MonsterKind::Wisp => 10.0,
        }
    }

    pub fn attack_cooldown(self) -> f32 {
        match self {
            MonsterKind::Imp => 0.9,
            MonsterKind::Slime => 1.15,
            MonsterKind::Brute => 1.35,
            MonsterKind::Wisp => 0.7,
        }
    }

    fn base_xp(self) -> i32 {
        match self {
            MonsterKind::Imp => 10,
            MonsterKind::Slime => 14,
            MonsterKind::Brute => 28,
            MonsterKind::Wisp => 18,
        }
    }
}

#[derive(Clone, Copy)]
struct EncounterWeight {
    kind: MonsterKind,
    weight: u32,
}

pub fn starter_items() -> Vec<Item> {
    vec![
        Item {
            name: "Copper Dirk".into(),
            base_name: "Dirk".into(),
            slot: Slot::Weapon,
            rarity: Rarity::Normal,
            item_level: 1,
            affixes: Vec::new(),
            power: 2,
            armor: 0,
            vitality: 0,
            haste: 1,
            value: 7,
        },
        Item {
            name: "Padded Vest".into(),
            base_name: "Vest".into(),
            slot: Slot::Armor,
            rarity: Rarity::Normal,
            item_level: 1,
            affixes: Vec::new(),
            power: 0,
            armor: 2,
            vitality: 1,
            haste: 0,
            value: 8,
        },
    ]
}

pub fn merchant_stock() -> Vec<Item> {
    vec![
        Item {
            name: "Keen Sabre".into(),
            base_name: "Sabre".into(),
            slot: Slot::Weapon,
            rarity: Rarity::Magic,
            item_level: 1,
            affixes: vec!["Keen".into(), "of Alacrity".into()],
            power: 4,
            armor: 0,
            vitality: 0,
            haste: 2,
            value: 24,
        },
        Item {
            name: "Iron Coat".into(),
            base_name: "Coat".into(),
            slot: Slot::Armor,
            rarity: Rarity::Magic,
            item_level: 1,
            affixes: vec!["Iron".into(), "of Vigor".into()],
            power: 0,
            armor: 5,
            vitality: 2,
            haste: 0,
            value: 26,
        },
        Item {
            name: "Lucky Bell".into(),
            base_name: "Bell".into(),
            slot: Slot::Charm,
            rarity: Rarity::Rare,
            item_level: 1,
            affixes: vec!["Lucky".into(), "Stalwart".into(), "of Fortune".into()],
            power: 2,
            armor: 1,
            vitality: 2,
            haste: 2,
            value: 36,
        },
    ]
}

pub fn roll_monster(rng: &mut StdRng, biome: Biome) -> MonsterKind {
    let table = encounter_table(biome);
    let total_weight: u32 = table.iter().map(|entry| entry.weight).sum();
    let mut roll = rng.random_range(0..total_weight);
    for entry in table {
        if roll < entry.weight {
            return entry.kind;
        }
        roll -= entry.weight;
    }
    table[0].kind
}

pub fn scaled_monster_level(world_level: i32, player_level: i32) -> i32 {
    let world_level = world_level.max(1);
    let player_level = player_level.max(1);
    world_level.max((world_level + player_level) / 2)
}

pub fn monster_max_hp(kind: MonsterKind, level: i32) -> f32 {
    kind.max_hp() * (1.0 + (level.max(1) - 1) as f32 * 0.35)
}

pub fn monster_damage(kind: MonsterKind, level: i32) -> f32 {
    kind.damage() * (1.0 + (level.max(1) - 1) as f32 * 0.22)
}

pub fn monster_xp(kind: MonsterKind, level: i32) -> i32 {
    (kind.base_xp() as f32 * (1.0 + (level.max(1) - 1) as f32 * 0.28)).round() as i32
}

pub fn roll_item(rng: &mut StdRng, item_level: i32) -> Item {
    let rarity = match rng.random_range(0..100) {
        0..=49 => Rarity::Normal,
        50..=82 => Rarity::Magic,
        83..=96 => Rarity::Rare,
        _ => Rarity::Unique,
    };
    let base = BASE_ITEMS[rng.random_range(0..BASE_ITEMS.len())];
    match rarity {
        Rarity::Normal => from_base(base, rarity, item_level, base.name.into(), Vec::new()),
        Rarity::Magic => {
            let mut affixes = vec![roll_affix(base.slot, rng)];
            if rng.random_bool(0.58) {
                affixes.push(roll_affix(base.slot, rng));
            }
            let name = magic_name(base.name, &affixes);
            from_base(base, rarity, item_level, name, affixes)
        }
        Rarity::Rare => {
            let affixes = vec![
                roll_affix(base.slot, rng),
                roll_affix(base.slot, rng),
                roll_affix(base.slot, rng),
            ];
            let name = format!(
                "{} {}",
                RARE_FIRST[rng.random_range(0..RARE_FIRST.len())],
                RARE_SECOND[rng.random_range(0..RARE_SECOND.len())]
            );
            from_base(base, rarity, item_level, name, affixes)
        }
        Rarity::Unique => roll_unique(base.slot, item_level, rng),
    }
}

const BASE_ITEMS: [BaseItem; 12] = [
    BaseItem {
        name: "Dirk",
        slot: Slot::Weapon,
        power: 2,
        armor: 0,
        vitality: 0,
        haste: 1,
        value: 7,
    },
    BaseItem {
        name: "Sabre",
        slot: Slot::Weapon,
        power: 3,
        armor: 0,
        vitality: 0,
        haste: 1,
        value: 10,
    },
    BaseItem {
        name: "Axe",
        slot: Slot::Weapon,
        power: 4,
        armor: 0,
        vitality: 0,
        haste: 0,
        value: 12,
    },
    BaseItem {
        name: "Mace",
        slot: Slot::Weapon,
        power: 5,
        armor: 0,
        vitality: 0,
        haste: 0,
        value: 14,
    },
    BaseItem {
        name: "Vest",
        slot: Slot::Armor,
        power: 0,
        armor: 2,
        vitality: 1,
        haste: 0,
        value: 8,
    },
    BaseItem {
        name: "Coat",
        slot: Slot::Armor,
        power: 0,
        armor: 3,
        vitality: 1,
        haste: 0,
        value: 11,
    },
    BaseItem {
        name: "Hauberk",
        slot: Slot::Armor,
        power: 0,
        armor: 4,
        vitality: 2,
        haste: 0,
        value: 15,
    },
    BaseItem {
        name: "Mantle",
        slot: Slot::Armor,
        power: 0,
        armor: 3,
        vitality: 2,
        haste: 1,
        value: 16,
    },
    BaseItem {
        name: "Coin",
        slot: Slot::Charm,
        power: 1,
        armor: 0,
        vitality: 0,
        haste: 1,
        value: 9,
    },
    BaseItem {
        name: "Fang",
        slot: Slot::Charm,
        power: 1,
        armor: 0,
        vitality: 1,
        haste: 0,
        value: 11,
    },
    BaseItem {
        name: "Ring",
        slot: Slot::Charm,
        power: 0,
        armor: 1,
        vitality: 1,
        haste: 1,
        value: 13,
    },
    BaseItem {
        name: "Bell",
        slot: Slot::Charm,
        power: 1,
        armor: 1,
        vitality: 1,
        haste: 1,
        value: 15,
    },
];

const WEAPON_AFFIXES: [Affix; 6] = [
    Affix {
        name: "Keen",
        power: 2,
        armor: 0,
        vitality: 0,
        haste: 0,
        value: 7,
    },
    Affix {
        name: "Jagged",
        power: 3,
        armor: 0,
        vitality: 0,
        haste: 0,
        value: 9,
    },
    Affix {
        name: "Swift",
        power: 0,
        armor: 0,
        vitality: 0,
        haste: 2,
        value: 8,
    },
    Affix {
        name: "of the Fox",
        power: 0,
        armor: 0,
        vitality: 1,
        haste: 1,
        value: 8,
    },
    Affix {
        name: "of Slaying",
        power: 4,
        armor: 0,
        vitality: 0,
        haste: 0,
        value: 12,
    },
    Affix {
        name: "of Alacrity",
        power: 1,
        armor: 0,
        vitality: 0,
        haste: 2,
        value: 10,
    },
];

const ARMOR_AFFIXES: [Affix; 6] = [
    Affix {
        name: "Sturdy",
        power: 0,
        armor: 2,
        vitality: 0,
        haste: 0,
        value: 7,
    },
    Affix {
        name: "Iron",
        power: 0,
        armor: 3,
        vitality: 0,
        haste: 0,
        value: 9,
    },
    Affix {
        name: "Stalwart",
        power: 0,
        armor: 1,
        vitality: 2,
        haste: 0,
        value: 10,
    },
    Affix {
        name: "of Vigor",
        power: 0,
        armor: 0,
        vitality: 3,
        haste: 0,
        value: 11,
    },
    Affix {
        name: "of Balance",
        power: 0,
        armor: 1,
        vitality: 1,
        haste: 1,
        value: 10,
    },
    Affix {
        name: "of Shelter",
        power: 0,
        armor: 4,
        vitality: 0,
        haste: 0,
        value: 13,
    },
];

const CHARM_AFFIXES: [Affix; 6] = [
    Affix {
        name: "Lucky",
        power: 1,
        armor: 0,
        vitality: 0,
        haste: 1,
        value: 8,
    },
    Affix {
        name: "Mending",
        power: 0,
        armor: 0,
        vitality: 2,
        haste: 0,
        value: 9,
    },
    Affix {
        name: "Quick",
        power: 0,
        armor: 0,
        vitality: 0,
        haste: 2,
        value: 9,
    },
    Affix {
        name: "of Fortune",
        power: 2,
        armor: 0,
        vitality: 1,
        haste: 0,
        value: 11,
    },
    Affix {
        name: "of Warding",
        power: 0,
        armor: 2,
        vitality: 0,
        haste: 0,
        value: 10,
    },
    Affix {
        name: "of the Lynx",
        power: 1,
        armor: 0,
        vitality: 0,
        haste: 2,
        value: 12,
    },
];

const RARE_FIRST: [&str; 6] = ["Doom", "Storm", "Rune", "Grim", "Blood", "Gale"];
const RARE_SECOND: [&str; 6] = ["Needle", "Shell", "Brand", "Ward", "Song", "Spur"];
const MEADOW_ENCOUNTERS: [EncounterWeight; 4] = [
    EncounterWeight {
        kind: MonsterKind::Imp,
        weight: 42,
    },
    EncounterWeight {
        kind: MonsterKind::Slime,
        weight: 38,
    },
    EncounterWeight {
        kind: MonsterKind::Wisp,
        weight: 15,
    },
    EncounterWeight {
        kind: MonsterKind::Brute,
        weight: 5,
    },
];
const FUNGAL_GROVE_ENCOUNTERS: [EncounterWeight; 4] = [
    EncounterWeight {
        kind: MonsterKind::Imp,
        weight: 12,
    },
    EncounterWeight {
        kind: MonsterKind::Slime,
        weight: 48,
    },
    EncounterWeight {
        kind: MonsterKind::Wisp,
        weight: 32,
    },
    EncounterWeight {
        kind: MonsterKind::Brute,
        weight: 8,
    },
];
const ASHFIELD_ENCOUNTERS: [EncounterWeight; 4] = [
    EncounterWeight {
        kind: MonsterKind::Imp,
        weight: 34,
    },
    EncounterWeight {
        kind: MonsterKind::Slime,
        weight: 10,
    },
    EncounterWeight {
        kind: MonsterKind::Wisp,
        weight: 24,
    },
    EncounterWeight {
        kind: MonsterKind::Brute,
        weight: 32,
    },
];
const OLD_RUINS_ENCOUNTERS: [EncounterWeight; 4] = [
    EncounterWeight {
        kind: MonsterKind::Imp,
        weight: 14,
    },
    EncounterWeight {
        kind: MonsterKind::Slime,
        weight: 8,
    },
    EncounterWeight {
        kind: MonsterKind::Wisp,
        weight: 34,
    },
    EncounterWeight {
        kind: MonsterKind::Brute,
        weight: 44,
    },
];

fn encounter_table(biome: Biome) -> &'static [EncounterWeight] {
    match biome {
        Biome::Town | Biome::Meadow => &MEADOW_ENCOUNTERS,
        Biome::FungalGrove => &FUNGAL_GROVE_ENCOUNTERS,
        Biome::Ashfield => &ASHFIELD_ENCOUNTERS,
        Biome::OldRuins => &OLD_RUINS_ENCOUNTERS,
    }
}

fn from_base(
    base: BaseItem,
    rarity: Rarity,
    item_level: i32,
    name: String,
    affixes: Vec<Affix>,
) -> Item {
    let mut item = Item {
        name,
        base_name: base.name.into(),
        slot: base.slot,
        rarity,
        item_level,
        affixes: affixes.iter().map(|affix| affix.name.into()).collect(),
        power: base.power,
        armor: base.armor,
        vitality: base.vitality,
        haste: base.haste,
        value: base.value,
    };
    for affix in affixes {
        item.power += affix.power;
        item.armor += affix.armor;
        item.vitality += affix.vitality;
        item.haste += affix.haste;
        item.value += affix.value;
    }
    apply_item_level_scaling(&mut item);
    item
}

fn apply_item_level_scaling(item: &mut Item) {
    let tier = (item.item_level - 1).max(0) / 2;
    match item.slot {
        Slot::Weapon => item.power += tier,
        Slot::Armor => {
            item.armor += tier;
            item.vitality += tier / 2;
        }
        Slot::Charm => {
            item.power += tier / 2;
            item.vitality += tier / 2;
            item.haste += tier / 3;
        }
    }
    item.value += tier * 5;
}

fn magic_name(base: &str, affixes: &[Affix]) -> String {
    match affixes {
        [first, second] if second.name.starts_with("of ") => {
            format!("{} {} {}", first.name, base, second.name)
        }
        [first, second] => format!("{} {} {}", first.name, second.name, base),
        [first] if first.name.starts_with("of ") => format!("{} {}", base, first.name),
        [first] => format!("{} {}", first.name, base),
        _ => base.into(),
    }
}

fn roll_affix(slot: Slot, rng: &mut StdRng) -> Affix {
    let table = match slot {
        Slot::Weapon => &WEAPON_AFFIXES[..],
        Slot::Armor => &ARMOR_AFFIXES[..],
        Slot::Charm => &CHARM_AFFIXES[..],
    };
    table[rng.random_range(0..table.len())]
}

fn roll_unique(slot: Slot, item_level: i32, rng: &mut StdRng) -> Item {
    let pool: &[BaseItem] = match slot {
        Slot::Weapon => &[
            BaseItem {
                name: "Stormbite",
                slot: Slot::Weapon,
                power: 9,
                armor: 0,
                vitality: 1,
                haste: 3,
                value: 42,
            },
            BaseItem {
                name: "Black Hook",
                slot: Slot::Weapon,
                power: 11,
                armor: 0,
                vitality: 0,
                haste: 1,
                value: 46,
            },
        ],
        Slot::Armor => &[
            BaseItem {
                name: "Saint's Mantle",
                slot: Slot::Armor,
                power: 0,
                armor: 8,
                vitality: 4,
                haste: 1,
                value: 44,
            },
            BaseItem {
                name: "Ashen Shell",
                slot: Slot::Armor,
                power: 0,
                armor: 10,
                vitality: 2,
                haste: 0,
                value: 48,
            },
        ],
        Slot::Charm => &[
            BaseItem {
                name: "Eye of Ember",
                slot: Slot::Charm,
                power: 4,
                armor: 2,
                vitality: 3,
                haste: 3,
                value: 50,
            },
            BaseItem {
                name: "Bell of Return",
                slot: Slot::Charm,
                power: 2,
                armor: 3,
                vitality: 4,
                haste: 2,
                value: 52,
            },
        ],
    };
    let unique = pool[rng.random_range(0..pool.len())];
    let mut item = Item {
        name: unique.name.into(),
        base_name: unique.name.into(),
        slot: unique.slot,
        rarity: Rarity::Unique,
        item_level,
        affixes: vec!["Unique".into()],
        power: unique.power,
        armor: unique.armor,
        vitality: unique.vitality,
        haste: unique.haste,
        value: unique.value,
    };
    apply_item_level_scaling(&mut item);
    item
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn generated_items_follow_quality_rules() {
        let mut rng = StdRng::seed_from_u64(9);
        for _ in 0..500 {
            let item_level = 1 + rng.random_range(0..=6);
            let item = roll_item(&mut rng, item_level);
            match item.rarity {
                Rarity::Normal => assert!(item.affixes.is_empty()),
                Rarity::Magic => assert!((1..=2).contains(&item.affixes.len())),
                Rarity::Rare => assert_eq!(item.affixes.len(), 3),
                Rarity::Unique => assert_eq!(item.affixes, vec!["Unique"]),
            }
            assert!(!item.base_name.is_empty());
            assert!(!item.name.is_empty());
        }
    }

    #[test]
    fn higher_item_levels_raise_floor_stats() {
        let base = BASE_ITEMS[0];
        let low = from_base(base, Rarity::Normal, 1, base.name.into(), Vec::new());
        let high = from_base(base, Rarity::Normal, 7, base.name.into(), Vec::new());
        assert!(high.power > low.power);
        assert!(high.value > low.value);
        assert_eq!(high.item_level, 7);
    }

    #[test]
    fn monsters_partly_follow_player_level_without_ignoring_world_level() {
        assert_eq!(scaled_monster_level(1, 1), 1);
        assert_eq!(scaled_monster_level(1, 9), 5);
        assert_eq!(scaled_monster_level(6, 1), 6);
        assert_eq!(scaled_monster_level(6, 10), 8);
    }

    #[test]
    fn higher_level_monsters_are_tougher_and_worth_more_xp() {
        assert!(monster_max_hp(MonsterKind::Brute, 5) > monster_max_hp(MonsterKind::Brute, 1));
        assert!(monster_damage(MonsterKind::Brute, 5) > monster_damage(MonsterKind::Brute, 1));
        assert!(monster_xp(MonsterKind::Brute, 5) > monster_xp(MonsterKind::Brute, 1));
    }

    #[test]
    fn biome_encounter_tables_have_distinct_identities() {
        assert_eq!(encounter_table(Biome::Meadow)[0].kind, MonsterKind::Imp);
        assert_eq!(
            encounter_table(Biome::FungalGrove)[1].kind,
            MonsterKind::Slime
        );
        assert_eq!(encounter_table(Biome::Ashfield)[3].kind, MonsterKind::Brute);
        assert!(
            encounter_table(Biome::OldRuins)[3].weight > encounter_table(Biome::Meadow)[3].weight
        );
    }

    #[test]
    fn higher_level_uniques_scale_too() {
        let mut low_rng = StdRng::seed_from_u64(3);
        let mut high_rng = StdRng::seed_from_u64(3);
        let low = roll_unique(Slot::Weapon, 1, &mut low_rng);
        let high = roll_unique(Slot::Weapon, 7, &mut high_rng);
        assert_eq!(low.name, high.name);
        assert!(high.power > low.power);
        assert!(high.value > low.value);
    }
}
