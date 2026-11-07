fn runtime_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    if command == "nocubes" || command == "nocubes config" {
        *ran = game.native_mod_screen_open("nocubes");
    }
}
