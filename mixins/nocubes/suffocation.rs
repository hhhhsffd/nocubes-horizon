use super::*;
/// The player push-out and bed tests use the same occupied surface as movement.
pub(super) fn occupied(world: &World, facts: &BlockFacts, pos: [i32; 3], body: Aabb) -> bool {
    let id = world.block(pos[0], pos[1], pos[2]);
    let f = facts.get(id);
    if !(f.full_collision && f.solid_render) {
        return false;
    }
    let cfg = horizon_core::nocubes::config();
    if !cfg.collisions || !horizon_core::nocubes::kinds::smoothable(&cfg, id.0) {
        return true;
    }
    let samples = horizon_core::nocubes::kinds::samples(&cfg);
    let sample = |q: [i32; 3]| {
        let id = world.block(pos[0] + q[0], pos[1] + q[1], pos[2] + q[2]);
        samples.get(id.0)
    };
    horizon_core::nocubes::collision_boxes_at(&sample, [0; 3], &cfg, pos)
        .iter()
        .any(|b| {
            Aabb::new(
                Vec3::new(
                    pos[0] as f64 + b[0],
                    pos[1] as f64 + b[1],
                    pos[2] as f64 + b[2],
                ),
                Vec3::new(
                    pos[0] as f64 + b[3],
                    pos[1] as f64 + b[4],
                    pos[2] as f64 + b[5],
                ),
            )
            .intersects(body)
        })
}
