fn motion_fluid(
    world: &crate::motion::MotionWorld<'_, '_, '_>,
    p: BlockPos,
) -> core::ops::ControlFlow<FluidState> {
    let cfg = horizon_core::nocubes::config();
    let raw = |q: [i32; 3]| {
        world
            .resources
            .fluids
            .fluid(world.scene.block(q[0], q[1], q[2]))
    };
    let fluid = horizon_core::nocubes::fluids::extended(
        [p.x, p.y, p.z],
        cfg.extend_fluids_range,
        |q| {
            world
                .assets
                .registry
                .state(world.scene.block(q[0], q[1], q[2]))
                .is_some_and(|s| {
                    cfg.smoothable_with(&s.name, |k| s.properties.get(k).map(String::as_str))
                })
        },
        |f: horizon_world::fluid::FluidState| f.kind() == horizon_world::fluid::FluidKind::Empty,
        |f: horizon_world::fluid::FluidState| f.is_source(),
        raw,
    );
    core::ops::ControlFlow::Break(fluid)
}
