use super::*;
/// Shapes are block local, allowing the normal movement solver to keep stepping,
/// borders, pistons, entity collision, and server authority.
pub fn shape(
    world: &crate::world::World,
    binding: &BlockCollisionRegistry<'_>,
    p: BlockPos,
) -> Option<io::Result<VoxelShape>> {
    lookup_shape(&|q| world.block(q[0], q[1], q[2]), binding, p)
}
pub fn lookup_shape(
    block: &impl Fn([i32; 3]) -> BlockStateId,
    binding: &BlockCollisionRegistry<'_>,
    p: BlockPos,
) -> Option<io::Result<VoxelShape>> {
    use horizon_core::nocubes;
    let cfg = nocubes::config();
    if !cfg.collisions {
        return None;
    }
    let here_id = block([p.x, p.y, p.z]);
    if !binding.may_collide(here_id)
        && binding.registry().state(here_id).is_some_and(|s| {
            cfg.smoothable_with(&s.name, |k| s.properties.get(k).map(String::as_str))
        })
    {
        return Some(Ok(VoxelShape::empty()));
    }
    let sample = |q: [i32; 3]| {
        let id = block(q);
        nocubes::kinds::sample(&cfg, id.0)
    };
    let pos = [p.x, p.y, p.z];
    let here = sample(pos);
    if !here.smooth && here.solid {
        return None;
    }
    let near = (-1..=1)
        .any(|z| (-1..=1).any(|y| (-1..=1).any(|x| sample([p.x + x, p.y + y, p.z + z]).smooth)));
    if !near {
        return None;
    }
    let boxes = nocubes::collision_boxes(&sample, pos, &cfg);
    Some((|| {
        let mut result = VoxelShape::empty();
        for b in boxes {
            let next = VoxelShape::from_box(Aabb::new(
                Vec3::new(b[0], b[1], b[2]),
                Vec3::new(b[3], b[4], b[5]),
            ))
            .map_err(io::Error::other)?;
            result = result
                .join(&next, horizon_core::geometry::ShapeOp::OR)
                .map_err(io::Error::other)?;
        }
        result.optimized().map_err(io::Error::other)
    })())
}
thread_local! {static NONPLAYER:std::cell::Cell<bool>=const {std::cell::Cell::new(false)};}
/// Context stays on the querying thread and restores correctly for nested calls and unwinding.
pub fn with_nonplayer<T>(f: impl FnOnce() -> T) -> T {
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            NONPLAYER.with(|v| v.set(self.0));
        }
    }
    let old = NONPLAYER.with(|v| v.replace(true));
    let _reset = Reset(old);
    f()
}
/// Shared lookup contract for resident and closure-backed engine queries.
pub trait QueryWorld {
    fn nocubes_block(&self, p: [i32; 3]) -> horizon_world::registry::BlockStateId;
    fn nocubes_falling(&self) -> bool {
        false
    }
    fn nocubes_player(&self) -> bool {
        false
    }
}
impl QueryWorld for SceneCollisions<'_, '_> {
    fn nocubes_block(&self, p: [i32; 3]) -> horizon_world::registry::BlockStateId {
        self.world.block(p[0], p[1], p[2])
    }
    fn nocubes_falling(&self) -> bool {
        self.subject.is_some_and(|s| s.falling_block)
    }
    fn nocubes_player(&self) -> bool {
        self.subject.is_some() && !NONPLAYER.with(|v| v.get())
    }
}
impl QueryWorld for LookupCollisions<'_, '_> {
    fn nocubes_block(&self, p: [i32; 3]) -> horizon_world::registry::BlockStateId {
        (self.block)(p[0], p[1], p[2])
    }
}
pub trait RegistryExt {
    fn nocubes_collision_shape<W>(
        &self,
        world: &mut W,
        state: BlockStateId,
        p: [i32; 3],
    ) -> std::result::Result<
        VoxelShape,
        horizon_world::block_collision::CollisionQueryError<<W as DynamicSupportWorld>::Error>,
    >
    where
        W: DynamicCollisionWorld + QueryWorld;
    fn nocubes_collision_shape_with_context<W, E>(
        &self,
        world: &mut W,
        state: BlockStateId,
        p: BlockPos,
        context: &CollisionContext<'_>,
    ) -> std::result::Result<VoxelShape, horizon_world::block_collision::CollisionQueryError<E>>
    where
        W: DynamicCollisionWorld<Error = E>
            + LiquidCollisionWorld<Error = E>
            + PowderSnowWorld<Error = E>
            + QueryWorld;
}
impl RegistryExt for BlockCollisionRegistry<'_> {
    fn nocubes_collision_shape<W>(
        &self,
        world: &mut W,
        state: BlockStateId,
        p: [i32; 3],
    ) -> std::result::Result<
        VoxelShape,
        horizon_world::block_collision::CollisionQueryError<<W as DynamicSupportWorld>::Error>,
    >
    where
        W: DynamicCollisionWorld + QueryWorld,
    {
        let cfg = horizon_core::nocubes::config();
        if state == world.nocubes_block(p)
            && !world.nocubes_falling()
            && !(cfg.player_collisions_only && !world.nocubes_player())
        {
            if let Some(shape) = lookup_shape(
                &|q| world.nocubes_block(q),
                self,
                BlockPos::new(p[0], p[1], p[2]),
            ) {
                return shape.map_err(horizon_world::block_collision::CollisionQueryError::Data);
            }
        }
        self.collision_shape(world, state, p)
    }
    fn nocubes_collision_shape_with_context<W, E>(
        &self,
        world: &mut W,
        state: BlockStateId,
        p: BlockPos,
        context: &CollisionContext<'_>,
    ) -> std::result::Result<VoxelShape, horizon_world::block_collision::CollisionQueryError<E>>
    where
        W: DynamicCollisionWorld<Error = E>
            + LiquidCollisionWorld<Error = E>
            + PowderSnowWorld<Error = E>
            + QueryWorld,
    {
        let cfg = horizon_core::nocubes::config();
        if state == world.nocubes_block([p.x, p.y, p.z])
            && !world.nocubes_falling()
            && !(cfg.player_collisions_only && !world.nocubes_player())
        {
            if let Some(shape) = lookup_shape(&|q| world.nocubes_block(q), self, p) {
                return shape.map_err(horizon_world::block_collision::CollisionQueryError::Data);
            }
        }
        self.collision_shape_with_context(world, state, p, context)
    }
}
