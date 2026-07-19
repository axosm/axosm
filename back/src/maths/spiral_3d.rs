pub struct Spiral3D {
    x: i32,
    y: i32,
    z: i32,
    layer: i32,
}

impl Spiral3D {
    pub fn new() -> Self {
        // If we do not want to start at (0, 0, 0), we simply add an offset
        Spiral3D {
            x: 0,
            y: 0,
            z: 0,
            layer: 0,
        }
    }
}

impl Iterator for Spiral3D {
    type Item = (i32, i32, i32);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let ret = (self.x, self.y, self.z);

        // If we are at the origin, pop out to the first layer
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

// usage
// fn main() {
//     let spiral = Spiral3D::new();
//     // Print the first 27 coordinates (fills the 0,0,0 core + the first 26-block shell)
//     for (x, y, z) in spiral.take(27) {
//         println!("{} {} {}", x, y, z);
//     }
// }
