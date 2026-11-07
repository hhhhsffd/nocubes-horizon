fn adapter(
    receiver: &mut &ArrowScene<'_, '_>,
    p: [i32; 3],
) -> core::ops::ControlFlow<Result<FluidState, Error>> {
    core::ops::ControlFlow::Break(nocubes_queries::NocubesQuery7::nocubes_query_7(receiver, p))
}
