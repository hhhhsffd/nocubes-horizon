fn adapter(
    receiver: &mut EntityFluidWorld<'_>,
    p: [i32; 3],
) -> core::ops::ControlFlow<Result<FluidState, Error>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_13(p))
}
