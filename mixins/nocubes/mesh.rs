fn mesh(
    mesher: &mut crate::mesher::Mesher<'_>,
    region: &[[Option<&crate::chunk::Chunk>; 3]; 3],
    sy: i32,
    out: &mut crate::mesher::SectionMesh,
    scratch: &mut crate::mesher::MeshScratch,
) -> core::ops::ControlFlow<SectionMesh> {
    let cfg = horizon_core::nocubes::config();
    if !cfg.render {
        return core::ops::ControlFlow::Continue(());
    }
    let samples = horizon_core::nocubes::kinds::samples(&cfg);
    let passes = mesher.nocubes_passes(region, sy, &samples);
    if !passes.into_iter().any(|p| p) {
        return core::ops::ControlFlow::Continue(());
    }
    core::ops::ControlFlow::Break(mesher.nocubes_mesh(
        region,
        sy,
        std::mem::take(out),
        scratch,
        &cfg,
        &samples,
        passes,
    ))
}
