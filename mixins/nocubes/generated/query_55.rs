fn adapter(
    world: &World,
    registry: &horizon_world::registry::BlockRegistry,
    facts: &crate::blocks::BlockFacts,
    collision: &horizon_world::block_collision::BlockCollisionRegistry<'_>,
    pos: BlockPos,
) -> core::ops::ControlFlow<Option<BubbleColumn>> {
    core::ops::ControlFlow::Break(nocubes_queries::nocubes_query_55(
        world, registry, facts, collision, pos,
    ))
}
