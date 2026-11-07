fn adapter(
    receiver: &mut Mob,
    w: &MobWorld,
    events: &mut Vec<MobEvent>,
) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(receiver.nocubes_query_44(w, events))
}
