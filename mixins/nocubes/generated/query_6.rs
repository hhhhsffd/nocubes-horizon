fn adapter(receiver: &ArrowScene<'_, '_>, at: Vec3) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_6(at))
}
