use crate::game::proc_gen::galaxy::Galaxy;
use crate::game::proc_gen::planet::Planet;
use crate::game::proc_gen::star_system::{BodyType, OrbitalBody, StarSystem};
use crate::game::proc_gen::tile::{DynamicTileProperties, TileType};
use crate::game::proc_gen::universe::should_spawn_galaxy;
use crate::maths::spiral_3d::Spiral3D;

#[derive(Debug, Clone)]
pub struct StartingLocation {
    pub galaxy: Galaxy,
    pub star_system: StarSystem,
    pub planet: Planet,
    pub tile: DynamicTileProperties,
}

impl StartingLocation {
    /// Determines if a planet is suitable for a player start.
    fn is_viable_planet(body: &OrbitalBody) -> bool {
        // Must be a terrestrial planet in the habitable zone
        body.body_type == BodyType::Terrestrial && body.is_in_habitable_zone
    }

    /// Determines if a specific tile is suitable as an initial spawn point.
    fn is_viable_start_tile(tile: &DynamicTileProperties) -> bool {
        // Player should start on hospitable land (e.g., Plains or Forest)
        matches!(tile.tile_type, TileType::Plains | TileType::Forest)
    }
}

pub fn find_starting_location(world_seed: u64) -> StartingLocation {
    let mut galaxy_spiral = Spiral3D::new(0, 0, 0);

    // 1. Iterate through Galaxy spatial coordinates
    while let Some(galaxy_pos) = galaxy_spiral.next() {
        if !should_spawn_galaxy(world_seed, galaxy_pos) {
            continue;
        }

        let galaxy = Galaxy::new(world_seed, galaxy_pos);
        let mut system_spiral = Spiral3D::new(0, 0, 0);

        // Limit star system search radius to avoid infinite loops inside a sparse galaxy
        let mut searched_systems = 0;
        const MAX_SYSTEM_SEARCHES: u32 = 500;

        // 2. Iterate through Star System coordinates within the current galaxy
        while let Some(star_pos) = system_spiral.next() {
            searched_systems += 1;
            if searched_systems > MAX_SYSTEM_SEARCHES {
                break; // Move on to the next galaxy if this one yields no valid starts
            }

            if !galaxy.should_spawn_star_system(star_pos) {
                continue;
            }

            // Bug fix from original code: pass `star_pos` here, not `galaxy.position`
            let star_system = StarSystem::new(galaxy.seed, galaxy.galaxy_type, star_pos);

            // 3. Find a terrestrial, habitable orbital body
            for body in &star_system.bodies {
                if !StartingLocation::is_viable_planet(body) {
                    continue;
                }

                // Temporary system ID for startup matching
                let star_system_id = 0;
                let planet = Planet::new(
                    star_system_id,
                    star_system.seed,
                    star_system.position,
                    body.index,
                    body.is_in_habitable_zone,
                    body.semi_major_axis_au,
                );

                // 4. Search planet tiles for a valid spawn biome
                for face in 0..6 {
                    for u in 0..planet.subdivision {
                        for v in 0..planet.subdivision {
                            let tile = planet.query_tile(face, u, v);

                            if StartingLocation::is_viable_start_tile(&tile) {
                                // Found a complete match across all hierarchy levels!
                                return StartingLocation {
                                    galaxy,
                                    star_system,
                                    planet,
                                    tile,
                                };
                            }
                        }
                    }
                }
            }
        }
    }

    panic!("Failed to find a valid starting location in the generated universe!");
}
