fn adapter(
    p: &PlayerMotion,
    world: &World,
    facts: &BlockFacts,
) -> core::ops::ControlFlow<[Option<f64>; 2]> {
    core::ops::ControlFlow::Break(nocubes_queries::nocubes_query_35(p, world, facts))
}
