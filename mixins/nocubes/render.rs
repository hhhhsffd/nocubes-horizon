use super::*;
const DIR_OFFSETS: [[i32; 3]; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];
impl Mesher<'_> {
    pub(super) fn nocubes_mesh(
        &mut self,
        region: &[[Option<&Chunk>; 3]; 3],
        sy: i32,
        mut out: SectionMesh,
        scratch: &mut MeshScratch,
    ) -> SectionMesh {
        let cfg = horizon_core::nocubes::config();
        let kinds = horizon_core::nocubes::kinds::snapshot();
        out.clear();
        let center = region[1][1].expect("centre chunk");
        let own = scratch.own.get_or_insert_with(|| Box::new([0; 4096]));
        scratch.padded.fill(region, sy, own);
        let pad = &scratch.padded;
        solid_rows(pad, &self.tables.solid, &mut scratch.solid_rows);
        let opaque = &scratch.solid_rows;
        out.visibility = visibility_rows(&open_rows(opaque));
        let base_y = center.min_y() + sy * 16;
        if self.leaf_culling.is_interior() {
            scratch
                .leaf
                .prepare(self.leaf_culling, pad, &self.tables.info, region, base_y);
        }
        let mut tints = Tints::new(
            region,
            self.biomes,
            self.zoom,
            self.blend_radius,
            [center.x, center.z, base_y],
        );
        let mut smooth = vec![false; self.tables.info.len()];
        let mut seen = std::collections::HashSet::new();
        for pz in -3..19 {
            for py in -3..19 {
                for px in -3..19 {
                    let rx = (px + 16_i32).div_euclid(16) as usize;
                    let rz = (pz + 16_i32).div_euclid(16) as usize;
                    let id = region[rz][rx].map_or(0, |c| c.block(px, base_y + py, pz).0);
                    if seen.insert(id) {
                        smooth[id as usize] = self
                            .assets
                            .registry
                            .state(BlockStateId(id))
                            .is_some_and(|s| {
                                cfg.render
                                    && cfg.smoothable_with(&s.name, |k| {
                                        s.properties.get(k).map(String::as_str)
                                    })
                            });
                    }
                }
            }
        }
        let info = &self.tables.info;
        let full_faces = &self.tables.full_faces;
        let catalog = &self.render.model_catalog;
        let mut models = Vec::new();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let id = pad.blocks[pi(x, y, z)];
                    if id == 0 {
                        continue;
                    }
                    let state = &info[id as usize];
                    if state.block_entity {
                        out.block_entities
                            .push(([center.x * 16 + x, base_y + y, center.z * 16 + z], id));
                    }
                    if state.emission > 0 {
                        out.lights
                            .push(([center.x * 16 + x, base_y + y, center.z * 16 + z], id));
                    }
                    if hidden(opaque, x, y, z)
                        && !DIR_OFFSETS
                            .iter()
                            .any(|d| smooth[pad.blocks[pi(x + d[0], y + d[1], z + d[2])] as usize])
                    {
                        continue;
                    }
                    let raw = |p: [i32; 3]| {
                        let rx = (p[0] + 16).div_euclid(16);
                        let rz = (p[2] + 16).div_euclid(16);
                        if !(0..3).contains(&rx) || !(0..3).contains(&rz) {
                            return 0;
                        }
                        region[rz as usize][rx as usize]
                            .map_or(0, |c| c.block(p[0], base_y + p[1], p[2]).0)
                    };
                    let extended = horizon_core::nocubes::fluids::extended(
                        [x, y, z],
                        cfg.extend_fluids_range,
                        |p| smooth[raw(p) as usize],
                        |f: FluidState| f.kind() == FluidKind::Empty,
                        |f: FluidState| f.is_source(),
                        |p| info[raw(p) as usize].fluid,
                    );
                    if extended.kind() != FluidKind::Empty {
                        let water = tints.get(Resolver::Water, x, y, z);
                        let shade = (
                            self.nether_shading,
                            self.shader_pack_lighting,
                            self.separate_ao,
                            self.shader_pack,
                        );
                        nocubes_fluid::tesselate(
                            self.tables,
                            shade,
                            pad,
                            water,
                            &mut out,
                            [x, y, z],
                            &smooth,
                            &raw,
                        );
                    }
                    if cfg.render && smooth[id as usize] {
                        continue;
                    }
                    if !state.model {
                        continue;
                    }
                    let world = BlockPos::new(center.x * 16 + x, base_y + y, center.z * 16 + z);
                    if catalog
                        .models_at(BlockStateId(id), world, &mut models)
                        .is_err()
                    {
                        continue;
                    }
                    // ModelBlockRenderer: the first part decides smooth lighting
                    // for all; light-emitting blocks are always lit flat.
                    let Some(first) = models.first() else {
                        continue;
                    };
                    let ao = self.smooth_lighting && state.emission == 0 && first.ambient_occlusion;
                    let mut origin = match offset(state, world.x, world.z) {
                        Some(o) => [
                            x as f32 + o[0] as f32,
                            y as f32 + o[1] as f32,
                            z as f32 + o[2] as f32,
                        ],
                        None => [x as f32, y as f32, z as f32],
                    };
                    if cfg.render
                        && cfg.fix_plant_height
                        && !state.solid
                        && state.fluid.kind() == FluidKind::Empty
                        && smooth[pad.blocks[pi(x, y - 1, z)] as usize]
                    {
                        let name = self
                            .assets
                            .registry
                            .state(BlockStateId(id))
                            .map(|s| s.name.as_str())
                            .unwrap_or("");
                        let plant = kinds.get(name).is_some_and(|k| k.short_plant);
                        if plant {
                            let sample = |p| self.nocubes_sample(region, sy, &smooth, p, &cfg);
                            let faces = horizon_core::nocubes::generate_at(
                                &sample,
                                [x - 1, y - 2, z - 1],
                                [x + 2, y + 1, z + 2],
                                &cfg,
                                [center.x * 16, base_y, center.z * 16],
                            );
                            let anchored = faces.iter().find(|f| {
                                let n = f.normal();
                                let owner = horizon_core::nocubes::material_position(f, cfg.mesher);
                                n[1] > n[0].abs().max(n[2].abs()) && owner == [x, y - 1, z]
                            });
                            if let Some(face) = anchored {
                                let c = horizon_core::nocubes::mul(
                                    face.vertices
                                        .into_iter()
                                        .fold([0.; 3], horizon_core::nocubes::add),
                                    0.25,
                                );
                                origin[0] += c[0] - 0.5 - x as f32;
                                origin[1] += c[1] - y as f32;
                                origin[2] += c[2] - 0.5 - z as f32;
                            }
                        }
                    }
                    let mut visible = [false; 6];
                    let interior_leaf = state.leaves
                        && self.leaf_culling.is_interior()
                        && scratch.leaf.interior(self.leaf_culling, x, y, z);
                    for d in Direction::ALL {
                        let [dx, dy, dz] = d.normal();
                        let neighbor = pad.blocks[pi(x + dx, y + dy, z + dz)];
                        // `should_render_face`'s first test, without its lookups: a
                        // full occlusion face toward this block hides the face.
                        if !smooth[neighbor as usize]
                            && full_faces[neighbor as usize] >> d.opposite() as u8 & 1 != 0
                        {
                            continue;
                        }
                        let other = &info[neighbor as usize];
                        // LeavesBlock.skipRendering with opaque leaves.
                        let opaque_leaves = !self.cutout_leaves && state.leaves && other.leaves;
                        let culled_leaf = match self.leaf_culling {
                            LeafCullingMode::Hollow | LeafCullingMode::Hidden => {
                                state.leaves
                                    && other.leaves
                                    && leaf_culling::in_section(x + dx, y + dy, z + dz)
                                    && (self.leaf_culling == LeafCullingMode::Hollow
                                        || leaf_culling::backed(
                                            pad,
                                            info,
                                            region,
                                            base_y,
                                            [x + 2 * dx, y + 2 * dy, z + 2 * dz],
                                        ))
                            }
                            _ => false,
                        };
                        visible[d as usize] = smooth[neighbor as usize]
                            || !opaque_leaves
                                && !culled_leaf
                                && self
                                    .visibility
                                    .should_render_face(
                                        BlockStateId(id),
                                        BlockStateId(neighbor),
                                        d,
                                        true,
                                    )
                                    .unwrap_or(true);
                    }
                    let connected = &self.render.connected;
                    for model in &models {
                        for face in &model.faces {
                            if face.quad.cull.is_some_and(|d| !visible[d as usize]) {
                                continue;
                            }
                            // A pane's top and bottom strips between stacked panes, with
                            // connected textures.
                            if state.pane
                                && !connected.is_empty()
                                && let Some(dy) = match face.quad.facing {
                                    ModelDirection::Up => Some(1),
                                    ModelDirection::Down => Some(-1),
                                    _ => None,
                                }
                            {
                                let p = &face.quad.positions;
                                // Only on the block's top or bottom plane: a pack's own pane
                                // model may have ledges between.
                                let plane = if dy > 0 { 1.0 } else { 0.0 };
                                let on_edge = p.iter().all(|v| (v[1] - plane).abs() < 1e-4);
                                let centre =
                                    [0, 2].map(|k| p.iter().map(|v| v[k]).sum::<f32>() / 4.0);
                                let other = pad.blocks[pi(x, y + dy, z)];
                                if on_edge && connected.stacked_pane_hides(id, other, centre) {
                                    continue;
                                }
                            }
                            self.face(
                                pad,
                                &mut tints,
                                &mut out,
                                [x, y, z],
                                origin,
                                face,
                                ao,
                                (id, state),
                                world,
                                interior_leaf,
                            );
                        }
                    }
                }
            }
        }
        if cfg.render && !(cfg.debug && cfg.debug_skip_rendering) {
            self.nocubes_surfaces(region, sy, pad, &smooth, &mut tints, &mut out);
        }
        out.visibility = ALL_VISIBLE;
        out.finish_sprite_layers();
        out
    }

    fn nocubes_sample(
        &self,
        region: &[[Option<&Chunk>; 3]; 3],
        sy: i32,
        smooth: &[bool],
        p: [i32; 3],
        cfg: &horizon_core::nocubes::Config,
    ) -> horizon_core::nocubes::Sample {
        use horizon_core::nocubes::Sample;
        let base_y = region[1][1].expect("centre chunk").min_y() + sy * 16;

        let rx = (p[0] + 16).div_euclid(16);
        let rz = (p[2] + 16).div_euclid(16);
        if !(0..3).contains(&rx) || !(0..3).contains(&rz) {
            return Sample::AIR;
        }
        let id =
            region[rz as usize][rx as usize].map_or(0, |c| c.block(p[0], base_y + p[1], p[2]).0);
        let smooth = smooth[id as usize];
        let mut sample = horizon_core::nocubes::kinds::sample(cfg, id);
        sample.smooth = smooth;
        if !smooth {
            sample.density = -1.;
        }
        sample
    }

    fn nocubes_surfaces(
        &self,
        region: &[[Option<&Chunk>; 3]; 3],
        sy: i32,
        pad: &Padded,
        smooth: &[bool],
        tints: &mut Tints,
        out: &mut SectionMesh,
    ) {
        use horizon_core::nocubes;
        let cfg = nocubes::config();
        let kinds = nocubes::kinds::snapshot();
        let center = region[1][1].expect("centre chunk");
        let base_y = center.min_y() + sy * 16;
        let sample = |p| self.nocubes_sample(region, sy, smooth, p, &cfg);
        let timer = std::time::Instant::now();
        let mut faces = Vec::new();
        for solid in [true, false] {
            let group = |p| {
                let mut s = sample(p);
                let is_solid = self.tables.info[s.state as usize].solid
                    || self
                        .assets
                        .registry
                        .state(BlockStateId(s.state))
                        .is_some_and(|s| s.name == "minecraft:dirt_path");
                if is_solid != solid {
                    s.smooth = false;
                    s.density = -1.;
                }
                s
            };
            faces.extend(nocubes::generate_at(
                &group,
                [0; 3],
                [16; 3],
                &cfg,
                [center.x * 16, base_y, center.z * 16],
            ));
        }
        let face_count = faces.len();
        let mut models = vec![];
        for surface in faces {
            let candidate = nocubes::material_position(&surface, cfg.mesher);
            let found = if sample(candidate).smooth {
                candidate
            } else {
                surface.block
            };
            let p = found.map(|v| v.clamp(0, 15));
            let mut id = sample(found).state;
            let normal = surface.normal();
            let is_snow = |id| {
                self.assets
                    .registry
                    .state(BlockStateId(id))
                    .is_some_and(|s| s.name == "minecraft:snow" || s.name == "minecraft:snow_block")
            };
            if (normal[1] > normal[0].abs().max(normal[2].abs()) || cfg.more_snow) && !is_snow(id) {
                for d in nocubes::MATERIAL_NEIGHBORS {
                    let q = std::array::from_fn(|i| found[i] + d[i]);
                    let snow = sample(q).state;
                    if is_snow(snow) {
                        id = snow;
                        break;
                    }
                }
            }
            if self
                .assets
                .registry
                .state(BlockStateId(id))
                .is_some_and(|s| s.properties.get("snowy").is_some_and(|v| v == "true"))
            {
                if let Some(snow) = self.assets.registry.default_state("minecraft:snow_block") {
                    id = snow.0;
                }
            }
            let world = BlockPos::new(center.x * 16 + p[0], base_y + p[1], center.z * 16 + p[2]);
            if self
                .render
                .model_catalog
                .models_at(BlockStateId(id), world, &mut models)
                .is_err()
            {
                continue;
            }
            let normal = surface.normal();
            let axis = (0..3)
                .max_by(|a, b| normal[*a].abs().total_cmp(&normal[*b].abs()))
                .unwrap();
            let direction = match (axis, normal[axis] > 0.) {
                (0, false) => ModelDirection::West,
                (0, true) => ModelDirection::East,
                (1, false) => ModelDirection::Down,
                (1, true) => ModelDirection::Up,
                (2, false) => ModelDirection::North,
                _ => ModelDirection::South,
            };
            let grass = cfg.better_grass_sides
                && self
                    .assets
                    .registry
                    .state(BlockStateId(id))
                    .is_some_and(|s| s.properties.contains_key("snowy"));
            let wanted = if grass { ModelDirection::Up } else { direction };
            let mut selected: Vec<_> = models
                .iter()
                .flat_map(|m| &m.faces)
                .filter(|f| f.quad.cull.is_none() || f.quad.facing == wanted)
                .collect();
            if selected.is_empty() {
                selected.extend(models.iter().flat_map(|m| &m.faces).take(1));
            }
            for face in selected {
                let mut quad = face.quad.clone();
                quad.positions = surface
                    .vertices
                    .map(|v| std::array::from_fn(|i| v[i] - p[i] as f32));
                quad.cull = None;
                quad.facing = direction;
                match direction {
                    ModelDirection::Down | ModelDirection::South | ModelDirection::West => {
                        quad.uv.rotate_right(1)
                    }
                    ModelDirection::Up => quad.uv.rotate_left(2),
                    _ => {}
                }
                let face = BakedFace {
                    quad,
                    layer: face.layer,
                    opaque: face.opaque,
                    translucent: face.translucent,
                    fusion: face.fusion,
                };
                let starts = (out.solid.len(), out.translucent.len());
                self.face(
                    pad,
                    tints,
                    out,
                    p,
                    p.map(|v| v as f32),
                    &face,
                    false,
                    (id, &self.tables.info[id as usize]),
                    world,
                    false,
                );
                if !self.tables.info[id as usize].solid
                    && !self
                        .assets
                        .registry
                        .state(BlockStateId(id))
                        .and_then(|s| kinds.get(&s.name))
                        .is_some_and(|k| k.single_sided)
                {
                    let mut back = face.quad.clone();
                    back.positions.reverse();
                    back.uv.reverse();
                    let back = BakedFace {
                        quad: back,
                        layer: face.layer,
                        opaque: face.opaque,
                        translucent: face.translucent,
                        fusion: None,
                    };
                    self.face(
                        pad,
                        tints,
                        out,
                        p,
                        p.map(|v| v as f32),
                        &back,
                        false,
                        (id, &self.tables.info[id as usize]),
                        world,
                        false,
                    );
                }
                let raw_light = |point: [i32; 3]| {
                    let rx = (point[0] + 16).div_euclid(16);
                    let rz = (point[2] + 16).div_euclid(16);
                    if !(0..3).contains(&rx) || !(0..3).contains(&rz) {
                        return 0;
                    }
                    let Some(chunk) = region[rz as usize][rx as usize] else {
                        return 0;
                    };
                    let id = chunk.block(point[0], base_y + point[1], point[2]);
                    lighting::packed_coords(
                        &self.tables.info[id.0 as usize],
                        chunk.light(point[0], base_y + point[1], point[2]),
                    )
                };
                for vertex in out.solid[starts.0..]
                    .iter_mut()
                    .chain(&mut out.translucent[starts.1..])
                {
                    vertex.light = pack_coords(lighting::with_min(
                        nocubes::vertex_light(vertex.pos, surface.area_normal(), &raw_light),
                        face.quad.light,
                    ));
                }
            }
            if cfg.grass_tufts
                && direction == ModelDirection::Up
                && self
                    .assets
                    .registry
                    .state(BlockStateId(sample(found).state))
                    .is_some_and(|s| s.properties.contains_key("snowy"))
                && !self
                    .assets
                    .registry
                    .state(BlockStateId(
                        sample([found[0], found[1] + 1, found[2]]).state,
                    ))
                    .and_then(|s| kinds.get(&s.name))
                    .is_some_and(|k| k.plant)
            {
                if let Some(grass) = self.assets.registry.default_state("minecraft:short_grass") {
                    let mut tuft_models = vec![];
                    if self
                        .render
                        .model_catalog
                        .models_at(grass, world, &mut tuft_models)
                        .is_ok()
                    {
                        if let Some(model) = tuft_models.iter().flat_map(|m| &m.faces).next() {
                            let offset =
                                offset(&self.tables.info[grass.0 as usize], world.x, world.z)
                                    .unwrap_or([0.; 3]);
                            for (a, b) in [(0, 2), (1, 3)] {
                                let start = std::array::from_fn::<_, 3, _>(|i| {
                                    surface.vertices[a][i] - p[i] as f32
                                        + if i == 1 { 0. } else { offset[i] as f32 }
                                });
                                let end = std::array::from_fn::<_, 3, _>(|i| {
                                    surface.vertices[b][i] - p[i] as f32
                                        + if i == 1 { 0. } else { offset[i] as f32 }
                                });
                                let mut q = model.quad.clone();
                                q.positions = [
                                    start,
                                    [start[0], start[1] + 0.4, start[2]],
                                    [end[0], end[1] + 0.4, end[2]],
                                    end,
                                ];
                                q.cull = None;
                                if id != surface.state {
                                    q.tint_index = -1;
                                    q.color = Some(-1);
                                }
                                for reverse in [false, true] {
                                    let mut q = q.clone();
                                    if reverse {
                                        q.positions.reverse();
                                        q.uv.reverse();
                                    }
                                    let tuft = BakedFace {
                                        quad: q,
                                        layer: model.layer,
                                        opaque: false,
                                        translucent: model.translucent,
                                        fusion: None,
                                    };
                                    self.face(
                                        pad,
                                        tints,
                                        out,
                                        p,
                                        p.map(|v| v as f32),
                                        &tuft,
                                        false,
                                        (grass.0, &self.tables.info[grass.0 as usize]),
                                        world,
                                        false,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        if cfg.debug && cfg.debug_mesh_performance {
            nocubes::record_mesh(timer.elapsed(), face_count);
        }
    }
}
