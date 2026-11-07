fn client_login(game: &mut crate::game::Game<'_>) {
    horizon_core::nocubes::set_client_connected();
    if game.native_on("nocubes") && horizon_core::nocubes::config().info_message {
        game.nocubes_notice("nocubes.notification.keys");
    }
}
