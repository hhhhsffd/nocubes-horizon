fn adapter(
    receiver: &ProjWorld<'_, '_>,
    from: Vec3,
    to: Vec3,
) -> core::ops::ControlFlow<Option<(Vec3, [i32; 3], [i32; 3])>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_16(from, to))
}
