use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl Game<'_> {
    pub(super) fn nocubes_query_29(&mut self, from: Vec3, velocity: Vec3, guest: Option<u32>) {
        let to = Vec3::new(
            from.x + velocity.x,
            from.y + velocity.y,
            from.z + velocity.z,
        );
        let mut scene = SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: None,
        };
        let hit = traverse_blocks(from, to, 1024, |p: BlockPos| {
            let state = self.world.block(p.x, p.y, p.z);
            if self.res.facts().is_air(state) {
                return None;
            }
            let shape = self
                .collision
                .nocubes_collision_shape(&mut scene, state, [p.x, p.y, p.z])
                .ok()?;
            shape.clip(from, to, p)
        });
        let Some(hit) = hit.ok().flatten() else {
            return;
        };
        let state = self.world.block(hit.block.x, hit.block.y, hit.block.z);
        if !self.res.facts().get(state).name.ends_with("lightning_rod") {
            return;
        }
        let edge = f64::from(1.0e-5f32);
        let clamp =
            |v: f64, origin: i32| v.clamp(f64::from(origin) + edge, f64::from(origin) + 1.0 - edge);
        self.channel_lightning(
            [
                clamp(hit.location.x, hit.block.x),
                clamp(hit.location.y, hit.block.y),
                clamp(hit.location.z, hit.block.z),
            ],
            guest,
        );
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_30(&mut self, t: &mut ThrownTrident, from: Vec3, to: Vec3) {
        let extinguished = {
            let rain = self.arrow_rain();
            let (ticks, extinguished) = self.arrow_scene(&rain).nocubes_apply_block_effects(
                from,
                to,
                t.fire_ticks,
                |_, _, _| {},
            );
            t.fire_ticks = ticks;
            extinguished
        };
        if extinguished {
            let pitch = 1.6 + (t.random.next_float() - t.random.next_float()) * 0.4;
            self.audio.play(
                "minecraft:entity.generic.extinguish_fire",
                Some([to.x, to.y, to.z]),
                0.7,
                pitch,
            );
        }
    }
}
