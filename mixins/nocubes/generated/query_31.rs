fn adapter(
    receiver: &GameLevel<'_>,
    pos: BlockPos,
    state: BlockStateId,
) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_31(pos, state))
}
