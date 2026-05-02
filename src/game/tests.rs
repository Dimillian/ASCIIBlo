use super::*;
use crate::content::{MonsterKind, monster_xp, roll_item};

#[test]
fn leveling_grants_spendable_points() {
    let mut game = Game::new(1);
    game.player.stats.xp = game.player.stats.next_xp - monster_xp(MonsterKind::Imp, 1);
    game.on_monster_killed(Monster {
        kind: MonsterKind::Imp,
        pos: game.player.pos + vec2(40.0, 0.0),
        vel: Vec2::ZERO,
        hp: 0.0,
        max_hp: MonsterKind::Imp.max_hp(),
        level: 1,
        attack_cd: 0.0,
        wobble: 0.0,
    });

    assert_eq!(game.player.stats.level, 2);
    assert_eq!(game.player.stats.unspent_stat_points, 3);
    assert_eq!(game.player.stats.unspent_skill_points, 1);
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

    assert_eq!(
        game.monsters
            .iter()
            .filter(|monster| monster.pos.distance(game.player.pos) <= MONSTER_LOCAL_RADIUS)
            .count(),
        48
    );
    assert!(
        game.monsters
            .iter()
            .all(|monster| monster.pos.distance(game.player.pos) <= MONSTER_DESPAWN_RADIUS)
    );
    assert!(
        game.monsters
            .iter()
            .all(|monster| monster.level >= game.world.biome_level(game.player.pos) - 2)
    );
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
            pos: game.player.pos + vec2(20.0, 0.0),
            vel: Vec2::ZERO,
            hp: 12.0,
            max_hp: 24.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
        },
        Monster {
            kind: MonsterKind::Brute,
            pos: game.player.pos + vec2(24.0, 0.0),
            vel: Vec2::ZERO,
            hp: 48.0,
            max_hp: 62.0,
            level: 2,
            attack_cd: 0.0,
            wobble: 0.0,
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
    game.player.stats.xp = game.player.stats.next_xp - monster_xp(MonsterKind::Imp, 1);
    game.monsters.push(Monster {
        kind: MonsterKind::Imp,
        pos: game.player.pos + vec2(32.0, 0.0),
        vel: Vec2::ZERO,
        hp: 1.0,
        max_hp: 1.0,
        level: 1,
        attack_cd: 0.0,
        wobble: 0.0,
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
            pos: game.player.pos + vec2(34.0, 0.0),
            vel: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
        },
        Monster {
            kind: MonsterKind::Slime,
            pos: game.player.pos + vec2(48.0, 10.0),
            vel: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
        },
        Monster {
            kind: MonsterKind::Brute,
            pos: game.player.pos + vec2(120.0, 0.0),
            vel: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
        },
    ];
    game.player.facing = Vec2::X;

    game.cast_fireball();
    game.update_projectiles(FIXED_DT);

    assert!(game.projectiles.is_empty());
    assert!(game.monsters[0].hp < 80.0);
    assert!(game.monsters[1].hp < 80.0);
    assert_eq!(game.monsters[2].hp, 80.0);
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
}

#[test]
fn cleave_hits_front_arc_without_hitting_behind() {
    let mut game = Game::new(8);
    game.monsters = vec![
        Monster {
            kind: MonsterKind::Imp,
            pos: game.player.pos + vec2(36.0, 0.0),
            vel: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
        },
        Monster {
            kind: MonsterKind::Slime,
            pos: game.player.pos + vec2(-36.0, 0.0),
            vel: Vec2::ZERO,
            hp: 80.0,
            max_hp: 80.0,
            level: 1,
            attack_cd: 0.0,
            wobble: 0.0,
        },
    ];
    game.player.facing = Vec2::X;

    game.cast_cleave();

    assert!(game.monsters[0].hp < 80.0);
    assert_eq!(game.monsters[1].hp, 80.0);
}

#[test]
fn skill_book_spends_points_on_selected_skill() {
    let mut game = Game::new(9);
    game.player.stats.unspent_skill_points = 1;
    game.skill_book_cursor = 2;
    let starting_rank = game.player.fireball_rank;
    game.input.inventory_equip_pressed = true;

    game.update_skill_book_controls();

    assert_eq!(game.player.fireball_rank, starting_rank + 1);
    assert_eq!(game.player.stats.unspent_skill_points, 0);
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
    game.input.inventory_down_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.skill_book_cursor, 1);

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
