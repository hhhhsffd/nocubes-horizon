use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl Game<'_> {
    pub(super) fn nocubes_query_27(&self, p: [i32; 3]) -> VoxelShape {
        let state = self.world.block(p[0], p[1], p[2]);
        let binding = self.collision;
        let mut scene = SceneCollisions {
            world: &self.world,
            binding,
            subject: None,
        };
        binding
            .nocubes_collision_shape_with_context(
                &mut scene,
                state,
                BlockPos::new(p[0], p[1], p[2]),
                &CollisionContext::empty(false),
            )
            .unwrap_or_else(|_| VoxelShape::empty())
    }
}
