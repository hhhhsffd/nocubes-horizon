fn adapter(
    receiver: &mut MotionWorld<'_, '_, '_>,
    state: BlockStateId,
    p: BlockPos,
    precise: bool,
) -> core::ops::ControlFlow<Result<()>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_54(state, p, precise))
}
