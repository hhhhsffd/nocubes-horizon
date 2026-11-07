fn boat_fluid_height(
    game: &crate::game::Game<'_>,
    x: i32,
    y: i32,
    z: i32,
    fluid: Option<Fluid>,
) -> core::ops::ControlFlow<Option<(f32, bool)>> {
    let cfg = horizon_core::nocubes::config();
    if cfg.extend_fluids_range == 0 {
        return core::ops::ControlFlow::Continue(());
    }
    let raw = |p: [i32; 3]| {
        game.res
            .motion()
            .fluids
            .fluid(game.world.block(p[0], p[1], p[2]))
    };
    let get = |p| {
        horizon_core::nocubes::fluids::extended(
            p,
            cfg.extend_fluids_range,
            |p| {
                game.pack
                    .assets
                    .registry
                    .state(game.world.block(p[0], p[1], p[2]))
                    .is_some_and(|s| {
                        cfg.smoothable_with(&s.name, |k| s.properties.get(k).map(String::as_str))
                    })
            },
            |f: horizon_world::fluid::FluidState| {
                f.kind() == horizon_world::fluid::FluidKind::Empty
            },
            |f: horizon_world::fluid::FluidState| f.is_source(),
            raw,
        )
    };
    let own = get([x, y, z]);
    let above = get([x, y + 1, z]);
    let kind =
        match own.kind() {
            horizon_world::fluid::FluidKind::Water
            | horizon_world::fluid::FluidKind::FlowingWater => Fluid::Water,
            horizon_world::fluid::FluidKind::Lava
            | horizon_world::fluid::FluidKind::FlowingLava => Fluid::Lava,
            _ => Fluid::None,
        };
    if kind == Fluid::None || fluid.is_some_and(|f| f != kind) {
        return core::ops::ControlFlow::Break(None);
    }
    core::ops::ControlFlow::Break(Some((
        if own.kind().same_family(above.kind()) {
            1.
        } else {
            own.own_height()
        },
        own.is_source(),
    )))
}
