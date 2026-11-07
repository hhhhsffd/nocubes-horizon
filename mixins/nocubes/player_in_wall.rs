fn player_in_wall(
    game: &crate::game::Game<'_>,
    at: [f64; 3],
    body: Dimensions,
) -> core::ops::ControlFlow<bool> {
    let c = horizon_core::nocubes::config();
    if !c.collisions {
        return core::ops::ControlFlow::Continue(());
    }
    let y = at[1] + f64::from(body.eye_height);
    let half = f64::from(body.width) * 0.4;
    let eye = horizon_core::geometry::Aabb::new(
        horizon_core::geometry::Vec3::new(at[0] - half, y - 0.000001, at[2] - half),
        horizon_core::geometry::Vec3::new(at[0] + half, y + 0.000001, at[2] + half),
    );
    let hit = ((at[0] - half).floor() as i32..=(at[0] + half).floor() as i32).any(|x| {
        ((at[2] - half).floor() as i32..=(at[2] + half).floor() as i32).any(|z| {
            let pos = [x, y.floor() as i32, z];
            let id = game.world.block(pos[0], pos[1], pos[2]);
            if !game.state_suffocating(id) {
                return false;
            }
            if let Some(shape) = horizon_client::collisions::nocubes_collision::shape(
                &game.world,
                game.collision,
                horizon_core::position::BlockPos::new(pos[0], pos[1], pos[2]),
            ) {
                shape.ok().is_some_and(|s| {
                    s.moved(horizon_core::geometry::Vec3::new(
                        pos[0] as f64,
                        pos[1] as f64,
                        pos[2] as f64,
                    ))
                    .boxes()
                    .iter()
                    .any(|b| b.intersects(eye))
                })
            } else {
                true
            }
        })
    });
    core::ops::ControlFlow::Break(hit)
}
