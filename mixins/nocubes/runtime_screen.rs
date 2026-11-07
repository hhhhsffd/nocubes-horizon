fn runtime_screen(
    game: &crate::game::Game<'_>,
    batch: &mut HudBatch,
    screen: &NativeScreenView<'_>,
) {
    game.nocubes_screen(batch, screen);
}
