use crate::game::proc_gen::{
    galaxy::GalaxyType,
    seed::{
        PLANET_SPAWN_TAG, STAR_SYSTEM_ATTR_TAG, STAR_SYSTEM_BODY_TYPE_TAG,
        STAR_SYSTEM_ORBIT_SPACING_TAG, STAR_SYSTEM_SMBH_SPAWN_TAG, STAR_SYSTEM_STAR_MASS_TAG,
        STAR_SYSTEM_TAG, derive_seed,
    },
};

const U64_TO_UNIT_F64: f64 = 1.0 / (u64::MAX as f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StarType {
    Spectral(SpectralType),
    StellarBlackHole,
    SupermassiveBlackHole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectralType {
    O,
    B,
    A,
    F,
    G,
    K,
    M,
}

#[derive(Debug, Clone)]
pub struct Star {
    pub star_type: StarType,
    pub mass: f32,         // Solar Masses (M_sun)
    pub radius: f32,       // Solar Radii (R_sun)
    pub luminosity: f32,   // Relative to Sun (L_sun)
    pub surface_temp: u32, // Kelvin
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BodyType {
    Terrestrial,
    GasGiant,
    IceGiant,
    AsteroidBelt,
}

#[derive(Debug, Clone)]
pub struct OrbitalBody {
    pub index: u32,
    pub body_type: BodyType,
    pub semi_major_axis_au: f32,
    pub is_in_habitable_zone: bool,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub struct StarSystem {
    pub seed: u64,
    pub position: (i32, i32, i32),
    pub star: Star,
    pub bodies: Vec<OrbitalBody>,
}

impl StarSystem {
    pub fn new(galaxy_seed: u64, galaxy_type: GalaxyType, position: (i32, i32, i32)) -> Self {
        let system_seed = derive_seed(
            galaxy_seed,
            STAR_SYSTEM_TAG,
            &[position.0 as i64, position.1 as i64, position.2 as i64],
        );

        let star = Self::generate_star(system_seed, galaxy_type, position);
        let bodies = Self::generate_orbital_bodies(system_seed, &star);

        Self {
            seed: system_seed,
            position,
            star,
            bodies,
        }
    }

    fn generate_star(system_seed: u64, galaxy_type: GalaxyType, position: (i32, i32, i32)) -> Star {
        let is_center = position == (0, 0, 0);

        // 1. Central Supermassive Black Hole Logic
        if is_center {
            let has_smbh = match galaxy_type {
                GalaxyType::Spiral | GalaxyType::Lenticular => true,
                GalaxyType::Elliptical => true,
                // Irregular galaxies only have a 20% chance of a central SMBH
                GalaxyType::Irregular => {
                    let center_roll = (derive_seed(system_seed, STAR_SYSTEM_SMBH_SPAWN_TAG, &[])
                        as f64)
                        * U64_TO_UNIT_F64;
                    center_roll < 0.20
                }
            };

            if has_smbh {
                let smbh_mass = match galaxy_type {
                    GalaxyType::Elliptical => 1_000_000_000.0, // Massive SMBHs
                    _ => 4_000_000.0, // Milky Way scale (~4 million solar masses)
                };

                return Star {
                    star_type: StarType::SupermassiveBlackHole,
                    mass: smbh_mass,
                    radius: 0.05, // Event horizon footprint scale
                    luminosity: 0.0,
                    surface_temp: 0,
                };
            }

        // 2. Galaxy-Specific Star & Stellar Black Hole Distributions
        let star_seed = derive_seed(system_seed, STAR_SYSTEM_ATTR_TAG, &[]);
        let roll = (star_seed as f64) * U64_TO_UNIT_F64;

        let derived_type = match galaxy_type {
            // Elliptical & Lenticular: "Red & Dead" - No young massive O/B stars
            GalaxyType::Elliptical | GalaxyType::Lenticular => {
                if roll < 0.0005 {
                    StarType::StellarBlackHole
                } else if roll < 0.0100 {
                    StarType::Spectral(SpectralType::F)
                } else if roll < 0.0800 {
                    StarType::Spectral(SpectralType::G)
                } else if roll < 0.2500 {
                    StarType::Spectral(SpectralType::K)
                } else {
                    StarType::Spectral(SpectralType::M)
                }
            }

            // Irregular: High Starburst - Increased ratio of massive O, B, A stars
            GalaxyType::Irregular => {
                if roll < 0.0020 {
                    StarType::StellarBlackHole
                } else if roll < 0.0050 {
                    StarType::Spectral(SpectralType::O)
                } else if roll < 0.0250 {
                    StarType::Spectral(SpectralType::B)
                } else if roll < 0.0800 {
                    StarType::Spectral(SpectralType::A)
                } else if roll < 0.2000 {
                    StarType::Spectral(SpectralType::F)
                } else if roll < 0.4000 {
                    StarType::Spectral(SpectralType::G)
                } else if roll < 0.6500 {
                    StarType::Spectral(SpectralType::K)
                } else {
                    StarType::Spectral(SpectralType::M)
                }
            }

            // Spiral: Standard balanced population
            GalaxyType::Spiral => {
                if roll < 0.0010 {
                    StarType::StellarBlackHole
                } else if roll < 0.0011 {
                    StarType::Spectral(SpectralType::O)
                } else if roll < 0.0023 {
                    StarType::Spectral(SpectralType::B)
                } else if roll < 0.0083 {
                    StarType::Spectral(SpectralType::A)
                } else if roll < 0.0383 {
                    StarType::Spectral(SpectralType::F)
                } else if roll < 0.1143 {
                    StarType::Spectral(SpectralType::G)
                } else if roll < 0.2353 {
                    StarType::Spectral(SpectralType::K)
                } else {
                    StarType::Spectral(SpectralType::M)
                }
            }
        };

        // 3. Attribute Resolution
        match derived_type {
            StarType::StellarBlackHole => {
                let mass_seed = derive_seed(star_seed, STAR_SYSTEM_STAR_MASS_TAG, &[]);
                let mass_roll = (mass_seed as f64) * U64_TO_UNIT_F64;
                let mass = (10.0 + mass_roll * 40.0) as f32; // 10 to 50 Solar Masses

                Star {
                    star_type: StarType::StellarBlackHole,
                    mass,
                    radius: 0.0001,
                    luminosity: 0.0,
                    surface_temp: 0,
                }
            }

            StarType::Spectral(spectral_type) => {
                let (min_m, max_m, min_temp, max_temp) = match spectral_type {
                    SpectralType::O => (16.0, 50.0, 30000, 45000),
                    SpectralType::B => (2.1, 16.0, 10000, 30000),
                    SpectralType::A => (1.4, 2.1, 7500, 10000),
                    SpectralType::F => (1.04, 1.4, 6000, 7500),
                    SpectralType::G => (0.8, 1.04, 5200, 6000),
                    SpectralType::K => (0.45, 0.8, 3700, 5200),
                    SpectralType::M => (0.08, 0.45, 2400, 3700),
                };

                let mass_seed = derive_seed(star_seed, STAR_SYSTEM_STAR_MASS_TAG, &[]);
                let mass_roll = (mass_seed as f64) * U64_TO_UNIT_F64;
                let mass = min_m + (max_m - min_m) * (mass_roll as f32);

                let luminosity = mass.powf(3.5);
                let radius = if mass < 1.0 {
                    mass.powf(0.8)
                } else {
                    mass.powf(0.57)
                };
                let mass_factor = (mass - min_m) / (max_m - min_m);
                let surface_temp =
                    (min_temp as f32 + (max_temp - min_temp) as f32 * mass_factor) as u32;

                Star {
                    star_type: StarType::Spectral(spectral_type),
                    mass,
                    radius,
                    luminosity,
                    surface_temp,
                }
            }

            _ => unreachable!(),
        }
    }

    fn generate_orbital_bodies(system_seed: u64, star: &Star) -> Vec<OrbitalBody> {
        let mut bodies = Vec::new();

        // Non-luminous systems (Black Holes) produce no habitable zones
        let hz_center = star.luminosity.sqrt();
        let hz_inner = hz_center * 0.75;
        let hz_outer = hz_center * 1.5;

        // Frost line distance (fallback to mass factor for zero-luminosity stars)
        let frost_line = if star.luminosity > 0.0 {
            4.85 * hz_center
        } else {
            2.0 * star.mass.sqrt()
        };

        let body_count_seed = derive_seed(system_seed, PLANET_SPAWN_TAG, &[0]);
        let body_count_roll = (body_count_seed as f64) * U64_TO_UNIT_F64;
        let max_bodies = 3 + (body_count_roll * 10.0) as u32;

        let mut current_au = 0.15 * star.mass.min(10.0); // Cap inner distance multiplier for extreme SMBH mass

        for idx in 0..max_bodies {
            let slot_seed = derive_seed(system_seed, PLANET_SPAWN_TAG, &[idx as i64 + 1]);

            let spacing_seed = derive_seed(slot_seed, STAR_SYSTEM_ORBIT_SPACING_TAG, &[]);
            let spacing_roll = (spacing_seed as f64) * U64_TO_UNIT_F64;

            let body_type_seed = derive_seed(slot_seed, STAR_SYSTEM_BODY_TYPE_TAG, &[]);
            let type_roll = (body_type_seed as f64) * U64_TO_UNIT_F64;

            let spacing_factor = 1.3 + (spacing_roll as f32 * 0.5);
            current_au *= spacing_factor;

            let body_type = if type_roll < 0.12 && current_au > 1.0 && current_au < frost_line {
                BodyType::AsteroidBelt
            } else if current_au < frost_line {
                BodyType::Terrestrial
            } else if current_au < frost_line * 3.5 {
                if type_roll < 0.70 {
                    BodyType::GasGiant
                } else {
                    BodyType::IceGiant
                }
            } else {
                BodyType::IceGiant
            };

            let is_in_habitable_zone =
                star.luminosity > 0.0 && current_au >= hz_inner && current_au <= hz_outer;

            bodies.push(OrbitalBody {
                index: idx,
                body_type,
                semi_major_axis_au: current_au,
                is_in_habitable_zone,
                seed: slot_seed,
            });
        }

        bodies
    }
}
