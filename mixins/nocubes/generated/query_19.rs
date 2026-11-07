fn adapter(receiver: &Game<'_>, g: &mut Gizmos) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_19(g))
}
