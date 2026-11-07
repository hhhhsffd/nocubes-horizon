fn adapter(
    receiver: &mut Game<'_>,
    partial: f32,
) -> core::ops::ControlFlow<Option<(glam::DVec3, f32, f32)>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_24(partial))
}
