fn adapter(
    receiver: &mut Game<'_>,
    t: &mut ThrownTrident,
    from: Vec3,
    to: Vec3,
) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_30(t, from, to))
}
