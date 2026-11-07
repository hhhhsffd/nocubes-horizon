fn view_blocking(
    game: &crate::game::Game<'_>,
    state: horizon_world::registry::BlockStateId,
    pos: [i32; 3],
    near_plane: Aabb,
) -> core::ops::ControlFlow<bool> {
    let c = horizon_core::nocubes::config();
    if c.render && game.state_suffocating(state) {
        if let Some(shape) = horizon_client::collisions::nocubes_collision::shape(
            &game.world,
            game.collision,
            horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]),
        ) {
            let blocking = shape.ok().is_some_and(|s| {
                s.moved(horizon_core::geometry::Vec3::new(
                    pos[0] as f64,
                    pos[1] as f64,
                    pos[2] as f64,
                ))
                .boxes()
                .iter()
                .any(|b| b.intersects(near_plane))
            });
            return core::ops::ControlFlow::Break(blocking);
        }
    }
    core::ops::ControlFlow::Continue(())
}
