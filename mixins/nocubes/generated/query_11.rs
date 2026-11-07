fn adapter(
    world: &World,
    fluids: &FluidResources,
    pos: [i32; 3],
) -> core::ops::ControlFlow<[f64; 3]> {
    core::ops::ControlFlow::Break(nocubes_queries::nocubes_query_11(world, fluids, pos))
}
