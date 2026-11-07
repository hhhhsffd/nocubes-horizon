fn runtime_input(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    game.nocubes_key(key, pressed);
}
