use crate::Point;

/// Fit a baked face's texture to a smooth face by position, independent of
/// either mesher's starting vertex, winding or repeated triangle vertex.
pub fn surface_uv(
    vertices: [Point; 4],
    axis: usize,
    model: [Point; 4],
    model_axis: usize,
    uv: [[f32; 2]; 4],
) -> [[f32; 2]; 4] {
    fn project(points: [Point; 4], axis: usize) -> [[f32; 2]; 4] {
        let axes = match axis {
            0 => [2, 1],
            1 => [0, 2],
            _ => [0, 1],
        };
        let min = axes.map(|a| points.iter().map(|p| p[a]).fold(f32::INFINITY, f32::min));
        let max = axes.map(|a| {
            points
                .iter()
                .map(|p| p[a])
                .fold(f32::NEG_INFINITY, f32::max)
        });
        points.map(|p| {
            std::array::from_fn(|i| {
                let extent = max[i] - min[i];
                if extent > 1e-6 {
                    (p[axes[i]] - min[i]) / extent
                } else {
                    0.5
                }
            })
        })
    }
    let source = project(model, model_axis);
    let a: [f32; 2] = std::array::from_fn(|i| source[1][i] - source[0][i]);
    let b: [f32; 2] = std::array::from_fn(|i| source[3][i] - source[0][i]);
    let det = a[0] * b[1] - a[1] * b[0];
    if det.abs() < 1e-6 {
        return uv;
    }
    project(vertices, axis).map(|p| {
        let d: [f32; 2] = std::array::from_fn(|i| p[i] - source[0][i]);
        let s = (d[0] * b[1] - d[1] * b[0]) / det;
        let t = (a[0] * d[1] - a[1] * d[0]) / det;
        std::array::from_fn(|i| {
            (1. - s) * (1. - t) * uv[0][i]
                + s * (1. - t) * uv[1][i]
                + s * t * uv[2][i]
                + (1. - s) * t * uv[3][i]
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const UV: [[f32; 2]; 4] = [[0., 0.], [0., 1.], [1., 1.], [1., 0.]];
    // Horizon's baked cube corners: down, up, north, south, west, east.
    const FACES: [[Point; 4]; 6] = [
        [[0., 0., 1.], [0., 0., 0.], [1., 0., 0.], [1., 0., 1.]],
        [[0., 1., 0.], [0., 1., 1.], [1., 1., 1.], [1., 1., 0.]],
        [[1., 1., 0.], [1., 0., 0.], [0., 0., 0.], [0., 1., 0.]],
        [[0., 1., 1.], [0., 0., 1.], [1., 0., 1.], [1., 1., 1.]],
        [[0., 1., 0.], [0., 0., 0.], [0., 0., 1.], [0., 1., 1.]],
        [[1., 1., 1.], [1., 0., 1.], [1., 0., 0.], [1., 1., 0.]],
    ];
    #[test]
    fn every_cube_direction_preserves_model_uv_for_any_vertex_order() {
        for (model, axis) in FACES.into_iter().zip([1, 1, 2, 2, 0, 0]) {
            for order in [[0, 1, 2, 3], [1, 2, 3, 0], [3, 2, 1, 0], [2, 0, 3, 1]] {
                assert_eq!(
                    surface_uv(order.map(|i| model[i]), axis, model, axis, UV),
                    order.map(|i| UV[i])
                );
            }
        }
    }
    #[test]
    fn side_texture_stays_upright_on_slopes_and_respects_baked_rotations() {
        let model = FACES[3];
        let slope = [[1., 0., 2.], [0., 0., 2.], [0., 1., 1.], [1., 1., 1.]];
        assert_eq!(
            surface_uv(slope, 2, model, 2, UV),
            [UV[2], UV[1], UV[0], UV[3]]
        );
        let rotated = [UV[1], UV[2], UV[3], UV[0]];
        assert_eq!(surface_uv(model, 2, model, 2, rotated), rotated);
    }
    #[test]
    fn triangular_faces_share_uv_at_the_repeated_vertex() {
        let model = FACES[1];
        let triangle = [model[2], model[0], model[1], model[1]];
        assert_eq!(
            surface_uv(triangle, 1, model, 1, UV),
            [UV[2], UV[0], UV[1], UV[1]]
        );
    }
}
