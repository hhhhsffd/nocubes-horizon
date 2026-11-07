fn adapter(m: &mut Mob, w: &MobWorld, events: &mut Vec<MobEvent>) -> core::ops::ControlFlow<()> {
    core::ops::ControlFlow::Break(nocubes_queries::nocubes_query_37(m, w, events))
}
