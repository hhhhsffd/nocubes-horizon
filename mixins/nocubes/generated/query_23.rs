fn adapter(receiver: &Game<'_>, pos: [i32; 3]) -> core::ops::ControlFlow<bool> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_23(pos))
}
