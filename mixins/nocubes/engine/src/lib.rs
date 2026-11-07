//! NoCubes terrain algorithms. Coordinates are block-centred density samples.
//! Derived from Cadiboo/NoCubes (LGPL-3.0); Surface Nets by Mikola Lysenko.
#![forbid(unsafe_code)]
mod defaults;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
pub type Point = [f32; 3];
pub type Pos = [i32; 3];
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum Mesher {
    #[default]
    SurfaceNets,
    SurfaceNets2x,
    MarchingCubes,
    MarchingCubes2x,
    CullingCubic,
    StupidCubic,
    OldNoCubes,
    CullingChamfer,
    Wulferis,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub render: bool,
    pub collisions: bool,
    pub mesher: Mesher,
    pub use_defaults: bool,
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
    pub state_overrides: std::collections::BTreeMap<String, bool>,
    pub better_grass_sides: bool,
    pub more_snow: bool,
    pub fix_plant_height: bool,
    pub roughness: f32,
    pub old_slopes: bool,
    pub collision_resolution: usize,
    pub sampled_collisions: bool,
    pub old_collision_enhancement: u8,
    pub only_old_collisions: bool,
    pub player_collisions_only: bool,
    pub force_visuals: bool,
    pub extend_fluids_range: u8,
    pub old_in_fluids: bool,
    pub grass_tufts: bool,
    pub info_message: bool,
    pub visuals_key: String,
    pub config_key: String,
    pub smooth_key: String,
    pub state_key: String,
    pub render_selection_box: bool,
    pub selection_color: String,
    pub debug: bool,
    pub debug_outline_smoothables: bool,
    pub debug_densities: bool,
    pub debug_collisions: bool,
    pub debug_mesh_collisions: bool,
    pub debug_mesh_performance: bool,
    pub debug_nearby_mesh: bool,
    pub debug_skip_rendering: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            render: true,
            collisions: true,
            mesher: Mesher::SurfaceNets,
            use_defaults: true,
            whitelist: vec![],
            blacklist: vec![],
            state_overrides: Default::default(),
            better_grass_sides: false,
            more_snow: false,
            fix_plant_height: false,
            roughness: 0.5,
            old_slopes: true,
            collision_resolution: 8,
            sampled_collisions: false,
            old_collision_enhancement: 0,
            only_old_collisions: false,
            player_collisions_only: false,
            force_visuals: false,
            extend_fluids_range: 1,
            old_in_fluids: true,
            grass_tufts: false,
            info_message: true,
            visuals_key: "Ctrl+KeyV".into(),
            config_key: "Ctrl+KeyO".into(),
            smooth_key: "KeyN".into(),
            state_key: "Shift+KeyN".into(),
            render_selection_box: true,
            selection_color: "#0006".into(),
            debug: false,
            debug_outline_smoothables: false,
            debug_densities: false,
            debug_collisions: false,
            debug_mesh_collisions: false,
            debug_mesh_performance: false,
            debug_nearby_mesh: false,
            debug_skip_rendering: false,
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !self.roughness.is_finite() || !(0.0..=1.0).contains(&self.roughness) {
            return Err("roughness must be in 0..=1.0".into());
        }
        if !(2..=16).contains(&self.collision_resolution) {
            return Err("collision_resolution must be in 2..=16".into());
        }
        if self.old_collision_enhancement > 4 || self.extend_fluids_range > 2 {
            return Err("collision enhancement must be 0..=4 and fluid range 0..=2".into());
        }
        parse_color(&self.selection_color)?;
        Ok(())
    }
    pub fn smoothable(
        &self,
        name: &str,
        properties: &std::collections::BTreeMap<String, String>,
    ) -> bool {
        self.smoothable_with(name, |k| properties.get(k).map(String::as_str))
    }
    pub fn smoothable_with<'a>(
        &self,
        name: &str,
        property: impl Fn(&str) -> Option<&'a str>,
    ) -> bool {
        let matches = |entry: &str| {
            let (block, props) = entry.split_once('[').unwrap_or((entry, ""));
            let block = if block.contains(':') {
                block.to_owned()
            } else {
                format!("minecraft:{block}")
            };
            block == name
                && (props.is_empty()
                    || props.strip_suffix(']').is_some_and(|p| {
                        p.split(',').all(|kv| {
                            kv.split_once('=')
                                .is_some_and(|(k, v)| property(k).is_some_and(|a| a == v))
                        })
                    }))
        };
        if let Some((_, value)) = self
            .state_overrides
            .iter()
            .filter(|(key, _)| matches(key))
            .max_by_key(|(key, _)| key.len())
        {
            return *value;
        }
        !self.blacklist.iter().any(|entry| matches(entry))
            && (self.whitelist.iter().any(|entry| matches(entry))
                || self.use_defaults
                    && (defaults::DEFAULT_SMOOTHABLES.contains(&name)
                        || defaults::MODDED.iter().any(|entry| matches(entry))))
    }
}
mod settings;
pub use settings::*;
mod collision;
pub use collision::{collision_boxes, collision_boxes_at, face_collision_boxes};
mod color;
pub use color::parse_color;
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub state: u32,
    pub density: f32,
    pub smooth: bool,
    pub solid: bool,
}
impl Sample {
    pub const AIR: Self = Self {
        state: 0,
        density: -1.0,
        smooth: false,
        solid: false,
    };
}
#[derive(Clone, Debug)]
pub struct Face {
    pub vertices: [Point; 4],
    pub block: Pos,
    pub state: u32,
}
impl Face {
    pub fn normal(&self) -> Point {
        let n = self.area_normal();
        let l = n.iter().map(|v| v * v).sum::<f32>().sqrt();
        if l > 1e-8 { mul(n, 1. / l) } else { [0.; 3] }
    }
    pub fn area_normal(&self) -> Point {
        let mut n = [0.; 3];
        for i in 0..4 {
            let a = sub(self.vertices[(i + 1) % 4], self.vertices[i]);
            let b = sub(self.vertices[(i + 3) % 4], self.vertices[i]);
            n = add(
                n,
                [
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                ],
            );
        }
        mul(n, 0.25)
    }
    pub fn triangles(&self) -> [[Point; 3]; 2] {
        let [a, b, c, d] = self.vertices;
        [[a, b, c], [a, c, d]]
    }
}
pub fn add(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: Point, b: Point) -> Point {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn mul(a: Point, b: f32) -> Point {
    a.map(|x| x * b)
}
pub fn normal(a: Point, b: Point, c: Point) -> Point {
    let u = sub(b, a);
    let v = sub(c, a);
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let l = n.iter().map(|v| v * v).sum::<f32>().sqrt();
    if l > 1e-8 { mul(n, 1.0 / l) } else { [0.; 3] }
}
fn offset(p: Pos, a: usize, n: i32) -> Pos {
    let mut p = p;
    p[a] += n;
    p
}
fn corner(p: Pos, c: usize) -> Pos {
    std::array::from_fn(|a| p[a] + ((c >> a) & 1) as i32)
}
fn field(sample: &impl Fn(Pos) -> Sample, p: Pos, smooth2: bool) -> f32 {
    if smooth2 {
        (0..8).map(|c| -sample(corner(p, c)).density).sum::<f32>() / 8.0
    } else {
        -sample(p).density
    }
}
fn cell_vertex(sample: &impl Fn(Pos) -> Sample, p: Pos, smooth2: bool) -> Option<Point> {
    let values: [f32; 8] = std::array::from_fn(|c| field(sample, corner(p, c), smooth2));
    if values.iter().all(|v| *v < 0.) || values.iter().all(|v| *v >= 0.) {
        return None;
    }
    let mut sum = [0.; 3];
    let mut count = 0;
    for c in 0..8 {
        for a in 0..3 {
            if c & (1 << a) != 0 {
                continue;
            }
            let d = c | (1 << a);
            if (values[c] < 0.) == (values[d] < 0.) {
                continue;
            }
            let t = values[c] / (values[c] - values[d]);
            let q = std::array::from_fn(|i| {
                p[i] as f32
                    + ((c >> i) & 1) as f32
                    + if i == a { t } else { 0. }
                    + if smooth2 { 1. } else { 0.5 }
            });
            sum = add(sum, q);
            count += 1;
        }
    }
    Some(mul(sum, 1. / count as f32))
}
/// Each crossing edge belongs to the negative-density block: deterministic across section seams.
pub fn generate(sample: &impl Fn(Pos) -> Sample, min: Pos, max: Pos, cfg: &Config) -> Vec<Face> {
    generate_at(sample, min, max, cfg, [0; 3])
}
/// The world origin ensures legacy roughness stays identical across section seams.
pub fn generate_at(
    sample: &impl Fn(Pos) -> Sample,
    min: Pos,
    max: Pos,
    cfg: &Config,
    origin: Pos,
) -> Vec<Face> {
    let cache = std::cell::RefCell::new(HashMap::<Pos, Sample>::new());
    let cached = |p| *cache.borrow_mut().entry(p).or_insert_with(|| sample(p));
    let sample = &cached;
    match cfg.mesher {
        Mesher::SurfaceNets | Mesher::SurfaceNets2x => {
            surface_nets(sample, min, max, cfg.mesher == Mesher::SurfaceNets2x)
        }
        Mesher::MarchingCubes | Mesher::MarchingCubes2x => {
            marching::generate(sample, min, max, cfg.mesher == Mesher::MarchingCubes2x)
        }
        Mesher::CullingChamfer => alternate::chamfer(sample, min, max),
        _ => cubic(sample, min, max, cfg, origin),
    }
}
fn surface_nets(sample: &impl Fn(Pos) -> Sample, min: Pos, max: Pos, smooth2: bool) -> Vec<Face> {
    let mut out = vec![];
    let mut cache = HashMap::<Pos, Option<Point>>::new();
    // Include one positive edge endpoint, but emit only owners in [min,max).
    for z in min[2] - 1..max[2] + 1 {
        for y in min[1] - 1..max[1] + 1 {
            for x in min[0] - 1..max[0] + 1 {
                let p = [x, y, z];
                for a in 0..3 {
                    let q = offset(p, a, 1);
                    let d0 = field(sample, p, smooth2);
                    let d1 = field(sample, q, smooth2);
                    if (d0 < 0.) == (d1 < 0.) {
                        continue;
                    }
                    let owner = if d0 < 0. { p } else { q };
                    if (0..3).any(|i| owner[i] < min[i] || owner[i] >= max[i]) {
                        continue;
                    }
                    let material = if smooth2 {
                        (0..8)
                            .map(|c| sample(corner(owner, c)))
                            .find(|s| s.smooth)
                            .unwrap_or(Sample::AIR)
                    } else {
                        sample(owner)
                    };
                    if !material.smooth {
                        continue;
                    }
                    let b = (a + 1) % 3;
                    let c = (a + 2) % 3;
                    let cells = [
                        p,
                        offset(p, b, -1),
                        offset(offset(p, b, -1), c, -1),
                        offset(p, c, -1),
                    ];
                    let mut verts = [[0.; 3]; 4];
                    let mut valid = true;
                    for (i, cell) in cells.into_iter().enumerate() {
                        match *cache
                            .entry(cell)
                            .or_insert_with(|| cell_vertex(sample, cell, smooth2))
                        {
                            Some(v) => verts[i] = v,
                            None => {
                                valid = false;
                                break;
                            }
                        }
                    }
                    if valid {
                        if d0 >= 0. {
                            verts.reverse();
                        }
                        out.push(Face {
                            vertices: verts,
                            block: owner,
                            state: material.state,
                        });
                    }
                }
            }
        }
    }
    out
}
const DIRS: [Pos; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];
const CUBE: [[usize; 4]; 6] = [
    [0, 1, 5, 4],
    [2, 6, 7, 3],
    [0, 2, 3, 1],
    [4, 5, 7, 6],
    [0, 4, 6, 2],
    [1, 3, 7, 5],
];
fn cubic(
    sample: &impl Fn(Pos) -> Sample,
    min: Pos,
    max: Pos,
    cfg: &Config,
    origin: Pos,
) -> Vec<Face> {
    let mut out = vec![];
    for z in min[2]..max[2] {
        for y in min[1]..max[1] {
            for x in min[0]..max[0] {
                let p = [x, y, z];
                let s = sample(p);
                if !s.smooth
                    || (matches!(cfg.mesher, Mesher::OldNoCubes | Mesher::Wulferis)
                        && s.density != 1.)
                {
                    continue;
                }
                if cfg.mesher == Mesher::OldNoCubes
                    && DIRS
                        .iter()
                        .map(|d| sample(std::array::from_fn(|i| p[i] + d[i])).density)
                        .sum::<f32>()
                        == -6.
                {
                    continue;
                }
                for (d, n) in DIRS.iter().enumerate() {
                    let neighbor = std::array::from_fn(|i| p[i] + n[i]);
                    if cfg.mesher != Mesher::StupidCubic
                        && if cfg.mesher == Mesher::OldNoCubes {
                            sample(neighbor).density == 1.
                        } else {
                            sample(neighbor).smooth
                        }
                    {
                        continue;
                    }
                    let mut vertices = CUBE[d].map(|c| corner(p, c).map(|v| v as f32));
                    for v in &mut vertices {
                        if cfg.mesher == Mesher::Wulferis {
                            *v = alternate::wulferis(sample, *v);
                        }
                        if cfg.mesher == Mesher::OldNoCubes {
                            *v = alternate::old(sample, *v, p, cfg, origin);
                        }
                    }
                    out.push(Face {
                        vertices,
                        block: p,
                        state: s.state,
                    });
                }
            }
        }
    }
    out
}
mod alternate;
mod marching;
/// Ray/triangle intersection, returns distance along a normalized ray.
pub fn ray_triangle(origin: Point, dir: Point, tri: [Point; 3]) -> Option<f32> {
    let e1 = sub(tri[1], tri[0]);
    let e2 = sub(tri[2], tri[0]);
    let cross = |a: Point, b: Point| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let dot = |a: Point, b: Point| a.iter().zip(b).map(|(a, b)| a * b).sum::<f32>();
    let p = cross(dir, e2);
    let det = dot(e1, p);
    if det.abs() < 1e-7 {
        return None;
    }
    let t = sub(origin, tri[0]);
    let u = dot(t, p) / det;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(t, e1);
    let v = dot(dir, q) / det;
    if v < 0. || u + v > 1. {
        return None;
    }
    let distance = dot(e2, q) / det;
    (distance >= 0.).then_some(distance)
}
/// Voxelize the same closed surface for physics. The full interior uses the signed field;
/// cells cut by triangles are conservatively included at configurable sub-block resolution.
pub fn sampled_collision_boxes(
    sample: &impl Fn(Pos) -> Sample,
    p: Pos,
    cfg: &Config,
) -> Vec<[f64; 6]> {
    let local = |q: Pos| sample(std::array::from_fn(|i| p[i] + q[i]));
    sampled_collision_boxes_local(&local, cfg, p)
}
fn sampled_collision_boxes_local(
    sample: &impl Fn(Pos) -> Sample,
    cfg: &Config,
    world_origin: Pos,
) -> Vec<[f64; 6]> {
    let p = [0; 3];
    let n = cfg.collision_resolution;
    let mut filled = vec![false; n * n * n];
    let faces = generate_at(
        sample,
        p.map(|x| x - 1),
        p.map(|x| x + 2),
        cfg,
        world_origin,
    );
    if faces.is_empty() {
        return if sample(p).density > 0. {
            vec![[0., 0., 0., 1., 1., 1.]]
        } else {
            vec![]
        };
    }
    for z in 0..n {
        for x in 0..n {
            let origin = [
                p[0] as f32 + (x as f32 + 0.5) / n as f32,
                p[1] as f32 - 2.,
                p[2] as f32 + (z as f32 + 0.5) / n as f32,
            ];
            let mut hits: Vec<(f32, f32)> = faces
                .iter()
                .flat_map(|f| f.triangles())
                .filter_map(|t| {
                    ray_triangle(origin, [0., 1., 0.], t)
                        .map(|d| (origin[1] + d, normal(t[0], t[1], t[2])[1]))
                })
                .collect();
            hits.sort_by(|a, b| a.0.total_cmp(&b.0));
            for y in 0..n {
                let height = p[1] as f32 + (y as f32 + 0.5) / n as f32;
                let above = hits.iter().find(|h| h.0 >= height);
                let below = hits.iter().rev().find(|h| h.0 < height);
                let inside = above
                    .map(|h| h.1 > 0.)
                    .or_else(|| below.map(|h| h.1 < 0.))
                    .unwrap_or_else(|| {
                        let origin = [origin[0], height, origin[2]];
                        [0, 2]
                            .into_iter()
                            .filter_map(|axis| {
                                let mut dir = [0.; 3];
                                dir[axis] = 1.;
                                faces
                                    .iter()
                                    .flat_map(|f| f.triangles())
                                    .filter_map(|t| {
                                        ray_triangle(origin, dir, t)
                                            .map(|d| (d, normal(t[0], t[1], t[2])[axis]))
                                    })
                                    .min_by(|a, b| a.0.total_cmp(&b.0))
                            })
                            .min_by(|a, b| a.0.total_cmp(&b.0))
                            .map_or(sample(p).density > 0., |h| h.1 > 0.)
                    });
                filled[(z * n + x) * n + y] = inside;
            }
        }
    }
    if filled.iter().all(|v| *v) {
        return vec![[0., 0., 0., 1., 1., 1.]];
    }
    let mut boxes = vec![];
    for z in 0..n {
        for x in 0..n {
            let mut y = 0;
            while y < n {
                if !filled[(z * n + x) * n + y] {
                    y += 1;
                    continue;
                }
                let start = y;
                while y < n && filled[(z * n + x) * n + y] {
                    y += 1;
                }
                boxes.push([
                    x as f64 / n as f64,
                    start as f64 / n as f64,
                    z as f64 / n as f64,
                    (x + 1) as f64 / n as f64,
                    y as f64 / n as f64,
                    (z + 1) as f64 / n as f64,
                ]);
            }
        }
    }
    boxes
}

pub mod fluids;
mod profiler;
pub mod ui;
pub use profiler::{block_mesh_profile, mesh_profile, profile_block, record_mesh};
/// NoCubes RenderableState.posForFace: half a block inward from the face centre.
pub fn material_position(face: &Face, mesher: Mesher) -> Pos {
    if mesher == Mesher::OldNoCubes {
        return face.block;
    }
    let centre = mul(face.vertices.into_iter().fold([0.; 3], add), 0.25);
    let mut n = [0.; 3];
    for i in 0..4 {
        let a = sub(face.vertices[(i + 1) % 4], face.vertices[i]);
        let b = sub(face.vertices[(i + 3) % 4], face.vertices[i]);
        n = add(
            n,
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ],
        );
    }
    let length = n.iter().map(|v| v * v).sum::<f32>().sqrt();
    let p = if length > 1e-8 {
        sub(centre, mul(n, 0.5 / length))
    } else {
        centre
    };
    p.map(|v| v.floor() as i32)
}
pub const MATERIAL_NEIGHBORS: [Pos; 26] = [
    [0, -1, 0],
    [0, 1, 0],
    [-1, 0, 0],
    [1, 0, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, -1, 0],
    [-1, 0, -1],
    [-1, 0, 1],
    [-1, 1, 0],
    [0, -1, -1],
    [0, -1, 1],
    [0, 1, -1],
    [0, 1, 1],
    [1, -1, 0],
    [1, 0, -1],
    [1, 0, 1],
    [1, 1, 0],
    [1, 1, 1],
    [1, 1, -1],
    [-1, 1, 1],
    [-1, 1, -1],
    [1, -1, 1],
    [1, -1, -1],
    [-1, -1, 1],
    [-1, -1, -1],
];
pub mod kinds;
/// Java Math.round and NoCubes LightCache's normal-biased light lookup.
pub fn vertex_light_position(point: Point, normal: Point) -> Pos {
    std::array::from_fn(|i| {
        (-0.5 + point[i] + (normal[i] * 4.).clamp(-1., 1.) + 0.5).floor() as i32
    })
}
pub fn vertex_light(point: Point, normal: Point, raw: &impl Fn(Pos) -> u32) -> u32 {
    let p = vertex_light_position(point, normal);
    for d in [
        [0, 0, 0],
        [0, -1, 0],
        [0, 1, 0],
        [-1, 0, 0],
        [1, 0, 0],
        [0, 0, -1],
        [0, 0, 1],
    ] {
        let value = raw(std::array::from_fn(|i| p[i] + d[i]));
        if value != 0 {
            return value;
        }
    }
    0
}

/// ModUtil.mapSnowHeight, mapped to the signed-density convention used by every mesher.
pub fn snow_density(layers: u8) -> f32 {
    -1. + f32::from(layers.clamp(1, 8) - 1) * 0.25
}
#[cfg(test)]
mod tests {
    use super::*;
    fn ground(p: Pos) -> Sample {
        if p[1] <= 0 {
            Sample {
                state: 1,
                density: 1.,
                smooth: true,
                solid: true,
            }
        } else {
            Sample::AIR
        }
    }
    #[test]
    fn flat_surface_is_at_block_top_and_points_up() {
        let faces = generate(&ground, [0; 3], [4; 3], &Config::default());
        assert_eq!(faces.len(), 16);
        for face in faces {
            assert!(face.normal()[1] > 0.99, "{:?}", face.normal());
            for p in face.vertices {
                assert!((p[1] - 1.).abs() < 1e-6);
            }
        }
    }
    #[test]
    fn empty_and_solid_have_no_surface() {
        for sample in [
            Sample::AIR,
            Sample {
                state: 1,
                density: 1.,
                smooth: true,
                solid: true,
            },
        ] {
            assert!(generate(&|_| sample, [0; 3], [3; 3], &Config::default()).is_empty());
        }
    }
    #[test]
    fn adjacent_sections_match_monolithic_surface() {
        let sample = |p: Pos| {
            if p[1] <= p[0].div_euclid(4) {
                Sample {
                    state: 1,
                    density: 1.,
                    smooth: true,
                    solid: true,
                }
            } else {
                Sample::AIR
            }
        };
        let cfg = Config::default();
        let mut split = generate(&sample, [-16, -4, -4], [0, 8, 4], &cfg);
        split.extend(generate(&sample, [0, -4, -4], [16, 8, 4], &cfg));
        let key = |faces: Vec<Face>| {
            let mut v: Vec<String> = faces
                .into_iter()
                .map(|f| format!("{:?}", f.vertices))
                .collect();
            v.sort();
            v
        };
        assert_eq!(
            key(split),
            key(generate(&sample, [-16, -4, -4], [16, 8, 4], &cfg))
        );
    }
    #[test]
    fn isolated_block_is_closed_and_preserved() {
        let faces = generate(
            &|p| if p == [0; 3] { ground(p) } else { Sample::AIR },
            [-2; 3],
            [3; 3],
            &Config::default(),
        );
        assert_eq!(faces.len(), 6);
        let mut edges = HashMap::new();
        for f in faces {
            for i in 0..4 {
                let a = f.vertices[i].map(f32::to_bits);
                let b = f.vertices[(i + 1) % 4].map(f32::to_bits);
                let e = if a < b { (a, b) } else { (b, a) };
                *edges.entry(e).or_insert(0) += 1;
            }
        }
        assert!(edges.values().all(|n| *n == 2));
    }
    #[test]
    fn marching_flat_surface_points_up() {
        let cfg = Config {
            mesher: Mesher::MarchingCubes,
            ..Config::default()
        };
        let faces = generate(&ground, [0; 3], [3; 3], &cfg);
        assert!(!faces.is_empty());
        for f in faces {
            assert!(f.normal()[1] > 0.99);
        }
    }
    #[test]
    fn whitelist_states_and_blacklist_precedence() {
        let mut cfg = Config::default();
        assert!(cfg.smoothable("minecraft:stone", &Default::default()));
        assert!(!cfg.smoothable("minecraft:stone_bricks", &Default::default()));
        cfg.whitelist.push("oak_stairs[facing=north]".into());
        let props = std::collections::BTreeMap::from([("facing".into(), "north".into())]);
        assert!(cfg.smoothable("minecraft:oak_stairs", &props));
        cfg.blacklist.push("oak_stairs".into());
        assert!(!cfg.smoothable("minecraft:oak_stairs", &props));
    }
    #[test]
    fn flat_collision_is_full_and_air_empty() {
        let cfg = Config {
            collision_resolution: 4,
            ..Config::default()
        };
        let boxes = sampled_collision_boxes(&ground, [0; 3], &cfg);
        let volume: f64 = boxes
            .iter()
            .map(|b| (b[3] - b[0]) * (b[4] - b[1]) * (b[5] - b[2]))
            .sum();
        assert_eq!(volume, 1.);
        assert!(sampled_collision_boxes(&ground, [0, 2, 0], &cfg).is_empty());
    }
    #[test]
    fn ray_hits_and_misses() {
        let tri = [[0., 0., 0.], [1., 0., 0.], [0., 0., 1.]];
        assert_eq!(ray_triangle([0.2, 2., 0.2], [0., -1., 0.], tri), Some(2.));
        assert_eq!(ray_triangle([2., 2., 2.], [0., -1., 0.], tri), None);
    }
    #[test]
    fn every_mesher_generates_finite_geometry() {
        let sample = |p: Pos| {
            if p.iter().all(|v| (-1..=1).contains(v)) {
                ground([0, 0, 0])
            } else {
                Sample::AIR
            }
        };
        for mesher in [
            Mesher::SurfaceNets,
            Mesher::SurfaceNets2x,
            Mesher::MarchingCubes,
            Mesher::MarchingCubes2x,
            Mesher::CullingCubic,
            Mesher::StupidCubic,
            Mesher::OldNoCubes,
            Mesher::CullingChamfer,
            Mesher::Wulferis,
        ] {
            let cfg = Config {
                mesher,
                ..Config::default()
            };
            let faces = generate(&sample, [-3; 3], [4; 3], &cfg);
            assert!(!faces.is_empty(), "{mesher:?}");
            assert!(
                faces
                    .iter()
                    .flat_map(|f| f.vertices)
                    .flatten()
                    .all(f32::is_finite)
            );
        }
    }
    #[test]
    fn cubic_faces_point_outward() {
        let sample = |p: Pos| if p == [0; 3] { ground(p) } else { Sample::AIR };
        let cfg = Config {
            mesher: Mesher::CullingCubic,
            ..Config::default()
        };
        for f in generate(&sample, [-2; 3], [3; 3], &cfg) {
            let n = f.normal();
            let c = mul(f.vertices.into_iter().fold([0.; 3], add), 0.25);
            assert!(
                n.into_iter()
                    .zip(sub(c, [0.5; 3]))
                    .map(|(a, b)| a * b)
                    .sum::<f32>()
                    > 0.
            );
        }
    }
    #[test]
    fn invalid_config_is_rejected() {
        assert!(serde_json::from_str::<Config>(r#"{"mseher":"SurfaceNets"}"#).is_err());
        assert!(
            Config {
                collision_resolution: 0,
                ..Config::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            Config {
                roughness: f32::NAN,
                ..Config::default()
            }
            .validate()
            .is_err()
        );
    }
    #[test]
    fn collision_rotation_matches_vertical_wall() {
        let wall = |p: Pos| ground([p[1], p[0], p[2]]);
        let cfg = Config {
            collision_resolution: 4,
            ..Config::default()
        };
        let boxes = sampled_collision_boxes(&wall, [0; 3], &cfg);
        let volume: f64 = boxes
            .iter()
            .map(|b| (b[3] - b[0]) * (b[4] - b[1]) * (b[5] - b[2]))
            .sum();
        assert_eq!(volume, 1.);
        assert!(sampled_collision_boxes(&wall, [2, 0, 0], &cfg).is_empty());
    }
    #[test]
    fn server_configuration_is_bounded_validated_and_reset() {
        clear_server_config();
        let server = Config {
            mesher: Mesher::MarchingCubes,
            render: false,
            collision_resolution: 4,
            ..Config::default()
        };
        apply_server_config(&serde_json::to_vec(&server).unwrap()).unwrap();
        assert_eq!(config().mesher, Mesher::MarchingCubes);
        assert_eq!(config().collision_resolution, 4);
        assert_eq!(config().render, local_config().render);
        assert!(apply_server_config(&vec![0; 30001]).is_err());
        assert!(apply_server_config(br#"{"collision_resolution":0}"#).is_err());
        clear_server_config();
        assert_eq!(config().mesher, local_config().mesher);
    }
}
