use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl Game<'_> {
    pub(super) fn nocubes_query_19(&self, g: &mut Gizmos) {
        if !self.debug_entry("visualize_collision_boxes") {
            return;
        }
        let Some(p) = &self.player else {
            return;
        };
        let bounds = p.bounds().inflate(Vec3::new(6.0, 6.0, 6.0));
        let (lo, hi) = (bounds.min_position(), bounds.max_position());
        let mut scene = horizon_client::collisions::SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: None,
        };
        for y in lo.y.floor() as i32..=hi.y.floor() as i32 {
            for z in lo.z.floor() as i32..=hi.z.floor() as i32 {
                for x in lo.x.floor() as i32..=hi.x.floor() as i32 {
                    let Ok(shape) = self.collision.nocubes_collision_shape_with_context(
                        &mut scene,
                        self.world.block(x, y, z),
                        BlockPos::new(x, y, z),
                        &CollisionContext::empty(false),
                    ) else {
                        continue;
                    };
                    let at = [f64::from(x), f64::from(y), f64::from(z)];
                    for b in shape.boxes() {
                        let (min, max) = (b.min_position(), b.max_position());
                        g.cuboid(
                            [at[0] + min.x, at[1] + min.y, at[2] + min.z],
                            [at[0] + max.x, at[1] + max.y, at[2] + max.z],
                            g::WHITE,
                            g::STROKE_WIDTH,
                        );
                    }
                }
            }
        }
    }
}
