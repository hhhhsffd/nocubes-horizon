fn adapter(receiver: &Game<'_>, from: Vec3, to: Vec3) -> core::ops::ControlFlow<Option<Vec3>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_28(from, to))
}
