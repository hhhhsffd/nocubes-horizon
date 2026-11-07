use super::*;
use horizon_client::world_primitives::WorldPrimitives;
use horizon_core::nocubes;
impl Game<'_> {
    pub(super) fn nocubes_overlay(&self, out: &mut WorldPrimitives) {
        let cfg = nocubes::config();
        if !cfg.render {
            return;
        }
        let hit = self.pick();
        let debug = cfg.debug;
        if hit.is_none() && !debug {
            return;
        }
        let base = hit
            .map(|h| h.pos)
            .or_else(|| {
                self.native_player()
                    .map(|p| [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32])
            })
            .unwrap_or([0; 3]);
        let samples = horizon_core::nocubes::kinds::samples(&cfg);
        let sample = |q: [i32; 3]| {
            let p = std::array::from_fn::<_, 3, _>(|i| base[i] + q[i]);
            let id = self.world.block(p[0], p[1], p[2]);
            samples.get(id.0)
        };
        let world = |p: [f32; 3]| std::array::from_fn(|i| f64::from(base[i]) + f64::from(p[i]));
        let selection = hit.is_some_and(|h| h.pos == base)
            && cfg.render_selection_box
            && !self.ui.hide_gui
            && (self.may_build()
                || self.mode == horizon_client::command::args::GameMode::Adventure)
            && sample([0; 3]).smooth;
        if selection || debug && cfg.debug_nearby_mesh {
            let radius = if debug && cfg.debug_nearby_mesh { 2 } else { 0 };
            let mut profile = if radius == 0 {
                nocubes::profile_block()
            } else {
                None
            };
            let faces =
                self.nocubes_query_mesh(base, [-radius; 3], [radius + 1; 3], &cfg, &samples);
            if let Some(p) = &mut profile {
                p.set_faces(faces.len());
            }
            let [r, g, b, a] = nocubes::parse_color(&cfg.selection_color).unwrap_or([0, 0, 0, 102]);
            let color = u32::from_be_bytes([a, r, g, b]);
            for face in faces.iter() {
                if selection && face.block != [0; 3] && radius == 0 {
                    continue;
                }
                for i in 0..4 {
                    out.line(
                        world(face.vertices[i]),
                        world(face.vertices[(i + 1) % 4]),
                        if radius > 0 { 0x99ff8800 } else { color },
                        2.,
                    );
                }
            }
        }
        if !debug {
            return;
        }
        for z in -2..=2 {
            for y in -2..=2 {
                for x in -2..=2 {
                    let p = [x, y, z];
                    let s = sample(p);
                    let origin = world(p.map(|v| v as f32));
                    if cfg.debug_outline_smoothables && s.smooth {
                        out.box_outline(origin, origin.map(|v| v + 1.), 0x8833ff66, 1.);
                    }
                    if cfg.debug_densities {
                        let c = std::array::from_fn(|i| origin[i] + 0.5);
                        let radius = (s.density.abs() as f64 * 0.125).max(0.02);
                        out.filled_box(
                            c.map(|v| v - radius),
                            c.map(|v| v + radius),
                            if s.density > 0. {
                                0x663366ff
                            } else {
                                0x66ff3333
                            },
                        );
                    }
                    if cfg.debug_collisions {
                        use horizon_client::collisions::{
                            SceneCollisions, nocubes_collision::RegistryExt,
                        };
                        let pos = std::array::from_fn::<_, 3, _>(|i| base[i] + p[i]);
                        let mut scene = SceneCollisions {
                            world: &self.world,
                            binding: self.collision,
                            subject: None,
                        };
                        let id = self.world.block(pos[0], pos[1], pos[2]);
                        if let Ok(shape) =
                            self.collision.nocubes_collision_shape(&mut scene, id, pos)
                        {
                            for b in shape.boxes() {
                                let lo = b.min_position();
                                let hi = b.max_position();
                                out.box_outline(
                                    [origin[0] + lo.x, origin[1] + lo.y, origin[2] + lo.z],
                                    [origin[0] + hi.x, origin[1] + hi.y, origin[2] + hi.z],
                                    0x9933ffff,
                                    1.,
                                );
                            }
                        }
                    }
                    if cfg.debug_mesh_collisions && s.smooth {
                        for b in nocubes::collision_boxes_at(&sample, p, &cfg, base) {
                            let lo = std::array::from_fn(|i| origin[i] + b[i]);
                            let hi = std::array::from_fn(|i| origin[i] + b[i + 3]);
                            out.box_outline(lo, hi, 0x99ffff00, 1.);
                        }
                    }
                }
            }
        }
        if cfg.debug_mesh_performance {
            let (n, ns, faces) = nocubes::mesh_profile();
            out.text(
                world([0., 2., 0.]),
                format!(
                    "NoCubes: {n} meshes; {} µs/mesh; {} faces/mesh",
                    ns / n.max(1) / 1000,
                    faces / n.max(1)
                ),
                0xffffffff,
                0.75,
            );
        }
    }
}
