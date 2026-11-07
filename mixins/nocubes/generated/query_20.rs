fn adapter(
    receiver: &Game<'_>,
    center: [f64; 3],
    min: [f64; 3],
    max: [f64; 3],
) -> core::ops::ControlFlow<f32> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_20(center, min, max))
}
