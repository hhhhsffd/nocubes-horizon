fn adapter(
    receiver: &mut Game<'_>,
    pos: BlockPos,
    old: horizon_world::registry::BlockStateId,
) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_26(pos, old))
}
