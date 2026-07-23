use crate::game::proc_gen::seed::{GALAXY_SPAWN_TAG, GALAXY_TYPE_TAG, WORLD_SEED, derive_seed};

// TODO add extragalactic exoplanet or objects that do not belong to a planet
// Does it even make sense? Is it possible?
// Idea : maybe add a type to a galaxy. Or make it possible to contain very few objects.

enum GalaxyType {
    Spiral,
    Elliptical,
    Lenticular,
    Irregular,
    // Ring,
    // BarredSpiral,
    // Starburst,
}

pub fn get_galaxy_type(galaxy_seed: u64) -> GalaxyType {
    // Derive a unique seed specifically for type, using galaxy_seed as the base
    let type_seed = derive_seed(
        galaxy_seed,
        GALAXY_TYPE_TAG,
        &[], // No coords needed here! galaxy_seed already encodes spatial uniqueness
    );

    let roll = (type_seed % 100) as u8;

    match roll {
        0..=59 => GalaxyType::Spiral,
        60..=74 => GalaxyType::Lenticular,
        75..=94 => GalaxyType::Elliptical,
        _ => GalaxyType::Irregular,
    }
}
