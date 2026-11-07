fn adapter(receiver: &BlockSpace<'_, '_>, m: &Mob, bounds: Aabb) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_45(m, bounds))
}
