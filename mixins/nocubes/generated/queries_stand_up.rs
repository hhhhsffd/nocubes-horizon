use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl StandUpLevel<'_, '_> {
    pub(super) fn nocubes_query_58(&self, p: [i32; 3]) -> VoxelShape {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        self.collision
            .nocubes_collision_shape_with_context(
                &mut scene,
                self.world.block(p[0], p[1], p[2]),
                BlockPos::new(p[0], p[1], p[2]),
                &CollisionContext::empty(false),
            )
            .unwrap_or_else(|_| VoxelShape::empty())
    }
}
