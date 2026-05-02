use std::collections::HashMap;

use macroquad::prelude::*;

use crate::{
    game::{Game, Monster, UiMode},
    ui,
    world::{TILE, World, hash3},
};

pub struct Renderer {
    camera: Vec2,
    tile_cache: HashMap<IVec2, CachedTileVisual>,
}

#[derive(Clone, Copy)]
struct CachedTileVisual {
    bg: Color,
    fg: Color,
    shimmer_seed: u64,
    tile: crate::world::Tile,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            camera: World::tile_center(ivec2(0, 0)),
            tile_cache: HashMap::new(),
        }
    }

    pub fn sync_camera(&mut self, target: Vec2, dt: f32) {
        self.camera = self.camera.lerp(target, 1.0 - 0.00008_f32.powf(dt));
    }

    pub fn screen_to_world(&self, screen: Vec2) -> Vec2 {
        screen - screen_center() + self.camera
    }

    pub fn draw(&mut self, game: &Game) {
        clear_background(Color::from_rgba(10, 12, 16, 255));
        let shake = if game.screen_shake > 0.0 {
            vec2(
                (game.elapsed * 73.0).sin() * game.screen_shake,
                (game.elapsed * 91.0).cos() * game.screen_shake,
            )
        } else {
            Vec2::ZERO
        };
        let camera = self.camera + shake;
        self.draw_world(game, camera);
        self.draw_loot(game, camera);
        self.draw_monsters(game, camera);
        self.draw_npcs(game, camera);
        self.draw_player(game, camera);
        self.draw_effects(game, camera);
        ui::draw_hud(game);
        match game.ui_mode {
            UiMode::Inventory => ui::draw_inventory(game),
            UiMode::Character => ui::draw_character(game),
            UiMode::WorldMap => ui::draw_world_map(game),
            UiMode::Merchant => ui::draw_shop(game),
            UiMode::Trainer => ui::draw_trainer(game),
            UiMode::Travel => ui::draw_travel(game),
            UiMode::None => {}
        }
    }

    fn draw_world(&mut self, game: &Game, camera: Vec2) {
        let view_min = World::world_to_tile(camera - screen_center() - vec2(TILE, TILE));
        let view_max = World::world_to_tile(camera + screen_center() + vec2(TILE, TILE));
        let shimmer_tick = game.elapsed as u64;
        for y in view_min.y..=view_max.y {
            for x in view_min.x..=view_max.x {
                let tile_pos = ivec2(x, y);
                let cached = *self.tile_cache.entry(tile_pos).or_insert_with(|| {
                    let tile = game.world.tile(tile_pos);
                    let (bg, fg) = tile.colors();
                    CachedTileVisual {
                        bg,
                        fg,
                        shimmer_seed: hash3(x, y, game.world.seed),
                        tile,
                    }
                });
                let world = World::tile_center(tile_pos);
                let screen = world_to_screen(world, camera);
                draw_rectangle(
                    screen.x - TILE * 0.5,
                    screen.y - TILE * 0.5,
                    TILE + 1.0,
                    TILE + 1.0,
                    cached.bg,
                );
                if (x + y).rem_euclid(2) != 0 {
                    continue;
                }
                let shimmer = cached.shimmer_seed + shimmer_tick;
                draw_text_centered(
                    cached.tile.glyph(shimmer),
                    screen,
                    18.0,
                    with_alpha(cached.fg, 0.72),
                );
            }
        }
    }

    fn draw_loot(&self, game: &Game, camera: Vec2) {
        for loot in &game.loot {
            let screen = world_to_screen(loot.pos, camera) + vec2(0.0, loot.bob.sin() * 4.0);
            draw_circle(
                screen.x,
                screen.y,
                10.0,
                with_alpha(loot.item.rarity.color(), 0.18),
            );
            draw_text_centered("*", screen, 24.0, loot.item.rarity.color());
        }
    }

    fn draw_monsters(&self, game: &Game, camera: Vec2) {
        for monster in &game.monsters {
            self.draw_monster(monster, camera);
        }
    }

    fn draw_monster(&self, monster: &Monster, camera: Vec2) {
        let screen = world_to_screen(monster.pos, camera) + vec2(0.0, monster.wobble.sin() * 2.5);
        let color = monster.kind.color();
        draw_circle(screen.x, screen.y + 6.0, 10.0, with_alpha(BLACK, 0.28));
        draw_circle(screen.x, screen.y, 13.0, with_alpha(color, 0.16));
        draw_text_centered(&monster.kind.glyph().to_string(), screen, 26.0, color);
        let width = 26.0;
        draw_rectangle(
            screen.x - width * 0.5,
            screen.y - 23.0,
            width,
            3.0,
            with_alpha(BLACK, 0.65),
        );
        draw_rectangle(
            screen.x - width * 0.5,
            screen.y - 23.0,
            width * (monster.hp / monster.max_hp).clamp(0.0, 1.0),
            3.0,
            Color::from_rgba(255, 100, 100, 255),
        );
        draw_text(
            &format!("{}", monster.level),
            screen.x + 12.0,
            screen.y + 10.0,
            14.0,
            with_alpha(WHITE, 0.78),
        );
    }

    fn draw_npcs(&self, game: &Game, camera: Vec2) {
        for npc in &game.npcs {
            let screen = world_to_screen(npc.pos, camera);
            draw_circle(screen.x, screen.y + 6.0, 10.0, with_alpha(BLACK, 0.28));
            draw_circle(screen.x, screen.y, 14.0, with_alpha(npc.kind.color(), 0.16));
            draw_text_centered(
                &npc.kind.glyph().to_string(),
                screen,
                26.0,
                npc.kind.color(),
            );
            if npc.pos.distance(game.player.pos) <= 42.0 && game.ui_mode == UiMode::None {
                ui::draw_world_hotkey_hint("F", "talk", screen + vec2(-24.0, -40.0));
            }
        }
    }

    fn draw_player(&self, game: &Game, camera: Vec2) {
        let screen = world_to_screen(game.player.pos, camera);
        draw_circle(screen.x, screen.y + 7.0, 12.0, with_alpha(BLACK, 0.32));
        draw_circle(screen.x, screen.y, 18.0, with_alpha(WHITE, 0.08));
        draw_text_centered("@", screen, 30.0, WHITE);
        let aim = screen + game.player.facing * 28.0;
        draw_line(
            screen.x,
            screen.y,
            aim.x,
            aim.y,
            2.0,
            with_alpha(WHITE, 0.6),
        );
    }

    fn draw_effects(&self, game: &Game, camera: Vec2) {
        for pulse in &game.pulses {
            let screen = world_to_screen(pulse.pos, camera);
            draw_circle_lines(
                screen.x,
                screen.y,
                pulse.radius,
                3.0,
                with_alpha(pulse.color, (pulse.ttl * 1.5).clamp(0.0, 1.0)),
            );
        }
        for particle in &game.particles {
            let screen = world_to_screen(particle.pos, camera);
            draw_circle(
                screen.x,
                screen.y,
                particle.radius,
                with_alpha(particle.color, (particle.ttl * 2.0).clamp(0.0, 1.0)),
            );
        }
        for text in &game.floating {
            let screen = world_to_screen(text.pos, camera);
            draw_text_centered(
                &text.text,
                screen,
                22.0,
                with_alpha(text.color, (text.ttl * 1.3).clamp(0.0, 1.0)),
            );
        }
    }
}

fn world_to_screen(world: Vec2, camera: Vec2) -> Vec2 {
    world - camera + screen_center()
}

fn screen_center() -> Vec2 {
    vec2(screen_width() * 0.5, screen_height() * 0.5)
}

fn draw_text_centered(text: &str, center: Vec2, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(
        text,
        center.x - dims.width * 0.5,
        center.y + dims.height * 0.35,
        size,
        color,
    );
}

pub(crate) fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha;
    color
}
