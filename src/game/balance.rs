use ::rand::{RngExt, SeedableRng, rngs::StdRng};
use macroquad::prelude::*;

use crate::{
    content::{
        MonsterKind, MonsterRank, Slot, monster_damage, monster_max_hp, roll_item, starter_items,
    },
    world::World,
};

use super::{AbilityKind, Equipment, FIXED_DT, Game, Monster};

const REPORT_LEVELS: [i32; 6] = [1, 3, 5, 8, 12, 20];
const MONSTER_KINDS: [MonsterKind; 8] = [
    MonsterKind::Imp,
    MonsterKind::Slime,
    MonsterKind::Brute,
    MonsterKind::Wisp,
    MonsterKind::Hound,
    MonsterKind::Beetle,
    MonsterKind::Cinderling,
    MonsterKind::Revenant,
];
const COMBAT_TRIALS: usize = 2_000;
const ENCOUNTER_TRIALS: usize = 8;
const LOOT_TRIALS: usize = 10_000;
const ENCOUNTER_TIMEOUT: f32 = 12.0;

#[derive(Clone, Copy)]
enum Loadout {
    Opening,
    StarterEquipped,
}

impl Loadout {
    fn label(self) -> &'static str {
        match self {
            Self::Opening => "opening",
            Self::StarterEquipped => "starter-equipped",
        }
    }
}

#[derive(Clone, Copy)]
enum EncounterBehavior {
    BasicOnly,
    StarterKit,
}

impl EncounterBehavior {
    fn label(self) -> &'static str {
        match self {
            Self::BasicOnly => "basic-only",
            Self::StarterKit => "starter-kit",
        }
    }
}

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    members: &'static [(MonsterKind, MonsterRank)],
}

const IMP_PACK: [(MonsterKind, MonsterRank); 4] = [
    (MonsterKind::Imp, MonsterRank::Normal),
    (MonsterKind::Imp, MonsterRank::Normal),
    (MonsterKind::Imp, MonsterRank::Normal),
    (MonsterKind::Imp, MonsterRank::Normal),
];
const HOUND_PACK: [(MonsterKind, MonsterRank); 5] = [
    (MonsterKind::Hound, MonsterRank::Normal),
    (MonsterKind::Hound, MonsterRank::Normal),
    (MonsterKind::Hound, MonsterRank::Normal),
    (MonsterKind::Hound, MonsterRank::Normal),
    (MonsterKind::Hound, MonsterRank::Normal),
];
const MIXED_PACK: [(MonsterKind, MonsterRank); 5] = [
    (MonsterKind::Imp, MonsterRank::Normal),
    (MonsterKind::Imp, MonsterRank::Normal),
    (MonsterKind::Hound, MonsterRank::Normal),
    (MonsterKind::Slime, MonsterRank::Normal),
    (MonsterKind::Wisp, MonsterRank::Normal),
];
const ELITE_PACK: [(MonsterKind, MonsterRank); 4] = [
    (MonsterKind::Imp, MonsterRank::Elite),
    (MonsterKind::Imp, MonsterRank::Normal),
    (MonsterKind::Imp, MonsterRank::Normal),
    (MonsterKind::Imp, MonsterRank::Normal),
];
const SCENARIOS: [Scenario; 4] = [
    Scenario {
        name: "4 imps",
        members: &IMP_PACK,
    },
    Scenario {
        name: "5 hounds",
        members: &HOUND_PACK,
    },
    Scenario {
        name: "mixed meadow",
        members: &MIXED_PACK,
    },
    Scenario {
        name: "elite imp pack",
        members: &ELITE_PACK,
    },
];

pub(crate) fn render_balance_report(seed: u64) -> String {
    let mut lines = Vec::new();
    lines.push("# ASCIIBlo balance report".into());
    lines.push(format!("seed: {seed}"));
    lines.push(
        "assumptions: later-level sheets spend earned stat points evenly; encounter sims stand still and use the real combat loop; loot rows sample independent item rolls."
            .into(),
    );
    lines.push(String::new());

    append_player_curve(&mut lines, seed);
    append_combat_matrix(&mut lines, seed);
    append_encounter_sims(&mut lines, seed);
    append_loot_sampler(&mut lines, seed);

    lines.join("\n")
}

fn append_player_curve(lines: &mut Vec<String>, seed: u64) {
    lines.push("## Player curve".into());
    lines.push("| level | loadout | hp | power | armor | speed | attack sec | crit |".into());
    lines.push("| ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |".into());
    for level in REPORT_LEVELS {
        for loadout in [Loadout::Opening, Loadout::StarterEquipped] {
            let game = report_game(seed, level, loadout);
            let player = &game.sim.player;
            lines.push(format!(
                "| {level} | {} | {:.0} | {} | {} | {:.0} | {:.2} | {:.0}% |",
                loadout.label(),
                player.max_hp(),
                player.power(),
                player.armor(),
                player.move_speed(),
                player.attack_interval(),
                player.crit_chance() * 100.0
            ));
        }
    }
    lines.push(String::new());
}

fn append_combat_matrix(lines: &mut Vec<String>, seed: u64) {
    lines.push("## Level 1 combat matrix".into());
    lines.push("| monster | hp | hit after armor (opening / geared) | avg basics to kill (opening / geared) | avg basic ttk sec (opening / geared) |".into());
    lines.push("| --- | ---: | ---: | ---: | ---: |".into());
    for kind in MONSTER_KINDS {
        let opening = report_game(seed, 1, Loadout::Opening);
        let geared = report_game(seed, 1, Loadout::StarterEquipped);
        let opening_hits = average_basic_hits(&opening, kind, seed ^ kind as u64);
        let geared_hits = average_basic_hits(&geared, kind, seed ^ ((kind as u64) << 8));
        let raw = monster_damage(kind, 1, MonsterRank::Normal);
        let opening_taken = (raw - opening.sim.player.armor() as f32).max(1.0);
        let geared_taken = (raw - geared.sim.player.armor() as f32).max(1.0);
        lines.push(format!(
            "| {} | {:.0} | {:.1} / {:.1} | {:.2} / {:.2} | {:.2} / {:.2} |",
            kind.name(),
            monster_max_hp(kind, 1, MonsterRank::Normal),
            opening_taken,
            geared_taken,
            opening_hits,
            geared_hits,
            opening_hits * opening.sim.player.attack_interval(),
            geared_hits * geared.sim.player.attack_interval(),
        ));
    }
    lines.push(String::new());
}

fn append_encounter_sims(lines: &mut Vec<String>, seed: u64) {
    lines.push("## Level 1 encounter sims".into());
    lines.push(
        "| scenario | loadout | behavior | clear rate | avg clear sec | avg hp left on clears |"
            .into(),
    );
    lines.push("| --- | --- | --- | ---: | ---: | ---: |".into());
    for scenario in SCENARIOS {
        for loadout in [Loadout::Opening, Loadout::StarterEquipped] {
            for behavior in [EncounterBehavior::BasicOnly, EncounterBehavior::StarterKit] {
                let summary = simulate_encounter_trials(seed, scenario, loadout, behavior);
                lines.push(format!(
                    "| {} | {} | {} | {:.0}% | {:.2} | {:.0} |",
                    scenario.name,
                    loadout.label(),
                    behavior.label(),
                    summary.clear_rate * 100.0,
                    summary.average_clear_time,
                    summary.average_hp_left,
                ));
            }
        }
    }
    lines.push(String::new());
}

fn append_loot_sampler(lines: &mut Vec<String>, seed: u64) {
    lines.push("## Loot sampler".into());
    lines.push("| item level | weapon avg pow | armor avg arm | armor avg vit | charm avg total stats | avg value | rare+ unique |".into());
    lines.push("| ---: | ---: | ---: | ---: | ---: | ---: | ---: |".into());
    for level in REPORT_LEVELS {
        let summary = sample_loot(seed ^ ((level as u64) << 16), level);
        lines.push(format!(
            "| {level} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.0}% |",
            summary.weapon_power,
            summary.armor_armor,
            summary.armor_vitality,
            summary.charm_total_stats,
            summary.average_value,
            summary.rare_or_unique_rate * 100.0,
        ));
    }
    lines.push(String::new());
}

fn report_game(seed: u64, level: i32, loadout: Loadout) -> Game {
    let mut game = Game::new(seed);
    game.sim.monsters.clear();
    game.sim.loot.clear();
    game.sim.player.stats.level = level;
    let gained_levels = (level - 1).max(0);
    game.sim.player.stats.strength = 5 + gained_levels * 2;
    game.sim.player.stats.agility = 5 + gained_levels * 2;
    game.sim.player.stats.vitality = 4 + gained_levels * 2;
    game.sim.player.stats.unspent_stat_points = 0;
    game.sim.player.equipment = match loadout {
        Loadout::Opening => Equipment {
            weapon: None,
            armor: None,
            charm: None,
        },
        Loadout::StarterEquipped => {
            let mut items = starter_items().into_iter();
            Equipment {
                weapon: items.next(),
                armor: items.next(),
                charm: None,
            }
        }
    };
    game.sim.player.hp = game.sim.player.max_hp();
    game.sim.player.mana = game.sim.player.max_mana();
    game
}

fn average_basic_hits(game: &Game, kind: MonsterKind, seed: u64) -> f32 {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut total_hits = 0usize;
    for _ in 0..COMBAT_TRIALS {
        let mut hp = monster_max_hp(kind, 1, MonsterRank::Normal);
        let mut hits = 0usize;
        while hp > 0.0 {
            hits += 1;
            hp -= sampled_player_damage(game, false, &mut rng);
        }
        total_hits += hits;
    }
    total_hits as f32 / COMBAT_TRIALS as f32
}

fn sampled_player_damage(game: &Game, skill: bool, rng: &mut StdRng) -> f32 {
    let base = game.sim.player.power() as f32 + rng.random_range(3.0..=8.0);
    let bonus = if skill { 4.0 } else { 0.0 };
    let damage = base + bonus;
    if rng.random_bool(game.sim.player.crit_chance() as f64) {
        damage * 2.0
    } else {
        damage
    }
}

struct EncounterSummary {
    clear_rate: f32,
    average_clear_time: f32,
    average_hp_left: f32,
}

fn simulate_encounter_trials(
    seed: u64,
    scenario: Scenario,
    loadout: Loadout,
    behavior: EncounterBehavior,
) -> EncounterSummary {
    let mut clears = 0usize;
    let mut clear_time_total = 0.0;
    let mut hp_left_total = 0.0;
    for trial in 0..ENCOUNTER_TRIALS {
        let outcome =
            simulate_encounter(seed ^ ((trial as u64) << 24), scenario, loadout, behavior);
        if outcome.cleared {
            clears += 1;
            clear_time_total += outcome.elapsed;
            hp_left_total += outcome.hp_left;
        }
    }
    EncounterSummary {
        clear_rate: clears as f32 / ENCOUNTER_TRIALS as f32,
        average_clear_time: if clears == 0 {
            0.0
        } else {
            clear_time_total / clears as f32
        },
        average_hp_left: if clears == 0 {
            0.0
        } else {
            hp_left_total / clears as f32
        },
    }
}

struct EncounterOutcome {
    cleared: bool,
    elapsed: f32,
    hp_left: f32,
}

fn simulate_encounter(
    seed: u64,
    scenario: Scenario,
    loadout: Loadout,
    behavior: EncounterBehavior,
) -> EncounterOutcome {
    let mut game = report_game(seed, 1, loadout);
    game.sim.player.pos = World::tile_center(ivec2(120, 120));
    game.sim.player.facing = Vec2::X;
    game.sim.player.stats.xp = 0;
    game.sim.player.stats.next_xp = i32::MAX;
    game.sim.monsters = scenario
        .members
        .iter()
        .enumerate()
        .map(|(index, (kind, rank))| {
            let pos =
                game.sim.player.pos + vec2(44.0 + index as f32 * 8.0, (index as f32 - 1.5) * 12.0);
            let max_hp = monster_max_hp(*kind, 1, *rank);
            Monster {
                kind: *kind,
                rank: *rank,
                quest_id: None,
                pack_id: 1,
                pack_center: pos,
                pos,
                vel: Vec2::ZERO,
                hit_offset: Vec2::ZERO,
                hp: max_hp,
                max_hp,
                level: 1,
                attack_cd: game.runtime.rng.random_range(0.0..kind.attack_cooldown()),
                engaged: true,
                wobble: 0.0,
                hit_flash: 0.0,
                chill_ttl: 0.0,
            }
        })
        .collect();

    let start_pos = game.sim.player.pos;
    let mut elapsed = 0.0;
    while elapsed < ENCOUNTER_TIMEOUT && !game.sim.monsters.is_empty() {
        if game.sim.player.pos != start_pos {
            return EncounterOutcome {
                cleared: false,
                elapsed,
                hp_left: 0.0,
            };
        }
        if let Some(target) = game.sim.monsters.first() {
            game.sim.player.facing = (target.pos - game.sim.player.pos).normalize_or_zero();
        }
        match behavior {
            EncounterBehavior::BasicOnly => {
                if game.sim.player.attack_cd <= 0.0 {
                    game.basic_attack();
                }
            }
            EncounterBehavior::StarterKit => {
                if game.sim.player.ability_cooldowns[AbilityKind::Cleave.index()] <= 0.0
                    && game.sim.player.mana >= AbilityKind::Cleave.mana_cost()
                {
                    game.cast_ability(AbilityKind::Cleave);
                } else if game.sim.player.ability_cooldowns[AbilityKind::Fireball.index()] <= 0.0
                    && game.sim.player.mana >= AbilityKind::Fireball.mana_cost()
                {
                    game.cast_ability(AbilityKind::Fireball);
                } else if game.sim.player.attack_cd <= 0.0 {
                    game.basic_attack();
                }
            }
        }
        tick_headless_combat(&mut game, FIXED_DT);
        elapsed += FIXED_DT;
    }
    EncounterOutcome {
        cleared: game.sim.monsters.is_empty(),
        elapsed,
        hp_left: game.sim.player.hp,
    }
}

fn tick_headless_combat(game: &mut Game, dt: f32) {
    game.sim.player.attack_cd = (game.sim.player.attack_cd - dt).max(0.0);
    for cooldown in &mut game.sim.player.ability_cooldowns {
        *cooldown = (*cooldown - dt).max(0.0);
    }
    game.sim.player.mana = (game.sim.player.mana + dt * game.sim.player.mana_regen_rate())
        .min(game.sim.player.max_mana());
    game.update_projectiles(dt);
    game.update_meteors(dt);
    game.update_monsters(dt);
}

struct LootSummary {
    weapon_power: f32,
    armor_armor: f32,
    armor_vitality: f32,
    charm_total_stats: f32,
    average_value: f32,
    rare_or_unique_rate: f32,
}

fn sample_loot(seed: u64, level: i32) -> LootSummary {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut weapon_count = 0usize;
    let mut armor_count = 0usize;
    let mut charm_count = 0usize;
    let mut weapon_power = 0;
    let mut armor_armor = 0;
    let mut armor_vitality = 0;
    let mut charm_total_stats = 0;
    let mut total_value = 0;
    let mut rare_or_unique = 0usize;
    for _ in 0..LOOT_TRIALS {
        let item = roll_item(&mut rng, level);
        match item.slot {
            Slot::Weapon => {
                weapon_count += 1;
                weapon_power += item.power;
            }
            Slot::Armor => {
                armor_count += 1;
                armor_armor += item.armor;
                armor_vitality += item.vitality;
            }
            Slot::Charm => {
                charm_count += 1;
                charm_total_stats += item.power + item.armor + item.vitality + item.haste;
            }
        }
        total_value += item.value;
        if matches!(
            item.rarity,
            crate::content::Rarity::Rare | crate::content::Rarity::Unique
        ) {
            rare_or_unique += 1;
        }
    }
    LootSummary {
        weapon_power: weapon_power as f32 / weapon_count as f32,
        armor_armor: armor_armor as f32 / armor_count as f32,
        armor_vitality: armor_vitality as f32 / armor_count as f32,
        charm_total_stats: charm_total_stats as f32 / charm_count as f32,
        average_value: total_value as f32 / LOOT_TRIALS as f32,
        rare_or_unique_rate: rare_or_unique as f32 / LOOT_TRIALS as f32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_curve_section_mentions_both_loadouts() {
        let mut lines = Vec::new();
        append_player_curve(&mut lines, 7);
        let section = lines.join("\n");
        assert!(section.contains("| 1 | opening |"));
        assert!(section.contains("| 1 | starter-equipped |"));
    }

    #[test]
    fn starter_loadout_is_stronger_than_opening_state() {
        let opening = report_game(7, 1, Loadout::Opening);
        let geared = report_game(7, 1, Loadout::StarterEquipped);
        assert!(geared.sim.player.power() > opening.sim.player.power());
        assert!(geared.sim.player.armor() > opening.sim.player.armor());
        assert!(geared.sim.player.max_hp() > opening.sim.player.max_hp());
    }
}
