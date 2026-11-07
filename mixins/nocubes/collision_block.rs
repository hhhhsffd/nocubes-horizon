fn collision_block(
    scene: &crate::collisions::SceneCollisions<'_, '_>,
    chunk: &crate::chunk::Chunk,
    p: BlockPos,
) -> core::ops::ControlFlow<io::Result<CollisionBlock>> {
    if !horizon_core::nocubes::config().collisions {
        return core::ops::ControlFlow::Continue(());
    }
    core::ops::ControlFlow::Break(
        scene
            .binding
            .collision_block(chunk.block(p.x, p.y, p.z))
            .map(|mut block| {
                block.empty_shape = false;
                block.large_collision_shape = true;
                block
            }),
    )
}
