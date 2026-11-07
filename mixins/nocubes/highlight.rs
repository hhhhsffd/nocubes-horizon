fn highlight(game: &crate::game::Game<'_>, state: BlockStateId, canceled: &mut bool) {
    let c = horizon_core::nocubes::config();
    if c.render
        && c.render_selection_box
        && game.pack.assets.registry.state(state).is_some_and(|s| {
            c.smoothable_with(&s.name, |k| s.properties.get(k).map(String::as_str))
        })
    {
        *canceled = true;
    }
}
