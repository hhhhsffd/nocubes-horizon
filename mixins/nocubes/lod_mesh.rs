fn lod_mesh(
    mesher: &mut Mesher,
    key: TileKey,
    columns: &[&Column],
    covered: &FxHashSet<(i32, i32)>,
    covered_bounds: Option<[i32; 4]>,
    context: &WorkerContext,
    neighbors: &[Vec<EdgeColumn<'_>>; 4],
    finer: &[TileKey],
) -> core::ops::ControlFlow<SectionMesh> {
    let cfg = horizon_core::nocubes::config();
    if !cfg.render
        || matches!(
            cfg.mesher,
            horizon_core::nocubes::Mesher::CullingCubic
                | horizon_core::nocubes::Mesher::StupidCubic
        )
    {
        return core::ops::ControlFlow::Continue(());
    }
    core::ops::ControlFlow::Break(nocubes_lod::mesh(
        mesher,
        key,
        columns,
        covered,
        covered_bounds,
        context,
        neighbors,
        finer,
        &cfg,
    ))
}
