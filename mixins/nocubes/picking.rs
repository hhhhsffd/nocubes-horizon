use super::*;
impl Game<'_> {
    fn nocubes_vanilla_ray(&self, origin: [f64; 3], dir: [f64; 3], reach: f64) -> Option<Hit> {
        let mut pos = origin.map(|v| v.floor() as i32);
        let step = dir.map(|d| if d > 0.0 { 1 } else { -1 });
        let delta = dir.map(|d| {
            if d == 0.0 {
                f64::INFINITY
            } else {
                (1.0 / d).abs()
            }
        });
        let mut t_max = [0.0; 3];
        for i in 0..3 {
            let boundary = if dir[i] > 0.0 {
                f64::from(pos[i]) + 1.0
            } else {
                f64::from(pos[i])
            };
            t_max[i] = if dir[i] == 0.0 {
                f64::INFINITY
            } else {
                (boundary - origin[i]) / dir[i]
            };
        }
        let mut face = [0; 3];
        let mut t = 0.0;
        while t <= reach {
            let state = self.world.block(pos[0], pos[1], pos[2]);
            let f = self.res.facts().get(state);
            if !self.pack.assets.registry.state(state).is_some_and(|s| {
                horizon_core::nocubes::config().render
                    && horizon_core::nocubes::config()
                        .smoothable_with(&s.name, |k| s.properties.get(k).map(String::as_str))
            }) && !self.res.facts().is_air(state)
                && (f.fluid == Fluid::None
                    || f.waterlogged
                    || &*f.name == "minecraft:kelp"
                    || f.render_shape == horizon_client::blocks::RenderShape::Model)
                && !matches!(
                    &*f.name,
                    "minecraft:water" | "minecraft:lava" | "minecraft:bubble_column"
                )
            {
                // The nearest entry into one of the outline's boxes (slab method).
                let mut best: Option<(f64, [i32; 3])> = None;
                for b in self.outline_boxes(pos) {
                    let (mut t0, mut t1, mut axis_in, mut sign) = (0.0f64, f64::INFINITY, None, 0);
                    let mut miss = false;
                    for a in 0..3 {
                        if dir[a] == 0.0 {
                            miss |= origin[a] < b[a] || origin[a] > b[a + 3];
                            continue;
                        }
                        let (lo, hi) =
                            ((b[a] - origin[a]) / dir[a], (b[a + 3] - origin[a]) / dir[a]);
                        let (near, far) = if lo < hi { (lo, hi) } else { (hi, lo) };
                        if near > t0 {
                            t0 = near;
                            axis_in = Some(a);
                            sign = if dir[a] > 0.0 { -1 } else { 1 };
                        }
                        t1 = t1.min(far);
                    }
                    if miss || t0 > t1 || t0 > reach {
                        continue;
                    }
                    let mut normal = face;
                    if let Some(a) = axis_in {
                        normal = [0; 3];
                        normal[a] = sign;
                    }
                    if best.is_none_or(|(bt, _)| t0 < bt) {
                        best = Some((t0, normal));
                    }
                }
                if let Some((t, mut face)) = best {
                    // `BlockGetter.clipWithInteractionOverride`: the hit stays at the
                    // outline's point, with the face of the interaction shape when
                    // the ray enters that first.
                    let name = &*f.name;
                    let facing = self
                        .pack
                        .assets
                        .registry
                        .state(state)
                        .and_then(|s| s.properties.get("facing"))
                        .map(String::as_str);
                    let inside = interaction_shapes::boxes(name, facing);
                    if !inside.is_empty() {
                        let at = pos.map(f64::from);
                        let world: Vec<[f64; 6]> = inside
                            .iter()
                            .map(|b| std::array::from_fn(|i| at[i % 3] + b[i]))
                            .collect();
                        if let Some((ti, normal)) =
                            interaction_shapes::entry(origin, dir, &world, reach, face)
                            && ti < t
                        {
                            face = normal;
                        }
                    }
                    return Some(Hit {
                        pos,
                        face,
                        point: [
                            origin[0] + dir[0] * t,
                            origin[1] + dir[1] * t,
                            origin[2] + dir[2] * t,
                        ],
                    });
                }
            }
            let axis = if t_max[0] < t_max[1] {
                if t_max[0] < t_max[2] { 0 } else { 2 }
            } else if t_max[1] < t_max[2] {
                1
            } else {
                2
            };
            t = t_max[axis];
            t_max[axis] += delta[axis];
            pos[axis] += step[axis];
            face = [0; 3];
            face[axis] = -step[axis];
        }
        None
    }
    pub(super) fn nocubes_ray(&self, origin: [f64; 3], dir: [f64; 3], reach: f64) -> Option<Hit> {
        use horizon_core::nocubes;
        let mut best = self.nocubes_vanilla_ray(origin, dir, reach);
        let cfg = nocubes::config();
        if !cfg.render || !reach.is_finite() || reach > 128. {
            return best;
        }
        let base = origin.map(|v| v.floor() as i32);
        let local = std::array::from_fn(|i| (origin[i] - f64::from(base[i])) as f32);
        let sample = |p: [i32; 3]| {
            let p = std::array::from_fn::<_, 3, _>(|i| p[i] + base[i]);
            let id = self.world.block(p[0], p[1], p[2]);
            nocubes::kinds::sample(&cfg, id.0)
        };
        let end = std::array::from_fn::<_, 3, _>(|i| local[i] + dir[i] as f32 * reach as f32);
        let lo = std::array::from_fn(|i| local[i].min(end[i]).floor() as i32 - 1);
        let hi = std::array::from_fn(|i| local[i].max(end[i]).ceil() as i32 + 1);
        let mut distance = best.map_or(reach, |h| {
            h.point
                .iter()
                .zip(origin)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt()
        });
        for surface in nocubes::generate_at(&sample, lo, hi, &cfg, base) {
            for triangle in surface.triangles() {
                if let Some(t) = nocubes::ray_triangle(local, dir.map(|v| v as f32), triangle)
                    && f64::from(t) < distance
                {
                    distance = f64::from(t);
                    let n = surface.normal();
                    let axis = (0..3)
                        .max_by(|a, b| n[*a].abs().total_cmp(&n[*b].abs()))
                        .unwrap();
                    let mut face = [0; 3];
                    face[axis] = if n[axis] > 0. { 1 } else { -1 };
                    best = Some(Hit {
                        pos: {
                            let candidate = nocubes::material_position(&surface, cfg.mesher);
                            let p = if sample(candidate).smooth {
                                candidate
                            } else {
                                surface.block
                            };
                            std::array::from_fn(|i| p[i] + base[i])
                        },
                        face,
                        point: std::array::from_fn(|i| origin[i] + dir[i] * distance),
                    });
                }
            }
        }
        best
    }
}
