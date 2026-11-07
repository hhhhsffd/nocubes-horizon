fn adapter(
    receiver: &mut MobLanding<'_, '_>,
    state: BlockStateId,
    p: BlockPos,
) -> core::ops::ControlFlow<Result<VoxelShape, String>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_40(state, p))
}
