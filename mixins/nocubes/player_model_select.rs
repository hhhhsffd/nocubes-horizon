fn player_model_select(
    game: &crate::game::Game<'_>,
    kind: &std::sync::Arc<horizon_client::mobs::MobType>,
    slim: bool,
) -> core::ops::ControlFlow<Option<std::sync::Arc<horizon_client::mobs::Part>>> {
    match game.nocubes_player_model(kind, slim) {
        Some(model) => core::ops::ControlFlow::Break(Some(model)),
        None => core::ops::ControlFlow::Continue(()),
    }
}
