use super::*;
use std::collections::{HashMap, HashSet};

use crate::content::{MonsterKind, MonsterRank, monster_xp, roll_item};

fn test_monster(kind: MonsterKind, pos: Vec2) -> Monster {
    Monster {
        kind,
        rank: MonsterRank::Normal,
        pack_id: 0,
        pack_center: pos,
        pos,
        vel: Vec2::ZERO,
        hit_offset: Vec2::ZERO,
        hp: 80.0,
        max_hp: 80.0,
        level: 1,
        attack_cd: 0.0,
        wobble: 0.0,
        hit_flash: 0.0,
        chill_ttl: 0.0,
    }
}

#[test]
fn leveling_grants_only_stat_points() {
    let mut game = Game::new(1);
    game.player.stats.xp =
        game.player.stats.next_xp - monster_xp(MonsterKind::Imp, 1, MonsterRank::Normal);
    game.on_monster_killed(Monster {
        kind: MonsterKind::Imp,
        rank: MonsterRank::Normal,
        pack_id: 0,
        pack_center: game.player.pos + vec2(40.0, 0.0),
        pos: game.player.pos + vec2(40.0, 0.0),
        vel: Vec2::ZERO,
        hit_offset: Vec2::ZERO,
        hp: 0.0,
        max_hp: MonsterKind::Imp.max_hp(),
        level: 1,
        attack_cd: 0.0,
        wobble: 0.0,
        hit_flash: 0.0,
        chill_ttl: 0.0,
    });

    assert_eq!(game.player.stats.level, 2);
    assert_eq!(game.player.stats.unspent_stat_points, 3);
}

#[test]
fn travel_destinations_reach_progressively_harder_biomes() {
    let world = World::new(1);
    let levels: Vec<i32> = TRAVEL_DESTINATIONS
        .iter()
        .map(|destination| world.biome_level(World::tile_center(destination.pos)))
        .collect();

    assert_eq!(levels, vec![0, 1, 2, 3, 4]);
}

#[test]
fn new_game_starts_in_town() {
    let game = Game::new(1);
    assert_eq!(game.world.biome_level(game.player.pos), 0);
    assert!(!game.known_tiles.is_empty());
}

#[test]
fn walking_far_from_town_repopulates_monsters_around_the_player() {
    let mut game = Game::new(1);
    game.player.pos = World::tile_center(ivec2(220, 0));

    game.fixed_update(FIXED_DT);

    let nearby_packs: HashSet<u64> = game
        .monsters
        .iter()
        .filter(|monster| monster.pack_center.distance(game.player.pos) <= MONSTER_LOCAL_RADIUS)
        .map(|monster| monster.pack_id)
        .collect();
    assert_eq!(nearby_packs.len(), MONSTER_LOCAL_PACK_TARGET);
    assert!(
        game.monsters
            .iter()
            .all(|monster| monster.pack_center.distance(game.player.pos) <= MONSTER_DESPAWN_RADIUS)
    );
    assert!(
        game.monsters
            .iter()
            .all(|monster| monster.level >= game.world.biome_level(game.player.pos) - 2)
    );
}

#[test]
fn spawned_monster_packs_are_homogeneous_and_clustered() {
    let mut game = Game::new(2);
    game.monsters.clear();
    game.next_monster_pack_id = 0;
    game.player.pos = World::tile_center(ivec2(220, 0));
    game.spawn_monster_packs(MONSTER_LOCAL_PACK_TARGET);

    let mut packs: HashMap<u64, Vec<&Monster>> = HashMap::new();
    for monster in &game.monsters {
        packs.entry(monster.pack_id).or_default().push(monster);
    }

    assert_eq!(packs.len(), MONSTER_LOCAL_PACK_TARGET);
    for pack in packs.values() {
        assert!((2..=7).contains(&pack.len()));
        assert!(pack.iter().all(|monster| monster.kind == pack[0].kind));
        assert!(
            pack.iter()
                .all(|monster| monster.pack_center == pack[0].pack_center)
        );
        assert!(pack.iter().all(|monster| {
            let distance = monster.pos.distance(monster.pack_center);
            (MONSTER_PACK_MEMBER_MIN_DISTANCE..=MONSTER_PACK_MEMBER_MAX_DISTANCE)
                .contains(&distance)
        }));
        assert!(
            pack.iter()
                .filter(|monster| monster.rank != MonsterRank::Normal)
                .count()
                <= 1
        );
    }
}

#[test]
fn spawned_pack_centers_keep_breathing_room() {
    let mut game = Game::new(3);
    game.monsters.clear();
    game.next_monster_pack_id = 0;
    game.player.pos = World::tile_center(ivec2(220, 0));
    game.spawn_monster_packs(MONSTER_LOCAL_PACK_TARGET);

    let mut centers_by_pack = HashMap::new();
    for monster in &game.monsters {
        centers_by_pack
            .entry(monster.pack_id)
            .or_insert(monster.pack_center);
    }
    let centers: Vec<Vec2> = centers_by_pack.into_values().collect();

    for (index, center) in centers.iter().enumerate() {
        for other in centers.iter().skip(index + 1) {
            assert!(center.distance(*other) >= MONSTER_PACK_SEPARATION);
        }
    }
}

#[test]
fn boss_pack_rolls_take_priority_over_elites() {
    assert_eq!(
        super::spawning::monster_pack_rank_for_roll(0),
        MonsterRank::Boss
    );
    assert_eq!(
        super::spawning::monster_pack_rank_for_roll(1),
        MonsterRank::Boss
    );
    assert_eq!(
        super::spawning::monster_pack_rank_for_roll(2),
        MonsterRank::Elite
    );
    assert_eq!(
        super::spawning::monster_pack_rank_for_roll(12),
        MonsterRank::Normal
    );
}

#[test]
fn pack_spawn_visibility_accounts_for_the_full_cluster() {
    let player = vec2(0.0, 0.0);
    let half_view = vec2(640.0, 380.0);

    assert!(super::spawning::pack_center_can_be_seen_with_view(
        vec2(700.0, 0.0),
        player,
        half_view
    ));
    assert!(!super::spawning::pack_center_can_be_seen_with_view(
        vec2(740.0, 0.0),
        player,
        half_view
    ));
    assert!(!super::spawning::pack_center_can_be_seen_with_view(
        vec2(0.0, 500.0),
        player,
        half_view
    ));
}

#[test]
fn world_map_tracks_discovery_and_supports_navigation() {
    let mut game = Game::new(1);
    let known_at_start = game.known_tiles.len();
    game.player.pos = World::tile_center(ivec2(32, 0));
    game.fixed_update(FIXED_DT);
    assert!(game.known_tiles.len() > known_at_start);

    game.input.world_map_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui_mode, UiMode::WorldMap);
    assert_eq!(game.world_map.center_tile, vec2(32.0, 0.0));

    game.input.movement = Vec2::X;
    game.input.map_zoom_delta = 1.0;
    let zoom_before = game.world_map.zoom;
    game.fixed_update(FIXED_DT);
    assert!(game.world_map.center_tile.x > 32.0);
    assert!(game.world_map.zoom > zoom_before);

    game.input.map_recenter_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.world_map.center_tile, vec2(32.0, 0.0));
}

#[test]
fn hovered_monster_prefers_the_enemy_under_the_cursor() {
    let mut game = Game::new(1);
    game.monsters = vec![
        Monster {
            kind: MonsterKind::Imp,
            rank: MonsterRank::Normal,
            pack_id: 0,
            pack_center: game.player.pos + vec2(20.0, 0.0),
            pos: game.player.pos + vec2(20.0, 0.0),
            vel: Vec2::ZERO,
            hit_offset: Vec2::ZERO,
            hp: 12.0,
            max_hp: 24.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
            hit_flash: 0.0,
            chill_ttl: 0.0,
        },
        Monster {
            kind: MonsterKind::Brute,
            rank: MonsterRank::Normal,
            pack_id: 1,
            pack_center: game.player.pos + vec2(24.0, 0.0),
            pos: game.player.pos + vec2(24.0, 0.0),
            vel: Vec2::ZERO,
            hit_offset: Vec2::ZERO,
            hp: 48.0,
            max_hp: 62.0,
            level: 2,
            attack_cd: 0.0,
            wobble: 0.0,
            hit_flash: 0.0,
            chill_ttl: 0.0,
        },
    ];
    game.input.aim_world = game.player.pos + vec2(24.0, 0.0);

    assert_eq!(
        game.hovered_monster().map(|monster| monster.kind),
        Some(MonsterKind::Brute)
    );
}

#[test]
fn combat_feed_scrolls_through_older_entries() {
    let mut game = Game::new(1);
    game.log = (0..10).map(|index| format!("Entry {}", index)).collect();
    game.input.log_scroll_delta = 2;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.log_scroll_offset, 2);

    game.input.log_scroll_delta = -1;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.log_scroll_offset, 1);

    game.input.log_scroll_delta = 99;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.log_scroll_offset, 4);
}

#[test]
fn equipping_moves_item_into_matching_slot() {
    let mut game = Game::new(1);
    game.inventory_cursor = 0;
    game.equip_selected_item();

    assert!(game.player.equipment.weapon.is_some());
    assert_eq!(game.player.inventory.len(), 1);
}

#[test]
fn shop_buy_and_sell_round_trip_updates_gold_and_inventory() {
    let mut game = Game::new(1);
    game.player.stats.gold = 100;
    game.shop_cursor = 0;
    let starting_inventory = game.player.inventory.len();

    game.buy_selected_item();
    assert_eq!(game.player.inventory.len(), starting_inventory + 1);
    assert_eq!(game.player.stats.gold, 76);

    game.shop_tab = ShopTab::Sell;
    game.shop_cursor = game.player.inventory.len() - 1;
    game.sell_selected_item();
    assert_eq!(game.player.inventory.len(), starting_inventory);
    assert_eq!(game.player.stats.gold, 90);
}

#[test]
fn gameplay_smoke_flow_reaches_combat_loot_shop_and_travel() {
    let mut game = Game::new(1);
    game.monsters.clear();
    game.player.facing = Vec2::X;
    game.player.stats.xp =
        game.player.stats.next_xp - monster_xp(MonsterKind::Imp, 1, MonsterRank::Normal);
    game.monsters.push(Monster {
        kind: MonsterKind::Imp,
        rank: MonsterRank::Normal,
        pack_id: 0,
        pack_center: game.player.pos + vec2(32.0, 0.0),
        pos: game.player.pos + vec2(32.0, 0.0),
        vel: Vec2::ZERO,
        hit_offset: Vec2::ZERO,
        hp: 1.0,
        max_hp: 1.0,
        level: 1,
        attack_cd: 0.0,
        wobble: 0.0,
        hit_flash: 0.0,
        chill_ttl: 0.0,
    });

    game.basic_attack();
    assert!(game.monsters.is_empty());
    assert!(game.floating.iter().any(|text| text.text.starts_with('-')));
    assert!(game.floating.iter().any(|text| text.text.contains("xp")));
    assert_eq!(game.player.stats.level, 2);

    game.loot.push(Loot {
        pos: game.player.pos,
        item: roll_item(&mut game.rng, 1),
        bob: 0.0,
    });
    let inventory_before_loot = game.player.inventory.len();
    game.pickup_loot();
    assert_eq!(game.player.inventory.len(), inventory_before_loot + 1);

    game.inventory_cursor = 0;
    game.equip_selected_item();
    assert!(game.player.equipment.weapon.is_some());

    game.player.pos = game
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Merchant)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui_mode, UiMode::Merchant);

    game.player.stats.gold = 100;
    game.buy_selected_item();
    assert!(game.player.inventory.len() >= 2);

    game.ui_mode = UiMode::None;
    game.player.pos = game
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Wayfinder)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui_mode, UiMode::Travel);
    game.travel_cursor = 4;
    game.input.inventory_equip_pressed = true;
    game.update_travel_controls();
    assert_eq!(game.world.biome_level(game.player.pos), 4);
}

#[test]
fn fireball_explodes_and_hits_nearby_monsters() {
    let mut game = Game::new(7);
    game.monsters = vec![
        Monster {
            kind: MonsterKind::Imp,
            rank: MonsterRank::Normal,
            pack_id: 0,
            pack_center: game.player.pos + vec2(34.0, 0.0),
            pos: game.player.pos + vec2(34.0, 0.0),
            vel: Vec2::ZERO,
            hit_offset: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
            hit_flash: 0.0,
            chill_ttl: 0.0,
        },
        Monster {
            kind: MonsterKind::Slime,
            rank: MonsterRank::Normal,
            pack_id: 1,
            pack_center: game.player.pos + vec2(48.0, 10.0),
            pos: game.player.pos + vec2(48.0, 10.0),
            vel: Vec2::ZERO,
            hit_offset: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
            hit_flash: 0.0,
            chill_ttl: 0.0,
        },
        Monster {
            kind: MonsterKind::Brute,
            rank: MonsterRank::Normal,
            pack_id: 2,
            pack_center: game.player.pos + vec2(120.0, 0.0),
            pos: game.player.pos + vec2(120.0, 0.0),
            vel: Vec2::ZERO,
            hit_offset: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
            hit_flash: 0.0,
            chill_ttl: 0.0,
        },
    ];
    game.player.facing = Vec2::X;

    game.cast_ability(AbilityKind::Fireball);
    game.update_projectiles(FIXED_DT);

    assert!(game.projectiles.is_empty());
    assert!(game.monsters[0].hp < 80.0);
    assert!(game.monsters[1].hp < 80.0);
    assert_eq!(game.monsters[2].hp, 80.0);
    assert_eq!(game.player.disciplines.magic.xp, 4);
}

#[test]
fn basic_attack_spawns_a_short_slash_arc() {
    let mut game = Game::new(10);
    game.player.facing = Vec2::X;

    game.basic_attack();

    assert_eq!(game.slash_arcs.len(), 1);
    assert_eq!(game.slash_arcs[0].direction, Vec2::X);
    assert_eq!(game.slash_arcs[0].radius, 34.0);
    assert_eq!(game.slash_arcs[0].ttl, 0.16);
    assert_eq!(game.player.disciplines.melee.xp, 0);
}

#[test]
fn hitting_monster_sets_flash_and_recoil() {
    let mut game = Game::new(12);
    game.player.facing = Vec2::X;
    game.monsters = vec![Monster {
        kind: MonsterKind::Imp,
        rank: MonsterRank::Normal,
        pack_id: 0,
        pack_center: game.player.pos + vec2(32.0, 0.0),
        pos: game.player.pos + vec2(32.0, 0.0),
        vel: Vec2::ZERO,
        hit_offset: Vec2::ZERO,
        hp: 80.0,
        max_hp: 80.0,
        level: 1,
        attack_cd: 0.0,
        wobble: 0.0,
        hit_flash: 0.0,
        chill_ttl: 0.0,
    }];

    game.basic_attack();

    assert!(game.monsters[0].hit_flash > 0.0);
    assert!(game.monsters[0].hit_offset.x > 0.0);
    assert_eq!(game.player.disciplines.melee.xp, 2);
}

#[test]
fn cleave_hits_front_arc_without_hitting_behind() {
    let mut game = Game::new(8);
    game.monsters = vec![
        Monster {
            kind: MonsterKind::Imp,
            rank: MonsterRank::Normal,
            pack_id: 0,
            pack_center: game.player.pos + vec2(36.0, 0.0),
            pos: game.player.pos + vec2(36.0, 0.0),
            vel: Vec2::ZERO,
            hit_offset: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
            hit_flash: 0.0,
            chill_ttl: 0.0,
        },
        Monster {
            kind: MonsterKind::Slime,
            rank: MonsterRank::Normal,
            pack_id: 1,
            pack_center: game.player.pos + vec2(-36.0, 0.0),
            pos: game.player.pos + vec2(-36.0, 0.0),
            vel: Vec2::ZERO,
            hit_offset: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
            hit_flash: 0.0,
            chill_ttl: 0.0,
        },
    ];
    game.player.facing = Vec2::X;

    game.cast_ability(AbilityKind::Cleave);

    assert!(game.monsters[0].hp < 80.0);
    assert_eq!(game.monsters[1].hp, 80.0);
}

#[test]
fn new_game_starts_with_only_starter_abilities_bound() {
    let mut game = Game::new(9);
    assert_eq!(
        game.player.bound_abilities,
        [AbilityKind::Cleave, AbilityKind::Fireball]
    );
    assert!(game.player.is_ability_unlocked(AbilityKind::Cleave));
    assert!(game.player.is_ability_unlocked(AbilityKind::Fireball));
    assert!(!game.player.is_ability_unlocked(AbilityKind::Rush));
    assert!(!game.player.is_ability_unlocked(AbilityKind::Nova));

    game.cast_ability(AbilityKind::Rush);
    game.cast_ability(AbilityKind::Nova);
    assert_eq!(
        game.player.ability_cooldowns[AbilityKind::Rush.index()],
        0.0
    );
    assert_eq!(
        game.player.ability_cooldowns[AbilityKind::Nova.index()],
        0.0
    );
}

#[test]
fn discipline_levels_unlock_combat_abilities() {
    let mut game = Game::new(13);

    game.award_discipline_xp(DisciplineKind::Melee, 24);
    game.award_discipline_xp(DisciplineKind::Magic, 24);

    assert!(game.player.is_ability_unlocked(AbilityKind::Rush));
    assert!(game.player.is_ability_unlocked(AbilityKind::Nova));
    assert!(
        game.log
            .iter()
            .any(|line| line.contains("Melee unlocks Rush"))
    );
    assert!(
        game.log
            .iter()
            .any(|line| line.contains("Magic unlocks Nova"))
    );
}

#[test]
fn discipline_xp_can_cross_multiple_levels_at_once() {
    let mut game = Game::new(17);

    game.award_discipline_xp(DisciplineKind::Melee, 100);

    assert_eq!(game.player.disciplines.melee.level, 3);
    assert_eq!(game.player.disciplines.melee.xp, 36);
    assert_eq!(game.player.disciplines.melee.next_xp, 88);
}

#[test]
fn binding_skills_replaces_and_swaps_slots_without_duplicates() {
    let mut game = Game::new(18);
    game.award_discipline_xp(DisciplineKind::Melee, 24);

    game.bind_ability(0, AbilityKind::Rush);
    assert_eq!(
        game.player.bound_abilities,
        [AbilityKind::Rush, AbilityKind::Fireball]
    );

    game.bind_ability(1, AbilityKind::Rush);
    assert_eq!(
        game.player.bound_abilities,
        [AbilityKind::Fireball, AbilityKind::Rush]
    );
}

#[test]
fn locked_skills_cannot_be_bound() {
    let mut game = Game::new(24);

    game.bind_ability(0, AbilityKind::Meteor);

    assert_eq!(
        game.player.bound_abilities,
        [AbilityKind::Cleave, AbilityKind::Fireball]
    );
}

#[test]
fn skill_book_navigation_moves_between_columns_and_prefers_useful_skills() {
    let mut game = Game::new(25);
    game.award_discipline_xp(DisciplineKind::Melee, 24);

    assert_eq!(game.skill_book_focus, SkillBookFocus::Disciplines);
    game.input.nav_right_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.skill_book_focus, SkillBookFocus::Skills);

    game.input = InputState::default();
    game.input.inventory_down_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.skill_book_ability_cursor, 1);

    game.input = InputState::default();
    game.input.nav_right_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.skill_book_focus, SkillBookFocus::Detail);

    game.input = InputState::default();
    game.input.nav_left_pressed = true;
    game.update_skill_book_controls();
    game.input = InputState::default();
    game.input.nav_left_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.skill_book_focus, SkillBookFocus::Disciplines);

    game.skill_book_cursor = 1;
    game.skill_book_ability_cursor = 0;
    game.input = InputState::default();
    game.input.inventory_up_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.skill_book_cursor, 0);
    assert_eq!(game.skill_book_ability_cursor, 1);
}

#[test]
fn melee_and_magic_unlocks_reach_levels_four_and_eight() {
    let mut game = Game::new(19);
    game.award_discipline_xp(DisciplineKind::Melee, 2_000);
    game.award_discipline_xp(DisciplineKind::Magic, 2_000);

    assert!(game.player.is_ability_unlocked(AbilityKind::Whirlwind));
    assert!(game.player.is_ability_unlocked(AbilityKind::Execute));
    assert!(game.player.is_ability_unlocked(AbilityKind::IceBolt));
    assert!(game.player.is_ability_unlocked(AbilityKind::Meteor));
}

#[test]
fn whirlwind_hits_nearby_enemies() {
    let mut game = Game::new(20);
    game.player.disciplines.melee.level = 4;
    game.monsters = vec![
        test_monster(MonsterKind::Imp, game.player.pos + vec2(24.0, 0.0)),
        test_monster(MonsterKind::Slime, game.player.pos + vec2(-24.0, 0.0)),
    ];

    game.cast_ability(AbilityKind::Whirlwind);

    assert!(game.monsters.iter().all(|monster| monster.hp < 80.0));
}

#[test]
fn execute_hits_harder_against_wounded_targets() {
    let mut game = Game::new(21);
    game.player.disciplines.melee.level = 8;
    game.player.facing = Vec2::X;
    game.monsters = vec![test_monster(
        MonsterKind::Imp,
        game.player.pos + vec2(32.0, 0.0),
    )];
    game.monsters[0].hp = 30.0;

    game.cast_ability(AbilityKind::Execute);

    assert!(game.monsters.is_empty() || game.monsters[0].hp < 12.0);
}

#[test]
fn ice_bolt_chills_the_first_enemy_hit() {
    let mut game = Game::new(22);
    game.player.disciplines.magic.level = 4;
    game.player.facing = Vec2::X;
    game.monsters = vec![test_monster(
        MonsterKind::Imp,
        game.player.pos + vec2(32.0, 0.0),
    )];

    game.cast_ability(AbilityKind::IceBolt);
    game.update_projectiles(FIXED_DT);

    assert!(game.monsters[0].chill_ttl > 0.0);
}

#[test]
fn meteor_waits_then_hits_its_target_area() {
    let mut game = Game::new(23);
    game.player.disciplines.magic.level = 8;
    game.input.aim_world = game.player.pos + vec2(40.0, 0.0);
    game.monsters = vec![test_monster(MonsterKind::Imp, game.input.aim_world)];

    game.cast_ability(AbilityKind::Meteor);
    assert_eq!(game.meteors.len(), 1);
    assert_eq!(game.monsters[0].hp, 80.0);

    game.update_meteors(0.73);

    assert!(game.meteors.is_empty());
    assert!(game.monsters.is_empty() || game.monsters[0].hp < 80.0);
}

#[test]
fn armor_mastery_gains_xp_when_damage_is_mitigated() {
    let mut game = Game::new(14);
    game.monsters = vec![Monster {
        kind: MonsterKind::Imp,
        rank: MonsterRank::Normal,
        pack_id: 0,
        pack_center: game.player.pos + vec2(12.0, 0.0),
        pos: game.player.pos + vec2(12.0, 0.0),
        vel: Vec2::ZERO,
        hit_offset: Vec2::ZERO,
        hp: 24.0,
        max_hp: 24.0,
        level: 1,
        attack_cd: 0.0,
        wobble: 0.0,
        hit_flash: 0.0,
        chill_ttl: 0.0,
    }];

    game.update_monsters(FIXED_DT);

    assert_eq!(game.player.disciplines.armor.xp, 2);
}

#[test]
fn agility_mastery_gains_xp_from_actual_travel() {
    let mut game = Game::new(15);
    game.agility_distance_bank = 143.9;
    game.input.movement = Vec2::X;

    game.fixed_update(FIXED_DT);

    assert_eq!(game.player.disciplines.agility.xp, 1);
}

#[test]
fn mastery_levels_improve_combat_and_movement_values() {
    let mut game = Game::new(16);
    let base_armor = game.player.armor();
    let base_speed = game.player.move_speed();
    let base_mana_regen = game.player.mana_regen_rate();

    game.award_discipline_xp(DisciplineKind::Melee, 24);
    game.award_discipline_xp(DisciplineKind::Magic, 24);
    game.award_discipline_xp(DisciplineKind::Armor, 24);
    game.award_discipline_xp(DisciplineKind::Agility, 24);

    assert_eq!(game.player.melee_damage_bonus(), 2);
    assert_eq!(game.player.magic_damage_bonus(), 2);
    assert_eq!(game.player.mana_regen_rate(), base_mana_regen + 0.5);
    assert_eq!(game.player.armor(), base_armor + 1);
    assert_eq!(game.player.move_speed(), base_speed + 6.0);
}

#[test]
fn mana_regenerates_more_slowly_at_first_and_scales_with_magic_mastery() {
    let mut novice = Game::new(26);
    novice.player.mana = 0.0;
    novice.fixed_update(1.0);
    assert_eq!(novice.player.mana, 3.0);

    let mut mage = Game::new(27);
    mage.player.mana = 0.0;
    mage.player.disciplines.magic.level = 4;
    mage.fixed_update(1.0);
    assert_eq!(mage.player.mana, 4.5);
}

#[test]
fn every_ui_window_supports_basic_navigation() {
    let mut game = Game::new(3);

    game.input.inventory_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui_mode, UiMode::Inventory);
    game.input.inventory_down_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.inventory_cursor, 1);

    game.ui_mode = UiMode::None;
    game.input = InputState::default();
    game.input.character_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui_mode, UiMode::Character);
    game.input.inventory_down_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.character_cursor, 1);

    game.ui_mode = UiMode::None;
    game.input = InputState::default();
    game.input.skill_book_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui_mode, UiMode::SkillBook);
    game.input.nav_right_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.skill_book_focus, SkillBookFocus::Skills);

    game.ui_mode = UiMode::None;
    game.input = InputState::default();
    game.input.world_map_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui_mode, UiMode::WorldMap);

    game.ui_mode = UiMode::None;
    game.player.pos = game
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Merchant)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui_mode, UiMode::Merchant);
    game.input = InputState::default();
    game.input.nav_right_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.shop_tab, ShopTab::Sell);

    game.ui_mode = UiMode::None;
    game.player.pos = game
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Trainer)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui_mode, UiMode::Trainer);
    game.input = InputState::default();
    game.input.inventory_equip_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui_mode, UiMode::SkillBook);

    game.ui_mode = UiMode::None;
    game.player.pos = game
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Wayfinder)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui_mode, UiMode::Travel);
    game.input = InputState::default();
    game.input.inventory_down_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.travel_cursor, 1);
}
