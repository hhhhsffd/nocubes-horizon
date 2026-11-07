fn adapter(
    world: &World,
    facts: &BlockFacts,
    collision: &BlockCollisionRegistry<'_>,
    from: Vec3,
    movement: Vec3,
) -> core::ops::ControlFlow<Option<(Vec3, BlockPos, [i32; 3])>> {
    core::ops::ControlFlow::Break(nocubes_queries::nocubes_query_3(
        world, facts, collision, from, movement,
    ))
}
