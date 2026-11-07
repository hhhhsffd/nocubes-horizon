fn adapter(
    receiver: &mut MotionWorld<'_, '_, '_>,
    state: BlockStateId,
    p: BlockPos,
) -> core::ops::ControlFlow<Result<VoxelShape>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_36(state, p))
}
