fn adapter(
    receiver: &MobWorld<'_>,
    from: Vec3,
    to: Vec3,
) -> core::ops::ControlFlow<Option<Option<[i32; 3]>>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_50(from, to))
}
