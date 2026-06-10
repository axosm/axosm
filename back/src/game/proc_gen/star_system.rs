enum GalaxyType {
    Spiral,
    Elliptical,
    Lenticular,
    Irregular,
    Ring,
    BarredSpiral,
    Starburst,
}

struct Galaxy {
    g_type: GalaxyType,
    seed: u64,
    // Add structural parameters like scale, tilt, bar_length, etc.
}

impl Galaxy {
    // Option A : too simplistic and should not contain rng
    fn new(global_pos: (f32, f32, f32)) -> Self {
        let seed = determine_seed(global_pos);
        let mut rng = SimpleRng::new(seed);

        // Roll for type based on cosmic proportions
        let roll = rng.next_f32(); // 0.0 .. 1.0
        let g_type = match roll {
            x if x < 0.60 => GalaxyType::Spiral,
            x if x < 0.75 => GalaxyType::Lenticular,
            x if x < 0.95 => GalaxyType::Elliptical,
            _ => GalaxyType::Irregular,
        };

        Galaxy { g_type, seed }
    }

    // Option B : What we should use
    // Check if a local coordinate within the galaxy bounds contains a star, see :
    // from https://gemini.google.com/share/96fdd16b6659
    //
    // Discusison on database type, min/max xyz value and possible underflow and NaN
    // https://gemini.google.com/share/2864aafc8d30
    change f32 to either i32 or i64, see discussion above
    fn evaluate_star_density(&self, local_pos: (f32, f32, f32)) -> bool {
        let (x, y, z) = local_pos;
        let r = (x * x + y * y + z * z).sqrt();
        let theta = y.atan2(x); // TODO are there faster function than atan2?

        if r > 1.0 { // TODO 1.0 is probably to small. The value depends on max xyz but its mostly emptyness between galaxies.
            return false;
        } // Edge of galaxy

        let density = match self.g_type {
            GalaxyType::Spiral => {
                // See what is linear winding optimization instead of .ln()
                let bulge = (-5.0 * r).exp();
                // 2 spiral arms, tightly wound (b=0.5)
                let arm_wave = (theta - (r.ln() / 0.5)).cos().powi(2);
                let disk = (-2.0 * r).exp() * (-5.0 * z.abs()).exp() * arm_wave;
                bulge + disk
            }
            GalaxyType::Elliptical => {
                // Purely radial, incredibly fast)
                //
                // E3 elongated galaxy example
                let r_ellipsoid = (x * x + (y / 0.7).powi(2) + (z / 0.5).powi(2)).sqrt();
                (-4.0 * r_ellipsoid.powf(0.25)).exp()
            }
            GalaxyType::Lenticular => {
                // Purely radial and flat, incredibly fast)
                //
                let bulge = (-4.0 * r).exp();
                let disk = (-2.0 * r).exp() * (-6.0 * z.abs()).exp();
                bulge + disk
            }
            GalaxyType::Irregular => {
                let noise = complex_noise_3d(x * 4.0, y * 4.0, z * 4.0);
                noise * (1.0 - r * r)
            }
            GalaxyType::Ring => {
                // Purely radial, gives a completely distinct layout for minimal cost
                //
                // A small central core + a prominent thin ring further out
                let bulge = (-6.0 * r).exp() * 0.8;

                let ring_radius = 0.65;
                let ring_thickness = 0.08;
                // Gaussian distribution around the ring radius
                let ring = (-(r - ring_radius).powi(2) / (2.0 * ring_thickness.powi(2))).exp();

                // Limit the vertical spread of the ring to keep it flat
                let vertical_falloff = (-8.0 * z.abs()).exp();

                (bulge + ring * vertical_falloff).clamp(0.0, 1.0)
            }

            GalaxyType::BarredSpiral => {
                // Compute intensive. Not sure to keep

                let bulge = (-5.0 * r).exp();

                // 1. Core Bar Logic
                // Transform coordinates to see if we are inside a central rectangular bar
                let bar_length = 0.4;
                let bar_thickness = 0.08;
                let bar_angle = 0.45; // Fixed rotation angle for this galaxy's bar

                // Rotate local frame to align with the bar axis
                let x_rot = x * bar_angle.cos() + y * bar_angle.sin();
                let y_rot = -x * bar_angle.sin() + y * bar_angle.cos();

                let in_bar =
                    x_rot.abs() < bar_length && y_rot.abs() < bar_thickness && z.abs() < 0.05;
                let bar_density = if in_bar {
                    // Smooth falloff towards the tips and edges of the bar
                    (1.0 - (x_rot / bar_length).powi(2))
                        * (1.0 - (y_rot / bar_thickness).powi(2))
                        * 0.7
                } else {
                    0.0
                };

                // 2. Spiral Arms (Tied to the ends of the bar)
                // Arms start at r = bar_length, winding outward
                let arm_wave = if r > bar_length {
                    // Pitch factor 'b' (0.35 = tighter wind)
                    // We offset theta based on the bar's angle so the arms physically attach to it
                    (theta - bar_angle - ((r - bar_length).ln() / 0.35))
                        .cos()
                        .powi(2)
                } else {
                    0.0
                };

                let disk = (-2.2 * r).exp() * (-6.0 * z.abs()).exp() * arm_wave;

                (bulge + bar_density + disk).clamp(0.0, 1.0)
            }

            GalaxyType::Starburst => {
                // Compute intensive. Not sure to keep
                //
                // Starburst is less about a unique shape and more about extreme, chaotic density.
                // We'll base it on an irregular/distorted spiral base profile, heavily amplified.
                let base_bulge = (-3.0 * r).exp();

                // Highly volatile, high-frequency starburst knots using 3D noise
                let dynamic_noise = complex_noise_3d(x * 6.0, y * 6.0, z * 6.0);
                let structural_noise = complex_noise_3d(x * 2.0, y * 2.0, z * 2.0);

                // Combine a loose physical profile with chaotic noise clusters
                let base_profile = (base_bulge + structural_noise * 0.4) * (1.0 - r * r);
                let localized_bursts = dynamic_noise.powi(2) * 0.6;

                // Scale up the overall intensity to simulate massive gas compression
                let global_amplification = 1.6;

                ((base_profile + localized_bursts) * global_amplification).clamp(0.0, 1.0)
            }
        };

        // You can layer a small high-frequency star-noise over this
        // to prevent stars from rendering perfectly smooth, or just threshold it:
        density > 0.25
    }
}

// // Simplistic "Clumping" (Galactic Arms)

// // Calculate distance squared from galactic center
// // let distance_sq = (target_sx * target_sx) + (target_sy * target_sy) + (target_sz * target_sz);

// // Systems further than 400 units out become incredibly rare
// let dynamic_threshold = if distance_sq < 200 * 200 {
//     15 // 15% chance to spawn near the core
// } else if distance_sq < 400 * 400 {
//     5  // 5% chance in the mid-rim
// } else {
//     1  // 1% chance in deep space outer rim
// };

// if spawn_roll > dynamic_threshold {
//     search_attempt += 1;
//     continue; // Empty space
// }
