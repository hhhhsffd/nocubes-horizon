fn registry_metadata(
    binding: &BlockCollisionRegistry<'_>,
    state: BlockStateId,
) -> core::ops::ControlFlow<io::Result<CollisionBlock>> {
    if !horizon_core::nocubes::config().collisions {
        return core::ops::ControlFlow::Continue(());
    }
    let result = binding
        .scan_flags
        .get(state.0 as usize)
        .copied()
        .ok_or_else(|| crate::compression::invalid("collision scan state outside registry"))
        .map(|flags| CollisionBlock {
            state,
            large_collision_shape: true,
            moving_piston: flags & 2 != 0,
            empty_shape: false,
        });
    core::ops::ControlFlow::Break(result)
}
