fn runtime_from_client(game: &mut crate::game::Game<'_>, player: u128, channel: &str, data: &[u8]) {
    game.nocubes_from_client(player, channel, data);
}
