use crate::{Point, add, mul};

/// One dual Surface Nets vertex at the existing LOD cell width. Density is
/// signed distance (positive inside); no block-resolution sampling is done.
pub fn lod_vertex(point: Point, width: f32, density: impl Fn(Point) -> Option<f32>) -> Point {
    let half = width * 0.5;
    let corners: [Point; 8] = std::array::from_fn(|c| {
        std::array::from_fn(|a| {
            let radius = if a == 1 { half.min(32.) } else { half };
            point[a] + if c >> a & 1 == 0 { -radius } else { radius }
        })
    });
    let mut values = [0.; 8];
    for (i, p) in corners.into_iter().enumerate() {
        let Some(d) = density(p) else { return point };
        values[i] = d;
    }
    let mut sum = [0.; 3];
    let mut count = 0;
    for c in 0..8 {
        for a in 0..3 {
            let next = c ^ (1 << a);
            if c >= next || (values[c] > 0.) == (values[next] > 0.) {
                continue;
            }
            let t = values[c] / (values[c] - values[next]);
            let p = std::array::from_fn(|i| corners[c][i] + t * (corners[next][i] - corners[c][i]));
            sum = add(sum, p);
            count += 1;
        }
    }
    if count == 0 {
        point
    } else {
        mul(sum, 1. / count as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flat_lod_surface_and_vertical_wall_stay_in_place_at_every_scale() {
        for width in [1., 2., 16., 64., 4096., 4194304.] {
            for axis in 0..3 {
                let p = [0., 70., 0.];
                let v = lod_vertex(p, width, |q| Some(p[axis] - q[axis]));
                for i in 0..3 {
                    assert!((v[i] - p[i]).abs() < 0.001, "{width}: {v:?}");
                }
            }
        }
    }
    #[test]
    fn coarse_step_becomes_a_slope_without_adding_density_samples() {
        let density = |p: Point| Some(if p[0] < 8. { 70. - p[1] } else { 78. - p[1] });
        let low = lod_vertex([8., 70., 8.], 8., density);
        let high = lod_vertex([8., 78., 8.], 8., density);
        assert!(low[1] > 70. && high[1] < 78.);
        assert!(low[0] < 8. && high[0] > 8.);
        // Count independently of width, including world-sized LOD cells.
        for width in [1., 16., 4096.] {
            let n = std::cell::Cell::new(0);
            lod_vertex([8., 70., 8.], width, |p| {
                n.set(n.get() + 1);
                density(p)
            });
            assert_eq!(n.get(), 8);
        }
    }
    #[test]
    fn unknown_or_protected_neighbor_pins_the_vertex() {
        let p = [8., 70., 8.];
        assert_eq!(
            lod_vertex(p, 8., |q| if q[0] < 8. { None } else { Some(74. - q[1]) }),
            p
        );
    }
}
