use xxhash_rust::xxh3::xxh3_64_with_seed;

pub const WORLD_SEED: u64 = 42;
pub const GENERATION_VERSION: u64 = 1;

pub const GALAXY_TAG: u64 = 1;
pub const SYSTEM_TAG: u64 = 2;
pub const PLANET_TAG: u64 = 3;
pub const TILE_TAG: u64 = 4;

pub const PLANET_SUBDIVISION_TAG: u64 = 12;

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
