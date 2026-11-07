fn adapter(
    receiver: &mut Game<'_>,
    from: Vec3,
    velocity: Vec3,
    guest: Option<u32>,
) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_29(from, velocity, guest))
}
