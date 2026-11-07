fn adapter(
    receiver: &mut Ticker<'_, '_, '_, '_>,
    p: [i32; 3],
) -> core::ops::ControlFlow<Result<FluidState, Error>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_9(p))
}
