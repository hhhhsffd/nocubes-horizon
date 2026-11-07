fn runtime_screen_key(
    game: &crate::game::Game<'_>,
    key: &str,
    repeat: bool,
    modifiers: u32,
    canceled: &mut bool,
) {
    game.nocubes_screen_key(key, repeat, modifiers, canceled);
}
