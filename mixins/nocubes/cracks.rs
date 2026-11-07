use super::*;
impl Game<'_> {
    pub(super) fn nocubes_crack_geometry(
        &self,
        pos: [i32; 3],
        model: &horizon_client::models::catalog::BakedModel,
        origin: glam::Vec3,
        translucent: bool,
        stage: [f32; 2],
        out: &mut Vec<Vertex>,
    ) {
        use horizon_core::nocubes;
        let cfg = nocubes::config();
        let samples = nocubes::kinds::samples(&cfg);
        let id = self.world.block(pos[0], pos[1], pos[2]);
        let state = self.pack.assets.registry.state(id);
        let smooth = samples.get(id.0).smooth;
        let plant = state.is_some_and(|s| {
            nocubes::kinds::snapshot()
                .get(&s.name)
                .is_some_and(|k| k.short_plant)
        });
        if !cfg.render || (!smooth && !(cfg.fix_plant_height && plant)) {
            horizon_client::breaking::geometry(model, origin, translucent, stage, out);
            return;
        }
        let mut profile = nocubes::profile_block();
        let faces = self.nocubes_query_mesh(
            pos,
            if plant { [0, -1, 0] } else { [0; 3] },
            [1; 3],
            &cfg,
            &samples,
        );
        if let Some(p) = &mut profile {
            p.set_faces(faces.len());
        }
        if plant {
            let offset = faces
                .iter()
                .find(|f| f.normal()[1] > 0.5)
                .map(|f| nocubes::mul(f.vertices.into_iter().fold([0.; 3], nocubes::add), 0.25))
                .unwrap_or([0.5, 0., 0.5]);
            horizon_client::breaking::geometry(
                model,
                origin + glam::Vec3::new(offset[0] - 0.5, offset[1], offset[2] - 0.5),
                translucent,
                stage,
                out,
            );
            return;
        }
        for face in faces.iter() {
            for reverse in [false, true] {
                if reverse
                    && (self.res.facts().get(id).solid_render
                        || state
                            .and_then(|s| {
                                nocubes::kinds::snapshot()
                                    .get(&s.name)
                                    .map(|k| k.single_sided)
                            })
                            .unwrap_or(false))
                {
                    continue;
                }
                let mut vertices = face.vertices;
                if reverse {
                    vertices.reverse();
                }
                for (point, uv) in
                    vertices
                        .into_iter()
                        .zip([[0., 0.], [0., 1.], [1., 1.], [1., 0.]])
                {
                    out.push(Vertex {
                        pos: (origin + glam::Vec3::from_array(point)).to_array(),
                        uv,
                        layer: if translucent { 12 } else { 11 },
                        color: 0xffffffff,
                        light: horizon_client::breaking::stage_light(stage),
                        normal: [0; 4],
                        glint_uv: [0.; 2],
                    });
                }
            }
        }
    }
}
