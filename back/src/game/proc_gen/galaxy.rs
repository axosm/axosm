use crate::game::proc_gen::seed::{GALAXY_TAG, GALAXY_TYPE_TAG, STAR_SPAWN_TAG, derive_seed};

const U64_TO_UNIT_F64: f64 = 1.0 / (u64::MAX as f64);
const GALAXY_RADIUS: f32 = 1000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GalaxyKind {
    Spiral = 0,
    Lenticular = 1,
    Elliptical = 2,
    Irregular = 3,
}

#[derive(Debug, Clone)]
pub struct Galaxy {
    pub seed: u64,
    pub kind: GalaxyKind,
    pub position: (i32, i32, i32),
}

impl Galaxy {
    pub fn new(world_seed: u64, position: (i32, i32, i32)) -> Self {
        let seed = derive_seed(
            world_seed,
            GALAXY_TAG,
            &[position.0 as i64, position.1 as i64, position.2 as i64],
        );

        let kind = Self::derive_type(seed);

        Self {
            seed,
            kind,
            position,
        }
    }

    fn derive_type(galaxy_seed: u64) -> GalaxyType {
        let type_seed = derive_seed(galaxy_seed, GALAXY_TYPE_TAG, &[]);
        let roll = (type_seed as f64) * U64_TO_UNIT_F64;

        match roll {
            x if x < 0.60 => GalaxyType::Spiral,
            x if x < 0.75 => GalaxyType::Lenticular,
            x if x < 0.95 => GalaxyType::Elliptical,
            _ => GalaxyType::Irregular,
        }
    }

    pub fn should_spawn_star_system(&self, star_system_pos: (i32, i32, i32)) -> bool {
        let density = compute_star_system_density(self.seed, self.kind, star_system_pos);

        if density <= 0.0001 {
            return false;
        }

        let star_system_seed = derive_seed(
            self.seed,
            STAR_SPAWN_TAG,
            &[
                star_system_pos.0 as i64,
                star_system_pos.1 as i64,
                star_system_pos.2 as i64,
            ],
        );

        let spawn_roll = (star_system_seed as f64) * U64_TO_UNIT_F64;
        spawn_roll < density as f64
    }
}

fn compute_star_system_density(
    seed: u64,
    kind: GalaxyType,
    star_system_position: (i32, i32, i32),
) -> f32 {
    let nx = star_system_position.0 as f32 / GALAXY_RADIUS;
    let ny = star_system_position.1 as f32 / GALAXY_RADIUS;
    let nz = star_system_position.2 as f32 / GALAXY_RADIUS;

    let r_sq = nx * nx + ny * ny + nz * nz;

    // Hard cutoff outside galactic radius
    if r_sq > 1.0 {
        return 0.0;
    }

    let r = r_sq.sqrt();

    match kind {
        GalaxyType::Spiral => {
            let theta = ny.atan2(nx);
            let bulge = (-5.0 * r).exp();

            // Linear winding for spiral arms (k = 8.0 pitch)
            let arm_wave = (8.0 * r - theta).cos();
            let disk = (-2.5 * r).exp() * (-8.0 * nz.abs()).exp() * (arm_wave * arm_wave);

            (bulge + disk).clamp(0.0, 1.0)
        }
        GalaxyType::Elliptical => {
            // E3 flattened ellipsoid
            let r_ellipsoid = (nx * nx + (ny / 0.7).powi(2) + (nz / 0.5).powi(2)).sqrt();
            (-3.5 * r_ellipsoid).exp().clamp(0.0, 1.0)
        }
        GalaxyType::Lenticular => {
            // Dense bulge with flat featureless disk
            let bulge = (-4.0 * r).exp();
            let disk = (-2.0 * r).exp() * (-6.0 * nz.abs()).exp();
            (bulge + disk).clamp(0.0, 1.0)
        }
        GalaxyType::Irregular => {
            // Clumpy 3D noise scaled by radial falloff
            let noise = fast_3d_noise(seed, nx * 3.0, ny * 3.0, nz * 3.0);
            (noise * (1.0 - r_sq)).clamp(0.0, 1.0)
        }
    }
}

/// Zero-dependency 3D Value Noise for Irregular Galaxies
fn fast_3d_noise(seed: u64, x: f32, y: f32, z: f32) -> f32 {
    let xi = x.floor() as i64;
    let yi = y.floor() as i64;
    let zi = z.floor() as i64;

    let fx = x - x.floor();
    let fy = y - y.floor();
    let fz = z - z.floor();

    let u = fx * fx * (3.0 - 2.0 * fx);
    let v = fy * fy * (3.0 - 2.0 * fy);
    let w = fz * fz * (3.0 - 2.0 * fz);

    let hash = |dx: i64, dy: i64, dz: i64| -> f32 {
        let s = derive_seed(seed, 0x4E4F_4953_4500, &[xi + dx, yi + dy, zi + dz]);
        (s as f32) * (1.0 / u64::MAX as f32)
    };

    let c000 = hash(0, 0, 0);
    let c100 = hash(1, 0, 0);
    let c010 = hash(0, 1, 0);
    let c110 = hash(1, 1, 0);
    let c001 = hash(0, 0, 1);
    let c101 = hash(1, 0, 1);
    let c011 = hash(0, 1, 1);
    let c111 = hash(1, 1, 1);

    let x00 = c000 + u * (c100 - c000);
    let x10 = c010 + u * (c110 - c010);
    let x01 = c001 + u * (c101 - c001);
    let x11 = c011 + u * (c111 - c011);

    let y0 = x00 + v * (x10 - x00);
    let y1 = x01 + v * (x11 - x01);

    y0 + w * (y1 - y0)
}
