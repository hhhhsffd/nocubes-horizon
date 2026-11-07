fn player_arm(
    game: &crate::game::Game<'_>,
    left: bool,
    m: glam::Mat4,
    light: u32,
    out: &mut Vec<Vertex>,
) -> core::ops::ControlFlow<()> {
    if game.nocubes_player_arm(left, m, light, out) {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
