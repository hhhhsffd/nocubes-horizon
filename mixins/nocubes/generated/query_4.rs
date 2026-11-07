fn adapter(
    receiver: &mut Ticker<'_, '_, '_, '_>,
    from: Vec3,
    to: Vec3,
) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_4(from, to))
}
