fn send_config(game: &mut crate::game::Game<'_>, player: u128) {
    if !game.native_on("nocubes") {
        return;
    }
    if let Ok(data) = horizon_core::nocubes::server_config_bytes() {
        if let Err(e) = game.native_net_send_to_player(player, "nocubes:config", &data) {
            eprintln!("[NoCubes] configuration sync: {e}");
        }
    }
}
