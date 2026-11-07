fn adapter(
    receiver: &PlacementWorld<'_, '_>,
    bounds: Aabb,
) -> core::ops::ControlFlow<Vec<horizon_core::geometry::VoxelShape>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_17(bounds))
}
