use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl PlacementWorld<'_, '_> {
    pub(super) fn nocubes_query_17(&self, bounds: Aabb) -> Vec<horizon_core::geometry::VoxelShape> {
        let binding = self.0.res.collision();
        let mut world = horizon_client::collisions::SceneCollisions {
            world: &self.0.world,
            binding,
            subject: None,
        };
        blocks_in(bounds.inflate(Vec3::new(1.0, 1.0, 1.0)))
            .filter_map(|p| {
                let state = self.0.world.block(p[0], p[1], p[2]);
                binding
                    .nocubes_collision_shape(&mut world, state, p)
                    .ok()
                    .map(|s| s.moved(Vec3::new(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))))
            })
            .collect()
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_18(&self, hit: &Hit) -> Option<(Vec3, Aabb)> {
        let mut adjusted = Hit {
            pos: hit.pos,
            face: hit.face,
            point: hit.point,
        };
        let state = self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
        if self.items().block_in_tag(
            "minecraft:cushion_uses_collision_shape",
            &self.res.facts().get(state).name,
        ) && let Some(eye) = self.eye_position()
        {
            let start = Vec3::new(eye[0], eye[1], eye[2]);
            let point = Vec3::new(hit.point[0], hit.point[1], hit.point[2]);
            let end = point.add_vector(point.subtract(start).normalize().scale(0.001));
            let binding = self.res.collision();
            let mut world = horizon_client::collisions::SceneCollisions {
                world: &self.world,
                binding,
                subject: None,
            };
            if let Ok(shape) = binding.nocubes_collision_shape(&mut world, state, hit.pos)
                && let Some(result) = shape.clip(
                    start,
                    end,
                    horizon_core::position::BlockPos::new(hit.pos[0], hit.pos[1], hit.pos[2]),
                )
            {
                adjusted.face = result.direction.normal();
                adjusted.point = [result.location.x, result.location.y, result.location.z];
            }
        }
        let hit = &adjusted;
        if hit.face != [0, 1, 0] {
            return None;
        }
        let clicked = self
            .res
            .facts()
            .get(self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]));
        let pos = if clicked.replaceable {
            hit.pos
        } else {
            std::array::from_fn(|i| hit.pos[i] + hit.face[i])
        };
        let y = hit.point[1];
        let position = Vec3::new(f64::from(pos[0]) + 0.5, y, f64::from(pos[2]) + 0.5);
        let bounds = aabb((
            [position.x - 0.5, position.y, position.z - 0.5],
            [position.x + 0.5, position.y + 0.25, position.z + 0.5],
        ));
        horizon_client::cushion::can_place(&PlacementWorld(self), bounds)
            .then_some((position, bounds))
    }
}
