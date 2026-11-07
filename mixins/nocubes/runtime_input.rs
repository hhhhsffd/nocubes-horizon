fn runtime_input(game: &mut crate::game::Game<'_>, event: &winit::event::KeyEvent, modifiers: u32) {
    game.nocubes_key(event, modifiers);
}
