use super::*;
use crate::collisions::nocubes_collision::RegistryExt;
pub(super) fn nocubes_query_62(level: &ParticleLevel, pos: [i32; 3]) -> bool {
    let state = level.world.block(pos[0], pos[1], pos[2]);
    if level.facts.is_air(state) || &*level.facts.get(state).name == "minecraft:water" {
        return true;
    }
    let mut scene = SceneCollisions {
        world: level.world,
        binding: level.collision,
        subject: None,
    };
    match level
        .collision
        .nocubes_collision_shape(&mut scene, state, pos)
    {
        Ok(shape) => shape.is_empty(),
        Err(_) => !level.facts.get(state).full_collision,
    }
}
pub(super) fn nocubes_query_63(level: &ParticleLevel, a: [f64; 3], b: [f64; 3]) -> bool {
    let (from, to) = (Vec3::new(a[0], a[1], a[2]), Vec3::new(b[0], b[1], b[2]));
    let mut scene = SceneCollisions {
        world: level.world,
        binding: level.collision,
        subject: None,
    };
    let hit = traverse_blocks(from, to, 512, |p: BlockPos| {
        let state = level.world.block(p.x, p.y, p.z);
        if level.facts.is_air(state) {
            return None;
        }
        let shape = level
            .collision
            .nocubes_collision_shape(&mut scene, state, [p.x, p.y, p.z])
            .ok()?;
        shape.clip(from, to, p).map(|_| ())
    });
    !matches!(hit, Ok(Some(())))
}
