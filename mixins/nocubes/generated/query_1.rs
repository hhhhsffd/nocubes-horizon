fn adapter(
    receiver: &ArrowScene<'_, '_>,
    state: BlockStateId,
    p: BlockPos,
) -> core::ops::ControlFlow<Vec<Aabb>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_1(state, p))
}
