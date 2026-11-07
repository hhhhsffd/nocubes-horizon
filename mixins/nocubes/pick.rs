fn pick(
    game: &crate::game::Game<'_>,
    origin: [f64; 3],
    dir: [f64; 3],
    reach: f64,
) -> core::ops::ControlFlow<Option<Hit>> {
    core::ops::ControlFlow::Break(game.nocubes_ray(origin, dir, reach))
}
