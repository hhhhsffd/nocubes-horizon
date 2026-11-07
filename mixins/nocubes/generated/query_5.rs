fn adapter(
    receiver: &mut Ticker<'_, '_, '_, '_>,
    state: BlockStateId,
    p: BlockPos,
) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_5(state, p))
}
