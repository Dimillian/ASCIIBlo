use super::*;
use std::collections::{HashMap, HashSet};

use crate::{
    content::{MonsterKind, MonsterRank, monster_xp, roll_item},
    stat_display::item_summary,
};

fn test_monster(kind: MonsterKind, pos: Vec2) -> Monster {
    Monster {
        kind,
        rank: MonsterRank::Normal,
        quest_id: None,
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

fn discover_towns(game: &mut Game, count: usize) {
    let sites = game.world.settlements_near_tile(ivec2(0, 0), 1_200);
    for site in sites
        .into_iter()
        .filter(|site| site.tier == crate::world::SettlementTier::Town)
    {
        game.reveal_around_tile(site.center, 1);
        if game.sim.travel_destinations.len() >= count {
            break;
        }
    }
}

fn origin_town(game: &Game) -> crate::world::SettlementSite {
    game.world
        .settlements_near_tile(ivec2(0, 0), 1)
        .into_iter()
        .find(|site| site.is_origin())
        .unwrap()
}

#[test]
fn leveling_grants_only_stat_points() {
    let mut game = Game::new(1);
    game.sim.player.stats.xp =
        game.sim.player.stats.next_xp - monster_xp(MonsterKind::Imp, 1, MonsterRank::Normal);
    game.on_monster_killed(Monster {
        kind: MonsterKind::Imp,
        rank: MonsterRank::Normal,
        quest_id: None,
        pack_id: 0,
        pack_center: game.sim.player.pos + vec2(40.0, 0.0),
        pos: game.sim.player.pos + vec2(40.0, 0.0),
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

    assert_eq!(game.sim.player.stats.level, 2);
    assert_eq!(game.sim.player.stats.unspent_stat_points, 3);
}

#[test]
fn travel_destinations_reach_progressively_harder_biomes() {
    let mut game = Game::new(1);
    discover_towns(&mut game, 5);
    let levels: Vec<i32> = game
        .sim
        .travel_destinations
        .iter()
        .map(|destination| destination.min_level)
        .collect();

    assert!(levels.len() >= 5);
    assert!(levels.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(levels[0], 0);
}

#[test]
fn towns_unlock_waypoints_while_villages_do_not() {
    let mut game = Game::new(9);
    let sites = game.world.settlements_near_tile(ivec2(0, 0), 1_200);
    let town = sites
        .iter()
        .copied()
        .find(|site| !site.is_origin() && site.tier == crate::world::SettlementTier::Town)
        .unwrap();
    let village = sites
        .iter()
        .copied()
        .find(|site| site.tier == crate::world::SettlementTier::Village)
        .unwrap();

    let initial_destinations = game.sim.travel_destinations.len();
    game.reveal_around_tile(village.center, 1);
    assert_eq!(game.sim.travel_destinations.len(), initial_destinations);

    game.reveal_around_tile(town.center, 1);
    assert_eq!(game.sim.travel_destinations.len(), initial_destinations + 1);
}

#[test]
fn new_game_starts_in_town() {
    let game = Game::new(1);
    assert_eq!(game.world.biome_level(game.sim.player.pos), 0);
    assert!(!game.sim.known_tiles.is_empty());
}

#[test]
fn bounty_boards_generate_one_valid_active_quest_from_origin_and_later_towns() {
    let mut game = Game::new(1);
    let origin = origin_town(&game);
    game.sim.player.pos = Game::quest_board_pos(origin);
    assert!(game.interact_with_nearby_quest_board());
    let quest = game.sim.active_quest.clone().unwrap();
    assert!(quest.goal > 0);
    let first_id = quest.id;
    assert!(game.interact_with_nearby_quest_board());
    assert_eq!(game.sim.active_quest.as_ref().unwrap().id, first_id);

    let mut later_game = Game::new(1);
    let later_town = later_game
        .world
        .settlements_near_tile(ivec2(0, 0), 1_200)
        .into_iter()
        .find(|site| !site.is_origin() && site.tier == crate::world::SettlementTier::Town)
        .unwrap();
    later_game.sim.player.pos = Game::quest_board_pos(later_town);
    assert!(later_game.interact_with_nearby_quest_board());
    assert!(later_game.sim.active_quest.is_some());
}

#[test]
fn each_quest_archetype_spawns_concrete_targets() {
    let mut kill_game = Game::new(2);
    let giver = origin_town(&kill_game);
    let kill = kill_game.generate_kill_pack_quest(giver).unwrap();
    assert_eq!(kill.kind, QuestKind::KillPack);
    assert_eq!(
        kill_game
            .sim
            .monsters
            .iter()
            .filter(|monster| monster.quest_id == Some(kill.id))
            .count(),
        kill.goal
    );

    let mut bounty_game = Game::new(3);
    let bounty = bounty_game
        .generate_bounty_quest(origin_town(&bounty_game))
        .unwrap();
    assert_eq!(bounty.kind, QuestKind::BountyBoss);
    assert!(bounty_game.sim.monsters.iter().any(|monster| {
        monster.quest_id == Some(bounty.id) && monster.rank == MonsterRank::Boss
    }));

    let mut meet_game = Game::new(1);
    let meet = meet_game
        .generate_meet_npc_quest(origin_town(&meet_game))
        .unwrap();
    assert_eq!(meet.kind, QuestKind::MeetNpc);
    assert!(meet_game.sim.npcs.iter().any(|npc| {
        npc.quest_id == Some(meet.id) && npc.kind == crate::content::NpcKind::QuestContact
    }));

    let mut recover_game = Game::new(1);
    let recover = recover_game
        .generate_recovery_quest(origin_town(&recover_game))
        .unwrap();
    assert_eq!(recover.kind, QuestKind::RecoverItems);
    assert_eq!(
        recover_game
            .sim
            .quest_items
            .iter()
            .filter(|item| item.quest_id == recover.id)
            .count(),
        recover.goal
    );
    assert!(
        recover_game
            .sim
            .quest_items
            .iter()
            .all(|item| { recover_game.world.walkable_at_world(item.pos) })
    );
}

#[test]
fn kill_and_bounty_quests_only_advance_from_tagged_targets() {
    let mut kill_game = Game::new(2);
    let kill = kill_game
        .generate_kill_pack_quest(origin_town(&kill_game))
        .unwrap();
    kill_game.sim.active_quest = Some(kill.clone());
    kill_game.on_monster_killed(test_monster(MonsterKind::Imp, kill.target_pos));
    assert_eq!(kill_game.sim.active_quest.as_ref().unwrap().progress, 0);
    for monster in kill_game
        .sim
        .monsters
        .iter()
        .filter(|monster| monster.quest_id == Some(kill.id))
        .cloned()
        .collect::<Vec<_>>()
    {
        kill_game.on_monster_killed(monster);
    }
    assert!(kill_game.sim.active_quest.as_ref().unwrap().is_ready());

    let mut bounty_game = Game::new(3);
    let bounty = bounty_game
        .generate_bounty_quest(origin_town(&bounty_game))
        .unwrap();
    bounty_game.sim.active_quest = Some(bounty.clone());
    let boss = bounty_game
        .sim
        .monsters
        .iter()
        .find(|monster| monster.quest_id == Some(bounty.id))
        .unwrap()
        .clone();
    bounty_game.on_monster_killed(boss);
    assert!(bounty_game.sim.active_quest.as_ref().unwrap().is_ready());
}

#[test]
fn meet_and_recovery_quests_complete_through_their_dedicated_interactions() {
    let mut meet_game = Game::new(1);
    let meet = meet_game
        .generate_meet_npc_quest(origin_town(&meet_game))
        .unwrap();
    meet_game.sim.active_quest = Some(meet.clone());
    let npc = meet_game
        .sim
        .npcs
        .iter()
        .find(|npc| npc.quest_id == Some(meet.id))
        .unwrap()
        .clone();
    assert!(meet_game.interact_with_quest_contact(&npc));
    assert!(meet_game.sim.active_quest.as_ref().unwrap().is_ready());

    let mut recover_game = Game::new(1);
    let recover = recover_game
        .generate_recovery_quest(origin_town(&recover_game))
        .unwrap();
    recover_game.sim.active_quest = Some(recover.clone());
    let item_positions = recover_game
        .sim
        .quest_items
        .iter()
        .map(|item| item.pos)
        .collect::<Vec<_>>();
    for pos in item_positions {
        recover_game.sim.player.pos = pos;
        assert!(recover_game.pickup_nearby_quest_item());
    }
    assert!(recover_game.sim.active_quest.as_ref().unwrap().is_ready());
}

#[test]
fn completed_quests_retarget_to_the_board_and_clean_up_on_turn_in() {
    let mut game = Game::new(2);
    let giver = origin_town(&game);
    let quest = game.generate_kill_pack_quest(giver).unwrap();
    game.sim.active_quest = Some(quest.clone());
    let monsters = game
        .sim
        .monsters
        .iter()
        .filter(|monster| monster.quest_id == Some(quest.id))
        .cloned()
        .collect::<Vec<_>>();
    for monster in monsters {
        game.on_monster_killed(monster);
    }
    assert_eq!(
        game.quest_navigation_target(),
        Some(Game::quest_board_pos(giver))
    );
    let gold_before = game.sim.player.stats.gold;
    let xp_before = game.sim.player.stats.xp;
    let level_before = game.sim.player.stats.level;
    game.sim.player.pos = Game::quest_board_pos(giver);
    assert!(game.interact_with_nearby_quest_board());
    assert!(game.sim.active_quest.is_none());
    assert!(game.sim.player.stats.gold > gold_before);
    assert!(game.sim.player.stats.level > level_before || game.sim.player.stats.xp > xp_before);
    assert!(
        game.sim
            .monsters
            .iter()
            .all(|monster| monster.quest_id != Some(quest.id))
    );
}

#[test]
fn completed_quest_sources_are_not_reissued() {
    let mut game = Game::new(1);
    let giver = origin_town(&game);
    let first = game.generate_recovery_quest(giver).unwrap();
    let first_signature = first.signature;
    game.sim.active_quest = Some(first.clone());
    let item_positions = game
        .sim
        .quest_items
        .iter()
        .filter(|item| item.quest_id == first.id)
        .map(|item| item.pos)
        .collect::<Vec<_>>();
    for pos in item_positions {
        game.sim.player.pos = pos;
        assert!(game.pickup_nearby_quest_item());
    }
    game.sim.player.pos = Game::quest_board_pos(giver);
    assert!(game.interact_with_nearby_quest_board());
    assert!(
        game.sim
            .completed_quest_signatures
            .contains(&first_signature)
    );

    let second = game.generate_recovery_quest(giver);
    assert!(
        second
            .as_ref()
            .is_none_or(|quest| quest.signature != first_signature)
    );
}

#[test]
fn shrines_and_wells_restore_once() {
    let mut game = Game::new(13);
    let landmark = (-500..=500)
        .step_by(7)
        .flat_map(|y| (-500..=500).step_by(7).map(move |x| ivec2(x, y)))
        .find_map(|tile| {
            let landmark = game.world.landmark_at_tile(tile)?;
            matches!(
                landmark.kind,
                crate::world::LandmarkKind::Shrine | crate::world::LandmarkKind::Well
            )
            .then_some(landmark)
        })
        .unwrap();
    game.sim.player.pos = World::tile_center(landmark.center);
    game.sim.player.hp = 1.0;
    game.sim.player.mana = 0.0;
    game.interact_with_nearby_world_entity();
    assert!(game.sim.used_landmarks.contains(&landmark.id));
    match landmark.kind {
        crate::world::LandmarkKind::Shrine => {
            assert_eq!(game.sim.player.mana, game.sim.player.max_mana())
        }
        crate::world::LandmarkKind::Well => {
            assert_eq!(game.sim.player.hp, game.sim.player.max_hp())
        }
        _ => unreachable!(),
    }
    let log_len = game.fx.log.len();
    game.interact_with_nearby_world_entity();
    assert_eq!(game.fx.log.len(), log_len + 1);
}

#[test]
fn walking_far_from_town_repopulates_monsters_around_the_player() {
    let mut game = Game::new(1);
    game.sim.player.pos = World::tile_center(ivec2(220, 0));

    game.fixed_update(FIXED_DT);

    let nearby_packs: HashSet<u64> = game
        .sim
        .monsters
        .iter()
        .filter(|monster| monster.pack_center.distance(game.sim.player.pos) <= MONSTER_LOCAL_RADIUS)
        .map(|monster| monster.pack_id)
        .collect();
    assert_eq!(nearby_packs.len(), MONSTER_LOCAL_PACK_TARGET);
    assert!(
        game.sim
            .monsters
            .iter()
            .all(|monster| monster.pack_center.distance(game.sim.player.pos)
                <= MONSTER_DESPAWN_RADIUS)
    );
    assert!(
        game.sim
            .monsters
            .iter()
            .all(|monster| monster.level >= game.world.biome_level(game.sim.player.pos) - 2)
    );
}

#[test]
fn spawned_monster_packs_are_homogeneous_and_clustered() {
    let mut game = Game::new(2);
    game.sim.monsters.clear();
    game.runtime.next_monster_pack_id = 0;
    game.sim.player.pos = World::tile_center(ivec2(220, 0));
    game.spawn_monster_packs(MONSTER_LOCAL_PACK_TARGET);

    let mut packs: HashMap<u64, Vec<&Monster>> = HashMap::new();
    for monster in &game.sim.monsters {
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
    game.sim.monsters.clear();
    game.runtime.next_monster_pack_id = 0;
    game.sim.player.pos = World::tile_center(ivec2(220, 0));
    game.spawn_monster_packs(MONSTER_LOCAL_PACK_TARGET);

    let mut centers_by_pack = HashMap::new();
    for monster in &game.sim.monsters {
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
    let known_at_start = game.sim.known_tiles.len();
    game.sim.player.pos = World::tile_center(ivec2(32, 0));
    game.fixed_update(FIXED_DT);
    assert!(game.sim.known_tiles.len() > known_at_start);

    game.runtime.input.world_map_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.mode, UiMode::WorldMap);
    assert_eq!(game.ui.world_map.center_tile, vec2(32.0, 0.0));

    game.runtime.input.movement = Vec2::X;
    game.runtime.input.map_zoom_delta = 1.0;
    let zoom_before = game.ui.world_map.zoom;
    game.fixed_update(FIXED_DT);
    assert!(game.ui.world_map.center_tile.x > 32.0);
    assert!(game.ui.world_map.zoom > zoom_before);

    game.runtime.input.map_recenter_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.world_map.center_tile, vec2(32.0, 0.0));
}

#[test]
fn hovered_monster_prefers_the_enemy_under_the_cursor() {
    let mut game = Game::new(1);
    game.sim.monsters = vec![
        Monster {
            kind: MonsterKind::Imp,
            rank: MonsterRank::Normal,
            quest_id: None,
            pack_id: 0,
            pack_center: game.sim.player.pos + vec2(20.0, 0.0),
            pos: game.sim.player.pos + vec2(20.0, 0.0),
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
            quest_id: None,
            pack_id: 1,
            pack_center: game.sim.player.pos + vec2(24.0, 0.0),
            pos: game.sim.player.pos + vec2(24.0, 0.0),
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
    game.runtime.input.aim_world = game.sim.player.pos + vec2(24.0, 0.0);

    assert_eq!(
        game.hovered_monster().map(|monster| monster.kind),
        Some(MonsterKind::Brute)
    );
}

#[test]
fn combat_feed_scrolls_through_older_entries() {
    let mut game = Game::new(1);
    game.fx.log = (0..10).map(|index| format!("Entry {}", index)).collect();
    game.runtime.input.log_scroll_delta = 2;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.fx.log_scroll_offset, 2);

    game.runtime.input.log_scroll_delta = -1;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.fx.log_scroll_offset, 1);

    game.runtime.input.log_scroll_delta = 99;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.fx.log_scroll_offset, 4);
}

#[test]
fn equipping_moves_item_into_matching_slot() {
    let mut game = Game::new(1);
    game.ui.inventory_cursor = 0;
    game.equip_selected_item();

    assert!(game.sim.player.equipment.weapon.is_some());
    assert_eq!(game.sim.player.inventory.len(), 1);
}

#[test]
fn shop_buy_and_sell_round_trip_updates_gold_and_inventory() {
    let mut game = Game::new(1);
    game.sim.player.stats.gold = 100;
    game.ui.shop_cursor = 0;
    let starting_inventory = game.sim.player.inventory.len();

    game.buy_selected_item();
    assert_eq!(game.sim.player.inventory.len(), starting_inventory + 1);
    assert_eq!(game.sim.player.stats.gold, 76);

    game.ui.shop_tab = ShopTab::Sell;
    game.ui.shop_cursor = game.sim.player.inventory.len() - 1;
    game.sell_selected_item();
    assert_eq!(game.sim.player.inventory.len(), starting_inventory);
    assert_eq!(game.sim.player.stats.gold, 90);
}

#[test]
fn gameplay_smoke_flow_reaches_combat_loot_shop_and_travel() {
    let mut game = Game::new(1);
    game.sim.monsters.clear();
    game.sim.player.facing = Vec2::X;
    game.sim.player.stats.xp =
        game.sim.player.stats.next_xp - monster_xp(MonsterKind::Imp, 1, MonsterRank::Normal);
    game.sim.monsters.push(Monster {
        kind: MonsterKind::Imp,
        rank: MonsterRank::Normal,
        quest_id: None,
        pack_id: 0,
        pack_center: game.sim.player.pos + vec2(32.0, 0.0),
        pos: game.sim.player.pos + vec2(32.0, 0.0),
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
    assert!(game.sim.monsters.is_empty());
    assert!(
        game.fx
            .floating
            .iter()
            .any(|text| text.text.starts_with('-'))
    );
    assert!(game.fx.floating.iter().any(|text| text.text.contains("xp")));
    assert_eq!(game.sim.player.stats.level, 2);

    game.sim.loot.push(Loot {
        pos: game.sim.player.pos,
        item: roll_item(&mut game.runtime.rng, 1),
        bob: 0.0,
    });
    let inventory_before_loot = game.sim.player.inventory.len();
    game.pickup_loot();
    assert_eq!(game.sim.player.inventory.len(), inventory_before_loot + 1);

    game.ui.inventory_cursor = 0;
    game.equip_selected_item();
    assert!(game.sim.player.equipment.weapon.is_some());

    game.sim.player.pos = game
        .sim
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Merchant)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui.mode, UiMode::Merchant);

    game.sim.player.stats.gold = 100;
    game.buy_selected_item();
    assert!(game.sim.player.inventory.len() >= 2);

    game.ui.mode = UiMode::None;
    discover_towns(&mut game, 5);
    game.sim.player.pos = game
        .sim
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Wayfinder)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui.mode, UiMode::Travel);
    game.ui.travel_cursor = game.sim.travel_destinations.len() - 1;
    game.runtime.input.inventory_equip_pressed = true;
    game.update_travel_controls();
    assert!(
        game.world.biome_level(game.sim.player.pos)
            >= game.sim.travel_destinations.last().unwrap().min_level
    );
}

#[test]
fn fireball_explodes_and_hits_nearby_monsters() {
    let mut game = Game::new(7);
    game.sim.monsters = vec![
        Monster {
            kind: MonsterKind::Imp,
            rank: MonsterRank::Normal,
            quest_id: None,
            pack_id: 0,
            pack_center: game.sim.player.pos + vec2(34.0, 0.0),
            pos: game.sim.player.pos + vec2(34.0, 0.0),
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
            quest_id: None,
            pack_id: 1,
            pack_center: game.sim.player.pos + vec2(48.0, 10.0),
            pos: game.sim.player.pos + vec2(48.0, 10.0),
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
            quest_id: None,
            pack_id: 2,
            pack_center: game.sim.player.pos + vec2(120.0, 0.0),
            pos: game.sim.player.pos + vec2(120.0, 0.0),
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
    game.sim.player.facing = Vec2::X;

    game.cast_ability(AbilityKind::Fireball);
    game.update_projectiles(FIXED_DT);

    assert!(game.fx.projectiles.is_empty());
    assert!(game.sim.monsters[0].hp < 80.0);
    assert!(game.sim.monsters[1].hp < 80.0);
    assert_eq!(game.sim.monsters[2].hp, 80.0);
    assert_eq!(game.sim.player.disciplines.magic.xp, 4);
}

#[test]
fn basic_attack_spawns_a_short_slash_arc() {
    let mut game = Game::new(10);
    game.sim.player.facing = Vec2::X;

    game.basic_attack();

    assert_eq!(game.fx.slash_arcs.len(), 1);
    assert_eq!(game.fx.slash_arcs[0].direction, Vec2::X);
    assert_eq!(game.fx.slash_arcs[0].radius, 34.0);
    assert_eq!(game.fx.slash_arcs[0].ttl, 0.16);
    assert_eq!(game.sim.player.disciplines.melee.xp, 0);
}

#[test]
fn hitting_monster_sets_flash_and_recoil() {
    let mut game = Game::new(12);
    game.sim.player.facing = Vec2::X;
    game.sim.monsters = vec![Monster {
        kind: MonsterKind::Imp,
        rank: MonsterRank::Normal,
        quest_id: None,
        pack_id: 0,
        pack_center: game.sim.player.pos + vec2(32.0, 0.0),
        pos: game.sim.player.pos + vec2(32.0, 0.0),
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

    assert!(game.sim.monsters[0].hit_flash > 0.0);
    assert!(game.sim.monsters[0].hit_offset.x > 0.0);
    assert_eq!(game.sim.player.disciplines.melee.xp, 2);
}

#[test]
fn cleave_hits_front_arc_without_hitting_behind() {
    let mut game = Game::new(8);
    game.sim.monsters = vec![
        Monster {
            kind: MonsterKind::Imp,
            rank: MonsterRank::Normal,
            quest_id: None,
            pack_id: 0,
            pack_center: game.sim.player.pos + vec2(36.0, 0.0),
            pos: game.sim.player.pos + vec2(36.0, 0.0),
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
            quest_id: None,
            pack_id: 1,
            pack_center: game.sim.player.pos + vec2(-36.0, 0.0),
            pos: game.sim.player.pos + vec2(-36.0, 0.0),
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
    game.sim.player.facing = Vec2::X;

    game.cast_ability(AbilityKind::Cleave);

    assert!(game.sim.monsters[0].hp < 80.0);
    assert_eq!(game.sim.monsters[1].hp, 80.0);
}

#[test]
fn new_game_starts_with_only_starter_abilities_bound() {
    let mut game = Game::new(9);
    assert_eq!(
        game.sim.player.bound_abilities,
        [AbilityKind::Cleave, AbilityKind::Fireball]
    );
    assert!(game.sim.player.is_ability_unlocked(AbilityKind::Cleave));
    assert!(game.sim.player.is_ability_unlocked(AbilityKind::Fireball));
    assert!(!game.sim.player.is_ability_unlocked(AbilityKind::Rush));
    assert!(!game.sim.player.is_ability_unlocked(AbilityKind::Nova));

    game.cast_ability(AbilityKind::Rush);
    game.cast_ability(AbilityKind::Nova);
    assert_eq!(
        game.sim.player.ability_cooldowns[AbilityKind::Rush.index()],
        0.0
    );
    assert_eq!(
        game.sim.player.ability_cooldowns[AbilityKind::Nova.index()],
        0.0
    );
}

#[test]
fn ability_definitions_keep_existing_tuning() {
    let expected = [
        (AbilityKind::Cleave, 1, 10.0, 2.2),
        (AbilityKind::Rush, 2, 8.0, 1.8),
        (AbilityKind::Whirlwind, 4, 16.0, 3.0),
        (AbilityKind::Execute, 8, 14.0, 2.6),
        (AbilityKind::Fireball, 1, 12.0, 1.2),
        (AbilityKind::Nova, 2, 14.0, 3.5),
        (AbilityKind::IceBolt, 4, 10.0, 1.0),
        (AbilityKind::Meteor, 8, 22.0, 4.0),
    ];

    for (ability, unlock_level, mana_cost, cooldown) in expected {
        assert_eq!(ability.unlock_level(), unlock_level);
        assert_eq!(ability.mana_cost(), mana_cost);
        assert_eq!(ability.cooldown(), cooldown);
    }
}

#[test]
fn discipline_levels_unlock_combat_abilities() {
    let mut game = Game::new(13);

    game.award_discipline_xp(DisciplineKind::Melee, 24);
    game.award_discipline_xp(DisciplineKind::Magic, 24);

    assert!(game.sim.player.is_ability_unlocked(AbilityKind::Rush));
    assert!(game.sim.player.is_ability_unlocked(AbilityKind::Nova));
    assert!(
        game.fx
            .log
            .iter()
            .any(|line| line.contains("Melee unlocks Rush"))
    );
    assert!(
        game.fx
            .log
            .iter()
            .any(|line| line.contains("Magic unlocks Nova"))
    );
}

#[test]
fn discipline_progression_keeps_feedback_side_effects() {
    let mut game = Game::new(28);

    game.award_discipline_xp(DisciplineKind::Melee, 24);

    assert_eq!(game.fx.skill_xp_toasts.len(), 1);
    assert_eq!(game.fx.skill_xp_toasts[0].amount, 24);
    assert!(
        game.fx
            .notifications
            .iter()
            .any(|notification| notification.text == "Melee reaches level 2")
    );
    assert!(
        game.fx
            .notifications
            .iter()
            .any(|notification| notification.text == "Unlocked Rush")
    );
}

#[test]
fn pickup_feedback_stays_the_same_after_event_routing() {
    let mut game = Game::new(29);
    let loot = Loot {
        pos: game.sim.player.pos,
        item: roll_item(&mut game.runtime.rng, 1),
        bob: 0.0,
    };
    let expected_log = format!(
        "Picked up {} [{}].",
        loot.item.name,
        item_summary(&loot.item)
    );
    game.sim.loot.push(loot);

    game.pickup_loot();

    assert!(game.fx.floating.iter().any(|text| text.text == "LOOT"));
    assert_eq!(game.fx.log.last(), Some(&expected_log));
}

#[test]
fn discipline_xp_can_cross_multiple_levels_at_once() {
    let mut game = Game::new(17);

    game.award_discipline_xp(DisciplineKind::Melee, 100);

    assert_eq!(game.sim.player.disciplines.melee.level, 3);
    assert_eq!(game.sim.player.disciplines.melee.xp, 36);
    assert_eq!(game.sim.player.disciplines.melee.next_xp, 88);
}

#[test]
fn binding_skills_replaces_and_swaps_slots_without_duplicates() {
    let mut game = Game::new(18);
    game.award_discipline_xp(DisciplineKind::Melee, 24);

    game.bind_ability(0, AbilityKind::Rush);
    assert_eq!(
        game.sim.player.bound_abilities,
        [AbilityKind::Rush, AbilityKind::Fireball]
    );

    game.bind_ability(1, AbilityKind::Rush);
    assert_eq!(
        game.sim.player.bound_abilities,
        [AbilityKind::Fireball, AbilityKind::Rush]
    );
}

#[test]
fn locked_skills_cannot_be_bound() {
    let mut game = Game::new(24);

    game.bind_ability(0, AbilityKind::Meteor);

    assert_eq!(
        game.sim.player.bound_abilities,
        [AbilityKind::Cleave, AbilityKind::Fireball]
    );
}

#[test]
fn skill_book_navigation_moves_between_columns_and_prefers_useful_skills() {
    let mut game = Game::new(25);
    game.award_discipline_xp(DisciplineKind::Melee, 24);

    assert_eq!(game.ui.skill_book_focus, SkillBookFocus::Disciplines);
    game.runtime.input.nav_right_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.ui.skill_book_focus, SkillBookFocus::Skills);

    game.runtime.input = InputState::default();
    game.runtime.input.inventory_down_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.ui.skill_book_ability_cursor, 1);

    game.runtime.input = InputState::default();
    game.runtime.input.nav_right_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.ui.skill_book_focus, SkillBookFocus::Detail);

    game.runtime.input = InputState::default();
    game.runtime.input.nav_left_pressed = true;
    game.update_skill_book_controls();
    game.runtime.input = InputState::default();
    game.runtime.input.nav_left_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.ui.skill_book_focus, SkillBookFocus::Disciplines);

    game.ui.skill_book_cursor = 1;
    game.ui.skill_book_ability_cursor = 0;
    game.runtime.input = InputState::default();
    game.runtime.input.inventory_up_pressed = true;
    game.update_skill_book_controls();
    assert_eq!(game.ui.skill_book_cursor, 0);
    assert_eq!(game.ui.skill_book_ability_cursor, 1);
}

#[test]
fn melee_and_magic_unlocks_reach_levels_four_and_eight() {
    let mut game = Game::new(19);
    game.award_discipline_xp(DisciplineKind::Melee, 2_000);
    game.award_discipline_xp(DisciplineKind::Magic, 2_000);

    assert!(game.sim.player.is_ability_unlocked(AbilityKind::Whirlwind));
    assert!(game.sim.player.is_ability_unlocked(AbilityKind::Execute));
    assert!(game.sim.player.is_ability_unlocked(AbilityKind::IceBolt));
    assert!(game.sim.player.is_ability_unlocked(AbilityKind::Meteor));
}

#[test]
fn whirlwind_hits_nearby_enemies() {
    let mut game = Game::new(20);
    game.sim.player.disciplines.melee.level = 4;
    game.sim.monsters = vec![
        test_monster(MonsterKind::Imp, game.sim.player.pos + vec2(24.0, 0.0)),
        test_monster(MonsterKind::Slime, game.sim.player.pos + vec2(-24.0, 0.0)),
    ];

    game.cast_ability(AbilityKind::Whirlwind);

    assert!(game.sim.monsters.iter().all(|monster| monster.hp < 80.0));
}

#[test]
fn execute_hits_harder_against_wounded_targets() {
    let mut game = Game::new(21);
    game.sim.player.disciplines.melee.level = 8;
    game.sim.player.facing = Vec2::X;
    game.sim.monsters = vec![test_monster(
        MonsterKind::Imp,
        game.sim.player.pos + vec2(32.0, 0.0),
    )];
    game.sim.monsters[0].hp = 30.0;

    game.cast_ability(AbilityKind::Execute);

    assert!(game.sim.monsters.is_empty() || game.sim.monsters[0].hp < 12.0);
}

#[test]
fn ice_bolt_chills_the_first_enemy_hit() {
    let mut game = Game::new(22);
    game.sim.player.disciplines.magic.level = 4;
    game.sim.player.facing = Vec2::X;
    game.sim.monsters = vec![test_monster(
        MonsterKind::Imp,
        game.sim.player.pos + vec2(32.0, 0.0),
    )];

    game.cast_ability(AbilityKind::IceBolt);
    game.update_projectiles(FIXED_DT);

    assert!(game.sim.monsters[0].chill_ttl > 0.0);
}

#[test]
fn meteor_waits_then_hits_its_target_area() {
    let mut game = Game::new(23);
    game.sim.player.disciplines.magic.level = 8;
    game.runtime.input.aim_world = game.sim.player.pos + vec2(40.0, 0.0);
    game.sim.monsters = vec![test_monster(MonsterKind::Imp, game.runtime.input.aim_world)];

    game.cast_ability(AbilityKind::Meteor);
    assert_eq!(game.fx.meteors.len(), 1);
    assert_eq!(game.sim.monsters[0].hp, 80.0);

    game.update_meteors(0.73);

    assert!(game.fx.meteors.is_empty());
    assert!(game.sim.monsters.is_empty() || game.sim.monsters[0].hp < 80.0);
}

#[test]
fn armor_mastery_gains_xp_when_damage_is_mitigated() {
    let mut game = Game::new(14);
    game.sim.monsters = vec![Monster {
        kind: MonsterKind::Imp,
        rank: MonsterRank::Normal,
        quest_id: None,
        pack_id: 0,
        pack_center: game.sim.player.pos + vec2(12.0, 0.0),
        pos: game.sim.player.pos + vec2(12.0, 0.0),
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

    assert_eq!(game.sim.player.disciplines.armor.xp, 2);
}

#[test]
fn agility_mastery_gains_xp_from_actual_travel() {
    let mut game = Game::new(15);
    game.runtime.agility_distance_bank = 143.9;
    game.runtime.input.movement = Vec2::X;

    game.fixed_update(FIXED_DT);

    assert_eq!(game.sim.player.disciplines.agility.xp, 1);
}

#[test]
fn new_game_starts_with_previous_actual_move_speed() {
    let game = Game::new(30);

    assert_eq!(game.sim.player.move_speed(), 170.0);
}

#[test]
fn new_game_displays_100_move_speed_rating() {
    let game = Game::new(30);

    assert_eq!(game.sim.player.move_speed_rating(), 100.0);
}

#[test]
fn mastery_levels_improve_combat_and_movement_values() {
    let mut game = Game::new(16);
    let base_armor = game.sim.player.armor();
    let base_speed = game.sim.player.move_speed();
    let base_mana_regen = game.sim.player.mana_regen_rate();

    game.award_discipline_xp(DisciplineKind::Melee, 24);
    game.award_discipline_xp(DisciplineKind::Magic, 24);
    game.award_discipline_xp(DisciplineKind::Armor, 24);
    game.award_discipline_xp(DisciplineKind::Agility, 24);

    assert_eq!(game.sim.player.melee_damage_bonus(), 2);
    assert_eq!(game.sim.player.magic_damage_bonus(), 2);
    assert_eq!(game.sim.player.mana_regen_rate(), base_mana_regen + 0.5);
    assert_eq!(game.sim.player.armor(), base_armor + 1);
    assert_eq!(game.sim.player.move_speed(), base_speed + 6.0);
}

#[test]
fn mana_regenerates_more_slowly_at_first_and_scales_with_magic_mastery() {
    let mut novice = Game::new(26);
    novice.sim.player.mana = 0.0;
    novice.fixed_update(1.0);
    assert_eq!(novice.sim.player.mana, 3.0);

    let mut mage = Game::new(27);
    mage.sim.player.mana = 0.0;
    mage.sim.player.disciplines.magic.level = 4;
    mage.fixed_update(1.0);
    assert_eq!(mage.sim.player.mana, 4.5);
}

#[test]
fn every_ui_window_supports_basic_navigation() {
    let mut game = Game::new(3);

    game.runtime.input.inventory_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.mode, UiMode::Inventory);
    game.runtime.input.inventory_down_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.inventory_cursor, 1);

    game.ui.mode = UiMode::None;
    game.runtime.input = InputState::default();
    game.runtime.input.character_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.mode, UiMode::Character);
    game.runtime.input.inventory_down_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.character_cursor, 1);

    game.ui.mode = UiMode::None;
    game.runtime.input = InputState::default();
    game.runtime.input.skill_book_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.mode, UiMode::SkillBook);
    game.runtime.input.nav_right_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.skill_book_focus, SkillBookFocus::Skills);

    game.ui.mode = UiMode::None;
    game.runtime.input = InputState::default();
    game.runtime.input.world_map_toggle_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.mode, UiMode::WorldMap);

    game.ui.mode = UiMode::None;
    game.sim.player.pos = game
        .sim
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Merchant)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui.mode, UiMode::Merchant);
    game.runtime.input = InputState::default();
    game.runtime.input.nav_right_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.shop_tab, ShopTab::Sell);

    game.ui.mode = UiMode::None;
    game.sim.player.pos = game
        .sim
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Trainer)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui.mode, UiMode::Trainer);
    game.runtime.input = InputState::default();
    game.runtime.input.inventory_equip_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.mode, UiMode::SkillBook);

    game.ui.mode = UiMode::None;
    game.sim.player.pos = game
        .sim
        .npcs
        .iter()
        .find(|npc| npc.kind == NpcKind::Wayfinder)
        .unwrap()
        .pos;
    game.interact_with_nearby_npc();
    assert_eq!(game.ui.mode, UiMode::Travel);
    discover_towns(&mut game, 2);
    game.runtime.input = InputState::default();
    game.runtime.input.inventory_down_pressed = true;
    game.fixed_update(FIXED_DT);
    assert_eq!(game.ui.travel_cursor, 1);
}
