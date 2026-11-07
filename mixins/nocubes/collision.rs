use super::*;
struct CachedShape {
    config: std::sync::Arc<horizon_core::nocubes::Config>,
    samples: horizon_core::nocubes::kinds::Samples,
    registry: usize,
    dependencies: Vec<([i32; 3], BlockStateId)>,
    shape: VoxelShape,
}
impl CachedShape {
    fn valid(
        &self,
        block: &impl Fn([i32; 3]) -> BlockStateId,
        config: &std::sync::Arc<horizon_core::nocubes::Config>,
        samples: &horizon_core::nocubes::kinds::Samples,
        registry: usize,
    ) -> bool {
        std::sync::Arc::ptr_eq(&self.config, config)
            && self.samples.same_registry(samples)
            && self.registry == registry
            && self.dependencies.iter().all(|(q, id)| block(*q) == *id)
    }
}
thread_local! {
    static SHAPES: std::cell::RefCell<std::collections::HashMap<[i32; 3], CachedShape>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    static SHAPE_ORDER: std::cell::RefCell<std::collections::VecDeque<[i32; 3]>> =
        const { std::cell::RefCell::new(std::collections::VecDeque::new()) };
}
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
    let samples = nocubes::kinds::samples(&cfg);
    let registry = binding.registry() as *const _ as usize;
    let pos = [p.x, p.y, p.z];
    if let Some(shape) = SHAPES.with(|cache| {
        let cache = cache.borrow();
        let entry = cache.get(&pos)?;
        entry
            .valid(block, &cfg, &samples, registry)
            .then(|| entry.shape.clone())
    }) {
        return Some(Ok(shape));
    }
    let here_id = block([p.x, p.y, p.z]);
    if !binding.may_collide(here_id) && samples.get(here_id.0).smooth {
        return Some(Ok(VoxelShape::empty()));
    }
    let dependencies = std::cell::RefCell::new(std::collections::HashMap::new());
    let sample = |q: [i32; 3]| {
        let id = block(q);
        dependencies.borrow_mut().insert(q, id);
        samples.get(id.0)
    };
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
    let shape = shape_from_boxes(boxes);
    if let Ok(shape) = &shape {
        SHAPES.with(|cache| {
            let mut cache = cache.borrow_mut();
            if !cache.contains_key(&pos) {
                SHAPE_ORDER.with(|order| {
                    let mut order = order.borrow_mut();
                    if cache.len() >= 1024 {
                        if let Some(old) = order.pop_front() {
                            cache.remove(&old);
                        }
                    }
                    order.push_back(pos);
                });
            }
            cache.insert(
                pos,
                CachedShape {
                    config: cfg,
                    samples,
                    registry,
                    dependencies: dependencies.into_inner().into_iter().collect(),
                    shape: shape.clone(),
                },
            );
        });
    }
    Some(shape)
}
fn shape_from_boxes(boxes: Vec<[f64; 6]>) -> io::Result<VoxelShape> {
    let mut result = VoxelShape::empty();
    for b in boxes {
        let next = VoxelShape::from_box(Aabb::new(
            Vec3::new(b[0], b[1], b[2]),
            Vec3::new(b[3], b[4], b[5]),
        ))
        .map_err(io::Error::other)?;
        result = result
            .join_unoptimized(&next, horizon_core::geometry::ShapeOp::OR)
            .map_err(io::Error::other)?;
    }
    result.optimized().map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collision_cache_invalidates_after_edits_config_or_registry_changes() {
        let config = std::sync::Arc::new(horizon_core::nocubes::Config::default());
        let samples = horizon_core::nocubes::kinds::Samples::default();
        let entry = CachedShape {
            config: config.clone(),
            samples: samples.clone(),
            registry: 7,
            dependencies: vec![([0; 3], BlockStateId(1)), ([1; 3], BlockStateId(2))],
            shape: VoxelShape::empty(),
        };
        let block = |p| BlockStateId(if p == [0; 3] { 1 } else { 2 });
        assert!(entry.valid(&block, &config, &samples, 7));
        assert!(!entry.valid(&|_| BlockStateId(1), &config, &samples, 7));
        assert!(!entry.valid(&block, &std::sync::Arc::new((*config).clone()), &samples, 7));
        assert!(!entry.valid(&block, &config, &samples, 8));
    }
    #[test]
    fn batched_shape_union_matches_incremental_optimization() {
        use horizon_core::nocubes::{self, Config, Mesher, Sample};
        for mesher in [
            Mesher::SurfaceNets,
            Mesher::SurfaceNets2x,
            Mesher::MarchingCubes,
            Mesher::MarchingCubes2x,
            Mesher::OldNoCubes,
            Mesher::Wulferis,
        ] {
            for level in [0, 2, 4] {
                let cfg = Config {
                    mesher,
                    old_collision_enhancement: level,
                    ..Default::default()
                };
                let sample = |p: [i32; 3]| {
                    if p[1] < 1 && p[0] + p[2] < 2 {
                        Sample {
                            state: 1,
                            smooth: true,
                            solid: true,
                            density: 1.,
                        }
                    } else {
                        Sample::AIR
                    }
                };
                let boxes = nocubes::collision_boxes(&sample, [0; 3], &cfg);
                let new = shape_from_boxes(boxes.clone()).unwrap();
                let mut old = VoxelShape::empty();
                for b in boxes {
                    let part = VoxelShape::from_box(Aabb::new(
                        Vec3::new(b[0], b[1], b[2]),
                        Vec3::new(b[3], b[4], b[5]),
                    ))
                    .unwrap();
                    old = old
                        .join(&part, horizon_core::geometry::ShapeOp::OR)
                        .unwrap();
                }
                assert!(
                    !new.join_is_not_empty(&old, horizon_core::geometry::ShapeOp::XOR)
                        .unwrap(),
                    "{mesher:?}, level {level}"
                );
            }
        }
    }
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
