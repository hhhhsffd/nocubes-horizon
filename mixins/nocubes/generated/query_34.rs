fn adapter(receiver: &mut Game<'_>, pos: [i32; 3]) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_34(pos))
}
