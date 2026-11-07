fn adapter(
    receiver: &ArrowScene<'_, '_>,
    from: Vec3,
    to: Vec3,
    ignored: Option<BlockPos>,
) -> core::ops::ControlFlow<Option<(Vec3, BlockPos, [i32; 3])>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_2(from, to, ignored))
}
