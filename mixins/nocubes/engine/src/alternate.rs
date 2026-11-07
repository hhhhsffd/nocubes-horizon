use super::*;
pub(super) fn chamfer(sample: &impl Fn(Pos) -> Sample, lo: Pos, hi: Pos) -> Vec<Face> {
    let mut out = vec![];
    for iz in lo[2]..hi[2] {
        for iy in lo[1]..hi[1] {
            for ix in lo[0]..hi[0] {
                let block = [ix, iy, iz];
                let s = sample(block);
                if !s.smooth {
                    continue;
                }
                let [x, y, z] = block.map(|v| v as f32);
                let min = 0.25;
                let max = 0.75;
                let pos_x = !sample(offset(block, 0, 1)).smooth;
                let neg_x = !sample(offset(block, 0, -1)).smooth;
                let pos_y = !sample(offset(block, 1, 1)).smooth;
                let neg_y = !sample(offset(block, 1, -1)).smooth;
                let pos_z = !sample(offset(block, 2, 1)).smooth;
                let neg_z = !sample(offset(block, 2, -1)).smooth;
                let max_x = if pos_x { max } else { 1. };
                let min_x = if neg_x { min } else { 0. };
                let max_y = if pos_y { max } else { 1. };
                let min_y = if neg_y { min } else { 0. };
                let max_z = if pos_z { max } else { 1. };
                let min_z = if neg_z { min } else { 0. };
                if pos_y {
                    out.push(Face {
                        vertices: [
                            [x + max_x, y + 1., z + max_z],
                            [x + max_x, y + 1., z + min_z],
                            [x + min_x, y + 1., z + min_z],
                            [x + min_x, y + 1., z + max_z],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_y {
                    out.push(Face {
                        vertices: [
                            [x + max_x, y + 0., z + max_z],
                            [x + min_x, y + 0., z + max_z],
                            [x + min_x, y + 0., z + min_z],
                            [x + max_x, y + 0., z + min_z],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if pos_z {
                    out.push(Face {
                        vertices: [
                            [x + max_x, y + max_y, z + 1.],
                            [x + min_x, y + max_y, z + 1.],
                            [x + min_x, y + min_y, z + 1.],
                            [x + max_x, y + min_y, z + 1.],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_z {
                    out.push(Face {
                        vertices: [
                            [x + max_x, y + max_y, z + 0.],
                            [x + max_x, y + min_y, z + 0.],
                            [x + min_x, y + min_y, z + 0.],
                            [x + min_x, y + max_y, z + 0.],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if pos_x {
                    out.push(Face {
                        vertices: [
                            [x + 1., y + max_y, z + max_z],
                            [x + 1., y + min_y, z + max_z],
                            [x + 1., y + min_y, z + min_z],
                            [x + 1., y + max_y, z + min_z],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_x {
                    out.push(Face {
                        vertices: [
                            [x + 0., y + max_y, z + max_z],
                            [x + 0., y + max_y, z + min_z],
                            [x + 0., y + min_y, z + min_z],
                            [x + 0., y + min_y, z + max_z],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if pos_y && pos_z {
                    out.push(Face {
                        vertices: [
                            [x + min_x, y + 1. - min, z + 1.],
                            [x + max_x, y + 1. - min, z + 1.],
                            [x + max_x, y + 1., z + 1. - min],
                            [x + min_x, y + 1., z + 1. - min],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if pos_y && neg_z {
                    out.push(Face {
                        vertices: [
                            [x + min_x, y + 1., z + 0. + min],
                            [x + max_x, y + 1., z + 0. + min],
                            [x + max_x, y + 1. - min, z + 0.],
                            [x + min_x, y + 1. - min, z + 0.],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if pos_y && pos_x {
                    out.push(Face {
                        vertices: [
                            [x + 1., y + 1. - min, z + max_z],
                            [x + 1., y + 1. - min, z + min_z],
                            [x + 1. - min, y + 1., z + min_z],
                            [x + 1. - min, y + 1., z + max_z],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if pos_y && neg_x {
                    out.push(Face {
                        vertices: [
                            [x + 0. + min, y + 1., z + max_z],
                            [x + 0. + min, y + 1., z + min_z],
                            [x + 0., y + 1. - min, z + min_z],
                            [x + 0., y + 1. - min, z + max_z],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_y && pos_z {
                    out.push(Face {
                        vertices: [
                            [x + min_x, y + 0., z + 1. - min],
                            [x + max_x, y + 0., z + 1. - min],
                            [x + max_x, y + 0. + min, z + 1.],
                            [x + min_x, y + 0. + min, z + 1.],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_y && neg_z {
                    out.push(Face {
                        vertices: [
                            [x + min_x, y + 0. + min, z + 0.],
                            [x + max_x, y + 0. + min, z + 0.],
                            [x + max_x, y + 0., z + 0. + min],
                            [x + min_x, y + 0., z + 0. + min],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_y && pos_x {
                    out.push(Face {
                        vertices: [
                            [x + 1. - min, y + 0., z + max_z],
                            [x + 1. - min, y + 0., z + min_z],
                            [x + 1., y + 0. + min, z + min_z],
                            [x + 1., y + 0. + min, z + max_z],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_y && neg_x {
                    out.push(Face {
                        vertices: [
                            [x + 0., y + 0. + min, z + max_z],
                            [x + 0., y + 0. + min, z + min_z],
                            [x + 0. + min, y + 0., z + min_z],
                            [x + 0. + min, y + 0., z + max_z],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if pos_x && pos_z {
                    out.push(Face {
                        vertices: [
                            [x + 1. - min, y + 0. + min_y, z + 1.],
                            [x + 1., y + 0. + min_y, z + 1. - min],
                            [x + 1., y + 0. + max_y, z + 1. - min],
                            [x + 1. - min, y + 0. + max_y, z + 1.],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if pos_x && neg_z {
                    out.push(Face {
                        vertices: [
                            [x + 1. - min, y + 0. + max_y, z + 0.],
                            [x + 1., y + 0. + max_y, z + 0. + min],
                            [x + 1., y + 0. + min_y, z + 0. + min],
                            [x + 1. - min, y + 0. + min_y, z + 0.],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_x && pos_z {
                    out.push(Face {
                        vertices: [
                            [x + 0. + min, y + 0. + max_y, z + 1.],
                            [x + 0., y + 0. + max_y, z + 1. - min],
                            [x + 0., y + 0. + min_y, z + 1. - min],
                            [x + 0. + min, y + 0. + min_y, z + 1.],
                        ],
                        block,
                        state: s.state,
                    });
                }
                if neg_x && neg_z {
                    out.push(Face {
                        vertices: [
                            [x + 0. + min, y + 0. + min_y, z + 0.],
                            [x + 0., y + 0. + min_y, z + 0. + min],
                            [x + 0., y + 0. + max_y, z + 0. + min],
                            [x + 0. + min, y + 0. + max_y, z + 0.],
                        ],
                        block,
                        state: s.state,
                    });
                }
            }
        }
    }
    out
}

fn density(sample: &impl Fn(Pos) -> Sample, p: Point) -> f32 {
    let q = p.map(|v| v - 0.5);
    let base = q.map(|v| v.floor() as i32);
    let f = std::array::from_fn::<_, 3, _>(|i| q[i] - base[i] as f32);
    (0..8)
        .map(|c| {
            let s = sample(corner(base, c));
            let d = if s.smooth {
                1.
            } else if s.solid {
                -1000.
            } else {
                -1.
            };
            d * (0..3)
                .map(|i| if c & (1 << i) != 0 { f[i] } else { 1. - f[i] })
                .product::<f32>()
        })
        .sum()
}
pub(super) fn wulferis(sample: &impl Fn(Pos) -> Sample, p: Point) -> Point {
    let mut gradient = [0.; 3];
    for (a, g) in gradient.iter_mut().enumerate() {
        let mut lo = p;
        let mut hi = p;
        lo[a] -= 0.5;
        hi[a] += 0.5;
        let d0 = density(sample, hi);
        let d1 = density(sample, lo);
        if d0.abs() > 5. || d1.abs() > 5. {
            return p;
        }
        *g = d0 - d1;
    }
    let length = gradient.iter().map(|v| v * v).sum::<f32>().sqrt();
    if length > 0.0001 {
        add(p, mul(gradient, -density(sample, p) * 0.75 / length))
    } else {
        p
    }
}
pub(super) fn old(
    sample: &impl Fn(Pos) -> Sample,
    mut v: Point,
    block: Pos,
    cfg: &Config,
    origin: Pos,
) -> Point {
    let mut protected = false;
    let mut top_clear = true;
    let mut bottom_clear = true;
    let mut side_clear = false;
    let mut below_clear = false;
    let mut above_solid = false;
    for i in 0..4 {
        let p = [
            v[0] as i32 - (i & 1),
            v[1] as i32,
            v[2] as i32 - ((i >> 1) & 1),
        ];
        let s = sample(p);
        let b = sample(offset(p, 1, -1));
        protected |= (s.solid && !s.smooth) || (b.solid && !b.smooth);
        top_clear &= !s.solid;
        bottom_clear &= !b.solid;
        side_clear |= !s.solid;
        below_clear |= !b.solid;
        above_solid |= sample(offset(p, 1, 1)).solid;
    }
    if protected {
        return v;
    }
    if cfg.old_slopes {
        if v[1] as i32 == block[1] && bottom_clear && side_clear && above_solid {
            v[1] = block[1] as f32 + 0.9999;
        } else if v[1] as i32 == block[1] + 1 && top_clear && below_clear {
            v[1] = block[1] as f32 + 0.0001;
        }
    }
    let world = std::array::from_fn::<_, 3, _>(|i| v[i] as f64 + origin[i] as f64);
    let mut h =
        (world[0] * 3129871.) as i64 ^ (world[1] as i64).wrapping_mul(116129781) ^ world[2] as i64;
    h = h
        .wrapping_mul(h)
        .wrapping_mul(42317861)
        .wrapping_add(h.wrapping_mul(11));
    for (i, a) in v.iter_mut().enumerate() {
        *a += (((h >> (16 + i * 4)) & 15) as f32 / 15. - 0.5) * cfg.roughness;
    }
    v
}
