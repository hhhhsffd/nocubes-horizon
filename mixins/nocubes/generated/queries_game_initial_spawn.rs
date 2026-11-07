use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl Columns<'_, '_> {
    pub(super) fn nocubes_query_22(&self, x: i32, y: i32, z: i32) -> bool {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.binding,
            subject: None,
        };
        let state = self.world.block(x, y, z);
        let Ok(shape) = self.binding.nocubes_collision_shape_with_context(
            &mut scene,
            state,
            BlockPos::new(x, y, z),
            &CollisionContext::empty(false),
        ) else {
            return false;
        };
        shape.face_shape(Direction::Up).is_ok_and(|face| {
            VoxelShape::block()
                .join_is_not_empty(&face, ShapeOp::XOR)
                .is_ok_and(|differs| !differs)
        })
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_23(&self, pos: [i32; 3]) -> bool {
        let mut scene = SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: None,
        };
        self.collision
            .nocubes_collision_shape_with_context(
                &mut scene,
                self.world.block(pos[0], pos[1], pos[2]),
                BlockPos::new(pos[0], pos[1], pos[2]),
                &CollisionContext::empty(false),
            )
            .is_ok_and(|shape| shape.is_empty())
    }
}
