use crate::game::proc_gen::galaxy::Galaxy;
use crate::game::proc_gen::seed::{GALAXY_TAG, SYSTEM_TAG, WORLD_SEED, derive_seed};
use crate::game::proc_gen::star_system::StarSystem;
use crate::game::proc_gen::universe::should_spawn_galaxy;
use crate::maths::spiral_3d::Spiral3D;

fn find_starting_galaxy_location() -> Galaxy {
    let mut spiral = Spiral3D::new(0, 0, 0);

    loop {
        let galaxy_pos = spiral.next().unwrap();
        if should_spawn_galaxy(WORLD_SEED, galaxy_pos) {
            return Galaxy::new(WORLD_SEED, galaxy_pos);
        }
    }
}

fn find_starting_star_system_location(galaxy: Galaxy) -> (i32, i32, i32) {
    let mut spiral = Spiral3D::new(0, 0, 0);

    loop {
        let star_system_pos = spiral.next().unwrap();
        if galaxy.should_spawn_star_system(star_system_pos) {
            return StarSystem::new(WORLD_SEED, galaxy.kind, galaxy.position);
        }
    }
}

pub fn find_starting_location() -> (i32, i32, i32, i32) {
    let galaxy = find_starting_galaxy_location();
    let star_system = find_starting_star_system_location(galaxy);

    let star_system_seed = derive_seed(
        GALAXY_TAG,
        SYSTEM_TAG,
        &[
            star_system_pos.0 as i64,
            star_system_pos.1 as i64,
            star_system_pos.2 as i64,
        ],
    );

    let star_system_pos = find_starting_star_system_location(galaxy_pos);

    // let mut search_attempt = 0i64;

    // // Configuration for the spiral's density
    // let spatial_step = 10.0; // Distance between points along the path
    // let turns_per_step = 0.5; // How fast it rotates

    // loop {
    //     // // 1. Map the search attempt to deterministic, distinct coordinates.
    //     // // For simplicity, we step along the X axis, but a 3D spiral algorithm is ideal.
    //     // let x = search_attempt * 100;
    //     // let y = 0i64;
    //     // let z = 0i64;

    //     // 1. Map the search attempt to deterministic, distinct 3D coordinates.
    //     let index = search_attempt as f64;

    //     // Calculate a growing radius and an angle based on the attempt number
    //     let radius = spatial_step * index.sqrt(); // .sqrt() keeps the point density even as it expands
    //     let angle = index * turns_per_step;

    //     // Map to 3D coordinates
    //     let x = (radius * angle.cos()).round() as i64;
    //     let y = (radius * angle.sin()).round() as i64;
    //     let z = (index * spatial_step * 0.5).round() as i64; // Slow climb along Z

    //     // 2. Derive the unique seed for THIS specific coordinate triplet
    //     // Tag 100 = Galaxy Seed derivation
    //     let galaxy_seed = derive_seed(WORLD_SEED, 100, &[x, y, z]);

    //     // 3. Check if the universe allows a galaxy to exist here
    //     if check_cosmic_density(galaxy_seed, x, y, z) {
    //         // Found a valid galaxy! Return coords and its unique seed.
    //         return (x, y, z, galaxy_seed);
    //     }

    //     search_attempt += 1;
    // }
}

// pub fn find_starting_galaxy(player_id: i64) -> (i64, i64, i64, u64) {
//     let mut search_attempt = 0i64;

//     // Configuration for the spiral's density
//     let spatial_step = 10.0; // Distance between points along the path
//     let turns_per_step = 0.5; // How fast it rotates

//     loop {
//         // // 1. Map the search attempt to deterministic, distinct coordinates.
//         // // For simplicity, we step along the X axis, but a 3D spiral algorithm is ideal.
//         // let x = search_attempt * 100;
//         // let y = 0i64;
//         // let z = 0i64;

//         // 1. Map the search attempt to deterministic, distinct 3D coordinates.
//         let index = search_attempt as f64;

//         // Calculate a growing radius and an angle based on the attempt number
//         let radius = spatial_step * index.sqrt(); // .sqrt() keeps the point density even as it expands
//         let angle = index * turns_per_step;

//         // Map to 3D coordinates
//         let x = (radius * angle.cos()).round() as i64;
//         let y = (radius * angle.sin()).round() as i64;
//         let z = (index * spatial_step * 0.5).round() as i64; // Slow climb along Z

//         // 2. Derive the unique seed for THIS specific coordinate triplet
//         // Tag 100 = Galaxy Seed derivation
//         let galaxy_seed = derive_seed(WORLD_SEED, 100, &[x, y, z]);

//         // 3. Check if the universe allows a galaxy to exist here
//         if check_cosmic_density(galaxy_seed, x, y, z) {
//             // Found a valid galaxy! Return coords and its unique seed.
//             return (x, y, z, galaxy_seed);
//         }

//         search_attempt += 1;
//     }
// }

// Might need this here (call it via repository). Thats how we get galaxy id.
// INSERT INTO galaxies (seed, x, y, z)
// VALUES (?1, ?2, ?3, ?4)
// ON CONFLICT(x, y, z) DO UPDATE SET x=x -- Prevents crashes if already explored
// RETURNING id;
