use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct TileProperties {
    // Coordinate identity
    pub face: u8,
    pub u: u32,
    pub v: u32,

    // Generated physical attributes (Never saved to SQLite)
    pub height: u32,
    pub biome: String,
    // pub fertility: f64,
    // pub max_mineral_capacity: u32,
}

pub fn get_tile_properties(planet_seed: u64, face: u8, u: u32, v: u32) -> TileProperties {
    // Step 1: Establish the individual tile's unique deterministic seed
    let tile_seed = derive_seed(planet_seed, TILE_TAG, &[face as i64, u as i64, v as i64]);

    // Step 2: Establish feature tags isolated from each other
    const TILE_HEIGHT_TAG: u64 = 100;
    const TILE_BIOME_TAG: u64 = 101;

    let height_seed = derive_seed(tile_seed, TILE_HEIGHT_TAG, &[]);
    let biome_seed = derive_seed(tile_seed, TILE_BIOME_TAG, &[]);

    // Step 3: Run your game's deterministic rules matching the design document
    let height = (height_seed % 100) as u32; // e.g. Height map value 0-99
    let biome = match biome_seed % 3 {
        0 => "Ocean",
        1 => "Desert",
        _ => "Continental",
    };

    TileProperties {
        face,
        u,
        v,
        height,
        biome: biome.to_string(),
    }
}
