use super::*;
pub(super) fn nocubes_query_11(world: &World, fluids: &FluidResources, pos: [i32; 3]) -> [f64; 3] {
    let mut velocity = Vec3::ZERO;
    let mut scene = EntityFluidWorld {
        bounds: Aabb::new(Vec3::ZERO, Vec3::ZERO),
        position: Vec3::ZERO,
        eye_y: 0.0,
        velocity: &mut velocity,
        world,
        fluids,
    };
    let at = BlockPos::new(pos[0], pos[1], pos[2]);
    let state = nocubes_entity_fluid(world, fluids, [pos[0], pos[1], pos[2]]);
    let flow = flow_at(&mut scene, state, at).unwrap_or(Vec3::ZERO);
    [flow.x, flow.y, flow.z]
}
impl EntityFluidWorld<'_> {
    pub(super) fn nocubes_query_12(&mut self, p: BlockPos) -> Result<FluidState, Error> {
        Ok(nocubes_entity_fluid(
            &self.world,
            &self.fluids,
            [p.x, p.y, p.z],
        ))
    }
}
impl EntityFluidWorld<'_> {
    pub(super) fn nocubes_query_13(&mut self, p: [i32; 3]) -> Result<FluidState, Error> {
        Ok(nocubes_entity_fluid(
            &self.world,
            &self.fluids,
            [p[0], p[1], p[2]],
        ))
    }
}
impl EntityFluidWorld<'_> {
    pub(super) fn nocubes_query_14(&mut self, p: BlockPos) -> Result<FluidState, Error> {
        Ok(nocubes_entity_fluid(
            &self.world,
            &self.fluids,
            [p.x, p.y, p.z],
        ))
    }
}
fn nocubes_entity_fluid(world: &World, fluids: &FluidResources, p: [i32; 3]) -> FluidState {
    let cfg = horizon_core::nocubes::config();
    horizon_core::nocubes::fluids::extended(
        p,
        cfg.extend_fluids_range,
        |q| horizon_core::nocubes::kinds::smoothable(&cfg, world.block(q[0], q[1], q[2]).0),
        |f: FluidState| f.kind() == FluidKind::Empty,
        |f: FluidState| f.is_source(),
        |q| fluids.fluid(world.block(q[0], q[1], q[2])),
    )
}
