fn adapter(receiver: &Columns<'_, '_>, x: i32, y: i32, z: i32) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_22(x, y, z))
}
