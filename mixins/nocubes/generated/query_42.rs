fn adapter(
    receiver: &MobWorld<'_>,
    bounds: Aabb,
) -> core::ops::ControlFlow<(CurrentAccumulator, CurrentAccumulator)> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_42(bounds))
}
