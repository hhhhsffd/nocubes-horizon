fn adapter(
    receiver: &mut EntityFluidWorld<'_>,
    p: BlockPos,
) -> core::ops::ControlFlow<Result<FluidState, Error>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_12(p))
}
