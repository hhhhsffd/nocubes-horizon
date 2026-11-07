//! Translation of CollisionHandler.generateShapes and its four enhancement levels.
use super::*;
pub fn face_collision_boxes(face: &Face, cfg: &Config) -> Vec<[f64; 6]> {
    let v = face.vertices;
    let centre = mul(v.into_iter().fold([0.; 3], add), 0.25);
    let inward = mul(
        face.area_normal(),
        if cfg.mesher == Mesher::OldNoCubes {
            -0.00001
        } else {
            -1.
        },
    );
    let mut boxes = vec![];
    let mut vertex = |p: Point| {
        boxes.push([
            p[0] as f64 - 0.125,
            p[1] as f64 - 0.125,
            p[2] as f64 - 0.125,
            p[0] as f64 + 0.125,
            p[1] as f64 + 0.125,
            p[2] as f64 + 0.125,
        ]);
    };
    let level = cfg.old_collision_enhancement;
    if level >= 1 {
        for p in v {
            vertex(p);
        }
        vertex(centre);
    }
    if level >= 2 {
        for i in 0..4 {
            vertex(mul(add(v[i], v[(i + 1) % 4]), 0.5));
        }
    }
    if level >= 3 {
        for i in 0..4 {
            for t in [0.25, 0.75] {
                vertex(add(v[i], mul(sub(v[(i + 1) % 4], v[i]), t)));
            }
        }
    }
    if level >= 4 {
        for p in v {
            vertex(mul(add(p, centre), 0.5));
        }
    }
    if !cfg.only_old_collisions {
        let ext = add(centre, inward);
        for p in v {
            let lo = std::array::from_fn::<_, 3, _>(|i| p[i].min(ext[i]) as f64);
            let hi = std::array::from_fn::<_, 3, _>(|i| p[i].max(ext[i]) as f64);
            if (0..3).all(|i| hi[i] > lo[i]) {
                boxes.push([lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]]);
            }
        }
    }
    boxes
}
pub fn collision_boxes(sample: &impl Fn(Pos) -> Sample, p: Pos, cfg: &Config) -> Vec<[f64; 6]> {
    collision_boxes_at(sample, p, cfg, [0; 3])
}
/// Sample coordinates are local to `world_origin`; returned boxes remain block-local.
pub fn collision_boxes_at(
    sample: &impl Fn(Pos) -> Sample,
    p: Pos,
    cfg: &Config,
    world_origin: Pos,
) -> Vec<[f64; 6]> {
    let local = |q: Pos| sample(std::array::from_fn(|i| q[i] + p[i]));
    let origin = std::array::from_fn(|i| world_origin[i] + p[i]);
    collision_boxes_local(&local, cfg, origin)
}
fn collision_boxes_local(
    sample: &impl Fn(Pos) -> Sample,
    cfg: &Config,
    origin: Pos,
) -> Vec<[f64; 6]> {
    let p = [0; 3];
    if cfg.sampled_collisions {
        return sampled_collision_boxes_local(sample, cfg, origin);
    }
    if matches!(
        cfg.mesher,
        Mesher::CullingCubic | Mesher::StupidCubic | Mesher::CullingChamfer
    ) {
        return if sample(p).smooth {
            vec![[0., 0., 0., 1., 1., 1.]]
        } else {
            vec![]
        };
    }
    if matches!(cfg.mesher, Mesher::OldNoCubes | Mesher::Wulferis) {
        if sample(p).smooth
            && DIRS
                .iter()
                .all(|d| sample(std::array::from_fn(|i| p[i] + d[i])).density == 1.)
        {
            return vec![[0., 0., 0., 1., 1., 1.]];
        }
        return generate_at(sample, p, p.map(|v| v + 1), cfg, origin)
            .iter()
            .flat_map(|face| face_collision_boxes(face, cfg))
            .map(|b| std::array::from_fn(|i| b[i] - p[i % 3] as f64))
            .collect();
    }
    let faces = generate_at(sample, p.map(|v| v - 1), p.map(|v| v + 2), cfg, origin);
    let mut boxes = vec![];
    // Entirely filled cells must remain solid, even if they produce no surface.
    for z in -1..=0 {
        for y in -1..=0 {
            for x in -1..=0 {
                let q = [p[0] + x, p[1] + y, p[2] + z];
                if (0..8).all(|c| {
                    field(
                        sample,
                        corner(q, c),
                        matches!(cfg.mesher, Mesher::SurfaceNets2x | Mesher::MarchingCubes2x),
                    ) < 0.
                }) {
                    let offset =
                        if matches!(cfg.mesher, Mesher::SurfaceNets2x | Mesher::MarchingCubes2x) {
                            1.
                        } else {
                            0.5
                        };
                    let lo = q.map(|v| v as f64 + offset);
                    boxes.push([lo[0], lo[1], lo[2], lo[0] + 1., lo[1] + 1., lo[2] + 1.]);
                }
            }
        }
    }
    for face in faces {
        boxes.extend(face_collision_boxes(&face, cfg));
    }
    boxes
        .into_iter()
        .filter_map(|b| {
            if (0..3).any(|i| b[i + 3] <= p[i] as f64 || b[i] >= p[i] as f64 + 1.) {
                return None;
            }
            Some(std::array::from_fn(|i| b[i] - p[i % 3] as f64))
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    fn top() -> Face {
        Face {
            vertices: [[0., 1., 0.], [0., 1., 1.], [1., 1., 1.], [1., 1., 0.]],
            block: [0; 3],
            state: 1,
        }
    }
    #[test]
    fn enhancement_levels_match_original_sample_counts() {
        for (level, count) in [(1, 5), (2, 9), (3, 17), (4, 21)] {
            let c = Config {
                old_collision_enhancement: level,
                only_old_collisions: true,
                ..Config::default()
            };
            let b = face_collision_boxes(&top(), &c);
            assert_eq!(b.len(), count);
            assert_eq!(b[0], [-0.125, 0.875, -0.125, 0.125, 1.125, 0.125]);
        }
    }
    #[test]
    fn new_collision_boxes_extend_inward() {
        let b = face_collision_boxes(&top(), &Config::default());
        assert_eq!(b.len(), 4);
        assert!(b.iter().all(|b| b[1] == 0. && b[4] == 1.));
    }
}
