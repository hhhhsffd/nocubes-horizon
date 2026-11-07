fn adapter(
    receiver: &mut SceneCollisions<'_, '_>,
    block: CollisionBlock,
    p: BlockPos,
    context: &CollisionContext<'_>,
) -> core::ops::ControlFlow<io::Result<VoxelShape>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_59(block, p, context))
}
