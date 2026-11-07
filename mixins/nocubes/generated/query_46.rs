fn adapter(
    receiver: &BlockSpace<'_, '_>,
    m: &Mob,
    p: [i32; 3],
    face: u8,
) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_46(m, p, face))
}
