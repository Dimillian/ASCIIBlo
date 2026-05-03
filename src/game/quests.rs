use ::rand::RngExt;
use macroquad::prelude::*;

use crate::{
    content::{MonsterKind, MonsterRank, NpcKind, roll_item, roll_monster},
    world::{Landmark, LandmarkKind, SettlementSite, SettlementTier, TILE, World},
};

use super::{
    Game, Loot, Monster, Npc, Quest, QuestItem, QuestKind, QuestReward, QuestSignature, QuestStage,
};

const QUEST_BOARD_OFFSET: IVec2 = IVec2::new(0, 5);
const QUEST_TARGET_MIN_RADIUS: i32 = 44;
const QUEST_TARGET_MAX_RADIUS: i32 = 118;
const QUEST_LANDMARK_RADIUS: i32 = 180;
const QUEST_TOWN_RADIUS: i32 = 420;

impl Game {
    pub fn quest_board_tile(site: SettlementSite) -> IVec2 {
        site.center + QUEST_BOARD_OFFSET
    }

    pub fn quest_board_pos(site: SettlementSite) -> Vec2 {
        World::tile_center(Self::quest_board_tile(site))
    }

    pub fn nearby_quest_boards(&self) -> Vec<SettlementSite> {
        let player_tile = World::world_to_tile(self.sim.player.pos);
        self.world
            .settlements_near_tile(player_tile, 42)
            .into_iter()
            .filter(|site| site.tier == SettlementTier::Town || site.is_origin())
            .collect()
    }

    pub fn active_quest_objective(&self) -> Option<(&str, String, String)> {
        self.sim.active_quest.as_ref().map(|quest| {
            let objective = if quest.is_ready() {
                format!("Return to the bounty board in {}", quest.giver.name())
            } else {
                quest.objective.clone()
            };
            (quest.title.as_str(), objective, quest.progress_text())
        })
    }

    pub fn quest_navigation_target(&self) -> Option<Vec2> {
        self.sim.active_quest.as_ref().map(|quest| {
            if quest.is_ready() {
                Self::quest_board_pos(quest.giver)
            } else {
                quest.target_pos
            }
        })
    }

    pub(super) fn interact_with_nearby_quest_board(&mut self) -> bool {
        let Some(site) = self
            .nearby_quest_boards()
            .into_iter()
            .find(|site| Self::quest_board_pos(*site).distance(self.sim.player.pos) <= 42.0)
        else {
            return false;
        };
        if self
            .sim
            .active_quest
            .as_ref()
            .is_some_and(|quest| quest.giver.id == site.id && quest.is_ready())
        {
            self.turn_in_active_quest();
            return true;
        }
        if self.sim.active_quest.is_some() {
            self.log("The board has no room for another promise.".into());
            return true;
        }
        if let Some(quest) = self.generate_quest_from_board(site) {
            self.log(format!("Accepted: {}.", quest.title));
            self.sim.active_quest = Some(quest);
        } else {
            self.log("The board is bare for now.".into());
        }
        true
    }

    fn generate_quest_from_board(&mut self, giver: SettlementSite) -> Option<Quest> {
        let start = self.runtime.rng.random_range(0..QuestKind::ALL.len());
        for offset in 0..QuestKind::ALL.len() {
            let kind = QuestKind::ALL[(start + offset) % QuestKind::ALL.len()];
            if let Some(quest) = match kind {
                QuestKind::KillPack => self.generate_kill_pack_quest(giver),
                QuestKind::BountyBoss => self.generate_bounty_quest(giver),
                QuestKind::MeetNpc => self.generate_meet_npc_quest(giver),
                QuestKind::RecoverItems => self.generate_recovery_quest(giver),
            } {
                return Some(quest);
            }
        }
        None
    }

    fn next_quest_id(&mut self) -> u64 {
        let id = self.runtime.next_quest_id;
        self.runtime.next_quest_id += 1;
        id
    }

    fn base_reward(&self, target_pos: Vec2, kind: QuestKind) -> QuestReward {
        let level = self.world.biome_level(target_pos).max(1);
        let weight = match kind {
            QuestKind::KillPack => 1,
            QuestKind::BountyBoss => 2,
            QuestKind::MeetNpc => 1,
            QuestKind::RecoverItems => 1,
        };
        QuestReward {
            gold: 18 + level * 5 * weight,
            xp: 14 + level * 7 * weight,
            item_chance: if kind == QuestKind::BountyBoss {
                0.28
            } else {
                0.16
            },
        }
    }

    pub(super) fn generate_kill_pack_quest(&mut self, giver: SettlementSite) -> Option<Quest> {
        let (target_pos, kind, signature) =
            self.find_new_monster_target(giver, QuestKind::KillPack)?;
        let quest_id = self.next_quest_id();
        let goal = self.spawn_quest_pack(quest_id, target_pos, kind);
        if goal == 0 {
            return None;
        }
        Some(Quest {
            id: quest_id,
            kind: QuestKind::KillPack,
            signature,
            stage: QuestStage::Active,
            giver,
            title: format!("Cull the {}", plural_monster_name(kind)),
            objective: format!(
                "Cull {} {} near {}",
                goal,
                plural_monster_name(kind).to_lowercase(),
                self.world.region_name(target_pos)
            ),
            target_pos,
            progress: 0,
            goal,
            reward: self.base_reward(target_pos, QuestKind::KillPack),
        })
    }

    pub(super) fn generate_bounty_quest(&mut self, giver: SettlementSite) -> Option<Quest> {
        let (target_pos, kind, signature) =
            self.find_new_monster_target(giver, QuestKind::BountyBoss)?;
        let quest_id = self.next_quest_id();
        let max_hp = crate::content::monster_max_hp(
            kind,
            self.world.biome_level(target_pos),
            MonsterRank::Boss,
        );
        let pack_id = self.runtime.next_monster_pack_id;
        self.runtime.next_monster_pack_id += 1;
        self.sim.monsters.push(Monster {
            kind,
            rank: MonsterRank::Boss,
            quest_id: Some(quest_id),
            pack_id,
            pack_center: target_pos,
            pos: target_pos,
            vel: Vec2::ZERO,
            hit_offset: Vec2::ZERO,
            hp: max_hp,
            max_hp,
            level: self.world.biome_level(target_pos).max(1),
            attack_cd: 0.0,
            wobble: 0.0,
            hit_flash: 0.0,
            chill_ttl: 0.0,
        });
        Some(Quest {
            id: quest_id,
            kind: QuestKind::BountyBoss,
            signature,
            stage: QuestStage::Active,
            giver,
            title: format!("Bounty: {}", kind.name()),
            objective: format!("Find and slay the Boss {}", kind.name()),
            target_pos,
            progress: 0,
            goal: 1,
            reward: self.base_reward(target_pos, QuestKind::BountyBoss),
        })
    }

    pub(super) fn generate_meet_npc_quest(&mut self, giver: SettlementSite) -> Option<Quest> {
        let target = self
            .world
            .settlements_near_tile(giver.center, QUEST_TOWN_RADIUS)
            .into_iter()
            .filter(|site| site.id != giver.id && site.tier == SettlementTier::Town)
            .filter(|site| {
                !self
                    .sim
                    .completed_quest_signatures
                    .contains(&QuestSignature::MeetNpc { town_id: site.id })
            })
            .min_by_key(|site| site.center.distance_squared(giver.center))?;
        let quest_id = self.next_quest_id();
        let name = quest_contact_name(quest_id);
        let target_pos = World::tile_center(target.center + ivec2(0, 4));
        self.sim.npcs.push(Npc {
            kind: NpcKind::QuestContact,
            name: name.clone(),
            quest_id: Some(quest_id),
            pos: target_pos,
        });
        Some(Quest {
            id: quest_id,
            kind: QuestKind::MeetNpc,
            signature: QuestSignature::MeetNpc { town_id: target.id },
            stage: QuestStage::Active,
            giver,
            title: format!("Message for {name}"),
            objective: format!("Meet {name} in {}", target.name()),
            target_pos,
            progress: 0,
            goal: 1,
            reward: self.base_reward(target_pos, QuestKind::MeetNpc),
        })
    }

    pub(super) fn generate_recovery_quest(&mut self, giver: SettlementSite) -> Option<Quest> {
        let landmark = self.find_recovery_landmark(giver)?;
        let item_tiles = self.recovery_item_tiles(landmark);
        if item_tiles.len() < 3 {
            return None;
        }
        let quest_id = self.next_quest_id();
        let goal = item_tiles.len();
        let target_pos = World::tile_center(landmark.center);
        for tile in item_tiles {
            self.sim.quest_items.push(QuestItem {
                quest_id,
                pos: World::tile_center(tile),
                name: recovery_item_name(landmark.kind).into(),
            });
        }
        Some(Quest {
            id: quest_id,
            kind: QuestKind::RecoverItems,
            signature: QuestSignature::RecoverItems {
                landmark_id: landmark.id,
            },
            stage: QuestStage::Active,
            giver,
            title: format!("Recover {}", recovery_item_name(landmark.kind)),
            objective: format!(
                "Recover {} {} at the {} near {}",
                goal,
                recovery_item_name(landmark.kind).to_lowercase(),
                landmark.kind.name(),
                self.world.region_name(target_pos)
            ),
            target_pos,
            progress: 0,
            goal,
            reward: self.base_reward(target_pos, QuestKind::RecoverItems),
        })
    }

    fn find_wilderness_target(&mut self, giver: SettlementSite) -> Option<Vec2> {
        let giver_level = self.world.biome_level_at_tile(giver.center).max(1);
        for _ in 0..256 {
            let angle = self.runtime.rng.random_range(0.0..std::f32::consts::TAU);
            let radius = self
                .runtime
                .rng
                .random_range(QUEST_TARGET_MIN_RADIUS..=QUEST_TARGET_MAX_RADIUS);
            let tile = giver.center
                + ivec2(
                    (angle.cos() * radius as f32).round() as i32,
                    (angle.sin() * radius as f32).round() as i32,
                );
            if !self.world.tile(tile).walkable
                || self.world.is_safe_zone(tile)
                || self.world.biome_level_at_tile(tile) < giver_level
            {
                continue;
            }
            return Some(World::tile_center(tile));
        }
        None
    }

    fn find_new_monster_target(
        &mut self,
        giver: SettlementSite,
        kind: QuestKind,
    ) -> Option<(Vec2, MonsterKind, QuestSignature)> {
        for _ in 0..64 {
            let target_pos = self.find_wilderness_target(giver)?;
            let monster_kind =
                roll_monster(&mut self.runtime.rng, self.world.biome_at_world(target_pos));
            let tile = World::world_to_tile(target_pos);
            let signature = match kind {
                QuestKind::KillPack => QuestSignature::KillPack {
                    tile_x: tile.x,
                    tile_y: tile.y,
                    kind: monster_kind,
                },
                QuestKind::BountyBoss => QuestSignature::BountyBoss {
                    tile_x: tile.x,
                    tile_y: tile.y,
                    kind: monster_kind,
                },
                QuestKind::MeetNpc | QuestKind::RecoverItems => unreachable!(),
            };
            if !self.sim.completed_quest_signatures.contains(&signature) {
                return Some((target_pos, monster_kind, signature));
            }
        }
        None
    }

    fn find_recovery_landmark(&self, giver: SettlementSite) -> Option<Landmark> {
        self.world
            .landmarks_near_tile(giver.center, QUEST_LANDMARK_RADIUS)
            .into_iter()
            .filter(|landmark| landmark.center.distance_squared(giver.center) > 40 * 40)
            .filter(|landmark| {
                !self
                    .sim
                    .completed_quest_signatures
                    .contains(&QuestSignature::RecoverItems {
                        landmark_id: landmark.id,
                    })
            })
            .min_by_key(|landmark| landmark.center.distance_squared(giver.center))
    }

    fn recovery_item_tiles(&self, landmark: Landmark) -> Vec<IVec2> {
        let mut candidates = Vec::new();
        for radius in 0_i32..=3 {
            for y in -radius..=radius {
                for x in -radius..=radius {
                    if x.abs().max(y.abs()) != radius {
                        continue;
                    }
                    let tile = landmark.center + ivec2(x, y);
                    if self.world.tile(tile).walkable {
                        candidates.push(tile);
                    }
                }
            }
        }
        candidates.sort_by_key(|tile| (tile.distance_squared(landmark.center), tile.y, tile.x));
        candidates.truncate(3);
        candidates
    }

    fn spawn_quest_pack(&mut self, quest_id: u64, center: Vec2, kind: MonsterKind) -> usize {
        let pack_id = self.runtime.next_monster_pack_id;
        self.runtime.next_monster_pack_id += 1;
        let level = self.world.biome_level(center).max(1);
        let mut pack = Vec::new();
        for offset in [
            vec2(0.0, 0.0),
            vec2(TILE * 1.5, 0.0),
            vec2(-TILE * 1.5, 0.0),
            vec2(0.0, TILE * 1.5),
        ] {
            let pos = center + offset;
            if !self.world.walkable_at_world(pos)
                || self.world.is_safe_zone(World::world_to_tile(pos))
            {
                continue;
            }
            let max_hp = crate::content::monster_max_hp(kind, level, MonsterRank::Normal);
            pack.push(Monster {
                kind,
                rank: MonsterRank::Normal,
                quest_id: Some(quest_id),
                pack_id,
                pack_center: center,
                pos,
                vel: Vec2::ZERO,
                hit_offset: Vec2::ZERO,
                hp: max_hp,
                max_hp,
                level,
                attack_cd: 0.0,
                wobble: 0.0,
                hit_flash: 0.0,
                chill_ttl: 0.0,
            });
        }
        let goal = pack.len();
        self.sim.monsters.extend(pack);
        goal
    }

    pub(super) fn on_quest_monster_killed(&mut self, monster: &Monster) {
        let Some(quest_id) = monster.quest_id else {
            return;
        };
        let Some(quest) = self.sim.active_quest.as_mut() else {
            return;
        };
        if quest.id != quest_id
            || !matches!(quest.kind, QuestKind::KillPack | QuestKind::BountyBoss)
            || quest.is_ready()
        {
            return;
        }
        quest.progress = (quest.progress + 1).min(quest.goal);
        self.finish_quest_if_ready();
    }

    pub(super) fn interact_with_quest_contact(&mut self, npc: &Npc) -> bool {
        let Some(quest_id) = npc.quest_id else {
            return false;
        };
        let Some(quest) = self.sim.active_quest.as_mut() else {
            return false;
        };
        if quest.id != quest_id || quest.kind != QuestKind::MeetNpc || quest.is_ready() {
            return false;
        }
        quest.progress = 1;
        self.log(format!("{}: {}", npc.name, npc.kind.greeting()));
        self.finish_quest_if_ready();
        true
    }

    pub(super) fn pickup_nearby_quest_item(&mut self) -> bool {
        let Some(index) = self
            .sim
            .quest_items
            .iter()
            .position(|item| item.pos.distance(self.sim.player.pos) <= 34.0)
        else {
            return false;
        };
        let item = self.sim.quest_items.remove(index);
        let Some(quest) = self.sim.active_quest.as_mut() else {
            return false;
        };
        if quest.id != item.quest_id || quest.kind != QuestKind::RecoverItems || quest.is_ready() {
            return false;
        }
        quest.progress = (quest.progress + 1).min(quest.goal);
        self.log(format!("Recovered {}.", item.name));
        self.finish_quest_if_ready();
        true
    }

    fn finish_quest_if_ready(&mut self) {
        let Some(quest) = self.sim.active_quest.as_mut() else {
            return;
        };
        if quest.progress < quest.goal || quest.is_ready() {
            return;
        }
        quest.stage = QuestStage::ReadyToTurnIn;
        let title = quest.title.clone();
        let giver_name = quest.giver.name();
        self.log(format!(
            "{} complete. Return to the bounty board in {}.",
            title, giver_name
        ));
    }

    fn turn_in_active_quest(&mut self) {
        let Some(quest) = self.sim.active_quest.take() else {
            return;
        };
        self.sim.completed_quest_signatures.insert(quest.signature);
        self.sim.player.stats.gold += quest.reward.gold;
        self.grant_player_xp(quest.reward.xp);
        self.log(format!(
            "Turned in {}. +{} gold, +{} xp.",
            quest.title, quest.reward.gold, quest.reward.xp
        ));
        if self.runtime.rng.random_bool(quest.reward.item_chance) {
            let item = roll_item(
                &mut self.runtime.rng,
                self.world.biome_level(quest.target_pos).max(1),
            );
            if self.sim.player.inventory.can_fit(&item) {
                self.log(format!("The board pays extra: {}.", item.name));
                self.sim
                    .player
                    .inventory
                    .insert_first_fit(item)
                    .expect("fitting quest reward should insert");
            } else {
                self.log(format!("The board leaves {} at your feet.", item.name));
                self.sim.loot.push(Loot {
                    pos: self.sim.player.pos,
                    item,
                    bob: 0.0,
                });
            }
        }
        self.sim
            .monsters
            .retain(|monster| monster.quest_id != Some(quest.id));
        self.sim.npcs.retain(|npc| npc.quest_id != Some(quest.id));
        self.sim
            .quest_items
            .retain(|item| item.quest_id != quest.id);
    }
}

fn plural_monster_name(kind: MonsterKind) -> String {
    match kind {
        MonsterKind::Brute => "Brutes".into(),
        MonsterKind::Wisp => "Wisps".into(),
        MonsterKind::Cinderling => "Cinderlings".into(),
        MonsterKind::Revenant => "Revenants".into(),
        _ => format!("{}s", kind.name()),
    }
}

fn quest_contact_name(id: u64) -> String {
    const FIRST: [&str; 8] = [
        "Ari", "Bram", "Celia", "Daro", "Edda", "Fenn", "Galen", "Hale",
    ];
    const SECOND: [&str; 8] = [
        "Ash", "Briar", "Cairn", "Dove", "Ember", "Frost", "Glen", "Hearth",
    ];
    format!(
        "{} {}",
        FIRST[(id as usize) % FIRST.len()],
        SECOND[((id / 3) as usize) % SECOND.len()]
    )
}

fn recovery_item_name(kind: LandmarkKind) -> &'static str {
    match kind {
        LandmarkKind::Shrine => "shrine shards",
        LandmarkKind::Well => "well tokens",
        LandmarkKind::Camp => "camp satchels",
        LandmarkKind::Graveyard => "grave seals",
        LandmarkKind::StandingStones => "stone rubbings",
        LandmarkKind::Cart => "cart ledgers",
    }
}
