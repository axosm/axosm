use crate::game::proc_gen::seed::{TILE_TAG, derive_seed};

// Tags for distinct feature derivations
const TILE_NOISE_OFFSET_TAG: u64 = 0x5449_4C45_4E4F; // "TILENO"
const TILE_DEPOSIT_TAG: u64 = 0x4445_504F_5349_5400; // "DEPOSIT"

const U64_TO_UNIT_F64: f64 = 1.0 / (u64::MAX as f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Plains,
    Forest,
    Mountain,
    Desert,
    Snow,
    Lava,
    Water,
    Ocean,
}

impl TileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Plains => "plains",
            Self::Forest => "forest",
            Self::Mountain => "mountain",
            Self::Desert => "desert",
            Self::Snow => "snow",
            Self::Lava => "lava",
            Self::Water => "water",
            Self::Ocean => "ocean",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicTileProperties {
    pub face: u8,
    pub u: u32,
    pub v: u32,
    pub tile_type: TileType,
    pub elevation: f32,     // 0.0 to 1.0
    pub yield_quality: f32, // 0.0 to 1.0 multiplier
    pub rare_deposit: Option<&'static str>,
}

/// Converts Goldberg (face, u, v) to a 3D point on a unit sphere to prevent edge seam artifacts.
fn goldberg_to_unit_sphere(face: u8, u: u32, v: u32, subdivision: u32) -> (f32, f32, f32) {
    let side_len = subdivision as f32;
    let fu = (u as f32) / side_len - 0.5;
    let fv = (v as f32) / side_len - 0.5;

    // Project faces onto cube / icosahedral axes
    let (x, y, z) = match face % 6 {
        0 => (1.0, fu, fv),
        1 => (-1.0, fu, fv),
        2 => (fu, 1.0, fv),
        3 => (fu, -1.0, fv),
        4 => (fu, fv, 1.0),
        _ => (fu, fv, -1.0),
    };

    let len = (x * x + y * y + z * z).sqrt();
    (x / len, y / len, z / len)
}

/// Deterministic calculation of tile state based on planet parameters.
pub fn calculate_tile_properties(
    planet_seed: u64,
    is_in_habitable_zone: bool,
    distance_from_star_au: f32,
    subdivision: u32,
    face: u8,
    u: u32,
    v: u32,
) -> DynamicTileProperties {
    let tile_seed = derive_seed(planet_seed, TILE_TAG, &[face as i64, u as i64, v as i64]);
    let (sx, sy, sz) = goldberg_to_unit_sphere(face, u, v, subdivision);

    // 1. Calculate Elevation via multi-frequency pseudo-noise
    let elevation_raw = (sx * 3.0).sin() * (sy * 3.0).cos() + (sz * 3.0).sin();
    let elevation = ((elevation_raw + 2.0) / 4.0).clamp(0.0, 1.0);

    // 2. Latitude-based temperature calculation (sz represents polar axis offset)
    let latitude = sz.abs(); // 0.0 at equator, 1.0 at poles
    let base_temp = 1.0 - latitude; // 1.0 hot, 0.0 cold

    // 3. Determine Biome (TileType)
    let tile_type = if distance_from_star_au < 0.4 {
        if elevation > 0.75 {
            TileType::Lava
        } else {
            TileType::Desert
        }
    } else if distance_from_star_au > 3.0 || base_temp < 0.2 {
        if elevation < 0.3 {
            TileType::Ocean
        } else {
            TileType::Snow
        }
    } else if is_in_habitable_zone {
        if elevation < 0.35 {
            TileType::Ocean
        } else if elevation < 0.45 {
            TileType::Water
        } else if elevation > 0.80 {
            TileType::Mountain
        } else if base_temp > 0.6 {
            TileType::Plains
        } else {
            TileType::Forest
        }
    } else {
        if elevation > 0.7 {
            TileType::Mountain
        } else {
            TileType::Desert
        }
    };

    // 4. Determine Yield Quality (0.0 .. 1.0)
    let yield_roll = (derive_seed(tile_seed, TILE_NOISE_OFFSET_TAG, &[]) as f64) * U64_TO_UNIT_F64;
    let yield_quality = (0.3 + (yield_roll as f32 * 0.7)).clamp(0.0, 1.0);

    // 5. Determine Rare Deposit Spawn
    let deposit_roll = (derive_seed(tile_seed, TILE_DEPOSIT_TAG, &[]) as f64) * U64_TO_UNIT_F64;
    let rare_deposit = if deposit_roll < 0.05 {
        match (deposit_roll * 100.0) as u32 % 6 {
            0 => Some("iron"),
            1 => Some("titanium"),
            2 => Some("uranium"),
            3 => Some("deuterium"),
            4 => Some("rare_earths"),
            _ => Some("dark_matter"),
        }
    } else {
        None
    };

    DynamicTileProperties {
        face,
        u,
        v,
        tile_type,
        elevation,
        yield_quality,
        rare_deposit,
    }
}

/// Returns coordinates of adjacent tiles on a hexagonal grid plane face.
pub fn get_hex_neighbors(face: u8, u: u32, v: u32, max_subdivision: u32) -> Vec<(u8, u32, u32)> {
    let mut neighbors = Vec::with_capacity(6);
    let du = u as i32;
    let dv = v as i32;

    // Hex axial directions
    let offsets = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

    for (ox, oy) in offsets {
        let nu = du + ox;
        let nv = dv + oy;

        // In-bounds on current face grid
        if nu >= 0 && nu < max_subdivision as i32 && nv >= 0 && nv < max_subdivision as i32 {
            neighbors.push((face, nu as u32, nv as u32));
        } else {
            // Edge-wrapping transitions across polyhedral faces (simplification boundary logic)
            let next_face = (face + 1) % 6;
            let wrapped_u = (nu.rem_euclid(max_subdivision as i32)) as u32;
            let wrapped_v = (nv.rem_euclid(max_subdivision as i32)) as u32;
            neighbors.push((next_face, wrapped_u, wrapped_v));
        }
    }

    neighbors
}

// use serde::{Deserialize, Serialize};

// #[derive(Serialize, Debug, Clone)]
// pub struct TileProperties {
//     // Coordinate identity
//     pub face: u8,
//     pub u: u32,
//     pub v: u32,

//     // Generated physical attributes (Never saved to SQLite)
//     pub height: u32,
//     pub biome: String,
//     // pub fertility: f64,
//     // pub max_mineral_capacity: u32,
// }

// pub fn get_tile_properties(planet_seed: u64, face: u8, u: u32, v: u32) -> TileProperties {
//     // Step 1: Establish the individual tile's unique deterministic seed
//     let tile_seed = derive_seed(planet_seed, TILE_TAG, &[face as i64, u as i64, v as i64]);

//     // Step 2: Establish feature tags isolated from each other
//     const TILE_HEIGHT_TAG: u64 = 100;
//     const TILE_BIOME_TAG: u64 = 101;

//     let height_seed = derive_seed(tile_seed, TILE_HEIGHT_TAG, &[]);
//     let biome_seed = derive_seed(tile_seed, TILE_BIOME_TAG, &[]);

//     // Step 3: Run your game's deterministic rules matching the design document
//     let height = (height_seed % 100) as u32; // e.g. Height map value 0-99
//     let biome = match biome_seed % 3 {
//         0 => "Ocean",
//         1 => "Desert",
//         _ => "Continental",
//     };

//     TileProperties {
//         face,
//         u,
//         v,
//         height,
//         biome: biome.to_string(),
//     }
// }
