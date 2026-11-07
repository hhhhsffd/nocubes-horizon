fn adapter(receiver: &Game<'_>, p: [i32; 3]) -> core::ops::ControlFlow<VoxelShape> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_27(p))
}
