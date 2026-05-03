use std::collections::HashMap;

use macroquad::prelude::*;

use crate::{
    game::{Game, Monster, UiMode},
    ui,
    world::{Biome, TILE, Tile, TileKind, World, hash3},
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
    accent_seed: u64,
    tile: Tile,
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
        let shake = if game.fx.screen_shake > 0.0 {
            vec2(
                (game.runtime.elapsed * 73.0).sin() * game.fx.screen_shake,
                (game.runtime.elapsed * 91.0).cos() * game.fx.screen_shake,
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
        match game.ui.mode {
            UiMode::Inventory => ui::draw_inventory(game),
            UiMode::Character => ui::draw_character(game),
            UiMode::SkillBook => ui::draw_skill_book(game),
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
        let shimmer_tick = game.runtime.elapsed as u64;
        let lights = collect_lights(game);
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
                        accent_seed: hash3(x, y, game.world.seed ^ 0xACCE_5510),
                        tile,
                    }
                });
                let world = World::tile_center(tile_pos);
                let screen = world_to_screen(world, camera);
                let biome = game.world.biome_at_tile(tile_pos);
                let light = light_at(world, biome, &lights);
                draw_rectangle(
                    screen.x - TILE * 0.5,
                    screen.y - TILE * 0.5,
                    TILE + 1.0,
                    TILE + 1.0,
                    apply_light(cached.bg, light),
                );
                self.draw_biome_edge(game, tile_pos, screen, biome);
                if !should_draw_tile_glyph(cached, tile_pos, game.world.seed) {
                    self.draw_tile_accent(
                        cached,
                        tile_pos,
                        screen,
                        game.world.seed,
                        game.runtime.elapsed,
                    );
                    continue;
                }
                let shimmer = cached.shimmer_seed + shimmer_tick;
                draw_text_centered(
                    cached.tile.glyph(shimmer),
                    screen,
                    18.0,
                    with_alpha(apply_light(cached.fg, light.scale(0.7)), 0.72),
                );
                self.draw_tile_accent(
                    cached,
                    tile_pos,
                    screen,
                    game.world.seed,
                    game.runtime.elapsed,
                );
            }
        }
    }

    fn draw_biome_edge(&self, game: &Game, tile_pos: IVec2, screen: Vec2, biome: Biome) {
        let edge = [
            (
                ivec2(-1, 0),
                vec2(-TILE * 0.5, -TILE * 0.5),
                vec2(3.0, TILE),
            ),
            (
                ivec2(1, 0),
                vec2(TILE * 0.5 - 3.0, -TILE * 0.5),
                vec2(3.0, TILE),
            ),
            (
                ivec2(0, -1),
                vec2(-TILE * 0.5, -TILE * 0.5),
                vec2(TILE, 3.0),
            ),
            (
                ivec2(0, 1),
                vec2(-TILE * 0.5, TILE * 0.5 - 3.0),
                vec2(TILE, 3.0),
            ),
        ];
        for (offset, rect_offset, size) in edge {
            if game.world.biome_at_tile(tile_pos + offset) != biome {
                draw_rectangle(
                    screen.x + rect_offset.x,
                    screen.y + rect_offset.y,
                    size.x,
                    size.y,
                    with_alpha(BLACK, 0.13),
                );
            }
        }
    }

    fn draw_tile_accent(
        &self,
        cached: CachedTileVisual,
        tile_pos: IVec2,
        screen: Vec2,
        world_seed: u64,
        elapsed: f32,
    ) {
        let cluster = hash3(
            tile_pos.x.div_euclid(4),
            tile_pos.y.div_euclid(4),
            world_seed ^ 0xC1A5_7E2D,
        ) % 100;
        let detail = cached.accent_seed % 100;
        match cached.tile.kind {
            TileKind::Grass if cluster > 62 && detail < 44 => {
                let lean = if cached.accent_seed & 1 == 0 {
                    -2.0
                } else {
                    2.0
                };
                draw_line(
                    screen.x - 5.0,
                    screen.y + 7.0,
                    screen.x - 5.0 + lean,
                    screen.y - 5.0,
                    1.5,
                    with_alpha(Color::from_rgba(140, 220, 128, 255), 0.55),
                );
                if detail < 20 {
                    draw_line(
                        screen.x + 3.0,
                        screen.y + 6.0,
                        screen.x + 5.0,
                        screen.y - 3.0,
                        1.2,
                        with_alpha(Color::from_rgba(140, 220, 128, 255), 0.42),
                    );
                }
            }
            TileKind::Flowers if cluster > 52 && detail < 48 => {
                draw_circle(
                    screen.x - 6.0,
                    screen.y + 5.0,
                    1.8,
                    with_alpha(Color::from_rgba(255, 142, 228, 255), 0.72),
                );
                if detail < 24 {
                    draw_circle(
                        screen.x + 5.0,
                        screen.y - 4.0,
                        1.4,
                        with_alpha(Color::from_rgba(255, 180, 236, 255), 0.52),
                    );
                }
            }
            TileKind::Fungus if cluster > 58 && detail < 42 => {
                let pulse =
                    ((elapsed * 1.8 + (cached.accent_seed % 17) as f32).sin() * 0.5 + 0.5) * 0.18;
                draw_circle_lines(
                    screen.x,
                    screen.y + 2.0,
                    4.0 + pulse * 8.0,
                    1.0,
                    with_alpha(Color::from_rgba(95, 228, 226, 255), 0.28 + pulse),
                );
            }
            TileKind::Ash if cluster > 54 && detail < 40 => {
                draw_line(
                    screen.x - 7.0,
                    screen.y - 4.0,
                    screen.x - 1.0,
                    screen.y + 1.0,
                    1.2,
                    with_alpha(Color::from_rgba(194, 194, 202, 255), 0.34),
                );
                draw_line(
                    screen.x - 1.0,
                    screen.y + 1.0,
                    screen.x + 6.0,
                    screen.y + 4.0,
                    1.2,
                    with_alpha(Color::from_rgba(194, 194, 202, 255), 0.34),
                );
                if detail < 14 {
                    let ember = ((elapsed * 2.4 + (cached.accent_seed % 11) as f32).sin() * 0.5
                        + 0.5)
                        * 0.35;
                    draw_circle(
                        screen.x + 6.0,
                        screen.y - 5.0,
                        1.2,
                        with_alpha(Color::from_rgba(255, 132, 64, 255), 0.3 + ember),
                    );
                }
            }
            TileKind::Ruins if cluster > 48 && detail < 46 => {
                draw_line(
                    screen.x - 7.0,
                    screen.y + 5.0,
                    screen.x,
                    screen.y - 5.0,
                    1.4,
                    with_alpha(Color::from_rgba(230, 188, 88, 255), 0.38),
                );
                draw_line(
                    screen.x,
                    screen.y - 5.0,
                    screen.x + 6.0,
                    screen.y - 1.0,
                    1.4,
                    with_alpha(Color::from_rgba(230, 188, 88, 255), 0.28),
                );
            }
            _ => {}
        }
    }

    fn draw_loot(&self, game: &Game, camera: Vec2) {
        for loot in &game.sim.loot {
            let screen = world_to_screen(loot.pos, camera) + vec2(0.0, loot.bob.sin() * 4.0);
            draw_glow(screen, 8.0, loot.item.rarity.color(), 0.8);
            draw_text_centered("*", screen, 24.0, loot.item.rarity.color());
        }
    }

    fn draw_monsters(&self, game: &Game, camera: Vec2) {
        for monster in &game.sim.monsters {
            self.draw_monster(monster, camera);
        }
    }

    fn draw_monster(&self, monster: &Monster, camera: Vec2) {
        let screen = world_to_screen(monster.pos + monster.hit_offset, camera)
            + vec2(0.0, monster.wobble.sin() * 2.5);
        let base_color = monster.kind.color();
        let color = if monster.hit_flash > 0.0 {
            light_color(base_color, 0.65)
        } else {
            base_color
        };
        if monster.rank != crate::content::MonsterRank::Normal {
            draw_circle_lines(
                screen.x,
                screen.y,
                if monster.rank == crate::content::MonsterRank::Boss {
                    21.0
                } else {
                    18.0
                },
                if monster.rank == crate::content::MonsterRank::Boss {
                    2.0
                } else {
                    1.5
                },
                with_alpha(monster.rank.accent_color(), 0.92),
            );
        }
        draw_circle(screen.x, screen.y + 6.0, 10.0, with_alpha(BLACK, 0.28));
        draw_circle(screen.x, screen.y, 13.0, with_alpha(color, 0.16));
        match monster.kind {
            crate::content::MonsterKind::Imp => {
                draw_line(
                    screen.x - 8.0,
                    screen.y - 8.0,
                    screen.x - 3.0,
                    screen.y - 14.0,
                    2.0,
                    with_alpha(color, 0.84),
                );
                draw_line(
                    screen.x + 8.0,
                    screen.y - 8.0,
                    screen.x + 3.0,
                    screen.y - 14.0,
                    2.0,
                    with_alpha(color, 0.84),
                );
            }
            crate::content::MonsterKind::Slime => {
                draw_line(
                    screen.x - 10.0,
                    screen.y + 10.0,
                    screen.x + 10.0,
                    screen.y + 10.0,
                    2.0,
                    with_alpha(color, 0.58),
                );
            }
            crate::content::MonsterKind::Brute => {
                draw_text_centered("B", screen + vec2(2.0, 2.0), 26.0, with_alpha(BLACK, 0.45));
                draw_line(
                    screen.x - 11.0,
                    screen.y - 9.0,
                    screen.x - 7.0,
                    screen.y - 14.0,
                    2.5,
                    with_alpha(color, 0.66),
                );
                draw_line(
                    screen.x + 11.0,
                    screen.y - 9.0,
                    screen.x + 7.0,
                    screen.y - 14.0,
                    2.5,
                    with_alpha(color, 0.66),
                );
            }
            crate::content::MonsterKind::Wisp => {
                draw_circle_lines(
                    screen.x,
                    screen.y,
                    17.0 + monster.wobble.sin() * 1.4,
                    1.0,
                    with_alpha(color, 0.28),
                );
            }
            crate::content::MonsterKind::Hound => {
                draw_line(
                    screen.x - 10.0,
                    screen.y + 5.0,
                    screen.x + 10.0,
                    screen.y + 5.0,
                    2.0,
                    with_alpha(color, 0.7),
                );
            }
            crate::content::MonsterKind::Beetle => {
                draw_circle_lines(screen.x, screen.y, 15.0, 2.0, with_alpha(color, 0.54));
            }
            crate::content::MonsterKind::Cinderling => {
                draw_line(
                    screen.x,
                    screen.y - 16.0,
                    screen.x,
                    screen.y - 8.0,
                    2.0,
                    with_alpha(color, 0.82),
                );
            }
            crate::content::MonsterKind::Revenant => {
                draw_line(
                    screen.x - 9.0,
                    screen.y - 11.0,
                    screen.x + 9.0,
                    screen.y - 11.0,
                    2.0,
                    with_alpha(color, 0.72),
                );
            }
        }
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
            monster.rank.accent_color(),
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
        for npc in &game.sim.npcs {
            let screen = world_to_screen(npc.pos, camera);
            draw_circle(screen.x, screen.y + 6.0, 10.0, with_alpha(BLACK, 0.28));
            draw_circle(screen.x, screen.y, 14.0, with_alpha(npc.kind.color(), 0.16));
            draw_text_centered(
                &npc.kind.glyph().to_string(),
                screen,
                26.0,
                npc.kind.color(),
            );
            if npc.pos.distance(game.sim.player.pos) <= 42.0 && game.ui.mode == UiMode::None {
                ui::draw_world_hotkey_hint("F", "talk", screen + vec2(-24.0, -40.0));
            }
        }
    }

    fn draw_player(&self, game: &Game, camera: Vec2) {
        let screen = world_to_screen(game.sim.player.pos, camera);
        draw_circle(screen.x, screen.y + 7.0, 12.0, with_alpha(BLACK, 0.32));
        draw_circle(screen.x, screen.y, 18.0, with_alpha(WHITE, 0.08));
        draw_text_centered("@", screen, 30.0, WHITE);
        let aim = screen + game.sim.player.facing * 28.0;
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
        for meteor in &game.fx.meteors {
            let screen = world_to_screen(meteor.pos, camera);
            let ratio = (meteor.ttl / 0.72).clamp(0.0, 1.0);
            draw_glow(
                screen,
                10.0 + (1.0 - ratio) * 8.0,
                Color::from_rgba(255, 132, 64, 255),
                0.92,
            );
            draw_circle_lines(
                screen.x,
                screen.y,
                meteor.radius,
                3.0,
                with_alpha(Color::from_rgba(255, 132, 64, 255), 0.8 - ratio * 0.45),
            );
            draw_circle(
                screen.x,
                screen.y,
                10.0 + (1.0 - ratio) * 8.0,
                with_alpha(Color::from_rgba(255, 132, 64, 255), 0.18),
            );
        }
        for projectile in &game.fx.projectiles {
            let screen = world_to_screen(projectile.pos, camera);
            let direction = projectile.vel.normalize_or_zero();
            for (index, scale) in [0.72, 0.46, 0.22].into_iter().enumerate() {
                let trail = screen - direction * (10.0 + index as f32 * 9.0);
                draw_circle(
                    trail.x,
                    trail.y,
                    projectile.radius * scale,
                    with_alpha(projectile.color, 0.22 * scale),
                );
            }
            draw_glow(screen, projectile.radius + 4.0, projectile.color, 1.0);
            draw_circle(screen.x, screen.y, projectile.radius, projectile.color);
        }
        for pulse in &game.fx.pulses {
            let screen = world_to_screen(pulse.pos, camera);
            draw_glow(
                screen,
                pulse.radius * 0.6 + 5.0,
                pulse.color,
                (pulse.ttl * 2.4).clamp(0.0, 0.9),
            );
            draw_circle_lines(
                screen.x,
                screen.y,
                pulse.radius,
                3.0,
                with_alpha(pulse.color, (pulse.ttl * 1.5).clamp(0.0, 1.0)),
            );
        }
        for slash in &game.fx.slash_arcs {
            let center = world_to_screen(slash.pos, camera);
            let base = slash.direction.y.atan2(slash.direction.x);
            let start = base - std::f32::consts::FRAC_PI_2;
            let segments = 10;
            for index in 0..segments {
                let a = start + std::f32::consts::PI * index as f32 / segments as f32;
                let b = start + std::f32::consts::PI * (index + 1) as f32 / segments as f32;
                let from = center + vec2(a.cos(), a.sin()) * slash.radius;
                let to = center + vec2(b.cos(), b.sin()) * slash.radius;
                draw_line(
                    from.x,
                    from.y,
                    to.x,
                    to.y,
                    4.0,
                    with_alpha(slash.color, (slash.ttl * 3.0).clamp(0.0, 1.0)),
                );
            }
        }
        for particle in &game.fx.particles {
            let screen = world_to_screen(particle.pos, camera);
            draw_circle(
                screen.x,
                screen.y,
                particle.radius,
                with_alpha(particle.color, (particle.ttl * 2.0).clamp(0.0, 1.0)),
            );
        }
        for text in &game.fx.floating {
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

#[derive(Clone, Copy)]
struct SceneLight {
    pos: Vec2,
    radius: f32,
    intensity: f32,
    color: Color,
}

#[derive(Clone, Copy, Debug)]
struct LightSample {
    r: f32,
    g: f32,
    b: f32,
}

impl LightSample {
    fn neutral(amount: f32) -> Self {
        Self {
            r: amount,
            g: amount,
            b: amount,
        }
    }

    fn add(mut self, color: Color, amount: f32) -> Self {
        self.r += color.r * amount;
        self.g += color.g * amount;
        self.b += color.b * amount;
        self
    }

    fn clamped(self) -> Self {
        Self {
            r: self.r.min(MAX_LIGHT_CHANNEL),
            g: self.g.min(MAX_LIGHT_CHANNEL),
            b: self.b.min(MAX_LIGHT_CHANNEL),
        }
    }

    fn scale(self, factor: f32) -> Self {
        Self {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
        }
    }
}

const MAX_LIGHT_CHANNEL: f32 = 0.34;

fn collect_lights(game: &Game) -> Vec<SceneLight> {
    let mut lights = vec![SceneLight {
        pos: game.sim.player.pos,
        radius: TILE * 8.0,
        intensity: 0.16,
        color: Color::from_rgba(255, 238, 214, 255),
    }];
    for projectile in &game.fx.projectiles {
        lights.push(SceneLight {
            pos: projectile.pos,
            radius: TILE * 4.0,
            intensity: 0.26,
            color: projectile.color,
        });
    }
    for loot in &game.sim.loot {
        lights.push(SceneLight {
            pos: loot.pos,
            radius: TILE * 2.5,
            intensity: 0.10,
            color: loot.item.rarity.color(),
        });
    }
    for pulse in &game.fx.pulses {
        lights.push(SceneLight {
            pos: pulse.pos,
            radius: pulse.radius.max(TILE * 2.5),
            intensity: 0.08 + pulse.ttl.min(0.4) * 0.20,
            color: pulse.color,
        });
    }
    for meteor in &game.fx.meteors {
        let ratio = (meteor.ttl / 0.72).clamp(0.0, 1.0);
        lights.push(SceneLight {
            pos: meteor.pos,
            radius: TILE * 5.0,
            intensity: 0.10 + (1.0 - ratio) * 0.14,
            color: Color::from_rgba(255, 132, 64, 255),
        });
    }
    lights
}

fn light_at(pos: Vec2, biome: Biome, lights: &[SceneLight]) -> LightSample {
    let ambient = if biome == Biome::Town { 0.08 } else { 0.0 };
    lights
        .iter()
        .fold(LightSample::neutral(ambient), |sample, light| {
            sample.add(light.color, light_falloff(pos, *light))
        })
        .clamped()
}

fn light_falloff(pos: Vec2, light: SceneLight) -> f32 {
    let ratio = (1.0 - pos.distance(light.pos) / light.radius).clamp(0.0, 1.0);
    ratio * ratio * light.intensity
}

fn apply_light(color: Color, light: LightSample) -> Color {
    Color::new(
        color.r + (1.0 - color.r) * light.r,
        color.g + (1.0 - color.g) * light.g,
        color.b + (1.0 - color.b) * light.b,
        color.a,
    )
}

fn light_color(color: Color, amount: f32) -> Color {
    Color::new(
        color.r + (1.0 - color.r) * amount,
        color.g + (1.0 - color.g) * amount,
        color.b + (1.0 - color.b) * amount,
        color.a,
    )
}

fn should_draw_tile_glyph(cached: CachedTileVisual, tile_pos: IVec2, world_seed: u64) -> bool {
    if (tile_pos.x + tile_pos.y).rem_euclid(2) != 0 {
        return false;
    }
    let cluster = hash3(
        tile_pos.x.div_euclid(5),
        tile_pos.y.div_euclid(5),
        world_seed ^ 0xA5C1_1B10,
    ) % 100;
    match cached.tile.kind {
        TileKind::Grass => cluster > 22 || cached.accent_seed % 5 == 0,
        TileKind::Flowers => cluster > 14 || cached.accent_seed % 4 == 0,
        TileKind::Fungus => cluster > 8 || cached.accent_seed % 3 == 0,
        TileKind::Ash => cluster > 18 || cached.accent_seed % 4 == 0,
        TileKind::Ruins => cluster > 10 || cached.accent_seed % 3 == 0,
        TileKind::Thorns => cluster > 12 || cached.accent_seed % 3 == 0,
        TileKind::Mire => cluster > 16 || cached.accent_seed % 4 == 0,
        TileKind::Road | TileKind::Bridge => cached.accent_seed % 3 != 0,
        TileKind::Wall
        | TileKind::Floor
        | TileKind::Tree
        | TileKind::DeadTree
        | TileKind::Rock
        | TileKind::MushroomCluster
        | TileKind::Fence
        | TileKind::Shrine
        | TileKind::Well
        | TileKind::Campfire
        | TileKind::Grave
        | TileKind::StandingStone
        | TileKind::Cart => true,
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

fn draw_glow(center: Vec2, radius: f32, color: Color, strength: f32) {
    for (scale, alpha) in [(3.0, 0.05), (2.1, 0.08), (1.45, 0.14)] {
        draw_circle(
            center.x,
            center.y,
            radius * scale,
            with_alpha(color, alpha * strength),
        );
    }
}

pub(crate) fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha;
    color
}

#[cfg(test)]
mod tests {
    use super::*;

    fn light(color: Color) -> SceneLight {
        SceneLight {
            pos: Vec2::ZERO,
            radius: 10.0,
            intensity: 0.24,
            color,
        }
    }

    #[test]
    fn light_falloff_is_stronger_at_the_source_than_the_edge() {
        let light = light(WHITE);

        assert!(light_falloff(Vec2::ZERO, light) > light_falloff(vec2(10.0, 0.0), light));
        assert_eq!(light_falloff(vec2(10.0, 0.0), light), 0.0);
    }

    #[test]
    fn fireball_light_warms_the_result_more_than_neutral_light() {
        let base = Color::from_rgba(40, 40, 40, 255);
        let fireball = Color::from_rgba(255, 132, 64, 255);
        let warm = apply_light(
            base,
            light_at(Vec2::ZERO, Biome::Meadow, &[light(fireball)]),
        );
        let neutral = apply_light(base, light_at(Vec2::ZERO, Biome::Meadow, &[light(WHITE)]));

        assert!(warm.r - warm.b > neutral.r - neutral.b);
    }

    #[test]
    fn overlapping_colored_lights_clamp_and_mix_deterministically() {
        let warm = light(Color::from_rgba(255, 132, 64, 255));
        let cool = light(Color::from_rgba(128, 214, 255, 255));
        let sample = light_at(Vec2::ZERO, Biome::Town, &[warm, cool]);
        let reversed = light_at(Vec2::ZERO, Biome::Town, &[cool, warm]);

        assert!(sample.r <= MAX_LIGHT_CHANNEL);
        assert!(sample.g <= MAX_LIGHT_CHANNEL);
        assert!(sample.b <= MAX_LIGHT_CHANNEL);
        assert!((sample.r - reversed.r).abs() < f32::EPSILON);
        assert!((sample.g - reversed.g).abs() < f32::EPSILON);
        assert!((sample.b - reversed.b).abs() < f32::EPSILON);
        assert!(sample.r > 0.0);
        assert!(sample.b > 0.0);
    }
}
