//! Repeatable CPU workload, independent of the GPU and game streaming.
use nocubes_engine::{Config, Mesher, Sample, collision_boxes, generate};
use std::{hint::black_box, time::Instant};
fn main() {
    let sample = |p: [i32; 3]| {
        if p[1] <= 7 + (p[0].div_euclid(4) + p[2].div_euclid(4)).rem_euclid(3) {
            Sample {
                state: 1,
                smooth: true,
                density: 1.,
                solid: true,
            }
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
            ..Default::default()
        };
        let start = Instant::now();
        let mut faces = 0;
        for _ in 0..20 {
            faces += black_box(generate(black_box(&sample), [0; 3], [16; 3], &cfg)).len();
        }
        let mesh_ms = start.elapsed().as_secs_f64() * 1000. / 20.;
        let start = Instant::now();
        let mut boxes = 0;
        for _ in 0..20 {
            for x in 0..16 {
                boxes += black_box(collision_boxes(black_box(&sample), [x, 8, 8], &cfg)).len();
            }
        }
        println!(
            "{mesher:?}: mesh_ms={mesh_ms:.4} collision_us={:.4} faces={} boxes={}",
            start.elapsed().as_secs_f64() * 1e6 / 320.,
            faces / 20,
            boxes / 20
        );
    }
}
