use std::path::PathBuf;

use macroquad::prelude::{Color, Vec2, ivec2};

use crate::{
    content::{Item, Rarity, Slot},
    game::{
        AbilityKind, DisciplineKind, Game, Loot, Monster, Notification, Projectile, Pulse, Quest,
        QuestKind, QuestReward, QuestStage, ShopTab, SkillBookFocus, SkillXpToast, UiMode,
    },
    world::{SettlementSite, SettlementTier},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewMode {
    Gameplay,
    Lighting,
    Pickup,
    Inventory,
    Character,
    SkillBook,
    WorldMap,
    ShopBuy,
    ShopSell,
    Trainer,
    Travel,
    Quest,
}

impl PreviewMode {
    pub const ALL: [PreviewMode; 12] = [
        PreviewMode::Gameplay,
        PreviewMode::Lighting,
        PreviewMode::Pickup,
        PreviewMode::Inventory,
        PreviewMode::Character,
        PreviewMode::SkillBook,
        PreviewMode::WorldMap,
        PreviewMode::ShopBuy,
        PreviewMode::ShopSell,
        PreviewMode::Trainer,
        PreviewMode::Travel,
        PreviewMode::Quest,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            PreviewMode::Gameplay => "gameplay",
            PreviewMode::Lighting => "lighting",
            PreviewMode::Pickup => "pickup",
            PreviewMode::Inventory => "inventory",
            PreviewMode::Character => "character",
            PreviewMode::SkillBook => "skill-book",
            PreviewMode::WorldMap => "world-map",
            PreviewMode::ShopBuy => "shop-buy",
            PreviewMode::ShopSell => "shop-sell",
            PreviewMode::Trainer => "trainer",
            PreviewMode::Travel => "travel",
            PreviewMode::Quest => "quest",
        }
    }

    fn from_slug(value: &str) -> Option<Self> {
        match value {
            "gameplay" => Some(PreviewMode::Gameplay),
            "lighting" => Some(PreviewMode::Lighting),
            "pickup" => Some(PreviewMode::Pickup),
            "inventory" => Some(PreviewMode::Inventory),
            "character" => Some(PreviewMode::Character),
            "skill-book" => Some(PreviewMode::SkillBook),
            "world-map" => Some(PreviewMode::WorldMap),
            "shop-buy" => Some(PreviewMode::ShopBuy),
            "shop-sell" => Some(PreviewMode::ShopSell),
            "trainer" => Some(PreviewMode::Trainer),
            "travel" => Some(PreviewMode::Travel),
            "quest" => Some(PreviewMode::Quest),
            _ => None,
        }
    }

    pub fn configure(self, game: &mut Game) {
        game.sim.player.stats.gold = 128;
        game.ui.shop_cursor = 0;
        game.ui.travel_cursor = 0;
        game.ui.inventory_focus = crate::game::InventoryFocus::Backpack;
        game.ui.inventory_backpack_cursor = 0;
        game.ui.inventory_equipment_cursor = 0;
        game.ui.character_cursor = 0;
        game.ui.skill_book_cursor = 0;
        game.ui.skill_book_ability_cursor = 0;
        game.ui.skill_book_focus = SkillBookFocus::Disciplines;
        game.sim.player.stats.unspent_stat_points = 0;
        for kind in DisciplineKind::ALL {
            let progress = game.sim.player.disciplines.get_mut(kind);
            progress.level = 1;
            progress.xp = 0;
            progress.next_xp = 24;
        }
        game.ui.mode = UiMode::None;
        game.ui.shop_tab = ShopTab::Buy;
        game.sim.loot.clear();
        game.sim.quest_items.clear();
        game.sim.active_quest = None;
        game.fx.floating.clear();
        game.fx.particles.clear();
        game.fx.pulses.clear();
        game.fx.slash_arcs.clear();
        game.fx.projectiles.clear();
        game.fx.meteors.clear();
        game.fx.skill_xp_toasts.clear();
        game.fx.notifications.clear();
        game.sim.player.hp = game.sim.player.max_hp();
        game.sim.player.mana = game.sim.player.max_mana();
        game.sim.player.attack_cd = 0.0;
        game.sim.player.ability_cooldowns = [0.0; 8];
        game.sim.player.bound_abilities = [AbilityKind::Cleave, AbilityKind::Fireball];
        game.fx.log = vec!["The bell in Ember Town rings. Go make trouble.".into()];
        game.runtime.preview_hover_screen = Some(Vec2::new(-1_000.0, -1_000.0));

        match self {
            PreviewMode::Gameplay => {
                game.sim.player.ability_cooldowns[AbilityKind::Cleave.index()] = 1.3;
                game.fx.log = vec![
                    "The bell in Ember Town rings. Go make trouble.".into(),
                    "You hit Slime for 14.".into(),
                    "You hit Slime for 15.".into(),
                    "You hit Slime for 17.".into(),
                    "You hit Slime for 16.".into(),
                    "Slime pops. +14 xp.".into(),
                    "Slime drops Swift Mace of Alacrity.".into(),
                    "Nothing close enough to pocket.".into(),
                ];
                game.fx.skill_xp_toasts.push(SkillXpToast {
                    kind: DisciplineKind::Melee,
                    amount: 6,
                    ttl: 2.4,
                });
                game.fx.notifications.push(Notification {
                    text: "Melee reaches level 2".into(),
                    color: DisciplineKind::Melee.color(),
                    ttl: 2.2,
                });
                game.fx.notifications.push(Notification {
                    text: "Unlocked Rush".into(),
                    color: DisciplineKind::Melee.color(),
                    ttl: 2.6,
                });
                game.sim.monsters.clear();
                game.sim.monsters.push(Monster {
                    kind: crate::content::MonsterKind::Brute,
                    rank: crate::content::MonsterRank::Elite,
                    quest_id: None,
                    pack_id: 0,
                    pack_center: game.sim.player.pos + Vec2::new(42.0, 0.0),
                    pos: game.sim.player.pos + Vec2::new(42.0, 0.0),
                    vel: Vec2::ZERO,
                    hit_offset: Vec2::ZERO,
                    hp: 41.0,
                    max_hp: 72.0,
                    level: 3,
                    attack_cd: 0.0,
                    wobble: 0.0,
                    hit_flash: 0.0,
                    chill_ttl: 0.0,
                });
                game.runtime.preview_hover_world = Some(game.sim.player.pos + Vec2::new(42.0, 0.0));
            }
            PreviewMode::Lighting => {
                game.sim.monsters.clear();
                game.fx.projectiles.push(Projectile {
                    ability: AbilityKind::Fireball,
                    pos: game.sim.player.pos + Vec2::new(76.0, -18.0),
                    vel: Vec2::new(160.0, 0.0),
                    ttl: 0.95,
                    radius: 7.0,
                    damage: 18.0,
                    aoe_radius: 34.0,
                    color: Color::from_rgba(255, 132, 64, 255),
                });
                game.fx.pulses.push(Pulse {
                    pos: game.sim.player.pos + Vec2::new(-72.0, 18.0),
                    radius: 18.0,
                    ttl: 0.42,
                    color: Color::from_rgba(128, 214, 255, 255),
                });
                game.sim.loot.push(Loot {
                    pos: game.sim.player.pos + Vec2::new(-118.0, -16.0),
                    item: Item {
                        name: "Frostglass Charm".into(),
                        base_name: "Charm".into(),
                        slot: Slot::Charm,
                        rarity: Rarity::Magic,
                        item_level: 4,
                        affixes: vec!["Frostglass".into()],
                        power: 0,
                        armor: 0,
                        vitality: 1,
                        haste: 2,
                        value: 31,
                    },
                    bob: 0.0,
                });
                game.fx.log = vec!["Warm and cool light cross the town square.".into()];
            }
            PreviewMode::Pickup => {
                game.sim.loot.push(Loot {
                    pos: game.sim.player.pos + Vec2::new(18.0, 0.0),
                    item: Item {
                        name: "Swift Dirk of the Fox".into(),
                        base_name: "Dirk".into(),
                        slot: Slot::Weapon,
                        rarity: Rarity::Magic,
                        item_level: 4,
                        affixes: vec!["Swift".into(), "of the Fox".into()],
                        power: 4,
                        armor: 0,
                        vitality: 1,
                        haste: 3,
                        value: 29,
                    },
                    bob: 0.0,
                });
            }
            PreviewMode::Inventory => {
                game.sim.player.inventory = crate::game::Backpack::from_items(vec![
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
                    Item {
                        name: "Ring of Warding".into(),
                        base_name: "Ring".into(),
                        slot: Slot::Charm,
                        rarity: Rarity::Magic,
                        item_level: 2,
                        affixes: vec!["of Warding".into()],
                        power: 0,
                        armor: 3,
                        vitality: 1,
                        haste: 0,
                        value: 23,
                    },
                    Item {
                        name: "Storm Mantle".into(),
                        base_name: "Mantle".into(),
                        slot: Slot::Armor,
                        rarity: Rarity::Rare,
                        item_level: 4,
                        affixes: vec!["Storm".into(), "of the Fox".into(), "Iron".into()],
                        power: 0,
                        armor: 5,
                        vitality: 3,
                        haste: 1,
                        value: 44,
                    },
                ]);
                game.sim.player.equipment.weapon = Some(Item {
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
                });
                game.ui.inventory_backpack_cursor = 2;
                game.ui.mode = UiMode::Inventory;
            }
            PreviewMode::Character => {
                game.sim.player.stats.unspent_stat_points = 3;
                game.ui.mode = UiMode::Character;
            }
            PreviewMode::SkillBook => {
                game.sim.player.disciplines.melee.xp = 18;
                game.sim.player.disciplines.magic.level = 8;
                game.sim.player.disciplines.magic.xp = 12;
                game.sim.player.disciplines.magic.next_xp = 808;
                game.sim.player.bound_abilities = [AbilityKind::Fireball, AbilityKind::Meteor];
                game.ui.skill_book_cursor = 1;
                game.ui.skill_book_ability_cursor = 2;
                game.ui.skill_book_focus = SkillBookFocus::Skills;
                game.ui.mode = UiMode::SkillBook;
            }
            PreviewMode::WorldMap => {
                game.ui.mode = UiMode::WorldMap;
                for center in [
                    ivec2(0, 0),
                    ivec2(0, -8),
                    ivec2(0, -16),
                    ivec2(8, 0),
                    ivec2(16, 0),
                    ivec2(24, 0),
                    ivec2(30, 0),
                    ivec2(0, 8),
                    ivec2(0, 16),
                    ivec2(0, 24),
                    ivec2(0, 32),
                    ivec2(0, 40),
                    ivec2(-8, 8),
                    ivec2(-16, 16),
                    ivec2(-24, 20),
                ] {
                    game.reveal_around_tile(center, 10);
                }
                reveal_preview_towns(game, 4);
            }
            PreviewMode::ShopBuy => game.ui.mode = UiMode::Merchant,
            PreviewMode::ShopSell => {
                game.ui.mode = UiMode::Merchant;
                game.ui.shop_tab = ShopTab::Sell;
            }
            PreviewMode::Trainer => game.ui.mode = UiMode::Trainer,
            PreviewMode::Travel => {
                reveal_preview_towns(game, 5);
                game.ui.mode = UiMode::Travel;
            }
            PreviewMode::Quest => {
                game.sim.active_quest = Some(Quest {
                    id: 99,
                    kind: QuestKind::RecoverItems,
                    signature: crate::game::QuestSignature::RecoverItems { landmark_id: 99 },
                    stage: QuestStage::Active,
                    giver: SettlementSite {
                        id: 0,
                        center: ivec2(0, 0),
                        tier: SettlementTier::Town,
                    },
                    title: "Recover cart ledgers".into(),
                    objective: "Recover 3 cart ledgers at the abandoned cart beyond Briarwatch"
                        .into(),
                    target_pos: game.sim.player.pos + Vec2::new(180.0, -64.0),
                    progress: 1,
                    goal: 3,
                    reward: QuestReward {
                        gold: 32,
                        xp: 28,
                        item_chance: 0.16,
                    },
                });
                game.fx.skill_xp_toasts.push(SkillXpToast {
                    kind: DisciplineKind::Magic,
                    amount: 14,
                    ttl: 2.4,
                });
            }
        }
    }
}

fn reveal_preview_towns(game: &mut Game, count: usize) {
    let sites = game.world.settlements_near_tile(ivec2(0, 0), 1_200);
    for site in sites
        .into_iter()
        .filter(|site| site.tier == crate::world::SettlementTier::Town)
    {
        game.reveal_around_tile(site.center, 2);
        if game.sim.travel_destinations.len() >= count {
            break;
        }
    }
}

pub enum PreviewRequest {
    None,
    Single { mode: PreviewMode, path: PathBuf },
    All { dir: PathBuf },
}

impl PreviewRequest {
    pub(crate) fn from_args<I>(mut args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let mut mode = None;
        let mut output = None;
        let mut output_dir = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--preview" => mode = args.next().and_then(|value| PreviewMode::from_slug(&value)),
                "--output" => output = args.next().map(PathBuf::from),
                "--preview-all" => output_dir = args.next().map(PathBuf::from),
                _ => {}
            }
        }

        if let Some(dir) = output_dir {
            return PreviewRequest::All { dir };
        }
        match (mode, output) {
            (Some(mode), Some(path)) => PreviewRequest::Single { mode, path },
            _ => PreviewRequest::None,
        }
    }

    pub fn is_preview(&self) -> bool {
        !matches!(self, PreviewRequest::None)
    }
}

pub struct PreviewRunner {
    queue: Vec<PreviewMode>,
    output_dir: PathBuf,
    frame_in_mode: usize,
    index: usize,
}

impl PreviewRunner {
    pub fn from_request(request: &PreviewRequest) -> Option<Self> {
        match request {
            PreviewRequest::None | PreviewRequest::Single { .. } => None,
            PreviewRequest::All { dir } => Some(Self {
                queue: PreviewMode::ALL.to_vec(),
                output_dir: dir.clone(),
                frame_in_mode: 0,
                index: 0,
            }),
        }
    }

    pub fn current_mode(&self) -> PreviewMode {
        self.queue[self.index]
    }

    pub fn current_path(&self) -> PathBuf {
        self.output_dir
            .join(format!("{}.png", self.current_mode().slug()))
    }

    pub fn tick(&mut self) -> PreviewTick {
        self.frame_in_mode += 1;
        if self.frame_in_mode < 3 {
            return PreviewTick::Continue;
        }
        let path = self.current_path();
        self.index += 1;
        self.frame_in_mode = 0;
        if self.index >= self.queue.len() {
            PreviewTick::CaptureAndFinish(path)
        } else {
            PreviewTick::CaptureAndAdvance(path, self.current_mode())
        }
    }
}

pub enum PreviewTick {
    Continue,
    CaptureAndAdvance(PathBuf, PreviewMode),
    CaptureAndFinish(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_preview_request() {
        let request = PreviewRequest::from_args(
            ["--preview", "inventory", "--output", "/tmp/inventory.png"]
                .into_iter()
                .map(String::from),
        );
        match request {
            PreviewRequest::Single { mode, path } => {
                assert_eq!(mode, PreviewMode::Inventory);
                assert_eq!(path, PathBuf::from("/tmp/inventory.png"));
            }
            _ => panic!("expected a single preview request"),
        }
    }

    #[test]
    fn configures_each_preview_mode() {
        let mut game = Game::new(4);
        PreviewMode::ShopSell.configure(&mut game);
        assert_eq!(game.ui.mode, UiMode::Merchant);
        assert_eq!(game.ui.shop_tab, ShopTab::Sell);

        PreviewMode::WorldMap.configure(&mut game);
        assert_eq!(game.ui.mode, UiMode::WorldMap);
        assert!(game.sim.known_tiles.len() > 600);

        PreviewMode::SkillBook.configure(&mut game);
        assert_eq!(game.ui.mode, UiMode::SkillBook);
        assert_eq!(game.ui.skill_book_cursor, 1);
        assert_eq!(game.ui.skill_book_focus, SkillBookFocus::Skills);

        PreviewMode::Lighting.configure(&mut game);
        assert_eq!(game.fx.projectiles.len(), 1);
        assert_eq!(game.fx.pulses.len(), 1);
        assert_eq!(game.sim.loot.len(), 1);

        PreviewMode::Travel.configure(&mut game);
        assert_eq!(game.ui.mode, UiMode::Travel);
        assert_eq!(game.ui.shop_tab, ShopTab::Buy);
    }

    #[test]
    fn preview_runner_advances_across_all_modes() {
        let request = PreviewRequest::All {
            dir: PathBuf::from("/tmp/previews"),
        };
        let mut runner = PreviewRunner::from_request(&request).expect("runner");
        assert_eq!(runner.current_mode(), PreviewMode::Gameplay);
        assert!(matches!(runner.tick(), PreviewTick::Continue));
        assert!(matches!(runner.tick(), PreviewTick::Continue));
        assert!(matches!(
            runner.tick(),
            PreviewTick::CaptureAndAdvance(_, PreviewMode::Lighting)
        ));
    }
}
