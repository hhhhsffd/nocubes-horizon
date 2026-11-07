fn receive_config(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    if channel != "nocubes:config" {
        return;
    }
    match horizon_core::nocubes::apply_server_config(data) {
        Ok(()) => {
            for (&(x, z), chunk) in &game.world.chunks {
                for sy in 0..chunk.sections.len() {
                    game.world.dirty_sections.insert((x, sy as i32, z));
                }
            }
        }
        Err(e) => eprintln!("[NoCubes] rejected server configuration: {e}"),
    }
}
