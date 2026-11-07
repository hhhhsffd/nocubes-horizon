use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl<'w, 'r> BlockSpace<'w, 'r> {
    pub(super) fn nocubes_query_45(&self, m: &Mob, bounds: Aabb) -> bool {
        use horizon_world::{
            entity_obstruction::EntityIdentity,
            world_movement::{CollisionCheckRequest, no_world_collision},
        };
        horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
            no_world_collision(
                &mut self.scene(m),
                CollisionCheckRequest {
                    entity: EntityIdentity(u64::from(m.id)),
                    context: Self::context(m),
                    bounds,
                    candidate_budget: 1_000_000,
                },
            )
        })
        .unwrap_or(false)
    }
}
impl<'w, 'r> BlockSpace<'w, 'r> {
    pub(super) fn nocubes_query_46(&self, m: &Mob, p: [i32; 3], face: u8) -> bool {
        if !self.facts.is_air(self.world.block(p[0], p[1], p[2]))
            && (p != block_pos(m)
                || self
                    .registry
                    .state(self.world.block(p[0], p[1], p[2]))
                    .map_or("", |s| s.name.as_str())
                    != "minecraft:moving_piston")
        {
            return false;
        }
        let s = face_step(face);
        let support_pos =
            horizon_core::position::BlockPos::new(p[0] + s[0], p[1] + s[1], p[2] + s[2]);
        let support = self
            .world
            .block(support_pos.x, support_pos.y, support_pos.z);
        let open = opposite(face);
        let Ok(shape) = horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
            self.binding.nocubes_collision_shape_with_context(
                &mut self.scene(m),
                support,
                support_pos,
                &Self::context(m),
            )
        }) else {
            return false;
        };
        let full = shape.is_block()
            || shape
                .face_shape(Direction::ALL[open as usize])
                .is_ok_and(|face| {
                    VoxelShape::block()
                        .join_is_not_empty(&face, ShapeOp::XOR)
                        .is_ok_and(|differs| !differs)
                });
        full && self.no_collision(m, attachment_box(p, face))
    }
}
