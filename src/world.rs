use macroquad::prelude::*;

pub const TILE: f32 = 24.0;
pub const TOWN_RADIUS: i32 = 8;
pub const BIOME_BAND: i32 = 18;
const PROVINCE_SIZE: i32 = 44;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Grass,
    Flowers,
    Fungus,
    Ash,
    Ruins,
    Road,
    Wall,
    Floor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    Town,
    Meadow,
    FungalGrove,
    Ashfield,
    OldRuins,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Town => "Ember Town",
            Biome::Meadow => "Meadow",
            Biome::FungalGrove => "Fungal Grove",
            Biome::Ashfield => "Ashfield",
            Biome::OldRuins => "Old Ruins",
        }
    }
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub kind: TileKind,
    pub walkable: bool,
}

impl Tile {
    pub fn colors(self) -> (Color, Color) {
        match self.kind {
            TileKind::Grass => (
                Color::from_rgba(23, 44, 28, 255),
                Color::from_rgba(86, 178, 96, 255),
            ),
            TileKind::Flowers => (
                Color::from_rgba(30, 46, 28, 255),
                Color::from_rgba(228, 108, 216, 255),
            ),
            TileKind::Fungus => (
                Color::from_rgba(16, 35, 39, 255),
                Color::from_rgba(95, 228, 226, 255),
            ),
            TileKind::Ash => (
                Color::from_rgba(35, 35, 38, 255),
                Color::from_rgba(164, 164, 170, 255),
            ),
            TileKind::Ruins => (
                Color::from_rgba(45, 38, 22, 255),
                Color::from_rgba(230, 188, 88, 255),
            ),
            TileKind::Road => (
                Color::from_rgba(56, 43, 28, 255),
                Color::from_rgba(216, 172, 92, 255),
            ),
            TileKind::Wall => (
                Color::from_rgba(20, 21, 24, 255),
                Color::from_rgba(140, 146, 154, 255),
            ),
            TileKind::Floor => (
                Color::from_rgba(33, 31, 34, 255),
                Color::from_rgba(128, 130, 138, 255),
            ),
        }
    }

    pub fn glyph(self, shimmer: u64) -> &'static str {
        match self.kind {
            TileKind::Grass => {
                if shimmer % 11 == 0 {
                    "\""
                } else {
                    "."
                }
            }
            TileKind::Flowers => {
                if shimmer % 5 == 0 {
                    "*"
                } else {
                    ","
                }
            }
            TileKind::Fungus => {
                if shimmer % 7 == 0 {
                    "o"
                } else {
                    ";"
                }
            }
            TileKind::Ash => {
                if shimmer % 9 == 0 {
                    "`"
                } else {
                    "."
                }
            }
            TileKind::Ruins => {
                if shimmer % 8 == 0 {
                    "#"
                } else {
                    ":"
                }
            }
            TileKind::Road => "=",
            TileKind::Wall => "#",
            TileKind::Floor => ".",
        }
    }
}

pub struct World {
    pub seed: u64,
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn tile(&self, pos: IVec2) -> Tile {
        if manhattan(pos) <= TOWN_RADIUS {
            town_tile(pos)
        } else {
            wilderness_tile(pos, self.seed, self.biome_at_tile(pos))
        }
    }

    pub fn world_to_tile(pos: Vec2) -> IVec2 {
        ivec2((pos.x / TILE).floor() as i32, (pos.y / TILE).floor() as i32)
    }

    pub fn tile_center(pos: IVec2) -> Vec2 {
        vec2(
            pos.x as f32 * TILE + TILE * 0.5,
            pos.y as f32 * TILE + TILE * 0.5,
        )
    }

    pub fn walkable_at_world(&self, pos: Vec2) -> bool {
        self.tile(Self::world_to_tile(pos)).walkable
    }

    pub fn collides_circle(&self, center: Vec2, radius: f32) -> bool {
        let points = [
            center,
            center + vec2(radius, 0.0),
            center + vec2(-radius, 0.0),
            center + vec2(0.0, radius),
            center + vec2(0.0, -radius),
        ];
        points
            .into_iter()
            .any(|point| !self.walkable_at_world(point))
    }

    pub fn biome_name(&self, pos: Vec2) -> &'static str {
        self.biome_at_world(pos).name()
    }

    pub fn biome_level(&self, pos: Vec2) -> i32 {
        self.biome_level_at_tile(Self::world_to_tile(pos))
    }

    pub fn biome_at_world(&self, pos: Vec2) -> Biome {
        self.biome_at_tile(Self::world_to_tile(pos))
    }

    pub fn biome_at_tile(&self, pos: IVec2) -> Biome {
        if manhattan(pos) <= TOWN_RADIUS {
            Biome::Town
        } else {
            province_biome(pos, self.seed)
        }
    }

    pub fn biome_level_at_tile(&self, pos: IVec2) -> i32 {
        if manhattan(pos) <= TOWN_RADIUS {
            0
        } else {
            let distance = organic_distance(pos, self.seed);
            1 + ((distance - TOWN_RADIUS as f32 - 1.0).max(0.0) / BIOME_BAND as f32).floor() as i32
        }
    }
}

fn manhattan(pos: IVec2) -> i32 {
    pos.x.abs() + pos.y.abs()
}

fn organic_distance(pos: IVec2, seed: u64) -> f32 {
    let radial = vec2(pos.x as f32, pos.y as f32).length();
    let broad_warp = sample_noise(pos, 42, seed ^ 0xA11C_E001) * 16.0;
    let local_warp = sample_noise(pos, 18, seed ^ 0xF00D_5EED) * 8.0;
    (radial + broad_warp + local_warp).max(0.0)
}

fn province_biome(pos: IVec2, seed: u64) -> Biome {
    let warped = vec2(
        pos.x as f32 + sample_noise(pos, 27, seed ^ 0xCAFE_1001) * 18.0,
        pos.y as f32 + sample_noise(pos, 31, seed ^ 0xCAFE_2002) * 18.0,
    );
    let cell = ivec2(
        (warped.x / PROVINCE_SIZE as f32).floor() as i32,
        (warped.y / PROVINCE_SIZE as f32).floor() as i32,
    );
    let mut best = (f32::INFINITY, Biome::Meadow);
    for y in cell.y - 1..=cell.y + 1 {
        for x in cell.x - 1..=cell.x + 1 {
            let site = province_site(ivec2(x, y), seed);
            let distance = warped.distance_squared(site);
            if distance < best.0 {
                best = (distance, province_kind(ivec2(x, y), seed));
            }
        }
    }
    best.1
}

fn province_site(cell: IVec2, seed: u64) -> Vec2 {
    let jitter_x = hash_unit(cell.x, cell.y, seed ^ 0xB10B_1001) * PROVINCE_SIZE as f32 * 0.34;
    let jitter_y = hash_unit(cell.x, cell.y, seed ^ 0xB10B_2002) * PROVINCE_SIZE as f32 * 0.34;
    vec2(
        (cell.x as f32 + 0.5) * PROVINCE_SIZE as f32 + jitter_x,
        (cell.y as f32 + 0.5) * PROVINCE_SIZE as f32 + jitter_y,
    )
}

fn province_kind(cell: IVec2, seed: u64) -> Biome {
    match hash3(cell.x, cell.y, seed ^ 0xB10B_3003) % 4 {
        0 => Biome::Meadow,
        1 => Biome::FungalGrove,
        2 => Biome::Ashfield,
        _ => Biome::OldRuins,
    }
}

fn town_tile(pos: IVec2) -> Tile {
    let on_road = pos.x == 0 || pos.y == 0;
    let house = matches!(
        (pos.x, pos.y),
        (-6..=-3, -6..=-2) | (3..=6, -6..=-2) | (-6..=-3, 2..=6) | (3..=6, 2..=6)
    );
    if house {
        let doorway = pos.x == -4 && pos.y == -2
            || pos.x == 4 && pos.y == -2
            || pos.x == -4 && pos.y == 2
            || pos.x == 4 && pos.y == 2;
        if doorway {
            Tile {
                kind: TileKind::Floor,
                walkable: true,
            }
        } else {
            Tile {
                kind: TileKind::Wall,
                walkable: false,
            }
        }
    } else if on_road {
        Tile {
            kind: TileKind::Road,
            walkable: true,
        }
    } else {
        Tile {
            kind: TileKind::Flowers,
            walkable: true,
        }
    }
}

fn wilderness_tile(pos: IVec2, seed: u64, biome: Biome) -> Tile {
    let detail = (hash3(pos.x, pos.y, seed ^ 0xDEAD_BEEF) % 100) as i32;
    let patch = sample_noise(pos, 10, seed ^ 0xC0FF_EE11);
    let sparse_patch = sample_noise(pos, 24, seed ^ 0xD15E_A5ED);
    let kind = match biome {
        Biome::Town => TileKind::Flowers,
        Biome::Meadow => {
            if patch > 0.18 || detail < 10 {
                TileKind::Flowers
            } else {
                TileKind::Grass
            }
        }
        Biome::FungalGrove => {
            if patch < -0.45 {
                TileKind::Grass
            } else {
                TileKind::Fungus
            }
        }
        Biome::Ashfield => {
            if sparse_patch > 0.56 {
                TileKind::Ruins
            } else {
                TileKind::Ash
            }
        }
        Biome::OldRuins => {
            if sparse_patch < -0.5 {
                TileKind::Ash
            } else {
                TileKind::Ruins
            }
        }
    };
    let obstacle = sample_noise(pos, 5, seed ^ 0xB10C_5EED);
    let blocked = matches!(kind, TileKind::Ruins) && obstacle > 0.62 && detail < 42;
    Tile {
        kind: if blocked { TileKind::Wall } else { kind },
        walkable: !blocked,
    }
}

fn sample_noise(pos: IVec2, cell_size: i32, seed: u64) -> f32 {
    let x0 = pos.x.div_euclid(cell_size);
    let y0 = pos.y.div_euclid(cell_size);
    let tx = smoothstep(pos.x.rem_euclid(cell_size) as f32 / cell_size as f32);
    let ty = smoothstep(pos.y.rem_euclid(cell_size) as f32 / cell_size as f32);

    let n00 = hash_unit(x0, y0, seed);
    let n10 = hash_unit(x0 + 1, y0, seed);
    let n01 = hash_unit(x0, y0 + 1, seed);
    let n11 = hash_unit(x0 + 1, y0 + 1, seed);
    let top = lerp(n00, n10, tx);
    let bottom = lerp(n01, n11, tx);
    lerp(top, bottom, ty)
}

fn hash_unit(x: i32, y: i32, seed: u64) -> f32 {
    let value = (hash3(x, y, seed) % 10_001) as f32 / 10_000.0;
    value * 2.0 - 1.0
}

fn smoothstep(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn hash3(x: i32, y: i32, seed: u64) -> u64 {
    let mut value = seed
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn danger_levels_progress_outward_from_town_forever() {
        let world = World::new(7);
        assert_eq!(world.biome_level(World::tile_center(ivec2(0, 0))), 0);
        assert!(world.biome_level(World::tile_center(ivec2(0, -18))) >= 1);
        assert!(world.biome_level(World::tile_center(ivec2(0, 180))) >= 8);
    }

    #[test]
    fn nearby_points_can_repeat_or_split_biomes_independently_of_level() {
        let world = World::new(7);
        let same_level_samples = [
            ivec2(32, 0),
            ivec2(24, 24),
            ivec2(0, 32),
            ivec2(-24, 24),
            ivec2(-32, 0),
        ];
        let biomes: Vec<_> = same_level_samples
            .into_iter()
            .map(|pos| world.biome_at_tile(pos))
            .collect();
        assert!(biomes.windows(2).any(|pair| pair[0] != pair[1]));
    }

    #[test]
    fn same_biome_can_reappear_in_distant_regions() {
        let world = World::new(7);
        let biome = world.biome_at_tile(ivec2(30, 0));
        assert!((1..=12).any(|step| {
            world.biome_at_tile(ivec2(30 + step * PROVINCE_SIZE, step * 7)) == biome
        }));
    }

    #[test]
    fn same_coordinate_always_generates_same_tile() {
        let world = World::new(7);
        let pos = ivec2(13_337, -42_424);
        assert_eq!(world.tile(pos).kind as u8, world.tile(pos).kind as u8);
        assert_eq!(world.tile(pos).walkable, world.tile(pos).walkable);
    }
}
