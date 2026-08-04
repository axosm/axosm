use crate::game::proc_gen::seed::{PLANET_TAG, derive_seed};
use crate::game::proc_gen::tile::{
    DynamicTileProperties, calculate_tile_properties, get_hex_neighbors,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanetType {
    Terrestrial,
    GasGiant,
    IceGiant,
    AsteroidField,
}

#[derive(Debug, Clone)]
pub struct Planet {
    pub id: Option<i64>,
    pub star_system_id: i64,
    pub seed: u64,
    pub planet_pos: (i32, i32, i32),
    pub class: PlanetType,
    pub semi_major_axis_au: f32,
    pub is_in_habitable_zone: bool,
    pub subdivision: u32,
}

impl Planet {
    pub fn new(
        star_system_id: i64,
        system_seed: u64,
        planet_pos: (i32, i32, i32),
        index: u32,
        is_in_habitable_zone: bool,
        semi_major_axis_au: f32,
    ) -> Self {
        let seed = derive_seed(
            system_seed,
            PLANET_TAG,
            &[
                planet_pos.0 as i64,
                planet_pos.1 as i64,
                planet_pos.2 as i64,
                index as i64,
            ],
        );

        // Resolution default for Goldberg Sphere: 32x32 tiles per face
        let subdivision = 32;

        let class = if semi_major_axis_au > 5.0 {
            PlanetClass::GasGiant
        } else if semi_major_axis_au > 3.0 {
            PlanetClass::IceGiant
        } else {
            PlanetClass::Terrestrial
        };

        Self {
            id: None,
            star_system_id,
            seed,
            planet_pos,
            class,
            semi_major_axis_au,
            is_in_habitable_zone,
            subdivision,
        }
    }

    /// Query exact physical state for a single tile on demand.
    pub fn query_tile(&self, face: u8, u: u32, v: u32) -> DynamicTileProperties {
        calculate_tile_properties(
            self.seed,
            self.is_in_habitable_zone,
            self.semi_major_axis_au,
            self.subdivision,
            face,
            u,
            v,
        )
    }

    /// Retrieve neighbor tile locations along with their calculated procedural states.
    pub fn query_tile_with_neighbors(
        &self,
        face: u8,
        u: u32,
        v: u32,
    ) -> (DynamicTileProperties, Vec<DynamicTileProperties>) {
        let center = self.query_tile(face, u, v);
        let neighbor_coords = get_hex_neighbors(face, u, v, self.subdivision);

        let neighbors = neighbor_coords
            .into_iter()
            .map(|(nf, nu, nv)| self.query_tile(nf, nu, nv))
            .collect();

        (center, neighbors)
    }
}
