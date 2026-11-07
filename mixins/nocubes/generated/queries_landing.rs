use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl MotionWorld<'_, '_, '_> {
    pub(super) fn nocubes_query_36(
        &mut self,
        state: BlockStateId,
        p: BlockPos,
    ) -> Result<VoxelShape> {
        let mut world = SceneCollisions {
            world: self.scene,
            binding: self.collision_binding,
            subject: None,
        };
        self.collision_binding
            .nocubes_collision_shape(&mut world, state, [p.x, p.y, p.z])
            .map_err(|e| e.to_string().into())
    }
}
