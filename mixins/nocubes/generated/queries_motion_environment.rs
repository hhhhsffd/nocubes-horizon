use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl MotionWorld<'_, '_, '_> {
    pub(super) fn nocubes_query_54(
        &mut self,
        state: BlockStateId,
        p: BlockPos,
        precise: bool,
    ) -> Result<()> {
        if !precise {
            return Ok(());
        }
        let drag = self
            .assets
            .registry
            .state(state)
            .and_then(|s| s.properties.get("drag"))
            .is_some_and(|d| d == "true");
        let above = self.block(p.offset(0, 1, 0));
        let binding = self.collision_binding;
        let context = CollisionContext::empty(false);
        let empty = binding
            .nocubes_collision_shape_with_context(&mut self.collisions(), above, p, &context)?
            .is_empty()
            && self.fluid_at(p.offset(0, 1, 0)).kind() == FluidKind::Empty;
        if self.player.flying {
            return Ok(());
        }
        let v = self.player.velocity;
        let y = match (empty, drag) {
            (true, true) => (-0.9_f64).max(v.y - 0.03),
            (true, false) => 1.8_f64.min(v.y + 0.1),
            (false, true) => (-0.3_f64).max(v.y - 0.03),
            (false, false) => 0.7_f64.min(v.y + 0.06),
        };
        self.player.velocity = Vec3::new(v.x, y, v.z);
        if empty {
            let random = &mut self.scene.random;
            for bubble in [false, true, false, true] {
                let x = f64::from(p.x) + random.next_double();
                let z = f64::from(p.z) + random.next_double();
                self.player.events.push(MotionEvent::BubbleColumnParticle {
                    bubble,
                    position: Vec3::new(x, f64::from(p.y + 1), z),
                });
            }
        } else {
            self.player.fall_distance = 0.0;
        }
        Ok(())
    }
}
