fn camera_fluids(
    world: &World,
    facts: &BlockFacts,
    b: [i32; 3],
) -> core::ops::ControlFlow<Option<(Fluid, f64)>> {
    let cfg = horizon_core::nocubes::config();
    if cfg.extend_fluids_range == 0 {
        return core::ops::ControlFlow::Continue(());
    }
    let raw = |p: [i32; 3]| {
        let f = facts.get(world.block(p[0], p[1], p[2]));
        (f.fluid, f.fluid_level, f.fluid_falling)
    };
    let get = |p| {
        horizon_core::nocubes::fluids::extended(
            p,
            cfg.extend_fluids_range,
            |p| horizon_core::nocubes::kinds::smoothable(&cfg, world.block(p[0], p[1], p[2]).0),
            |f: (Fluid, u8, bool)| f.0 == Fluid::None,
            |f: (Fluid, u8, bool)| f.1 >= 8 && !f.2,
            raw,
        )
    };
    let own = get(b);
    if own.0 == Fluid::None {
        return core::ops::ControlFlow::Break(None);
    }
    let above = get([b[0], b[1] + 1, b[2]]);
    let full = facts.get(world.block(b[0], b[1] + 1, b[2])).full_collision;
    let height = if own.0 == above.0 || (own.1 >= 8 && !own.2 && full) {
        1.
    } else {
        f64::from(own.1) / 9.
    };
    core::ops::ControlFlow::Break(Some((own.0, height)))
}
