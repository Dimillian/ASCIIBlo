use std::{env, path::PathBuf};

use macroquad::prelude::{Vec2, ivec2};

use crate::{
    content::{Item, Rarity, Slot},
    game::{Game, Loot, Monster, ShopTab, UiMode},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewMode {
    Gameplay,
    Pickup,
    Inventory,
    Character,
    SkillBook,
    WorldMap,
    ShopBuy,
    ShopSell,
    Trainer,
    Travel,
}

impl PreviewMode {
    pub const ALL: [PreviewMode; 10] = [
        PreviewMode::Gameplay,
        PreviewMode::Pickup,
        PreviewMode::Inventory,
        PreviewMode::Character,
        PreviewMode::SkillBook,
        PreviewMode::WorldMap,
        PreviewMode::ShopBuy,
        PreviewMode::ShopSell,
        PreviewMode::Trainer,
        PreviewMode::Travel,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            PreviewMode::Gameplay => "gameplay",
            PreviewMode::Pickup => "pickup",
            PreviewMode::Inventory => "inventory",
            PreviewMode::Character => "character",
            PreviewMode::SkillBook => "skill-book",
            PreviewMode::WorldMap => "world-map",
            PreviewMode::ShopBuy => "shop-buy",
            PreviewMode::ShopSell => "shop-sell",
            PreviewMode::Trainer => "trainer",
            PreviewMode::Travel => "travel",
        }
    }

    fn from_slug(value: &str) -> Option<Self> {
        match value {
            "gameplay" => Some(PreviewMode::Gameplay),
            "pickup" => Some(PreviewMode::Pickup),
            "inventory" => Some(PreviewMode::Inventory),
            "character" => Some(PreviewMode::Character),
            "skill-book" => Some(PreviewMode::SkillBook),
            "world-map" => Some(PreviewMode::WorldMap),
            "shop-buy" => Some(PreviewMode::ShopBuy),
            "shop-sell" => Some(PreviewMode::ShopSell),
            "trainer" => Some(PreviewMode::Trainer),
            "travel" => Some(PreviewMode::Travel),
            _ => None,
        }
    }

    pub fn configure(self, game: &mut Game) {
        game.player.stats.gold = 128;
        game.shop_cursor = 0;
        game.travel_cursor = 0;
        game.inventory_cursor = 0;
        game.character_cursor = 0;
        game.skill_book_cursor = 0;
        game.player.stats.unspent_stat_points = 0;
        game.player.stats.unspent_skill_points = 0;
        game.ui_mode = UiMode::None;
        game.shop_tab = ShopTab::Buy;
        game.loot.clear();
        game.floating.clear();
        game.particles.clear();
        game.pulses.clear();
        game.slash_arcs.clear();
        game.projectiles.clear();
        game.player.hp = game.player.max_hp();
        game.player.mana = game.player.max_mana();
        game.player.attack_cd = 0.0;
        game.player.rush_cd = 0.0;
        game.player.nova_cd = 0.0;
        game.player.fireball_cd = 0.0;
        game.player.cleave_cd = 0.0;
        game.log = vec!["The bell in Ember Town rings. Go make trouble.".into()];
        game.preview_hover_screen = Some(Vec2::new(-1_000.0, -1_000.0));

        match self {
            PreviewMode::Gameplay => {
                game.log = vec![
                    "The bell in Ember Town rings. Go make trouble.".into(),
                    "You hit Slime for 14.".into(),
                    "You hit Slime for 15.".into(),
                    "You hit Slime for 17.".into(),
                    "You hit Slime for 16.".into(),
                    "Slime pops. +14 xp.".into(),
                    "Slime drops Swift Mace of Alacrity.".into(),
                    "Nothing close enough to pocket.".into(),
                ];
                game.monsters.clear();
                game.monsters.push(Monster {
                    kind: crate::content::MonsterKind::Brute,
                    pos: game.player.pos + Vec2::new(42.0, 0.0),
                    vel: Vec2::ZERO,
                    hp: 41.0,
                    max_hp: 72.0,
                    level: 3,
                    attack_cd: 0.0,
                    wobble: 0.0,
                });
                game.preview_hover_world = Some(game.player.pos + Vec2::new(42.0, 0.0));
            }
            PreviewMode::Pickup => {
                game.loot.push(Loot {
                    pos: game.player.pos + Vec2::new(18.0, 0.0),
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
            PreviewMode::Inventory => game.ui_mode = UiMode::Inventory,
            PreviewMode::Character => {
                game.player.stats.unspent_stat_points = 3;
                game.ui_mode = UiMode::Character;
            }
            PreviewMode::SkillBook => {
                game.player.stats.unspent_skill_points = 1;
                game.skill_book_cursor = 2;
                game.ui_mode = UiMode::SkillBook;
            }
            PreviewMode::WorldMap => {
                game.ui_mode = UiMode::WorldMap;
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
            }
            PreviewMode::ShopBuy => game.ui_mode = UiMode::Merchant,
            PreviewMode::ShopSell => {
                game.ui_mode = UiMode::Merchant;
                game.shop_tab = ShopTab::Sell;
            }
            PreviewMode::Trainer => game.ui_mode = UiMode::Trainer,
            PreviewMode::Travel => game.ui_mode = UiMode::Travel,
        }
    }
}

pub enum PreviewRequest {
    None,
    Single { mode: PreviewMode, path: PathBuf },
    All { dir: PathBuf },
}

impl PreviewRequest {
    pub fn from_env() -> Self {
        Self::from_args(env::args().skip(1))
    }

    fn from_args<I>(mut args: I) -> Self
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
        assert_eq!(game.ui_mode, UiMode::Merchant);
        assert_eq!(game.shop_tab, ShopTab::Sell);

        PreviewMode::WorldMap.configure(&mut game);
        assert_eq!(game.ui_mode, UiMode::WorldMap);
        assert!(game.known_tiles.len() > 600);

        PreviewMode::SkillBook.configure(&mut game);
        assert_eq!(game.ui_mode, UiMode::SkillBook);
        assert_eq!(game.skill_book_cursor, 2);

        PreviewMode::Travel.configure(&mut game);
        assert_eq!(game.ui_mode, UiMode::Travel);
        assert_eq!(game.shop_tab, ShopTab::Buy);
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
            PreviewTick::CaptureAndAdvance(_, PreviewMode::Pickup)
        ));
    }
}
