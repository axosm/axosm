https://share.gemini.google/AE5QonfqYUJL

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum GalaxyType {
    Spiral = 0,
    Lenticular = 1,
    Elliptical = 2,
    Irregular = 3,
}

const GALAXY_SPAWN_TAG: u64 = 0x4741_4C5F_5350; // "GAL_SP"
const GALAXY_TYPE_TAG: u64 = 0x4741_4C5F_5459; // "GAL_TY"
const STAR_SPAWN_TAG: u64 = 0x5354_4152_5F53; // "STAR_S"

const GALAXY_RADIUS: f32 = 1000.0; // Scaled visual radius of galaxies

/// 1. Check if a galaxy exists at a given universe (X, Y, Z) block
pub fn should_spawn_galaxy(world_seed: u64, pos: (i64, i64, i64)) -> bool {
    let cosmic_seed = derive_seed(world_seed, GALAXY_SPAWN_TAG, &[pos.0, pos.1, pos.2]);
    let spawn_roll = (cosmic_seed % 1000) as f64 / 1000.0;

    // Example: 5% chance of a galaxy spawning in this block of universe space
    spawn_roll < 0.05
}

/// 2. Get the type of the galaxy from its unique galaxy seed
pub fn get_galaxy_type(galaxy_seed: u64) -> GalaxyType {
    let type_seed = derive_seed(galaxy_seed, GALAXY_TYPE_TAG, &[]);
    let roll = (type_seed % 100) as u8;

    match roll {
        0..=59 => GalaxyType::Spiral,      // 60%
        60..=74 => GalaxyType::Lenticular, // 15%
        75..=94 => GalaxyType::Elliptical, // 20%
        _ => GalaxyType::Irregular,        // 5%
    }
}

/// 3. Evaluate if a star system spawns at local_pos relative to galaxy center
pub fn check_star_system_density(
    galaxy_seed: u64,
    galaxy_type: GalaxyType,
    star_pos: (i64, i64, i64),
    local_pos: (f32, f32, f32),
) -> bool {
    // A. Early Out: Outside max bounding sphere
    let nx = local_pos.0 / GALAXY_RADIUS;
    let ny = local_pos.1 / GALAXY_RADIUS;
    let nz = local_pos.2 / GALAXY_RADIUS;

    let r_sq = nx * nx + ny * ny + nz * nz;
    if r_sq > 1.0 {
        return false; // Intergalactic void
    }

    let r = r_sq.sqrt();

    // B. Mathematical Density Map [0.0 .. 1.0]
    let density = match galaxy_type {
        GalaxyType::Spiral => {
            let theta = ny.atan2(nx);
            let bulge = (-5.0 * r).exp();

            // Linear spiral winding (k=8.0 controls tightness, avoids r.ln())
            let arm_wave = (8.0 * r - theta).cos();
            let disk = (-2.5 * r).exp() * (-8.0 * nz.abs()).exp() * (arm_wave * arm_wave);

            (bulge + disk).min(1.0)
        }
        GalaxyType::Elliptical => {
            // Flattened E3 ellipsoid profile
            let r_ellipsoid = (nx * nx + (ny / 0.7).powi(2) + (nz / 0.5).powi(2)).sqrt();
            (-3.5 * r_ellipsoid).exp().min(1.0)
        }
        GalaxyType::Lenticular => {
            // Flat featureless disk + core bulge
            let bulge = (-4.0 * r).exp();
            let disk = (-2.0 * r).exp() * (-6.0 * nz.abs()).exp();
            (bulge + disk).min(1.0)
        }
        GalaxyType::Irregular => {
            // Lightweight 3D procedural noise mask scaled by radial falloff
            let noise = fast_3d_noise(galaxy_seed, nx * 3.0, ny * 3.0, nz * 3.0);
            (noise * (1.0 - r_sq)).clamp(0.0, 1.0)
        }
    };

    // C. Deterministic System Roll
    let system_seed = derive_seed(
        galaxy_seed,
        STAR_SPAWN_TAG,
        &[star_pos.0, star_pos.1, star_pos.2],
    );

    let roll = (system_seed % 10_000) as f32 / 10_000.0;
    roll < density
}

/// Zero-dependency 3D Value Noise for Irregular Galaxies
fn fast_3d_noise(seed: u64, x: f32, y: f32, z: f32) -> f32 {
    let xi = x.floor() as i64;
    let yi = y.floor() as i64;
    let zi = z.floor() as i64;

    let fx = x - x.floor();
    let fy = y - y.floor();
    let fz = z - z.floor();

    // Smoothstep interpolation curves (3t^2 - 2t^3)
    let u = fx * fx * (3.0 - 2.0 * fx);
    let v = fy * fy * (3.0 - 2.0 * fy);
    let w = fz * fz * (3.0 - 2.0 * fz);

    // Hash lattice corners using derive_seed
    let hash = |dx: i64, dy: i64, dz: i64| -> f32 {
        let s = derive_seed(seed, 0x4E4F_4953_4500, &[xi + dx, yi + dy, zi + dz]);
        (s % 1000) as f32 / 1000.0
    };

    // Trilinear interpolation
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

// enum GalaxyType {
//     Spiral,
//     Elliptical,
//     Lenticular,
//     Irregular,
//     // Ring,
//     // BarredSpiral,
//     // Starburst,
// }

// struct Galaxy {
//     g_type: GalaxyType,
//     seed: u64,
//     // Add structural parameters like scale, tilt, bar_length, etc.
// }

// impl Galaxy {
//     // Should not contain rng
//     fn new(global_pos: (f32, f32, f32)) -> Self {
//         let seed = determine_seed(global_pos);
//         let mut rng = SimpleRng::new(seed);

//         // Roll for type based on cosmic proportions
//         let roll = rng.next_f32(); // 0.0 .. 1.0
//         let g_type = match roll {
//             x if x < 0.60 => GalaxyType::Spiral,
//             x if x < 0.75 => GalaxyType::Lenticular,
//             x if x < 0.95 => GalaxyType::Elliptical,
//             _ => GalaxyType::Irregular,
//         };

//         Galaxy { g_type, seed }
//     }

//     // Check if a local coordinate within the galaxy bounds contains a star, see :
//     // from https://gemini.google.com/share/96fdd16b6659
//     //
//     // Discusison on database type, min/max xyz value and possible underflow and NaN
//     // https://gemini.google.com/share/2864aafc8d30
//     // change f32 to either i32 or i64, see discussion above
//     fn evaluate_star_density(&self, local_pos: (f32, f32, f32)) -> bool {
//         let (x, y, z) = local_pos;
//         let r = (x * x + y * y + z * z).sqrt();
//         let theta = y.atan2(x); // TODO are there faster function than atan2?

//         if r > 1.0 {
//             // TODO 1.0 is probably to small. The value depends on max xyz but its mostly emptyness between galaxies.
//             return false;
//         } // Edge of galaxy

//         let density = match self.g_type {
//             GalaxyType::Spiral => {
//                 // See what is linear winding optimization instead of .ln()
//                 let bulge = (-5.0 * r).exp();
//                 // 2 spiral arms, tightly wound (b=0.5)
//                 let arm_wave = (theta - (r.ln() / 0.5)).cos().powi(2);
//                 let disk = (-2.0 * r).exp() * (-5.0 * z.abs()).exp() * arm_wave;
//                 bulge + disk
//             }
//             GalaxyType::Elliptical => {
//                 // Purely radial, incredibly fast)
//                 //
//                 // E3 elongated galaxy example
//                 let r_ellipsoid = (x * x + (y / 0.7).powi(2) + (z / 0.5).powi(2)).sqrt();
//                 (-4.0 * r_ellipsoid.powf(0.25)).exp()
//             }
//             GalaxyType::Lenticular => {
//                 // Purely radial and flat, incredibly fast)
//                 //
//                 let bulge = (-4.0 * r).exp();
//                 let disk = (-2.0 * r).exp() * (-6.0 * z.abs()).exp();
//                 bulge + disk
//             }
//             GalaxyType::Irregular => {
//                 let noise = complex_noise_3d(x * 4.0, y * 4.0, z * 4.0);
//                 noise * (1.0 - r * r)
//             } // GalaxyType::Ring => {
//               //     // Purely radial, gives a completely distinct layout for minimal cost
//               //     //
//               //     // A small central core + a prominent thin ring further out
//               //     let bulge = (-6.0 * r).exp() * 0.8;

//               //     let ring_radius = 0.65;
//               //     let ring_thickness = 0.08;
//               //     // Gaussian distribution around the ring radius
//               //     let ring = (-(r - ring_radius).powi(2) / (2.0 * ring_thickness.powi(2))).exp();

//               //     // Limit the vertical spread of the ring to keep it flat
//               //     let vertical_falloff = (-8.0 * z.abs()).exp();

//               //     (bulge + ring * vertical_falloff).clamp(0.0, 1.0)
//               // }

//               // GalaxyType::BarredSpiral => {
//               //     // Compute intensive. Not sure to keep

//               //     let bulge = (-5.0 * r).exp();

//               //     // 1. Core Bar Logic
//               //     // Transform coordinates to see if we are inside a central rectangular bar
//               //     let bar_length = 0.4;
//               //     let bar_thickness = 0.08;
//               //     let bar_angle = 0.45; // Fixed rotation angle for this galaxy's bar

//               //     // Rotate local frame to align with the bar axis
//               //     let x_rot = x * bar_angle.cos() + y * bar_angle.sin();
//               //     let y_rot = -x * bar_angle.sin() + y * bar_angle.cos();

//               //     let in_bar =
//               //         x_rot.abs() < bar_length && y_rot.abs() < bar_thickness && z.abs() < 0.05;
//               //     let bar_density = if in_bar {
//               //         // Smooth falloff towards the tips and edges of the bar
//               //         (1.0 - (x_rot / bar_length).powi(2))
//               //             * (1.0 - (y_rot / bar_thickness).powi(2))
//               //             * 0.7
//               //     } else {
//               //         0.0
//               //     };

//               //     // 2. Spiral Arms (Tied to the ends of the bar)
//               //     // Arms start at r = bar_length, winding outward
//               //     let arm_wave = if r > bar_length {
//               //         // Pitch factor 'b' (0.35 = tighter wind)
//               //         // We offset theta based on the bar's angle so the arms physically attach to it
//               //         (theta - bar_angle - ((r - bar_length).ln() / 0.35))
//               //             .cos()
//               //             .powi(2)
//               //     } else {
//               //         0.0
//               //     };

//               //     let disk = (-2.2 * r).exp() * (-6.0 * z.abs()).exp() * arm_wave;

//               //     (bulge + bar_density + disk).clamp(0.0, 1.0)
//               // }

//               // GalaxyType::Starburst => {
//               //     // Compute intensive. Not sure to keep
//               //     //
//               //     // Starburst is less about a unique shape and more about extreme, chaotic density.
//               //     // We'll base it on an irregular/distorted spiral base profile, heavily amplified.
//               //     let base_bulge = (-3.0 * r).exp();

//               //     // Highly volatile, high-frequency starburst knots using 3D noise
//               //     let dynamic_noise = complex_noise_3d(x * 6.0, y * 6.0, z * 6.0);
//               //     let structural_noise = complex_noise_3d(x * 2.0, y * 2.0, z * 2.0);

//               //     // Combine a loose physical profile with chaotic noise clusters
//               //     let base_profile = (base_bulge + structural_noise * 0.4) * (1.0 - r * r);
//               //     let localized_bursts = dynamic_noise.powi(2) * 0.6;

//               //     // Scale up the overall intensity to simulate massive gas compression
//               //     let global_amplification = 1.6;

//               //     ((base_profile + localized_bursts) * global_amplification).clamp(0.0, 1.0)
//               // }
//         };

//         // You can layer a small high-frequency star-noise over this
//         // to prevent stars from rendering perfectly smooth, or just threshold it:
//         density > 0.25
//     }
// }

// // // Simplistic "Clumping" (Galactic Arms)

// // // Calculate distance squared from galactic center
// // // let distance_sq = (target_sx * target_sx) + (target_sy * target_sy) + (target_sz * target_sz);

// // // Systems further than 400 units out become incredibly rare
// // let dynamic_threshold = if distance_sq < 200 * 200 {
// //     15 // 15% chance to spawn near the core
// // } else if distance_sq < 400 * 400 {
// //     5  // 5% chance in the mid-rim
// // } else {
// //     1  // 1% chance in deep space outer rim
// // };

// // if spawn_roll > dynamic_threshold {
// //     search_attempt += 1;
// //     continue; // Empty space
// // }
