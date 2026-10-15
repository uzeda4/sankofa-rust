//! Perspective projection of a rectangle rotated around its center, for the cover tilt.

/// Corners (top-left, top-right, bottom-right, bottom-left) of the rect `min`..`max` after
/// rotating by `ax` around X and `ay` around Y (radians), seen from `distance` px away.
pub fn project(min: [f32; 2], max: [f32; 2], ax: f32, ay: f32, distance: f32) -> [[f32; 2]; 4] {
    let c = [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0];
    let (sx, cx) = ax.sin_cos();
    let (sy, cy) = ay.sin_cos();
    [[min[0], min[1]], [max[0], min[1]], [max[0], max[1]], [min[0], max[1]]].map(|p| {
        let (x, y) = (p[0] - c[0], p[1] - c[1]);
        // rotate around Y, then X
        let (x1, z1) = (x * cy, -x * sy);
        let (y2, z2) = (y * cx - z1 * sx, y * sx + z1 * cx);
        let s = distance / (distance - z2);
        [c[0] + x1 * s, c[1] + y2 * s]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_angle_keeps_the_rect() {
        let p = project([10.0, 20.0], [110.0, 170.0], 0.0, 0.0, 800.0);
        assert_eq!(p[0], [10.0, 20.0]);
        assert_eq!(p[2], [110.0, 170.0]);
    }

    #[test]
    fn rotating_around_y_narrows_the_rect() {
        let p = project([0.0, 0.0], [100.0, 100.0], 0.0, 0.8, 800.0);
        assert!((p[1][0] - p[0][0]) < 100.0);
    }
}
