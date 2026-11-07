fn runtime_char(game: &crate::game::Game<'_>, c: char, canceled: &mut bool) {
    game.nocubes_char(c, canceled);
}
