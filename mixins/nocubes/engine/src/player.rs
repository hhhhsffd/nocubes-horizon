//! Bounded rounded cuboids for skinned player parts. UVs remain in each
//! original face's rectangle; geometry never samples or substitutes a skin.
use super::{Point, add, mul, normal, sub};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkinQuad {
    pub normal: Point,
    pub vertices: [[f32; 5]; 4],
}

/// Nine quads per original face, retaining its winding and bilinear UV map.
/// Unsupported/non-cuboid custom parts are deliberately left to their owner.
pub fn rounded_skin_box(faces: &[SkinQuad]) -> Option<Vec<(usize, SkinQuad)>> {
    if faces.len() != 6 {
        return None;
    }
    let mut lo = [f32::INFINITY; 3];
    let mut hi = [f32::NEG_INFINITY; 3];
    for face in faces {
        for vertex in face.vertices {
            if vertex.iter().any(|v| !v.is_finite()) {
                return None;
            }
            for axis in 0..3 {
                lo[axis] = lo[axis].min(vertex[axis]);
                hi[axis] = hi[axis].max(vertex[axis]);
            }
        }
    }
    let size = sub(hi, lo);
    if size.iter().any(|v| *v <= 1e-4) {
        return None;
    }
    let radius = size.into_iter().fold(f32::INFINITY, f32::min) * 0.3;
    let mut seen = 0u8;
    let mut out = Vec::with_capacity(54);
    for (source, face) in faces.iter().enumerate() {
        let axis = (0..3).find(|&a| face.normal[a].abs() == 1.)?;
        if (0..3).any(|a| a != axis && face.normal[a] != 0.) {
            return None;
        }
        let bit = 1 << (axis + usize::from(face.normal[axis] < 0.) * 3);
        if seen & bit != 0 {
            return None;
        }
        seen |= bit;
        let plane = if face.normal[axis] > 0. {
            hi[axis]
        } else {
            lo[axis]
        };
        let [a, b, c, d] = face.vertices;
        let u = sub(pos(b), pos(a));
        let v = sub(pos(d), pos(a));
        let width = length(u);
        let height = length(v);
        // Each source face must be one complete, axis-aligned box face.
        if face.vertices.iter().any(|p| {
            (p[axis] - plane).abs() > 1e-4
                || (0..3).any(|i| (p[i] - lo[i]).abs() > 1e-4 && (p[i] - hi[i]).abs() > 1e-4)
        }) || width <= 1e-4
            || height <= 1e-4
            || (0..3).any(|i| (a[i] + c[i] - b[i] - d[i]).abs() > 1e-4)
            || dot(u, v).abs() > 1e-4
            || u.iter().filter(|x| x.abs() > 1e-4).count() != 1
            || v.iter().filter(|x| x.abs() > 1e-4).count() != 1
        {
            return None;
        }
        let us = [0., radius / width, 1. - radius / width, 1.];
        let vs = [0., radius / height, 1. - radius / height, 1.];
        let uv_bounds: [(f32, f32); 2] = std::array::from_fn(|i| {
            face.vertices
                .iter()
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), p| {
                    (lo.min(p[i + 3]), hi.max(p[i + 3]))
                })
        });
        for y in 0..3 {
            for x in 0..3 {
                let at = |u: f32, v: f32| {
                    let mut vertex = std::array::from_fn(|i| {
                        (1. - v) * (a[i] * (1. - u) + b[i] * u) + v * (d[i] * (1. - u) + c[i] * u)
                    });
                    let p = pos(vertex);
                    let inner = std::array::from_fn(|i| p[i].clamp(lo[i] + radius, hi[i] - radius));
                    let delta = sub(p, inner);
                    let rounded = add(inner, mul(delta, radius / length(delta)));
                    vertex[..3].copy_from_slice(&rounded);
                    for i in 0..2 {
                        vertex[i + 3] = vertex[i + 3].clamp(uv_bounds[i].0, uv_bounds[i].1);
                    }
                    vertex
                };
                let vertices = [
                    at(us[x], vs[y]),
                    at(us[x + 1], vs[y]),
                    at(us[x + 1], vs[y + 1]),
                    at(us[x], vs[y + 1]),
                ];
                let mut n = normal(pos(vertices[0]), pos(vertices[1]), pos(vertices[2]));
                if dot(n, face.normal) < 0. {
                    n = mul(n, -1.);
                }
                out.push((
                    source,
                    SkinQuad {
                        normal: n,
                        vertices,
                    },
                ));
            }
        }
    }
    Some(out)
}
fn pos(v: [f32; 5]) -> Point {
    [v[0], v[1], v[2]]
}
fn dot(a: Point, b: Point) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}
fn length(p: Point) -> f32 {
    dot(p, p).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn cube(size: Point) -> Vec<SkinQuad> {
        let mut faces = Vec::new();
        for axis in 0..3 {
            for sign in [-1., 1.] {
                let mut n = [0.; 3];
                n[axis] = sign;
                let corners = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
                faces.push(SkinQuad {
                    normal: n,
                    vertices: corners.map(|[u, v]| {
                        let mut p = [0.; 5];
                        p[axis] = if sign > 0. { size[axis] } else { 0. };
                        p[(axis + 1) % 3] = u * size[(axis + 1) % 3];
                        p[(axis + 2) % 3] = v * size[(axis + 2) % 3];
                        p[3] = 0.125 + u * 0.0625;
                        p[4] = 0.25 + v * 0.125;
                        p
                    }),
                });
            }
        }
        faces
    }
    #[test]
    fn classic_slim_head_and_body_keep_skin_uvs_bounds_and_nonzero_normals() {
        for size in [[8., 8., 8.], [8., 12., 4.], [4., 12., 4.], [3., 12., 4.]] {
            let faces = cube(size);
            let rounded = rounded_skin_box(&faces).unwrap();
            assert_eq!(rounded.len(), 54);
            for (source, q) in &rounded {
                assert!((length(q.normal) - 1.).abs() < 1e-4);
                assert!(dot(q.normal, faces[*source].normal) > 0.);
                for vertex in q.vertices {
                    assert!(vertex.iter().all(|v| v.is_finite()));
                    for i in 0..3 {
                        assert!((-1e-4..=size[i] + 1e-4).contains(&vertex[i]));
                    }
                    assert!((0.125..=0.1875).contains(&vertex[3]));
                    assert!((0.25..=0.375).contains(&vertex[4]));
                }
            }
        }
    }
    #[test]
    fn rounded_edges_join_across_skin_faces_without_cracks() {
        let rounded = rounded_skin_box(&cube([8.; 3])).unwrap();
        // Quantize only to tolerate independent interpolation on shared edges.
        let mut edges = std::collections::HashMap::new();
        for (_, face) in rounded {
            for i in 0..4 {
                let a = pos(face.vertices[i]).map(|v| (v * 10000.).round() as i32);
                let b = pos(face.vertices[(i + 1) % 4]).map(|v| (v * 10000.).round() as i32);
                *edges
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_insert(0) += 1;
            }
        }
        assert!(edges.values().all(|&count| count == 2));
    }
    #[test]
    fn mirrored_and_rotated_skin_uvs_follow_the_original_face_not_an_atlas_guess() {
        let mut faces = cube([8.; 3]);
        for face in &mut faces {
            for p in &mut face.vertices {
                let u = p[3];
                p[3] = p[4];
                p[4] = 1. - u;
            }
        }
        let out = rounded_skin_box(&faces).unwrap();
        for (index, q) in out.iter().enumerate() {
            let source = &faces[index / 9];
            if index % 9 == 0 {
                assert_eq!(&q.1.vertices[0][3..], &source.vertices[0][3..]);
            }
            if index % 9 == 8 {
                assert_eq!(&q.1.vertices[2][3..], &source.vertices[2][3..]);
            }
        }
    }
    #[test]
    fn custom_non_box_or_degenerate_geometry_is_not_rewritten() {
        assert!(rounded_skin_box(&cube([0., 8., 8.])).is_none());
        let mut faces = cube([8.; 3]);
        faces[0].vertices[0][0] += 0.5;
        assert!(rounded_skin_box(&faces).is_none());
        let mut faces = cube([8.; 3]);
        faces[0].vertices[0][3] = f32::NAN;
        assert!(rounded_skin_box(&faces).is_none());
        assert!(rounded_skin_box(&cube([8.; 3])[..5]).is_none());
    }
}
