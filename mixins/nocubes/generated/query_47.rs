fn adapter(receiver: &mut LayEgg, m: &mut Mob, cx: &mut Cx) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_47(m, cx))
}
