// A hypothetical Cosmic Density Function using xxh3 and math
fn check_cosmic_density(galaxy_seed: u64, x: i64, y: i64, z: i64) -> bool {
    // Convert your seed/coordinates into a float value between 0.0 and 1.0
    // Using simple trig functions as you mentioned:
    let fx = x as f64 * 0.1;
    let fy = y as f64 * 0.1;
    let fz = z as f64 * 0.1;

    let density = (fx.sin() + fy.cos() + fz.sin()).abs() / 3.0;

    // If the density passes a threshold, a galaxy exists here!
    density > 0.65
}

fn find_starting_galaxy(player_id: i64) -> (i64, i64, i64, u64) {
    let mut search_attempt = 0i64;

    loop {
        // 1. Map the search attempt to deterministic, distinct coordinates.
        // For simplicity, we step along the X axis, but a 3D spiral algorithm is ideal.
        let x = search_attempt * 100;
        let y = 0i64;
        let z = 0i64;

        // 2. Derive the unique seed for THIS specific coordinate triplet
        // Tag 100 = Galaxy Seed derivation
        let galaxy_seed = derive_seed(proc_gen::WORLD_SEED, 100, &[x, y, z]);

        // 3. Check if the universe allows a galaxy to exist here
        if check_cosmic_density(galaxy_seed, x, y, z) {
            // Found a valid galaxy! Return coords and its unique seed.
            return (x, y, z, galaxy_seed);
        }

        search_attempt += 1;
    }
}
