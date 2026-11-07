fn runtime_screen_key(game: &crate::game::Game<'_>, key: &str, canceled: &mut bool) {
    game.nocubes_screen_key(key, canceled);
}
