fn adapter(
    b: &Blocks,
    pos: [i32; 3],
    state: BlockStateId,
    above: BlockStateId,
) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(nocubes_queries::nocubes_query_39(b, pos, state, above))
}
