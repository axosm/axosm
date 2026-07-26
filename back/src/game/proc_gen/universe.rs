use crate::game::proc_gen::seed::{GALAXY_SPAWN_TAG, derive_seed};

// Pre-computed constant for maximum precision and zero division overhead
const U64_TO_UNIT_F64: f64 = 1.0 / (u64::MAX as f64);

pub fn should_spawn_galaxy(world_seed: u64, galaxy_pos: (i32, i32, i32)) -> bool {
    let density = compute_cosmic_density(galaxy_pos);

    let galaxy_spawn_seed = derive_seed(
        world_seed,
        GALAXY_SPAWN_TAG,
        &[
            galaxy_pos.0 as i64,
            galaxy_pos.1 as i64,
            galaxy_pos.2 as i64,
        ],
    );

    let roll = (galaxy_spawn_seed as f64) * U64_TO_UNIT_F64;

    let threshold = 0.002 + 0.5 * density.powi(3);

    roll < threshold
}

fn compute_cosmic_density(galaxy_pos: (i32, i32, i32)) -> f64 {
    let x = galaxy_pos.0 as f64 * 0.08;
    let y = galaxy_pos.1 as f64 * 0.08;
    let z = galaxy_pos.2 as f64 * 0.08;

    // Coupled primary waves (cosmic filaments)
    let base_structure = (x.sin() * y.cos()) + (y.sin() * z.cos()) + (z.sin() * x.cos());

    // Higher frequency octave (sub-clusters & fine void detail)
    let detail = (x * 2.3).cos() * (z * 2.3).sin() * 0.5;

    let raw_density = base_structure + detail; // Range: [-3.5 .. 3.5]

    // Correctly normalize [-3.5, 3.5] -> [0.0, 1.0]
    let normalized = (raw_density + 3.5) / 7.0;
    normalized.clamp(0.0, 1.0)
}
