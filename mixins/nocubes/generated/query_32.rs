fn adapter(receiver: &Game<'_>) -> core::ops::ControlFlow<[Option<f64>; 2]> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_32())
}
