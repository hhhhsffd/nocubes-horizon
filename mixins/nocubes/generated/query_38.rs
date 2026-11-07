fn adapter(
    receiver: &Blocks<'_, '_>,
    s: BlockStateId,
    key: &str,
    value: &str,
) -> core::ops::ControlFlow<Option<BlockStateId>> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_38(s, key, value))
}
