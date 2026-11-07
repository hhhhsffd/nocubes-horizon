fn mob_fluid(
    world: &crate::mobs::MobWorld<'_>,
    x: i32,
    y: i32,
    z: i32,
) -> core::ops::ControlFlow<Fluid> {
    let cfg = horizon_core::nocubes::config();
    let raw = |q: [i32; 3]| {
        world
            .fluid_tables
            .fluid(world.world.block(q[0], q[1], q[2]))
    };
    let fluid = horizon_core::nocubes::fluids::extended(
        [x, y, z],
        cfg.extend_fluids_range,
        |q| horizon_core::nocubes::kinds::smoothable(&cfg, world.world.block(q[0], q[1], q[2]).0),
        |f: horizon_world::fluid::FluidState| f.kind() == horizon_world::fluid::FluidKind::Empty,
        |f: horizon_world::fluid::FluidState| f.is_source(),
        raw,
    );
    core::ops::ControlFlow::Break(match fluid.kind() {
        horizon_world::fluid::FluidKind::Water | horizon_world::fluid::FluidKind::FlowingWater => {
            Fluid::Water
        }
        horizon_world::fluid::FluidKind::Lava | horizon_world::fluid::FluidKind::FlowingLava => {
            Fluid::Lava
        }
        _ => Fluid::None,
    })
}
