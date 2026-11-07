fn mob_fluid_height(
    world: &crate::mobs::MobWorld<'_>,
    x: i32,
    y: i32,
    z: i32,
) -> core::ops::ControlFlow<(Fluid, f64)> {
    let cfg = horizon_core::nocubes::config();
    if cfg.extend_fluids_range == 0 {
        return core::ops::ControlFlow::Continue(());
    }
    let raw = |p: [i32; 3]| {
        world
            .fluid_tables
            .fluid(world.world.block(p[0], p[1], p[2]))
    };
    let get = |p| {
        horizon_core::nocubes::fluids::extended(
            p,
            cfg.extend_fluids_range,
            |p| {
                horizon_core::nocubes::kinds::smoothable(
                    &cfg,
                    world.world.block(p[0], p[1], p[2]).0,
                )
            },
            |f: FluidState| f.kind() == FluidKind::Empty,
            |f: FluidState| f.is_source(),
            raw,
        )
    };
    let own = get([x, y, z]);
    let above = get([x, y + 1, z]);
    let kind = match own.kind() {
        FluidKind::Water | FluidKind::FlowingWater => Fluid::Water,
        FluidKind::Lava | FluidKind::FlowingLava => Fluid::Lava,
        _ => Fluid::None,
    };
    let height = if own.kind().same_family(above.kind()) {
        1.
    } else {
        f64::from(own.own_height())
    };
    core::ops::ControlFlow::Break((kind, if kind == Fluid::None { 0. } else { height }))
}
