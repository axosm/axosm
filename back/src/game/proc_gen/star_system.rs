use crate::game::proc_gen::seed::{PLANET_SPAWN_TAG, STAR_ATTR_TAG, STAR_SYSTEM_TAG, derive_seed};

// Pre-computed constant for maximum precision floating-point conversions
const U64_TO_UNIT_F64: f64 = 1.0 / (u64::MAX as f64);

/// Stellar Spectral Classification following the Morgan–Keenan system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpectralType {
    O, // Extremely rare, massive, bright blue
    B, // Deep blue-white
    A, // White
    F, // Yellow-white
    G, // Yellow (Sun-like)
    K, // Orange
    M, // Red dwarf (Most common)
}

/// Key physical attributes of the primary star in Solar units ($M_\odot, R_\odot, L_\odot$)
#[derive(Debug, Clone)]
pub struct Star {
    pub spectral_type: SpectralType,
    pub mass: f32,         // In Solar Masses (M_sun)
    pub radius: f32,       // In Solar Radii (R_sun)
    pub luminosity: f32,   // Relative to Sun (L_sun)
    pub surface_temp: u32, // In Kelvin
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
    pub body_index: u32,
    pub body_type: BodyType,
    pub semi_major_axis_au: f32, // Distance from star in AU
    pub is_in_habitable_zone: bool,
    pub seed: u64,
}

#[derive(Debug, Clone)]
pub struct StarSystem {
    pub seed: u64,
    pub star_system_pos: (i32, i32, i32),
    pub star: Star,
    pub bodies: Vec<OrbitalBody>,
}

impl StarSystem {
    pub fn new(galaxy_seed: u64, star_system_pos: (i32, i32, i32)) -> Self {
        let system_seed = derive_seed(
            galaxy_seed,
            STAR_SYSTEM_TAG,
            &[
                star_system_pos.0 as i64,
                star_system_pos.1 as i64,
                star_system_pos.2 as i64,
            ],
        );

        let star = Self::generate_star(system_seed);
        let bodies = Self::generate_orbital_bodies(system_seed, &star);

        Self {
            seed: system_seed,
            star_system_pos,
            star,
            bodies,
        }
    }

    /// Derives the primary star's characteristics based on real stellar initial mass functions.
    fn generate_star(system_seed: u64) -> Star {
        let star_seed = derive_seed(system_seed, STAR_ATTR_TAG, &[]);
        let roll = (star_seed as f64) * U64_TO_UNIT_F64;

        // Weighted distribution based on realistic stellar populations
        let (spectral_type, min_m, max_m, min_temp, max_temp) = match roll {
            x if x < 0.0001 => (SpectralType::O, 16.0, 50.0, 30000, 45000),
            x if x < 0.0013 => (SpectralType::B, 2.1, 16.0, 10000, 30000),
            x if x < 0.0073 => (SpectralType::A, 1.4, 2.1, 7500, 10000),
            x if x < 0.0373 => (SpectralType::F, 1.04, 1.4, 6000, 7500),
            x if x < 0.1133 => (SpectralType::G, 0.8, 1.04, 5200, 6000),
            x if x < 0.2343 => (SpectralType::K, 0.45, 0.8, 3700, 5200),
            _ => (SpectralType::M, 0.08, 0.45, 2400, 3700),
        };

        // Secondary roll for continuous interpolation within class ranges
        let param_roll = ((star_seed.rotate_left(13)) as f64) * U64_TO_UNIT_F64;
        let mass = min_m + (max_m - min_m) * (param_roll as f32);
        let surface_temp = min_temp + ((max_temp - min_temp) as f64 * param_roll) as u32;

        // Mass-Luminosity Relationship: L ~ M^3.5 (standard main sequence approximation)
        let luminosity = mass.powf(3.5);

        // Mass-Radius Relationship: R ~ M^0.8 for M < 1, R ~ M^0.57 for M >= 1
        let radius = if mass < 1.0 {
            mass.powf(0.8)
        } else {
            mass.powf(0.57)
        };

        Star {
            spectral_type,
            mass,
            radius,
            luminosity,
            surface_temp,
        }
    }

    /// Spawns planetary bodies using exponential Titius-Bode style orbital distance scaling.
    fn generate_orbital_bodies(system_seed: u64, star: &Star) -> Vec<OrbitalBody> {
        let mut bodies = Vec::new();

        // Calculate Habitable Zone boundaries based on star luminosity: AU = sqrt(L_sun)
        let hz_center = star.luminosity.sqrt();
        let hz_inner = hz_center * 0.75;
        let hz_outer = hz_center * 1.5;

        // Frost line (Gas giant formation threshold)
        let frost_line = 4.85 * hz_center;

        // Determine total potential orbital slots based on system scale
        let body_count_seed = derive_seed(system_seed, PLANET_SPAWN_TAG, &[0]);
        let body_count_roll = (body_count_seed as f64) * U64_TO_UNIT_F64;
        let max_bodies = 3 + (body_count_roll * 10.0) as u32; // 3 to 12 bodies

        let mut current_au = 0.15 * star.mass; // Inner boundary scales with star mass

        for idx in 0..max_bodies {
            let slot_seed = derive_seed(system_seed, PLANET_SPAWN_TAG, &[idx as i64 + 1]);
            let slot_roll = (slot_seed as f64) * U64_TO_UNIT_F64; // Could there be any correlation?

            // Step distance to next orbit (Titius-Bode variation)
            let spacing_factor = 1.3 + (slot_roll as f32 * 0.5); // 1.3x to 1.8x distance growth
            current_au *= spacing_factor;

            // Determine orbital body type based on distance from frost line
            let body_type = if slot_roll < 0.12 && current_au > 1.0 && current_au < frost_line {
                BodyType::AsteroidBelt
            } else if current_au < frost_line {
                BodyType::Terrestrial
            } else if current_au < frost_line * 3.5 {
                if slot_roll < 0.70 {
                    BodyType::GasGiant
                } else {
                    BodyType::IceGiant
                }
            } else {
                BodyType::IceGiant
            };

            let is_in_habitable_zone = current_au >= hz_inner && current_au <= hz_outer;

            bodies.push(OrbitalBody {
                body_index: idx,
                body_type,
                semi_major_axis_au: current_au,
                is_in_habitable_zone,
                seed: slot_seed,
            });
        }

        bodies
    }
}
