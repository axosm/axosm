use xxhash_rust::xxh3::xxh3_64_with_seed;

// # Golden Rules
// ## Rule 1
// Every object derives its seed from its parent.
// ```text
// World → Galaxy → System → Planet → Tile
// ```
// ## Rule 2
// Every independent decision gets its own tag.
// ```text
// STAR_TYPE_TAG
// PLANET_COUNT_TAG
// TILE_HEIGHT_TAG
// ```
// ## Rule 3
// Store only player-created state.
// Everything else is regenerated from:
// ```text
// world_seed + location + tag

pub const WORLD_SEED: u64 = 42;

// If generation changes later:
// GENERATION_VERSION = 1;
// New worlds use new generation.
// Old saves keep version 1.
pub const GENERATION_VERSION: u64 = 1;

pub const GALAXY_TAG: u64 = 1;
pub const SYSTEM_TAG: u64 = 2;
pub const PLANET_TAG: u64 = 3;
pub const TILE_TAG: u64 = 4;

pub const PLANET_SUBDIVISION_TAG: u64 = 12;

pub const GALAXY_SPAWN_TAG: u64 = 222;

// # Adding Features Later
// Do not change hierarchy.
// Add new tags.
// const STAR_TYPE_TAG: u64 = 10;
// const PLANET_COUNT_TAG: u64 = 11;
// const TILE_HEIGHT_TAG: u64 = 100;
// const TILE_BIOME_TAG: u64 = 101;
// const ASTEROID_COUNT_TAG: u64 = 200;
// const STATION_COUNT_TAG: u64 = 201;

pub fn derive_seed(base_seed: u64, tag: u64, components: &[i64]) -> u64 {
    // Start with a mix of our global generation version, base seed, and specific feature tag
    let mut current_hash = xxh3_64_with_seed(&tag.to_le_bytes(), base_seed);
    current_hash = xxh3_64_with_seed(&GENERATION_VERSION.to_le_bytes(), current_hash);

    // Append all location/coordinate constraints sequentially
    for &comp in components {
        current_hash = xxh3_64_with_seed(&comp.to_le_bytes(), current_hash);
    }

    current_hash
}
