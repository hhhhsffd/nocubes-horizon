fn runtime_tick(game: &mut crate::game::Game<'_>) {
    game.nocubes_tick();
    game.nocubes_pending();
}
