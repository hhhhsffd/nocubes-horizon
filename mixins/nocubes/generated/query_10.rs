fn adapter(
    receiver: &mut Ticker<'_, '_, '_, '_>,
    p: BlockPos,
) -> core::ops::ControlFlow<Result<FluidState, Error>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_10(p))
}
