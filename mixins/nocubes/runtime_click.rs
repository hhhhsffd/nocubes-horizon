fn runtime_click(
    game: &crate::game::Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    game.nocubes_screen_click(screen, button, canceled);
}
