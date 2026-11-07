fn adapter(receiver: &MobWorld<'_>, x: i32, y: i32, z: i32) -> core::ops::ControlFlow<f64> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_48(x, y, z))
}
