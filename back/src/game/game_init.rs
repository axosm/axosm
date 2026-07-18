use crate::game::proc_gen::galaxy::check_cosmic_density;
use crate::game::proc_gen::seed::{WORLD_SEED, derive_seed};

pub fn find_starting_galaxy(player_id: i64) -> (i64, i64, i64, u64) {
    let mut search_attempt = 0i64;

    // Configuration for the spiral's density
    let spatial_step = 10.0; // Distance between points along the path
    let turns_per_step = 0.5; // How fast it rotates

    loop {
        // // 1. Map the search attempt to deterministic, distinct coordinates.
        // // For simplicity, we step along the X axis, but a 3D spiral algorithm is ideal.
        // let x = search_attempt * 100;
        // let y = 0i64;
        // let z = 0i64;

        // 1. Map the search attempt to deterministic, distinct 3D coordinates.
        let index = search_attempt as f64;

        // Calculate a growing radius and an angle based on the attempt number
        let radius = spatial_step * index.sqrt(); // .sqrt() keeps the point density even as it expands
        let angle = index * turns_per_step;

        // Map to 3D coordinates
        let x = (radius * angle.cos()).round() as i64;
        let y = (radius * angle.sin()).round() as i64;
        let z = (index * spatial_step * 0.5).round() as i64; // Slow climb along Z

        // 2. Derive the unique seed for THIS specific coordinate triplet
        // Tag 100 = Galaxy Seed derivation
        let galaxy_seed = derive_seed(WORLD_SEED, 100, &[x, y, z]);

        // 3. Check if the universe allows a galaxy to exist here
        if check_cosmic_density(galaxy_seed, x, y, z) {
            // Found a valid galaxy! Return coords and its unique seed.
            return (x, y, z, galaxy_seed);
        }

        search_attempt += 1;
    }
}

// Might need this here (call it via repository). Thats how we get galaxy id.
// INSERT INTO galaxies (seed, x, y, z)
// VALUES (?1, ?2, ?3, ?4)
// ON CONFLICT(x, y, z) DO UPDATE SET x=x -- Prevents crashes if already explored
// RETURNING id;
