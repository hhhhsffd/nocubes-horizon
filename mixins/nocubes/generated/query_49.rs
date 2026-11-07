fn adapter(
    receiver: &MobWorld<'_>,
    feet_y: Option<f64>,
    from: Vec3,
    to: Vec3,
    fluids_block: bool,
) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_49(feet_y, from, to, fluids_block))
}
