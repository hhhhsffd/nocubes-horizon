fn adapter(
    receiver: &MobWorld<'_>,
    state: horizon_world::registry::BlockStateId,
    p: [i32; 3],
) -> core::ops::ControlFlow<Option<horizon_core::geometry::Aabb>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_53(state, p))
}
