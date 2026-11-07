fn adapter(
    receiver: &mut MotionWorld<'_, '_, '_>,
    from: Vec3,
    to: Vec3,
    visited: &mut crate::fast_hash::FxHashSet<i64>,
    entered: &mut Vec<(i32, BlockPos)>,
    max_iterations: i32,
    damage_only: bool,
) -> core::ops::ControlFlow<Result<i32>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_56(
        from,
        to,
        visited,
        entered,
        max_iterations,
        damage_only,
    ))
}
