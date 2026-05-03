use macroquad::prelude::*;

pub const TILE: f32 = 24.0;
pub const TOWN_RADIUS: i32 = 12;
pub const BIOME_BAND: i32 = 26;
const PROVINCE_SIZE: i32 = 104;
const SETTLEMENT_CELL: i32 = 120;
const ROAD_SEARCH_RADIUS_CELLS: i32 = 3;
const FEATURE_CELL: i32 = 28;
const STARTER_TOWN_DRY_BUFFER: i32 = 28;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    Grass,
    Flowers,
    Fungus,
    Ash,
    Ruins,
    Thorns,
    Mire,
    Road,
    Bridge,
    Wall,
    Floor,
    Tree,
    DeadTree,
    Rock,
    MushroomCluster,
    Fence,
    Shrine,
    Well,
    Campfire,
    Grave,
    StandingStone,
    Cart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Biome {
    Town,
    Meadow,
    FungalGrove,
    Ashfield,
    OldRuins,
    Thornwood,
    Mistfen,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Town => "Settlement",
            Biome::Meadow => "Meadow",
            Biome::FungalGrove => "Fungal Grove",
            Biome::Ashfield => "Ashfield",
            Biome::OldRuins => "Old Ruins",
            Biome::Thornwood => "Thornwood",
            Biome::Mistfen => "Mistfen",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementTier {
    Village,
    Town,
}

impl SettlementTier {
    pub fn label(self) -> &'static str {
        match self {
            SettlementTier::Village => "Village",
            SettlementTier::Town => "Town",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettlementSite {
    pub id: u64,
    pub center: IVec2,
    pub tier: SettlementTier,
}

impl SettlementSite {
    pub fn name(self) -> String {
        if self.center == IVec2::ZERO {
            return "Ember Town".into();
        }
        const FIRST: [&str; 10] = [
            "Ash", "Briar", "Cinder", "Dusk", "Elder", "Fallow", "Gloam", "Hearth", "Moss", "Rill",
        ];
        const SECOND: [&str; 10] = [
            "brook", "cross", "field", "fen", "ford", "gate", "hollow", "mere", "rest", "watch",
        ];
        let first = FIRST[(self.id % FIRST.len() as u64) as usize];
        let second = SECOND[((self.id / 11) % SECOND.len() as u64) as usize];
        format!("{first}{second}")
    }

    pub fn is_origin(self) -> bool {
        self.center == IVec2::ZERO
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LandmarkKind {
    Shrine,
    Well,
    Camp,
    Graveyard,
    StandingStones,
    Cart,
}

impl LandmarkKind {
    pub fn name(self) -> &'static str {
        match self {
            LandmarkKind::Shrine => "Shrine",
            LandmarkKind::Well => "Well",
            LandmarkKind::Camp => "Camp",
            LandmarkKind::Graveyard => "Graveyard",
            LandmarkKind::StandingStones => "Standing Stones",
            LandmarkKind::Cart => "Abandoned Cart",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Landmark {
    pub id: u64,
    pub center: IVec2,
    pub kind: LandmarkKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
            TileKind::Thorns => (
                Color::from_rgba(22, 35, 24, 255),
                Color::from_rgba(164, 194, 92, 255),
            ),
            TileKind::Mire => (
                Color::from_rgba(18, 31, 28, 255),
                Color::from_rgba(114, 174, 146, 255),
            ),
            TileKind::Road => (
                Color::from_rgba(56, 43, 28, 255),
                Color::from_rgba(216, 172, 92, 255),
            ),
            TileKind::Bridge => (
                Color::from_rgba(28, 38, 42, 255),
                Color::from_rgba(214, 170, 102, 255),
            ),
            TileKind::Wall => (
                Color::from_rgba(20, 21, 24, 255),
                Color::from_rgba(140, 146, 154, 255),
            ),
            TileKind::Floor => (
                Color::from_rgba(33, 31, 34, 255),
                Color::from_rgba(128, 130, 138, 255),
            ),
            TileKind::Tree => (
                Color::from_rgba(20, 38, 22, 255),
                Color::from_rgba(92, 160, 84, 255),
            ),
            TileKind::DeadTree => (
                Color::from_rgba(33, 29, 24, 255),
                Color::from_rgba(154, 128, 92, 255),
            ),
            TileKind::Rock => (
                Color::from_rgba(31, 32, 36, 255),
                Color::from_rgba(154, 158, 166, 255),
            ),
            TileKind::MushroomCluster => (
                Color::from_rgba(16, 35, 39, 255),
                Color::from_rgba(124, 236, 232, 255),
            ),
            TileKind::Fence => (
                Color::from_rgba(47, 35, 24, 255),
                Color::from_rgba(174, 126, 74, 255),
            ),
            TileKind::Shrine => (
                Color::from_rgba(42, 36, 27, 255),
                Color::from_rgba(255, 224, 96, 255),
            ),
            TileKind::Well => (
                Color::from_rgba(24, 35, 42, 255),
                Color::from_rgba(112, 180, 255, 255),
            ),
            TileKind::Campfire => (
                Color::from_rgba(44, 28, 20, 255),
                Color::from_rgba(255, 132, 64, 255),
            ),
            TileKind::Grave => (
                Color::from_rgba(34, 34, 38, 255),
                Color::from_rgba(188, 188, 198, 255),
            ),
            TileKind::StandingStone => (
                Color::from_rgba(32, 33, 38, 255),
                Color::from_rgba(180, 184, 196, 255),
            ),
            TileKind::Cart => (
                Color::from_rgba(48, 34, 22, 255),
                Color::from_rgba(182, 134, 84, 255),
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
            TileKind::Thorns => {
                if shimmer % 6 == 0 {
                    "v"
                } else {
                    "+"
                }
            }
            TileKind::Mire => {
                if shimmer % 8 == 0 {
                    "~"
                } else {
                    ","
                }
            }
            TileKind::Road => "=",
            TileKind::Bridge => "=",
            TileKind::Wall => "#",
            TileKind::Floor => ".",
            TileKind::Tree => "T",
            TileKind::DeadTree => "Y",
            TileKind::Rock => "^",
            TileKind::MushroomCluster => "o",
            TileKind::Fence => "|",
            TileKind::Shrine => "+",
            TileKind::Well => "O",
            TileKind::Campfire => "*",
            TileKind::Grave => "+",
            TileKind::StandingStone => "I",
            TileKind::Cart => "%",
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
        if let Some(site) = self.settlement_at_tile(pos) {
            return settlement_tile(pos, site);
        }
        if self.road_at_tile(pos) {
            let terrain = wilderness_tile(pos, self.seed, self.biome_at_tile(pos));
            return Tile {
                kind: if terrain.kind == TileKind::Mire {
                    TileKind::Bridge
                } else {
                    TileKind::Road
                },
                walkable: true,
            };
        }
        if let Some(tile) = self.landmark_tile(pos) {
            return tile;
        }
        if let Some(tile) = self.prop_tile(pos) {
            return tile;
        }
        wilderness_tile(pos, self.seed, self.biome_at_tile(pos))
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

    pub fn region_name(&self, pos: Vec2) -> String {
        if let Some(site) = self.settlement_at_world(pos) {
            return if site.is_origin() {
                site.name()
            } else {
                format!("{} {}", site.name(), site.tier.label())
            };
        }
        self.biome_name(pos).into()
    }

    pub fn biome_level(&self, pos: Vec2) -> i32 {
        self.biome_level_at_tile(Self::world_to_tile(pos))
    }

    pub fn biome_at_world(&self, pos: Vec2) -> Biome {
        self.biome_at_tile(Self::world_to_tile(pos))
    }

    pub fn biome_at_tile(&self, pos: IVec2) -> Biome {
        if self.settlement_at_tile(pos).is_some() {
            Biome::Town
        } else if pos.distance_squared(IVec2::ZERO)
            <= STARTER_TOWN_DRY_BUFFER * STARTER_TOWN_DRY_BUFFER
        {
            Biome::Meadow
        } else {
            province_biome(pos, self.seed)
        }
    }

    pub fn biome_level_at_tile(&self, pos: IVec2) -> i32 {
        if self.settlement_at_tile(pos).is_some() {
            let nearest_center = self
                .settlement_at_tile(pos)
                .map(|site| site.center)
                .unwrap_or(pos);
            if nearest_center == IVec2::ZERO {
                return 0;
            }
        }
        let distance = organic_distance(pos, self.seed);
        1 + ((distance - TOWN_RADIUS as f32 - 1.0).max(0.0) / BIOME_BAND as f32).floor() as i32
    }

    pub fn settlement_at_world(&self, pos: Vec2) -> Option<SettlementSite> {
        self.settlement_at_tile(Self::world_to_tile(pos))
    }

    pub fn settlement_at_tile(&self, pos: IVec2) -> Option<SettlementSite> {
        let origin = origin_settlement();
        if inside_settlement(pos, origin) {
            return Some(origin);
        }
        let cell = settlement_cell(pos);
        for y in cell.y - 1..=cell.y + 1 {
            for x in cell.x - 1..=cell.x + 1 {
                if let Some(site) = generated_settlement(ivec2(x, y), self.seed)
                    && inside_settlement(pos, site)
                {
                    return Some(site);
                }
            }
        }
        None
    }

    pub fn settlements_near_tile(&self, center: IVec2, radius: i32) -> Vec<SettlementSite> {
        let mut sites = Vec::new();
        let origin = origin_settlement();
        if origin.center.distance_squared(center) <= radius * radius {
            sites.push(origin);
        }
        let min_cell = settlement_cell(center - ivec2(radius, radius));
        let max_cell = settlement_cell(center + ivec2(radius, radius));
        for y in min_cell.y - 1..=max_cell.y + 1 {
            for x in min_cell.x - 1..=max_cell.x + 1 {
                if let Some(site) = generated_settlement(ivec2(x, y), self.seed)
                    && site.center.distance_squared(center) <= radius * radius
                {
                    sites.push(site);
                }
            }
        }
        sites.sort_by_key(|site| site.id);
        sites.dedup_by_key(|site| site.id);
        sites
    }

    pub fn is_safe_zone(&self, pos: IVec2) -> bool {
        self.settlement_at_tile(pos).is_some()
    }

    pub fn landmark_at_tile(&self, pos: IVec2) -> Option<Landmark> {
        let cell = feature_cell(pos);
        for y in cell.y - 1..=cell.y + 1 {
            for x in cell.x - 1..=cell.x + 1 {
                if let Some(landmark) = generated_landmark(ivec2(x, y), self.seed)
                    && self.settlement_at_tile(landmark.center).is_none()
                    && pos.distance_squared(landmark.center) <= 9
                {
                    return Some(landmark);
                }
            }
        }
        None
    }

    pub fn landmark_at_world(&self, pos: Vec2) -> Option<Landmark> {
        self.landmark_at_tile(Self::world_to_tile(pos))
    }

    pub fn landmarks_near_tile(&self, center: IVec2, radius: i32) -> Vec<Landmark> {
        let mut landmarks = Vec::new();
        let min_cell = feature_cell(center - ivec2(radius, radius));
        let max_cell = feature_cell(center + ivec2(radius, radius));
        for y in min_cell.y - 1..=max_cell.y + 1 {
            for x in min_cell.x - 1..=max_cell.x + 1 {
                if let Some(landmark) = generated_landmark(ivec2(x, y), self.seed)
                    && self.settlement_at_tile(landmark.center).is_none()
                    && landmark.center.distance_squared(center) <= radius * radius
                {
                    landmarks.push(landmark);
                }
            }
        }
        landmarks.sort_by_key(|landmark| landmark.id);
        landmarks.dedup_by_key(|landmark| landmark.id);
        landmarks
    }

    fn road_at_tile(&self, pos: IVec2) -> bool {
        if self.settlement_at_tile(pos).is_some() {
            return false;
        }
        let search_center = settlement_cell(pos);
        for y in
            search_center.y - ROAD_SEARCH_RADIUS_CELLS..=search_center.y + ROAD_SEARCH_RADIUS_CELLS
        {
            for x in search_center.x - ROAD_SEARCH_RADIUS_CELLS
                ..=search_center.x + ROAD_SEARCH_RADIUS_CELLS
            {
                let Some(site) = generated_settlement(ivec2(x, y), self.seed) else {
                    continue;
                };
                for neighbor in self.road_neighbors(site) {
                    if site.id >= neighbor.id {
                        continue;
                    }
                    if organic_segment_distance(pos, site.center, neighbor.center, self.seed)
                        <= 1.15
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn road_neighbors(&self, site: SettlementSite) -> Vec<SettlementSite> {
        let mut candidates = Vec::new();
        let cell = settlement_cell(site.center);
        for y in cell.y - 2..=cell.y + 2 {
            for x in cell.x - 2..=cell.x + 2 {
                if let Some(other) = generated_settlement(ivec2(x, y), self.seed)
                    && other.id != site.id
                {
                    candidates.push(other);
                }
            }
        }
        let origin = origin_settlement();
        if origin.id != site.id && site.center.distance_squared(origin.center) <= 220 * 220 {
            candidates.push(origin);
        }
        candidates.sort_by(|a, b| {
            site.center
                .distance_squared(a.center)
                .cmp(&site.center.distance_squared(b.center))
        });
        candidates.into_iter().take(2).collect()
    }

    fn landmark_tile(&self, pos: IVec2) -> Option<Tile> {
        let landmark = self.landmark_at_tile(pos)?;
        if self.settlement_at_tile(landmark.center).is_some() {
            return None;
        }
        if wilderness_tile(
            landmark.center,
            self.seed,
            self.biome_at_tile(landmark.center),
        )
        .kind
            == TileKind::Mire
        {
            return None;
        }
        let local = pos - landmark.center;
        let tile = match landmark.kind {
            LandmarkKind::Shrine => {
                if local == IVec2::ZERO {
                    Some(Tile {
                        kind: TileKind::Shrine,
                        walkable: true,
                    })
                } else if local.x.abs() <= 1 && local.y.abs() <= 1 {
                    Some(Tile {
                        kind: TileKind::Floor,
                        walkable: true,
                    })
                } else {
                    None
                }
            }
            LandmarkKind::Well => {
                if local == IVec2::ZERO {
                    Some(Tile {
                        kind: TileKind::Well,
                        walkable: true,
                    })
                } else if local.x.abs() <= 1 && local.y.abs() <= 1 {
                    Some(Tile {
                        kind: TileKind::Road,
                        walkable: true,
                    })
                } else {
                    None
                }
            }
            LandmarkKind::Camp => match (local.x, local.y) {
                (0, 0) => Some(Tile {
                    kind: TileKind::Campfire,
                    walkable: true,
                }),
                (-1..=1, -1..=1) => Some(Tile {
                    kind: TileKind::Road,
                    walkable: true,
                }),
                (-2, -1..=1) | (2, -1..=1) => Some(Tile {
                    kind: TileKind::Fence,
                    walkable: false,
                }),
                _ => None,
            },
            LandmarkKind::Graveyard => match (local.x, local.y) {
                (0, 2) => Some(Tile {
                    kind: TileKind::Road,
                    walkable: true,
                }),
                (-2..=2, -2..=2) if local.x.abs() == 2 || local.y.abs() == 2 => Some(Tile {
                    kind: TileKind::Fence,
                    walkable: false,
                }),
                (-1, -1) | (1, -1) | (-1, 1) | (1, 1) => Some(Tile {
                    kind: TileKind::Grave,
                    walkable: false,
                }),
                _ if local.x.abs() <= 1 && local.y.abs() <= 1 => Some(Tile {
                    kind: TileKind::Floor,
                    walkable: true,
                }),
                _ => None,
            },
            LandmarkKind::StandingStones => match (local.x, local.y) {
                (-2, 0) | (2, 0) | (0, -2) | (0, 2) => Some(Tile {
                    kind: TileKind::StandingStone,
                    walkable: false,
                }),
                _ if local.x.abs() <= 1 && local.y.abs() <= 1 => Some(Tile {
                    kind: TileKind::Floor,
                    walkable: true,
                }),
                _ => None,
            },
            LandmarkKind::Cart => match (local.x, local.y) {
                (0, 0) | (1, 0) => Some(Tile {
                    kind: TileKind::Cart,
                    walkable: false,
                }),
                (-1..=2, -1..=1) => Some(Tile {
                    kind: TileKind::Road,
                    walkable: true,
                }),
                _ => None,
            },
        };
        tile
    }

    fn prop_tile(&self, pos: IVec2) -> Option<Tile> {
        let biome = self.biome_at_tile(pos);
        if wilderness_tile(pos, self.seed, biome).kind == TileKind::Mire {
            return None;
        }
        let roll = hash3(pos.x, pos.y, self.seed ^ 0xBEEF_CAFE) % 100;
        match biome {
            Biome::Town => None,
            Biome::Meadow if roll < 2 => Some(Tile {
                kind: TileKind::Tree,
                walkable: true,
            }),
            Biome::Meadow if roll < 4 => Some(Tile {
                kind: TileKind::Rock,
                walkable: true,
            }),
            Biome::FungalGrove if roll < 5 => Some(Tile {
                kind: TileKind::MushroomCluster,
                walkable: true,
            }),
            Biome::Ashfield if roll < 4 => Some(Tile {
                kind: TileKind::DeadTree,
                walkable: true,
            }),
            Biome::OldRuins if roll < 5 => Some(Tile {
                kind: TileKind::Rock,
                walkable: true,
            }),
            Biome::Thornwood if roll < 7 => Some(Tile {
                kind: TileKind::Tree,
                walkable: true,
            }),
            Biome::Mistfen if roll < 5 => Some(Tile {
                kind: TileKind::DeadTree,
                walkable: true,
            }),
            _ => None,
        }
    }
}

fn origin_settlement() -> SettlementSite {
    SettlementSite {
        id: 0,
        center: IVec2::ZERO,
        tier: SettlementTier::Town,
    }
}

fn settlement_cell(pos: IVec2) -> IVec2 {
    ivec2(
        pos.x.div_euclid(SETTLEMENT_CELL),
        pos.y.div_euclid(SETTLEMENT_CELL),
    )
}

fn generated_settlement(cell: IVec2, seed: u64) -> Option<SettlementSite> {
    if cell == IVec2::ZERO {
        return None;
    }
    let roll = hash3(cell.x, cell.y, seed ^ 0x51E7_7E11) % 100;
    if roll >= 28 {
        return None;
    }
    let jitter_x = hash_unit(cell.x, cell.y, seed ^ 0x51E7_1001) * SETTLEMENT_CELL as f32 * 0.28;
    let jitter_y = hash_unit(cell.x, cell.y, seed ^ 0x51E7_2002) * SETTLEMENT_CELL as f32 * 0.28;
    let center = ivec2(
        ((cell.x as f32 + 0.5) * SETTLEMENT_CELL as f32 + jitter_x).round() as i32,
        ((cell.y as f32 + 0.5) * SETTLEMENT_CELL as f32 + jitter_y).round() as i32,
    );
    if center.distance_squared(IVec2::ZERO) < 72 * 72 {
        return None;
    }
    Some(SettlementSite {
        id: hash3(cell.x, cell.y, seed ^ 0x51E7_3003),
        center,
        tier: if roll < 8 {
            SettlementTier::Town
        } else {
            SettlementTier::Village
        },
    })
}

fn inside_settlement(pos: IVec2, site: SettlementSite) -> bool {
    let local = pos - site.center;
    let radius = match site.tier {
        SettlementTier::Village => 9,
        SettlementTier::Town => 14,
    };
    local.x.abs() <= radius && local.y.abs() <= radius
}

fn settlement_tile(pos: IVec2, site: SettlementSite) -> Tile {
    let local = pos - site.center;
    let half = match site.tier {
        SettlementTier::Village => 9,
        SettlementTier::Town => 14,
    };
    let on_outer_road = local.x == 0 || local.y == 0;
    let plaza = local.x.abs() <= 2 && local.y.abs() <= 2;
    let wall = local.x.abs() == half || local.y.abs() == half;
    let gate = (local.x == 0 && local.y.abs() == half) || (local.y == 0 && local.x.abs() == half);
    if wall && !gate {
        return Tile {
            kind: TileKind::Fence,
            walkable: false,
        };
    }
    if plaza || on_outer_road || gate {
        return Tile {
            kind: TileKind::Road,
            walkable: true,
        };
    }
    if let Some(tile) = building_tile(local, site.tier, site.id) {
        return tile;
    }
    Tile {
        kind: TileKind::Flowers,
        walkable: true,
    }
}

fn building_tile(local: IVec2, tier: SettlementTier, id: u64) -> Option<Tile> {
    let mut buildings = vec![
        RectI::new(-7, -7, 5, 5),
        RectI::new(3, -7, 5, 5),
        RectI::new(-7, 3, 5, 5),
        RectI::new(3, 3, 5, 5),
    ];
    if tier == SettlementTier::Town {
        buildings.extend([
            RectI::new(-12, -4, 4, 7),
            RectI::new(9, -4, 4, 7),
            RectI::new(-3, -12, 7, 4),
            RectI::new(-3, 9, 7, 4),
        ]);
    } else if id % 2 == 0 {
        buildings.pop();
    }
    for (index, rect) in buildings.iter().enumerate() {
        if !rect.contains(local) {
            continue;
        }
        let door = match index % 4 {
            0 => ivec2(rect.x + rect.w - 1, rect.y + rect.h / 2),
            1 => ivec2(rect.x, rect.y + rect.h / 2),
            2 => ivec2(rect.x + rect.w - 1, rect.y + rect.h / 2),
            _ => ivec2(rect.x, rect.y + rect.h / 2),
        };
        if local == door {
            return Some(Tile {
                kind: TileKind::Floor,
                walkable: true,
            });
        }
        let interior = local.x > rect.x
            && local.x < rect.x + rect.w - 1
            && local.y > rect.y
            && local.y < rect.y + rect.h - 1;
        return Some(Tile {
            kind: if interior {
                TileKind::Floor
            } else {
                TileKind::Wall
            },
            walkable: interior,
        });
    }
    None
}

#[derive(Clone, Copy)]
struct RectI {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl RectI {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    fn contains(self, pos: IVec2) -> bool {
        pos.x >= self.x && pos.x < self.x + self.w && pos.y >= self.y && pos.y < self.y + self.h
    }
}

fn organic_segment_distance(pos: IVec2, a: IVec2, b: IVec2, seed: u64) -> f32 {
    let point = vec2(pos.x as f32, pos.y as f32);
    let start = vec2(a.x as f32, a.y as f32);
    let end = vec2(b.x as f32, b.y as f32);
    let segment = end - start;
    let len_sq = segment.length_squared().max(1.0);
    let t = ((point - start).dot(segment) / len_sq).clamp(0.0, 1.0);
    let mut closest = start + segment * t;
    let warp = sample_noise(pos, 18, seed ^ 0xA11C_700D) * 2.4;
    let normal = vec2(-segment.y, segment.x).normalize_or_zero();
    closest += normal * warp;
    point.distance(closest)
}

fn feature_cell(pos: IVec2) -> IVec2 {
    ivec2(
        pos.x.div_euclid(FEATURE_CELL),
        pos.y.div_euclid(FEATURE_CELL),
    )
}

fn generated_landmark(cell: IVec2, seed: u64) -> Option<Landmark> {
    let roll = hash3(cell.x, cell.y, seed ^ 0xFA7E_7001) % 100;
    if roll >= 22 {
        return None;
    }
    let jitter_x = hash_unit(cell.x, cell.y, seed ^ 0xFA7E_1001) * FEATURE_CELL as f32 * 0.3;
    let jitter_y = hash_unit(cell.x, cell.y, seed ^ 0xFA7E_2002) * FEATURE_CELL as f32 * 0.3;
    let center = ivec2(
        ((cell.x as f32 + 0.5) * FEATURE_CELL as f32 + jitter_x).round() as i32,
        ((cell.y as f32 + 0.5) * FEATURE_CELL as f32 + jitter_y).round() as i32,
    );
    Some(Landmark {
        id: hash3(cell.x, cell.y, seed ^ 0xFA7E_3003),
        center,
        kind: match roll % 6 {
            0 => LandmarkKind::Shrine,
            1 => LandmarkKind::Well,
            2 => LandmarkKind::Camp,
            3 => LandmarkKind::Graveyard,
            4 => LandmarkKind::StandingStones,
            _ => LandmarkKind::Cart,
        },
    })
}

fn organic_distance(pos: IVec2, seed: u64) -> f32 {
    let radial = vec2(pos.x as f32, pos.y as f32).length();
    let broad_warp = sample_noise(pos, 60, seed ^ 0xA11C_E001) * 22.0;
    let local_warp = sample_noise(pos, 24, seed ^ 0xF00D_5EED) * 10.0;
    (radial + broad_warp + local_warp).max(0.0)
}

fn province_biome(pos: IVec2, seed: u64) -> Biome {
    let warped = vec2(
        pos.x as f32 + sample_noise(pos, 58, seed ^ 0xCAFE_1001) * 32.0,
        pos.y as f32 + sample_noise(pos, 64, seed ^ 0xCAFE_2002) * 32.0,
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
    let jitter_x = hash_unit(cell.x, cell.y, seed ^ 0xB10B_1001) * PROVINCE_SIZE as f32 * 0.38;
    let jitter_y = hash_unit(cell.x, cell.y, seed ^ 0xB10B_2002) * PROVINCE_SIZE as f32 * 0.38;
    vec2(
        (cell.x as f32 + 0.5) * PROVINCE_SIZE as f32 + jitter_x,
        (cell.y as f32 + 0.5) * PROVINCE_SIZE as f32 + jitter_y,
    )
}

fn province_kind(cell: IVec2, seed: u64) -> Biome {
    match hash3(cell.x, cell.y, seed ^ 0xB10B_3003) % 6 {
        0 => Biome::Meadow,
        1 => Biome::FungalGrove,
        2 => Biome::Ashfield,
        3 => Biome::OldRuins,
        4 => Biome::Thornwood,
        _ => Biome::Mistfen,
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
        Biome::Thornwood => {
            if patch > 0.28 {
                TileKind::Flowers
            } else {
                TileKind::Thorns
            }
        }
        Biome::Mistfen => {
            if sparse_patch < -0.38 {
                TileKind::Grass
            } else {
                TileKind::Mire
            }
        }
    };
    let obstacle = sample_noise(pos, 8, seed ^ 0xB10C_5EED);
    let blocked = matches!(kind, TileKind::Ruins | TileKind::Thorns) && obstacle > 0.74;
    Tile {
        kind: if blocked { TileKind::Wall } else { kind },
        walkable: !blocked && kind != TileKind::Mire,
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
        assert!(world.biome_level(World::tile_center(ivec2(0, -28))) >= 1);
        assert!(world.biome_level(World::tile_center(ivec2(0, 280))) >= 8);
    }

    #[test]
    fn nearby_points_can_repeat_or_split_biomes_independently_of_level() {
        let world = World::new(7);
        let same_level_samples = [
            ivec2(88, 0),
            ivec2(64, 64),
            ivec2(0, 88),
            ivec2(-64, 64),
            ivec2(-88, 0),
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
        let biome = world.biome_at_tile(ivec2(140, 0));
        assert!((1..=12).any(|step| {
            world.biome_at_tile(ivec2(140 + step * PROVINCE_SIZE, step * 17)) == biome
        }));
    }

    #[test]
    fn same_coordinate_always_generates_same_tile() {
        let world = World::new(7);
        let pos = ivec2(13_337, -42_424);
        assert_eq!(world.tile(pos), world.tile(pos));
    }

    #[test]
    fn starter_town_uses_real_floorplans() {
        let world = World::new(7);
        assert_eq!(world.tile(ivec2(0, 0)).kind, TileKind::Road);
        assert_eq!(world.tile(ivec2(-7, -7)).kind, TileKind::Wall);
        assert_eq!(world.tile(ivec2(-3, -5)).kind, TileKind::Floor);
    }

    #[test]
    fn starter_town_has_a_dry_meadow_buffer() {
        let world = World::new(1);
        for pos in [ivec2(0, 16), ivec2(18, 0), ivec2(-12, 20)] {
            assert_eq!(world.biome_at_tile(pos), Biome::Meadow);
            assert_ne!(world.tile(pos).kind, TileKind::Mire);
        }
    }

    #[test]
    fn graveyards_have_a_walkable_gate() {
        let world = World::new(13);
        let graveyard = (-500..=500)
            .step_by(7)
            .flat_map(|y| (-500..=500).step_by(7).map(move |x| ivec2(x, y)))
            .find_map(|tile| {
                let landmark = world.landmark_at_tile(tile)?;
                (landmark.kind == LandmarkKind::Graveyard).then_some(landmark)
            })
            .unwrap();

        assert!(world.tile(graveyard.center + ivec2(0, 2)).walkable);
    }

    #[test]
    fn mire_is_blocking_and_roads_cross_it_as_bridges() {
        let world = World::new(7);
        let mire = (-900..=900)
            .step_by(3)
            .flat_map(|y| (-900..=900).step_by(3).map(move |x| ivec2(x, y)))
            .find(|pos| world.tile(*pos).kind == TileKind::Mire)
            .unwrap();
        assert!(!world.tile(mire).walkable);

        let bridge = (-900..=900)
            .flat_map(|y| (-900..=900).map(move |x| ivec2(x, y)))
            .find(|pos| world.tile(*pos).kind == TileKind::Bridge)
            .unwrap();
        assert!(world.tile(bridge).walkable);
    }

    #[test]
    fn loose_world_props_do_not_create_single_tile_blockers() {
        let world = World::new(7);
        let prop = (-240..=240)
            .flat_map(|y| (-240..=240).map(move |x| ivec2(x, y)))
            .find(|pos| {
                matches!(
                    world.tile(*pos).kind,
                    TileKind::Tree
                        | TileKind::DeadTree
                        | TileKind::Rock
                        | TileKind::MushroomCluster
                )
            })
            .unwrap();
        assert!(world.tile(prop).walkable);
    }

    #[test]
    fn generated_settlements_are_safe_and_deterministic() {
        let world = World::new(7);
        let sites = world.settlements_near_tile(ivec2(240, 0), 260);
        assert!(sites.iter().any(|site| !site.is_origin()));
        let site = sites.into_iter().find(|site| !site.is_origin()).unwrap();
        assert_eq!(world.settlement_at_tile(site.center), Some(site));
        assert!(world.is_safe_zone(site.center));
        assert_eq!(
            world.settlement_at_tile(site.center),
            world.settlement_at_tile(site.center)
        );
    }

    #[test]
    fn new_biomes_exist_in_the_world() {
        let world = World::new(7);
        let mut found_thornwood = false;
        let mut found_mistfen = false;
        for y in (-600..=600).step_by(64) {
            for x in (-600..=600).step_by(64) {
                match world.biome_at_tile(ivec2(x, y)) {
                    Biome::Thornwood => found_thornwood = true,
                    Biome::Mistfen => found_mistfen = true,
                    _ => {}
                }
            }
        }
        assert!(found_thornwood && found_mistfen);
    }
}
