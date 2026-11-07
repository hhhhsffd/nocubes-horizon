fn adapter(receiver: &StandUpLevel<'_, '_>, p: [i32; 3]) -> core::ops::ControlFlow<VoxelShape> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_58(p))
}
