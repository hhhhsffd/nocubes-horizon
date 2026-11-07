fn adapter(
    receiver: &mut MotionWorld<'_, '_, '_>,
    p: BlockPos,
) -> core::ops::ControlFlow<Result<horizon_core::geometry::VoxelShape>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_57(p))
}
