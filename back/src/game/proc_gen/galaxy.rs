// TODO add extragalactic exoplanet or objects that do not belong to a planet
// Does it even make sense? Is it possible?
// Idea : maybe add a type to a galaxy. Or make it possible to contain very few objects.

// Galaxies are spread like filaments

// A) A hypothetical Cosmic Density Function using xxh3 and math
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

// B) Another Cosmic Density Function. Not sure what is the difference with check_cosmic_density.
fn get_cosmic_density(cx: i64, cy: i64, cz: i64) -> f64 {
    // Convert to floats and apply scaling factors to control the size of voids/clusters
    // Smaller scale numbers = larger, grander cosmic structures
    let x = (cx as f64) * 0.05;
    let y = (cy as f64) * 0.05;
    let z = (cz as f64) * 0.05;

    // Combine overlapping frequencies to create irregular, organic shapes (filaments)
    let raw_density = (x.sin() * y.cos())
        + (y.sin() * z.cos())
        + (z.sin() * x.cos())
        + ((x * 2.3).cos() * (z * 2.3).sin() * 0.5); // Higher frequency detail

    // Normalize the result to a clean 0.0 to 1.0 range
    let normalized = (raw_density + 1.5) / 3.0;
    normalized.clamp(0.0, 1.0)
}

TODO add spawn rate

fn with spawn () {
// 1. Determine local cosmic density
let density = get_cosmic_density(target_cx, target_cy, target_cz);

// 2. Get your deterministic seed for this cosmic slot
let cosmic_seed = derive_seed(
    proc_gen::WORLD_SEED, 
    222, /* GALAXY_SPAWN_TAG */ 
    &[target_cx, target_cy, target_cz]
);
let spawn_roll = ((cosmic_seed % 1000).abs() as f64) / 1000.0; // 0.0 to 1.0

// 3. Apply a threshold filter
let galaxy_exists = if density < 0.4 {
    // --- THE VOIDS ---
    // 40% of the entire universe has zero chance of spawning anything.
    false 
} else if density < 0.7 {
    // --- THE FILAMENTS ---
    // In the stringy bridges, there's a low-to-medium chance of a galaxy spawning.
    spawn_roll < 0.08 // 8% spawn chance
} else {
    // --- THE NODES / CLUSTERS ---
    // In the high-density intersection hubs, galaxies clump heavily.
    spawn_roll < 0.45 // 45% spawn chance
};

if !galaxy_exists {
    // Move on to the next cosmic coordinate set
    search_attempt += 1;
    continue;
}

// Success! A galaxy is born here. 
// Now you can derive its unique `target_galaxy_id` and proceed inward...
}
