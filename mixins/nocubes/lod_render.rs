// Smooth only the already reduced LOD columns. Keep native masks, fluids,
// scheduling, distance levels, and greedy faces that do not change shape.
use super::*;
use horizon_core::nocubes::{self, Point};

struct Grid<'a> {
    key: TileKey,
    columns: &'a [&'a Column],
    neighbors: &'a [Vec<EdgeColumn<'a>>; 4],
    samples: nocubes::kinds::Samples,
}
impl Grid<'_> {
    fn width(&self) -> f32 {
        (1 << self.key.level) as f32
    }
    fn column(&self, x: f32, z: f32) -> Option<&Column> {
        let width = self.width();
        let (cx, cz) = ((x / width).floor() as i32, (z / width).floor() as i32);
        if (0..16).contains(&cx) && (0..16).contains(&cz) {
            return Some(self.columns[(cz * 16 + cx) as usize]);
        }
        let size = self.key.size() as f32;
        let (side, along) = if x < 0. && (0.0..size).contains(&z) {
            (0, z)
        } else if x >= size && (0.0..size).contains(&z) {
            (1, z)
        } else if z < 0. && (0.0..size).contains(&x) {
            (2, x)
        } else if z >= size && (0.0..size).contains(&x) {
            (3, x)
        } else {
            return None;
        };
        self.neighbors[side]
            .iter()
            .find(|e| {
                !e.unknown
                    && e.end - e.start == width as i32
                    && along >= e.start as f32
                    && along < e.end as f32
            })
            .map(|e| e.column)
    }
    fn density(&self, p: Point) -> Option<f32> {
        let column = self.column(p[0], p[2])?;
        let radius = (self.width() * 0.5).min(32.);
        let mut best = -radius;
        let mut run: Option<(f32, f32)> = None;
        let mut add = |lo: f32, hi: f32| {
            best = best.max((p[1] - lo).min(hi - p[1]));
        };
        for span in &column.spans {
            let (lo, hi) = (f32::from(span.min), f32::from(span.max));
            if !self.samples.get(span.state.0).smooth {
                if lo <= p[1] && p[1] <= hi {
                    return None;
                }
                continue;
            }
            if let Some((start, end)) = run {
                if lo == end {
                    run = Some((start, hi));
                } else {
                    add(start, end);
                    run = Some((lo, hi));
                }
            } else {
                run = Some((lo, hi));
            }
        }
        if let Some((lo, hi)) = run {
            add(lo, hi);
        }
        Some(best.clamp(-radius, radius))
    }
    fn protected(&self, p: Point) -> bool {
        let half = self.width() * 0.5;
        [-half, half].into_iter().any(|dx| {
            [-half, half].into_iter().any(|dz| {
                self.column(p[0] + dx, p[2] + dz).is_none_or(|column| {
                    column.spans.iter().any(|s| {
                        !self.samples.get(s.state.0).smooth
                            && f32::from(s.min) <= p[1] + 0.01
                            && f32::from(s.max) >= p[1] - 0.01
                    })
                })
            })
        })
    }
    fn owner(&self, quad: &[TerrainVertex; 4], normal: Point) -> Option<Span> {
        let p: Point = std::array::from_fn(|a| {
            quad.iter().map(|v| v.pos[a]).sum::<f32>() * 0.25 - normal[a] * 0.01
        });
        self.column(p[0], p[2])?
            .spans
            .iter()
            .find(|s| f32::from(s.min) <= p[1] && p[1] < f32::from(s.max))
            .copied()
    }
}
fn direction(n: Point) -> Direction {
    let axis = (0..3)
        .max_by(|&a, &b| n[a].abs().total_cmp(&n[b].abs()))
        .unwrap();
    match (axis, n[axis] > 0.) {
        (0, false) => Direction::West,
        (0, true) => Direction::East,
        (1, false) => Direction::Down,
        (1, true) => Direction::Up,
        (2, false) => Direction::North,
        _ => Direction::South,
    }
}
fn cuts(min: f32, max: f32, axis: usize, width: f32, heights: &[f32]) -> Vec<f32> {
    let mut out = vec![min];
    if axis == 1 {
        out.extend(heights.iter().copied().filter(|y| *y > min && *y < max));
    } else {
        let mut at = ((min / width).floor() + 1.) * width;
        while at < max {
            out.push(at);
            at += width;
        }
    }
    out.push(max);
    out
}
fn clipped(p: Point, key: TileKey, covered: &FxHashSet<(i32, i32)>, finer: &[TileKey]) -> bool {
    let [ox, _, oz] = key.origin();
    let covered_at = |x: f32, z: f32| {
        let (x, z) = (ox + x.floor() as i32, oz + z.floor() as i32);
        covered.contains(&(x.div_euclid(16), z.div_euclid(16)))
            || finer.iter().any(|k| {
                let [kx, _, kz] = k.origin();
                x >= kx && x < kx + k.size() && z >= kz && z < kz + k.size()
            })
    };
    let states = [-0.25, 0.25].map(|dx| [-0.25, 0.25].map(|dz| covered_at(p[0] + dx, p[2] + dz)));
    states.iter().flatten().any(|v| *v) && states.iter().flatten().any(|v| !*v)
}

pub(super) fn mesh(
    mesher: &mut Mesher,
    key: TileKey,
    columns: &[&Column],
    covered: &FxHashSet<(i32, i32)>,
    covered_bounds: Option<[i32; 4]>,
    context: &WorkerContext,
    neighbors: &[Vec<EdgeColumn<'_>>; 4],
    finer: &[TileKey],
    cfg: &std::sync::Arc<nocubes::Config>,
) -> SectionMesh {
    let source = mesh_columns_with_edges(
        key,
        columns,
        covered,
        covered_bounds,
        neighbors,
        finer,
        |span, d, at| mesher.material(span, d, at, context),
    );
    let samples = nocubes::kinds::samples(cfg);
    if !columns
        .iter()
        .any(|c| c.spans.iter().any(|s| samples.get(s.state.0).smooth))
    {
        return source;
    }
    // Match Horizon's heightmap reduction at its coarse levels. Preserve the
    // source's above-ground caves at finer levels and its water at every level.
    let reduced: Vec<Column>;
    let refs: Vec<&Column>;
    let reduced_edges: [Vec<Column>; 4];
    let edge_refs: [Vec<EdgeColumn<'_>>; 4];
    let neighbors = if key.level >= HEIGHTMAP_LEVEL {
        reduced_edges = neighbors.each_ref().map(|side| {
            side.iter()
                .map(|edge| {
                    heightmap(edge.column, |s| {
                        mesher.material(s, Direction::Up, [0; 2], context).fluid != 0
                    })
                })
                .collect()
        });
        edge_refs = std::array::from_fn(|side| {
            neighbors[side]
                .iter()
                .zip(&reduced_edges[side])
                .map(|(e, column)| EdgeColumn {
                    start: e.start,
                    end: e.end,
                    column,
                    unknown: e.unknown,
                })
                .collect()
        });
        &edge_refs
    } else {
        neighbors
    };
    let columns = if key.level >= HEIGHTMAP_LEVEL {
        reduced = columns
            .iter()
            .map(|c| {
                heightmap(c, |s| {
                    mesher.material(s, Direction::Up, [0; 2], context).fluid != 0
                })
            })
            .collect();
        refs = reduced.iter().collect();
        &refs[..]
    } else {
        columns
    };
    let grid = Grid {
        key,
        columns,
        neighbors,
        samples,
    };
    smooth(
        source,
        &grid,
        covered,
        finer,
        cfg,
        |state| {
            context
                .assets
                .registry
                .state(state)
                .is_some_and(|s| s.properties.contains_key("snowy"))
        },
        |span, at| mesher.material(span, Direction::Up, at, context).faces[0],
    )
}
fn smooth(
    mut source: SectionMesh,
    grid: &Grid<'_>,
    covered: &FxHashSet<(i32, i32)>,
    finer: &[TileKey],
    cfg: &nocubes::Config,
    grass: impl Fn(BlockStateId) -> bool,
    mut grass_top: impl FnMut(&Span, [i32; 2]) -> Option<Paint>,
) -> SectionMesh {
    let key = grid.key;
    let mut heights: Vec<f32> = grid
        .columns
        .iter()
        .copied()
        .chain(grid.neighbors.iter().flatten().map(|e| e.column))
        .flat_map(|c| {
            c.spans
                .iter()
                .flat_map(|s| [f32::from(s.min), f32::from(s.max)])
        })
        .collect();
    heights.sort_unstable_by(f32::total_cmp);
    heights.dedup();
    let mut cache: FxHashMap<[u32; 3], Point> = FxHashMap::default();
    let mut replaced = FxHashSet::default();
    let mut solid = Vec::new();
    let mut translucent = Vec::new();
    let budget = ((source.solid.len() + source.translucent.len()) * 2).clamp(4096, 32768);
    let [ox, _, oz] = key.origin();
    let mut over_budget = false;
    'emit: for (input, output) in [
        (&source.solid, &mut solid),
        (&source.translucent, &mut translucent),
    ] {
        for quad in input.as_chunks::<4>().0 {
            let n = nocubes::normal(quad[0].pos, quad[1].pos, quad[2].pos);
            let Some(span) = grid.owner(quad, n) else {
                output.extend_from_slice(quad);
                continue;
            };
            if !grid.samples.get(span.state.0).smooth {
                output.extend_from_slice(quad);
                continue;
            }
            let d = direction(n);
            let axis = match d {
                Direction::Up | Direction::Down => 1,
                Direction::East | Direction::West => 0,
                _ => 2,
            };
            let axes = match axis {
                0 => [2, 1],
                1 => [0, 2],
                _ => [0, 1],
            };
            let min: Point = std::array::from_fn(|a| {
                quad.iter().map(|v| v.pos[a]).fold(f32::INFINITY, f32::min)
            });
            let max: Point = std::array::from_fn(|a| {
                quad.iter()
                    .map(|v| v.pos[a])
                    .fold(f32::NEG_INFINITY, f32::max)
            });
            let u = cuts(min[axes[0]], max[axes[0]], axes[0], grid.width(), &heights);
            let v = cuts(min[axes[1]], max[axes[1]], axes[1], grid.width(), &heights);
            // Bound both CPU work and output for pathological layered columns.
            // A complex tile retains its original reduced mesh, never full chunks.
            if u.len().saturating_mul(v.len()) > 8192 {
                over_budget = true;
                break 'emit;
            }
            let before = output.len();
            let mut changed = false;
            let grass = grass(span.state);
            for uw in u.windows(2) {
                for vw in v.windows(2) {
                    let original: [Point; 4] = quad.map(|vertex| {
                        let mut p = vertex.pos;
                        p[axes[0]] = if p[axes[0]] == min[axes[0]] {
                            uw[0]
                        } else {
                            uw[1]
                        };
                        p[axes[1]] = if p[axes[1]] == min[axes[1]] {
                            vw[0]
                        } else {
                            vw[1]
                        };
                        p
                    });
                    let points = original.map(|p| {
                        *cache.entry(p.map(f32::to_bits)).or_insert_with(|| {
                            if clipped(p, key, covered, finer) || grid.protected(p) {
                                p
                            } else {
                                nocubes::lod_vertex(p, grid.width(), |q| grid.density(q))
                            }
                        })
                    });
                    changed |= points != original;
                    let normal = nocubes::normal(points[0], points[1], points[2]);
                    if normal == [0.; 3] {
                        continue;
                    }
                    let top =
                        grass && (normal[1] > 1e-4 || cfg.better_grass_sides && normal[1] >= 0.);
                    let paint = if top && d != Direction::Up {
                        // Replace both side layers with just one tinted grass top.
                        if !replaced.insert(original.map(|p| p.map(f32::to_bits))) {
                            continue;
                        }
                        changed = true;
                        let at = [ox + original[0][0] as i32, oz + original[0][2] as i32];
                        grass_top(&span, at)
                    } else {
                        None
                    };
                    let edge_a = nocubes::sub(quad[1].pos, quad[0].pos);
                    let edge_b = nocubes::sub(quad[3].pos, quad[0].pos);
                    let dot =
                        |a: Point, b: Point| a.into_iter().zip(b).map(|(a, b)| a * b).sum::<f32>();
                    let source_uv =
                        quad.map(|p| p.uv.map(|u| f32::from(u) / crate::mesher::TILED_UV_STEPS));
                    let mut uv = original.map(|p| {
                        let delta = nocubes::sub(p, quad[0].pos);
                        let s = dot(delta, edge_a) / dot(edge_a, edge_a);
                        let t = dot(delta, edge_b) / dot(edge_b, edge_b);
                        std::array::from_fn(|i| {
                            source_uv[0][i]
                                + s * (source_uv[1][i] - source_uv[0][i])
                                + t * (source_uv[3][i] - source_uv[0][i])
                        })
                    });
                    if paint.is_some() {
                        uv = points.map(|p| [p[0], -p[2]]);
                        let corner = [0, 1].map(|i| {
                            uv.iter()
                                .map(|p| p[i])
                                .fold(f32::INFINITY, f32::min)
                                .floor()
                        });
                        let scale = (1024. / grid.width()).min(1.);
                        uv = uv.map(|p| std::array::from_fn(|i| (p[i] - corner[i]) * scale));
                    }
                    output.extend((0..4).map(|i| {
                        let (layer, color, light) = paint
                            .map_or((quad[0].layer, quad[0].color, quad[0].light), |p| {
                                (p.layer, p.color, p.light)
                            });
                        TerrainVertex::tiled(points[i], uv[i], layer, color, light)
                    }));
                    if output.len() > budget || cache.len() > 16384 {
                        over_budget = true;
                        break 'emit;
                    }
                }
            }
            if !changed {
                output.truncate(before);
                output.extend_from_slice(quad);
            }
        }
    }
    if over_budget || solid.len() + translucent.len() > budget {
        return source;
    }
    source.solid = solid;
    source.translucent = translucent;
    let layers: Vec<_> = source
        .solid
        .iter()
        .chain(&source.translucent)
        .map(|v| v.layer)
        .collect();
    for layer in layers {
        source.note_sprite_layer(layer);
    }
    source.finish_sprite_layers();
    source
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_material(s: &Span, d: Direction, _: [i32; 2]) -> Material {
        let water = s.state.0 == 2;
        let paint = |layer| {
            Some(Paint {
                layer,
                light: 240,
                color: u32::MAX,
                translucent: water,
            })
        };
        Material {
            faces: if water {
                [paint(20), None]
            } else if matches!(d, Direction::Up | Direction::Down) {
                [paint(10), None]
            } else {
                [paint(11), paint(12)]
            },
            fluid: u8::from(water),
            opaque: !water,
        }
    }
    fn fixture(columns: &[Column], level: u8) -> (SectionMesh, SectionMesh) {
        let key = TileKey { x: 0, z: 0, level };
        let refs: Vec<_> = columns.iter().collect();
        let neighbors = std::array::from_fn(|_| Vec::new());
        let grid = Grid {
            key,
            columns: &refs,
            neighbors: &neighbors,
            samples: samples(),
        };
        let make = || {
            mesh_columns_with_edges(
                key,
                &refs,
                &FxHashSet::default(),
                None,
                &neighbors,
                &[],
                test_material,
            )
        };
        let original = make();
        let smoothed = smooth(
            make(),
            &grid,
            &FxHashSet::default(),
            &[],
            &nocubes::Config::default(),
            |_| true,
            |s, at| test_material(s, Direction::Up, at).faces[0],
        );
        (original, smoothed)
    }
    fn samples() -> nocubes::kinds::Samples {
        nocubes::kinds::install_resolver(3, |_, state| nocubes::Sample {
            state,
            smooth: state == 1,
            solid: state == 1,
            density: if state == 1 { 1. } else { -1. },
        });
        nocubes::kinds::samples(&std::sync::Arc::new(nocubes::Config::default()))
    }
    fn span(min: i16, max: i16, state: u32) -> Span {
        Span {
            min,
            max,
            state: BlockStateId(state),
            biome: 0,
            light: 240,
        }
    }
    #[test]
    fn contiguous_material_runs_do_not_create_a_false_air_surface() {
        let columns = [Column {
            surface: None,
            spans: vec![span(-64, 69, 1), span(69, 70, 1)].into(),
        }; 1];
        let refs = vec![&columns[0]; 256];
        let neighbors = std::array::from_fn(|_| Vec::new());
        let grid = Grid {
            key: TileKey {
                x: 0,
                z: 0,
                level: 3,
            },
            columns: &refs,
            neighbors: &neighbors,
            samples: samples(),
        };
        assert_eq!(grid.density([4., 69., 4.]), Some(1.));
        assert_eq!(
            nocubes::lod_vertex([8., 70., 8.], 8., |p| grid.density(p)),
            [8., 70., 8.]
        );
    }
    #[test]
    fn water_non_smooth_blocks_and_mixed_level_edges_anchor_geometry() {
        let column = Column {
            surface: None,
            spans: vec![span(-64, 70, 1), span(70, 74, 2)].into(),
        };
        let refs = vec![&column; 256];
        let mut neighbors = std::array::from_fn(|_| Vec::new());
        neighbors[0].push(EdgeColumn {
            start: 0,
            end: 16,
            column: &column,
            unknown: false,
        });
        let key = TileKey {
            x: 0,
            z: 0,
            level: 3,
        };
        let grid = Grid {
            key,
            columns: &refs,
            neighbors: &neighbors,
            samples: samples(),
        };
        assert_eq!(grid.density([4., 72., 4.]), None);
        assert!(
            grid.column(-4., 4.).is_none(),
            "a different LOD width pins the seam"
        );
        assert_eq!(
            nocubes::lod_vertex([0., 70., 8.], 8., |p| grid.density(p)),
            [0., 70., 8.]
        );
    }
    #[test]
    fn same_level_neighbors_produce_the_same_shared_edge_vertex() {
        let low = Column {
            surface: None,
            spans: vec![span(-64, 70, 1)].into(),
        };
        let high = Column {
            surface: None,
            spans: vec![span(-64, 78, 1)].into(),
        };
        let left = vec![&low; 256];
        let right = vec![&high; 256];
        let mut le = std::array::from_fn(|_| Vec::new());
        let mut re = std::array::from_fn(|_| Vec::new());
        for z in 0..16 {
            le[1].push(EdgeColumn {
                start: z * 8,
                end: (z + 1) * 8,
                column: &high,
                unknown: false,
            });
            re[0].push(EdgeColumn {
                start: z * 8,
                end: (z + 1) * 8,
                column: &low,
                unknown: false,
            });
        }
        let a = Grid {
            key: TileKey {
                x: 0,
                z: 0,
                level: 3,
            },
            columns: &left,
            neighbors: &le,
            samples: samples(),
        };
        let b = Grid {
            key: TileKey {
                x: 1,
                z: 0,
                level: 3,
            },
            columns: &right,
            neighbors: &re,
            samples: samples(),
        };
        let va = nocubes::lod_vertex([128., 70., 8.], 8., |p| a.density(p));
        let vb = nocubes::lod_vertex([0., 70., 8.], 8., |p| b.density(p));
        for i in 0..3 {
            assert!((va[i] - vb[i] - if i == 0 { 128. } else { 0. }).abs() < 0.001);
        }
        assert!(va[1] > 70.);
    }
    #[test]
    fn clipping_stays_on_the_native_footprint_and_cell_count_stays_lod_sized() {
        let key = TileKey {
            x: -1,
            z: -1,
            level: 3,
        };
        let mut covered = FxHashSet::default();
        covered.insert((-7, -7));
        assert!(clipped([16., 70., 24.], key, &covered, &[]));
        assert!(!clipped([8., 70., 8.], key, &covered, &[]));
        let fine = TileKey {
            x: -7,
            z: -7,
            level: 0,
        };
        assert!(clipped(
            [16., 70., 24.],
            key,
            &FxHashSet::default(),
            &[fine]
        ));
        for level in [0, 3, 12, 22] {
            let width = (1u32 << level) as f32;
            assert_eq!(cuts(0., 16. * width, 0, width, &[]).len(), 17);
        }
    }
    #[test]
    fn flat_tiles_retain_greedy_mesh_and_water_vertices_are_unchanged() {
        let flat = vec![
            Column {
                surface: None,
                spans: vec![span(-64, 70, 1)].into()
            };
            256
        ];
        let (old, new) = fixture(&flat, 3);
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&old.solid),
            bytemuck::cast_slice::<_, u8>(&new.solid)
        );
        let sea = vec![
            Column {
                surface: None,
                spans: vec![span(-64, 70, 1), span(70, 74, 2)].into()
            };
            256
        ];
        let (old, new) = fixture(&sea, 3);
        assert_eq!(
            bytemuck::cast_slice::<_, u8>(&old.translucent),
            bytemuck::cast_slice::<_, u8>(&new.translucent)
        );
    }
    #[test]
    fn reduced_steps_are_smoothed_and_sloping_grass_uses_one_top_layer() {
        let columns: Vec<_> = (0..256)
            .map(|i| Column {
                surface: None,
                spans: vec![span(-64, if i % 16 < 8 { 70 } else { 78 }, 1)].into(),
            })
            .collect();
        let (old, new) = fixture(&columns, 3);
        assert!(
            new.solid.iter().any(|v| v.pos[1] > 70. && v.pos[1] < 78.),
            "the LOD step must actually change shape"
        );
        assert!(new.solid.len() <= (old.solid.len() * 2).clamp(4096, 32768));
        let slopes: Vec<_> = new
            .solid
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|q| {
                let n = nocubes::normal(q[0].pos, q[1].pos, q[2].pos);
                n[1] > 0.001 && n[1] < 0.999
            })
            .collect();
        assert!(!slopes.is_empty());
        assert!(
            slopes.iter().all(|q| q.iter().all(|v| v.layer == 10)),
            "no side overlay may cover an upward slope"
        );
    }
}
