use crate::{
    content::{Item, Slot},
    game::Player,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicStat {
    Strength,
    Agility,
    Vitality,
    Life,
    Mana,
    Power,
    Armor,
    CritChance,
    AttackDelay,
    MoveSpeed,
    MeleeDamage,
    SpellDamage,
    ManaRegen,
}

impl PublicStat {
    pub(crate) fn label(self) -> &'static str {
        match self {
            PublicStat::Strength => "Strength",
            PublicStat::Agility => "Agility",
            PublicStat::Vitality => "Vitality",
            PublicStat::Life => "Life",
            PublicStat::Mana => "Mana",
            PublicStat::Power => "Power",
            PublicStat::Armor => "Armor",
            PublicStat::CritChance => "Crit chance",
            PublicStat::AttackDelay => "Attack delay",
            PublicStat::MoveSpeed => "Move speed",
            PublicStat::MeleeDamage => "Melee damage",
            PublicStat::SpellDamage => "Spell damage",
            PublicStat::ManaRegen => "Mana regeneration",
        }
    }

    pub(crate) fn compact_label(self) -> &'static str {
        match self {
            PublicStat::Strength => "STR",
            PublicStat::Agility => "AGI",
            PublicStat::Vitality => "VIT",
            PublicStat::Life => "Life",
            PublicStat::Mana => "Mana",
            PublicStat::Power => "POW",
            PublicStat::Armor => "ARM",
            PublicStat::CritChance => "Crit",
            PublicStat::AttackDelay => "Delay",
            PublicStat::MoveSpeed => "Speed",
            PublicStat::MeleeDamage => "Melee",
            PublicStat::SpellDamage => "Spell",
            PublicStat::ManaRegen => "Regen",
        }
    }

    pub(crate) fn value(self, player: &Player) -> String {
        match self {
            PublicStat::Strength => player.stats.strength.to_string(),
            PublicStat::Agility => player.stats.agility.to_string(),
            PublicStat::Vitality => {
                let gear_bonus = player.equipment.bonus_vitality();
                if gear_bonus == 0 {
                    player.stats.vitality.to_string()
                } else {
                    format!("{} (+{} gear)", player.stats.vitality, gear_bonus)
                }
            }
            PublicStat::Life => format!(
                "{} / {}",
                player.hp.round() as i32,
                player.max_hp().round() as i32
            ),
            PublicStat::Mana => format!(
                "{} / {}",
                player.mana.round() as i32,
                player.max_mana().round() as i32
            ),
            PublicStat::Power => player.power().to_string(),
            PublicStat::Armor => player.armor().to_string(),
            PublicStat::CritChance => format!("{:.0}%", player.crit_chance() * 100.0),
            PublicStat::AttackDelay => format!("{:.2}s", player.attack_interval()),
            PublicStat::MoveSpeed => format!("{:.0}", player.move_speed_rating()),
            PublicStat::MeleeDamage => format!("+{}", player.melee_damage_bonus()),
            PublicStat::SpellDamage => format!("+{}", player.magic_damage_bonus()),
            PublicStat::ManaRegen => format!("+{:.1}/s", player.magic_regen_bonus()),
        }
    }

    pub(crate) fn detail(self, player: &Player) -> String {
        match self {
            PublicStat::Strength => format!(
                "Strength grants 2 Power per point. Current base Strength {} contributes {} Power before gear.",
                player.stats.strength,
                player.stats.strength * 2
            ),
            PublicStat::Agility => format!(
                "Agility increases Move speed, shortens Attack delay, and raises Crit chance. Current Agility sets Crit chance to {:.0}%.",
                player.crit_chance() * 100.0
            ),
            PublicStat::Vitality => {
                let gear_bonus = player.equipment.bonus_vitality();
                if gear_bonus == 0 {
                    "Vitality grants 7 maximum Life per point and 1 Armor every 2 base points."
                        .into()
                } else {
                    format!(
                        "Vitality grants 7 maximum Life per point and 1 Armor every 2 base points. Gear adds {} Vitality.",
                        gear_bonus
                    )
                }
            }
            PublicStat::Life => {
                "Life is lost when monsters hit you. If it reaches 0, you wake in town and lose some gold."
                    .into()
            }
            PublicStat::Mana => {
                "Mana fuels active skills. Maximum Mana rises with level, while Magic mastery improves Mana regeneration."
                    .into()
            }
            PublicStat::Power => format!(
                "Power is your weapon baseline. It equals Strength x2 plus {} gear Power before attack rolls.",
                player.equipment.bonus_power()
            ),
            PublicStat::Armor => format!(
                "Armor subtracts from incoming monster damage, but hits still deal at least 1. Current value includes {} gear Armor.",
                player.equipment.bonus_armor()
            ),
            PublicStat::CritChance => {
                "Crit chance starts at 8%, gains 1% per Agility, and caps at 35%. Critical hits deal double damage."
                    .into()
            }
            PublicStat::AttackDelay => {
                "Basic attacks recover faster as Agility and certain gear bonuses improve. The interval cannot go below 0.16 seconds."
                    .into()
            }
            PublicStat::MoveSpeed => {
                "Move speed starts at 100 and increases with Agility and certain gear bonuses.".into()
            }
            PublicStat::MeleeDamage => {
                "Extra physical damage added to basic attacks and melee skills.".into()
            }
            PublicStat::SpellDamage => "Extra damage added to Magic skills.".into(),
            PublicStat::ManaRegen => {
                "Extra Mana restored each second from Magic mastery.".into()
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct StatPreview {
    pub power: i32,
    pub armor: i32,
    pub vitality: i32,
    pub max_life: i32,
    pub move_speed: i32,
    pub attack_delay: f32,
}

impl StatPreview {
    fn current(player: &Player) -> Self {
        Self {
            power: player.power(),
            armor: player.armor(),
            vitality: player.stats.vitality + player.equipment.bonus_vitality(),
            max_life: player.max_hp().round() as i32,
            move_speed: player.move_speed_rating().round() as i32,
            attack_delay: player.attack_interval(),
        }
    }

    fn with_item(player: &Player, item: &Item) -> Self {
        let equipped = equipped_item(player, item.slot);
        let power_bonus =
            player.equipment.bonus_power() - equipped.map_or(0, |item| item.power) + item.power;
        let armor_bonus =
            player.equipment.bonus_armor() - equipped.map_or(0, |item| item.armor) + item.armor;
        let vitality_bonus = player.equipment.bonus_vitality()
            - equipped.map_or(0, |item| item.vitality)
            + item.vitality;
        let speed_bonus =
            player.equipment.bonus_haste() - equipped.map_or(0, |item| item.haste) + item.haste;
        let speed_factor = player.stats.agility + speed_bonus;

        Self {
            power: player.stats.strength * 2 + power_bonus,
            armor: player.stats.vitality / 2 + armor_bonus + player.armor_mastery_bonus(),
            vitality: player.stats.vitality + vitality_bonus,
            max_life: (64.0 + (player.stats.vitality + vitality_bonus) as f32 * 7.0).round() as i32,
            move_speed: (80.0 + speed_factor as f32 * 4.0 + player.agility_mastery_bonus() as f32)
                .round() as i32,
            attack_delay: (0.5 - speed_factor as f32 * 0.018).max(0.16),
        }
    }
}

pub(crate) fn item_bonus_labels(item: &Item) -> Vec<String> {
    let mut labels = Vec::new();
    if item.power != 0 {
        labels.push(format!("{} +{}", PublicStat::Power.label(), item.power));
    }
    if item.armor != 0 {
        labels.push(format!("{} +{}", PublicStat::Armor.label(), item.armor));
    }
    if item.vitality != 0 {
        labels.push(format!(
            "{} +{}",
            PublicStat::Vitality.label(),
            item.vitality
        ));
    }
    if item.haste != 0 {
        labels.push(format!(
            "{} +{}",
            PublicStat::MoveSpeed.label(),
            item.haste * 4
        ));
    }
    labels
}

pub(crate) fn item_summary(item: &Item) -> String {
    let mut parts = Vec::new();
    if item.power != 0 {
        parts.push(format!(
            "+{} {}",
            item.power,
            PublicStat::Power.compact_label()
        ));
    }
    if item.armor != 0 {
        parts.push(format!(
            "+{} {}",
            item.armor,
            PublicStat::Armor.compact_label()
        ));
    }
    if item.vitality != 0 {
        parts.push(format!(
            "+{} {}",
            item.vitality,
            PublicStat::Vitality.compact_label()
        ));
    }
    if item.haste != 0 {
        parts.push(format!(
            "+{} {}",
            item.haste * 4,
            PublicStat::MoveSpeed.compact_label()
        ));
    }
    parts.join(" ")
}

pub(crate) fn item_comparison_labels(player: &Player, item: &Item) -> Vec<String> {
    let current = StatPreview::current(player);
    let preview = StatPreview::with_item(player, item);
    let mut labels = Vec::new();

    push_int_change(&mut labels, PublicStat::Power, current.power, preview.power);
    push_int_change(&mut labels, PublicStat::Armor, current.armor, preview.armor);
    push_int_change(
        &mut labels,
        PublicStat::Vitality,
        current.vitality,
        preview.vitality,
    );
    push_int_change(
        &mut labels,
        PublicStat::Life,
        current.max_life,
        preview.max_life,
    );
    push_int_change(
        &mut labels,
        PublicStat::MoveSpeed,
        current.move_speed,
        preview.move_speed,
    );
    if (current.attack_delay - preview.attack_delay).abs() >= 0.005 {
        labels.push(format!(
            "{} {:.2}s -> {:.2}s",
            PublicStat::AttackDelay.label(),
            current.attack_delay,
            preview.attack_delay
        ));
    }

    labels
}

fn push_int_change(labels: &mut Vec<String>, stat: PublicStat, before: i32, after: i32) {
    if before != after {
        labels.push(format!("{} {} -> {}", stat.label(), before, after));
    }
}

fn equipped_item(player: &Player, slot: Slot) -> Option<&Item> {
    match slot {
        Slot::Weapon => player.equipment.weapon.as_ref(),
        Slot::Armor => player.equipment.armor.as_ref(),
        Slot::Charm => player.equipment.charm.as_ref(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        content::{Item, Rarity, Slot},
        game::Game,
    };

    use super::{PublicStat, item_bonus_labels, item_comparison_labels, item_summary};

    fn haste_charm() -> Item {
        Item {
            name: "Swift Charm".into(),
            base_name: "Charm".into(),
            slot: Slot::Charm,
            rarity: Rarity::Magic,
            item_level: 1,
            affixes: vec!["Swift".into()],
            power: 0,
            armor: 0,
            vitality: 1,
            haste: 2,
            value: 10,
        }
    }

    #[test]
    fn public_stat_vocabulary_hides_haste_from_item_bonuses() {
        let labels = item_bonus_labels(&haste_charm());

        assert_eq!(labels, vec!["Vitality +1", "Move speed +8"]);
        assert!(!labels.iter().any(|label| label.contains("Haste")));
    }

    #[test]
    fn compact_item_summaries_use_the_shared_public_labels() {
        assert_eq!(item_summary(&haste_charm()), "+1 VIT +8 Speed");
    }

    #[test]
    fn item_comparisons_use_character_sheet_stats() {
        let game = Game::new(1);
        let labels = item_comparison_labels(&game.sim.player, &haste_charm());

        assert_eq!(
            labels,
            vec![
                "Vitality 4 -> 5",
                "Life 92 -> 99",
                "Move speed 100 -> 108",
                "Attack delay 0.41s -> 0.37s",
            ]
        );
    }

    #[test]
    fn public_stat_labels_and_values_are_consistent() {
        let game = Game::new(2);

        assert_eq!(PublicStat::MoveSpeed.label(), "Move speed");
        assert_eq!(PublicStat::MoveSpeed.compact_label(), "Speed");
        assert_eq!(PublicStat::MoveSpeed.value(&game.sim.player), "100");
    }

    #[test]
    fn vitality_hides_empty_gear_bonus_but_keeps_real_bonus_visible() {
        let mut game = Game::new(3);

        assert_eq!(PublicStat::Vitality.value(&game.sim.player), "4");
        assert_eq!(
            PublicStat::Vitality.detail(&game.sim.player),
            "Vitality grants 7 maximum Life per point and 1 Armor every 2 base points."
        );

        game.sim.player.equipment.charm = Some(haste_charm());

        assert_eq!(PublicStat::Vitality.value(&game.sim.player), "4 (+1 gear)");
        assert_eq!(
            PublicStat::Vitality.detail(&game.sim.player),
            "Vitality grants 7 maximum Life per point and 1 Armor every 2 base points. Gear adds 1 Vitality."
        );
    }
}
