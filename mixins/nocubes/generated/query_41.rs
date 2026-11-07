fn adapter(receiver: &MobWorld<'_>, bounds: Aabb, eye_y: f64) -> core::ops::ControlFlow<Fluids> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_41(bounds, eye_y))
}
