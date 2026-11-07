fn adapter(
    receiver: &mut SceneCollisions<'_, '_>,
    state: BlockStateId,
    p: [i32; 3],
) -> core::ops::ControlFlow<io::Result<VoxelShape>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_60(state, p))
}
