fn adapter(receiver: &mut Game<'_>, pushes: &Vec<Push>) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_25(pushes.clone()))
}
