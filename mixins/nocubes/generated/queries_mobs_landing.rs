use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl MobLanding<'_, '_> {
    pub(super) fn nocubes_query_40(
        &mut self,
        state: BlockStateId,
        p: BlockPos,
    ) -> Result<VoxelShape, String> {
        let mut scene = SceneCollisions {
            world: self.w.world,
            binding: self.w.collision,
            subject: None,
        };
        horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
            self.w
                .collision
                .nocubes_collision_shape(&mut scene, state, [p.x, p.y, p.z])
        })
        .map_err(|e| e.to_string())
    }
}
