pub struct Spiral3D {
    start_x: i32,
    start_y: i32,
    start_z: i32,
    x: i32, // Current relative X offset
    y: i32, // Current relative Y offset
    z: i32, // Current relative Z offset
    layer: i32,
}

impl Spiral3D {
    /// Creates a new 3D spiral starting at the given target coordinates.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Spiral3D {
            start_x: x,
            start_y: y,
            start_z: z,
            x: 0,
            y: 0,
            z: 0,
            layer: 0,
        }
    }
}

impl Default for Spiral3D {
    /// Defaults to starting at (0, 0, 0).
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Iterator for Spiral3D {
    type Item = (i32, i32, i32);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Offset the internal relative coordinates by the starting origin
        let ret = (
            self.start_x + self.x,
            self.start_y + self.y,
            self.start_z + self.z,
        );

        // If we are at the origin layer, pop out to the first layer
        if self.layer == 0 {
            self.layer = 1;
            self.x = 1;
            self.y = -1;
            self.z = -1;
            return Some(ret);
        }

        // State Machine: Systematically walk the 6 outer faces of the current cube layer
        if self.x == self.layer && self.y == -self.layer && self.z == -self.layer {
            // Whole 3D layer shell is complete! Jump outward to the next cube layer.
            self.layer += 1;
            self.x = self.layer;
            self.y = -self.layer;
            self.z = -self.layer;
        } else if self.x == self.layer && self.y < self.layer {
            self.y += 1; // Face 1: Move +Y along the front-right edge
        } else if self.y == self.layer && self.x > -self.layer {
            self.x -= 1; // Face 2: Move -X along the front-top edge
        } else if self.x == -self.layer && self.y > -self.layer {
            self.y -= 1; // Face 3: Move -Y along the back-left edge
        } else if self.y == -self.layer && self.x < self.layer - 1 {
            // Note the -1 constraint: we stop just short of completing the bottom ring
            // so we can start stepping upward into the Z dimension.
            self.x += 1;
        } else if self.z < self.layer {
            // Face 5 & 6: We have processed the base ring; now we increment Z
            // and spiral inwards/outwards for the inner columns.
            self.z += 1;
            // Reset XY to the start of the ring for this specific Z height layer
            self.x = self.layer;
            self.y = -self.layer;
        }

        Some(ret)
    }
}

// Usage example
// fn main() {
//     // Start spiral centered at (10, 20, 30)
//     let spiral = Spiral3D::new(10, 20, 30);

//     for (x, y, z) in spiral.take(5) {
//         println!("{} {} {}", x, y, z);
//     }
// }
