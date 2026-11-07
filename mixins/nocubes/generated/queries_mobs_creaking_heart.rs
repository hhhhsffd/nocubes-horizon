use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl Blocks<'_, '_> {
    pub(super) fn nocubes_query_38(
        &self,
        s: BlockStateId,
        key: &str,
        value: &str,
    ) -> Option<BlockStateId> {
        let mut st = self.w.registry.state(s)?.clone();
        st.properties.insert(key.to_owned(), value.to_owned());
        horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
            self.w.registry.resolve(&st)
        })
        .ok()
    }
}
pub(super) fn nocubes_query_39(
    b: &Blocks,
    pos: [i32; 3],
    state: BlockStateId,
    above: BlockStateId,
) -> bool {
    let mut scene = SceneCollisions {
        world: b.w.world,
        binding: b.w.collision,
        subject: None,
    };
    let above_pos = [pos[0], pos[1] + 1, pos[2]];
    let above_empty = horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
        b.w.collision
            .nocubes_collision_shape(&mut scene, above, above_pos)
    })
    .is_ok_and(|s| s.is_empty());
    above_empty
        && !b.w.has_tag("minecraft:leaves", b.name(state))
        && b.w.facts.get(state).sturdy_full & (1 << 1) != 0
}
