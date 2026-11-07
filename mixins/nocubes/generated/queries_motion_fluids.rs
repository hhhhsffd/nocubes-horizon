use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
pub(super) fn nocubes_query_55(
    world: &World,
    registry: &horizon_world::registry::BlockRegistry,
    facts: &crate::blocks::BlockFacts,
    collision: &horizon_world::block_collision::BlockCollisionRegistry<'_>,
    pos: BlockPos,
) -> Option<BubbleColumn> {
    let state = world.block(pos.x, pos.y, pos.z);
    if &*facts.get(state).name != "minecraft:bubble_column" {
        return None;
    }
    let drag = registry
        .state(state)
        .and_then(|s| s.properties.get("drag"))
        .is_some_and(|d| d == "true");
    let above = world.block(pos.x, pos.y + 1, pos.z);
    let mut scene = SceneCollisions {
        world,
        binding: collision,
        subject: None,
    };
    let context = horizon_world::collision_context::CollisionContext::empty(false);
    let surface = collision
        .nocubes_collision_shape_with_context(&mut scene, above, pos, &context)
        .is_ok_and(|shape| shape.is_empty())
        && {
            let cfg = horizon_core::nocubes::config();
            horizon_core::nocubes::fluids::extended(
                [pos.x, pos.y + 1, pos.z],
                cfg.extend_fluids_range,
                |q| horizon_core::nocubes::kinds::smoothable(&cfg, world.block(q[0], q[1], q[2]).0),
                |f: (crate::blocks::Fluid, u8, bool)| f.0 == crate::blocks::Fluid::None,
                |f: (crate::blocks::Fluid, u8, bool)| f.1 >= 8 && !f.2,
                |q| {
                    let f = facts.get(world.block(q[0], q[1], q[2]));
                    (f.fluid, f.fluid_level, f.fluid_falling)
                },
            )
            .0
        } == crate::blocks::Fluid::None;
    Some(BubbleColumn { drag, surface })
}
