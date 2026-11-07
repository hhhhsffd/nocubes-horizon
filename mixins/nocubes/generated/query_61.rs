fn adapter(
    receiver: &mut LookupCollisions<'_, '_>,
    block: CollisionBlock,
    p: BlockPos,
    context: &CollisionContext<'_>,
) -> core::ops::ControlFlow<io::Result<VoxelShape>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_61(block, p, context))
}
