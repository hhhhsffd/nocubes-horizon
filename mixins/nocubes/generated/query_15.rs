fn adapter(receiver: &Game<'_>, bb: Aabb) -> core::ops::ControlFlow<f32> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_15(bb))
}
