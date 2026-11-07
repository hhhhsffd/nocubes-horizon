fn collision_shape(
    scene: &crate::collisions::SceneCollisions<'_, '_>,
    p: BlockPos,
) -> core::ops::ControlFlow<io::Result<VoxelShape>> {
    match crate::collisions::nocubes_collision::shape(scene.world, scene.binding, p) {
        Some(shape) => core::ops::ControlFlow::Break(shape),
        None => core::ops::ControlFlow::Continue(()),
    }
}
