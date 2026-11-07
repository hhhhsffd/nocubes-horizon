fn highlight(state: BlockStateId, canceled: &mut bool) {
    let c = horizon_core::nocubes::config();
    if c.render && c.render_selection_box && horizon_core::nocubes::kinds::smoothable(&c, state.0) {
        *canceled = true;
    }
}
