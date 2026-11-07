fn adapter(receiver: &MobWorld<'_>, x: i32, y: i32, z: i32) -> core::ops::ControlFlow<Vec<Aabb>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_43(x, y, z))
}
