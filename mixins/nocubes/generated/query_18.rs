fn adapter(receiver: &Game<'_>, hit: &Hit) -> core::ops::ControlFlow<Option<(Vec3, Aabb)>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_18(hit))
}
