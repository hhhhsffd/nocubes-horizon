use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl ArrowScene<'_, '_> {
    #[doc = " `Entity.applyEffectsFromBlocks` for arrows and player-thrown tridents."]
    #[doc = " The caller handles `isAffectedByBlocks`, initial water/rain clearing and"]
    #[doc = " `baseTick`. `visit` preserves arrow contact hooks and bubble pushes."]
    #[doc = " Returns the fire timer and whether to play the extinguishing sound."]
    pub fn nocubes_apply_block_effects(
        &self,
        from: Vec3,
        to: Vec3,
        mut fire_ticks: i32,
        mut visit: impl FnMut(BlockStateId, BlockPos, bool),
    ) -> (i32, bool) {
        let previous_fire = fire_ticks;
        let mut visited = HashSet::new();
        let mut effects = Collector::default();
        let used =
            self.nocubes_check_inside_blocks(from, to, &mut visited, &mut effects, 16, &mut visit);
        if 16 - used <= 0 {
            self.nocubes_check_inside_blocks(to, to, &mut visited, &mut effects, 1, &mut visit);
        }
        for effect in effects.finish() {
            match effect {
                Effect::ClearFreeze => {}
                Effect::FireIgnite => {
                    if fire_ticks < 0 {
                        fire_ticks += 1;
                    }
                    if fire_ticks >= 0 {
                        fire_ticks = fire_ticks.max(160);
                    }
                }
                Effect::LavaIgnite => fire_ticks = fire_ticks.max(300),
                Effect::Extinguish => fire_ticks = fire_ticks.min(0),
            }
        }
        if self.in_rain(to) {
            fire_ticks = fire_ticks.min(0);
        }
        let extinguished = previous_fire > 0 && fire_ticks <= 0;
        let ignited = fire_ticks > previous_fire;
        if fire_ticks <= 0 && !ignited {
            fire_ticks = 0;
        }
        (fire_ticks, extinguished)
    }
}
impl ArrowScene<'_, '_> {
    #[doc = " `Entity.checkInsideBlocks(from, to, ...)`: returns the iterations used."]
    fn nocubes_check_inside_blocks(
        &self,
        from: Vec3,
        to: Vec3,
        visited: &mut HashSet<BlockPos>,
        effects: &mut Collector,
        max_iterations: i32,
        visit: &mut impl FnMut(BlockStateId, BlockPos, bool),
    ) -> i32 {
        let deflated = bounds(to).deflate(Vec3::new(
            f64::from(1.0e-5f32),
            f64::from(1.0e-5f32),
            f64::from(1.0e-5f32),
        ));
        let far = from.distance_squared(to) > 0.999_990_000_000_252_6f64 * 0.999_990_000_000_252_6;
        let mut hits = Vec::new();
        let mut iterations = 0;
        let _ = sweep_intersected_blocks(from, to, deflated, 1_000_000, |p, iteration| {
            if iteration >= max_iterations {
                return false;
            }
            iterations = iteration;
            hits.push((p, iteration));
            true
        });
        for (p, step) in hits {
            let state = self.block(p);
            if self.is_air(state) {
                continue;
            }
            let inside_block = true;
            let fluid = nocubes_arrow_fluid(self, p);
            let inside_fluid = self.collided_with_fluid(fluid, p, from, to);
            if !(inside_block || inside_fluid) || !visited.insert(p) {
                continue;
            }
            effects.advance(step);
            let name = &*self.facts.get(state).name;
            let precise = far
                || deflated.intersects(Aabb::new(
                    Vec3::new(f64::from(p.x), f64::from(p.y), f64::from(p.z)),
                    Vec3::new(f64::from(p.x + 1), f64::from(p.y + 1), f64::from(p.z + 1)),
                ));
            visit(state, p, precise);
            if matches!(name, "minecraft:fire" | "minecraft:soul_fire") {
                effects.apply(Effect::ClearFreeze);
                effects.apply(Effect::FireIgnite);
            }
            if inside_fluid {
                effects.advance(step);
                match fluid.kind() {
                    FluidKind::Water | FluidKind::FlowingWater => effects.apply(Effect::Extinguish),
                    FluidKind::Lava | FluidKind::FlowingLava => {
                        effects.apply(Effect::ClearFreeze);
                        effects.apply(Effect::LavaIgnite);
                    }
                    FluidKind::Empty => {}
                }
            }
        }
        iterations + 1
    }
}
impl ArrowScene<'_, '_> {
    pub(super) fn nocubes_query_1(&self, state: BlockStateId, p: BlockPos) -> Vec<Aabb> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        self.collision
            .nocubes_collision_shape(&mut scene, state, [p.x, p.y, p.z])
            .map(|s| s.boxes().to_vec())
            .unwrap_or_default()
    }
}
impl ArrowScene<'_, '_> {
    pub(super) fn nocubes_query_2(
        &self,
        from: Vec3,
        to: Vec3,
        ignored: Option<BlockPos>,
    ) -> Option<(Vec3, BlockPos, [i32; 3])> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        let context = horizon_world::collision_context::CollisionContext::position(from.y);
        let hit = horizon_core::geometry::traverse_blocks(from, to, 1024, |p: BlockPos| {
            if Some(p) == ignored {
                return None;
            }
            let state = self.block(p);
            if self.is_air(state) {
                return None;
            }
            let shape = self
                .collision
                .nocubes_collision_shape_with_context(&mut scene, state, p, &context)
                .ok()?;
            shape.clip(from, to, p)
        });
        let hit = hit.ok().flatten()?;
        Some((hit.location, hit.block, hit.direction.normal()))
    }
}
pub(super) fn nocubes_query_3(
    world: &World,
    facts: &BlockFacts,
    collision: &BlockCollisionRegistry<'_>,
    from: Vec3,
    movement: Vec3,
) -> Option<(Vec3, BlockPos, [i32; 3])> {
    let to = Vec3::new(
        from.x + movement.x,
        from.y + movement.y,
        from.z + movement.z,
    );
    let mut scene = SceneCollisions {
        world,
        binding: collision,
        subject: None,
    };
    let context = horizon_world::collision_context::CollisionContext::position(from.y);
    let hit = horizon_core::geometry::traverse_blocks(from, to, 1024, |p: BlockPos| {
        let state = world.block(p.x, p.y, p.z);
        if facts.is_air(state) {
            return None;
        }
        let shape = collision
            .nocubes_collision_shape_with_context(&mut scene, state, p, &context)
            .ok()?;
        shape.clip(from, to, p)
    })
    .ok()
    .flatten()?;
    Some((hit.location, hit.block, hit.direction.normal()))
}
impl Ticker<'_, '_, '_, '_> {
    pub(super) fn nocubes_query_4(&mut self, from: Vec3, to: Vec3) {
        if self.st.removed || self.st.no_physics {
            return;
        }
        let scene = self.scene;
        let (fire_ticks, extinguished) =
            scene.nocubes_apply_block_effects(from, to, self.st.fire_ticks, |state, p, precise| {
                let name = &*scene.facts.get(state).name;
                crate::block_contact::emit(crate::block_contact::Event::Inside(
                    crate::block_contact::Contact {
                        entity: self.p.id as i32,
                        kind: self.p.kind.id(),
                        block: name,
                        state,
                        position: p,
                    },
                ));
                if name == "minecraft:bubble_column" && precise {
                    self.bubble_column(state, p);
                }
            });
        self.st.fire_ticks = fire_ticks;
        if extinguished {
            let pitch = 1.6 + (self.st.random.next_float() - self.st.random.next_float()) * 0.4;
            self.st.events.push(ArrowEvent::Sound {
                sound: ArrowSound::Extinguish,
                volume: 0.7,
                pitch,
            });
        }
    }
}
impl Ticker<'_, '_, '_, '_> {
    pub(super) fn nocubes_query_5(&mut self, state: BlockStateId, p: BlockPos) {
        let drag = self.scene.property(state, "drag") == Some("true");
        let above = self.scene.block(BlockPos::new(p.x, p.y + 1, p.z));
        let nothing_above = self.scene.collision_boxes(above, p).is_empty()
            && nocubes_arrow_fluid(&self.scene, BlockPos::new(p.x, p.y + 1, p.z)).kind()
                == FluidKind::Empty;
        if self.st.in_ground {
            return;
        }
        if nothing_above {
            self.add_velocity(if drag { -0.03 } else { 0.1 });
            self.st
                .events
                .push(ArrowEvent::BubbleColumnParticles { pos: p });
        } else {
            self.add_velocity(if drag { -0.03 } else { 0.06 });
            self.st.fall_distance = 0.0;
        }
    }
}
impl ArrowScene<'_, '_> {
    pub(super) fn nocubes_query_6(&self, at: Vec3) -> bool {
        let b = bounds(at).deflate(Vec3::new(0.001, 0.001, 0.001));
        let lo = b.min_position();
        let hi = b.max_position();
        let low = BlockPos::containing(lo);
        let high = BlockPos::new(
            hi.x.ceil() as i32 - 1,
            hi.y.ceil() as i32 - 1,
            hi.z.ceil() as i32 - 1,
        );
        for x in (low.x - 1) >> 4..=(high.x + 1) >> 4 {
            for z in (low.z - 1) >> 4..=(high.z + 1) >> 4 {
                if !self.world.chunks.contains_key(&(x, z)) {
                    return false;
                }
            }
        }
        for x in low.x..=high.x {
            for y in low.y..=high.y {
                for z in low.z..=high.z {
                    let fluid = nocubes_arrow_fluid(self, BlockPos::new(x, y, z));
                    let mut scene = self;
                    if matches!(fluid.kind(), FluidKind::Water | FluidKind::FlowingWater)
                        && let Ok(height) = fluid.height_at(&mut scene, [x, y, z])
                        && f64::from(y) + f64::from(height) >= lo.y
                        && f64::from(y) + f64::from(height) > at.y
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
}
pub(super) trait NocubesQuery7 {
    fn nocubes_query_7(&mut self, p: [i32; 3]) -> Result<FluidState, Error>;
}
impl NocubesQuery7 for &ArrowScene<'_, '_> {
    fn nocubes_query_7(&mut self, p: [i32; 3]) -> Result<FluidState, Error> {
        Ok(nocubes_arrow_fluid(self, BlockPos::new(p[0], p[1], p[2])))
    }
}
impl Ticker<'_, '_, '_, '_> {
    pub(super) fn nocubes_query_8(&mut self, p: BlockPos) -> Result<FluidState, Error> {
        Ok(nocubes_arrow_fluid(&self.scene, p))
    }
}
impl Ticker<'_, '_, '_, '_> {
    pub(super) fn nocubes_query_9(&mut self, p: [i32; 3]) -> Result<FluidState, Error> {
        Ok(nocubes_arrow_fluid(
            &self.scene,
            BlockPos::new(p[0], p[1], p[2]),
        ))
    }
}
impl Ticker<'_, '_, '_, '_> {
    pub(super) fn nocubes_query_10(&mut self, p: BlockPos) -> Result<FluidState, Error> {
        Ok(nocubes_arrow_fluid(&self.scene, p))
    }
}
fn nocubes_arrow_fluid(scene: &ArrowScene<'_, '_>, p: BlockPos) -> FluidState {
    let cfg = horizon_core::nocubes::config();
    horizon_core::nocubes::fluids::extended(
        [p.x, p.y, p.z],
        cfg.extend_fluids_range,
        |q| {
            scene
                .registry
                .state(scene.block(BlockPos::new(q[0], q[1], q[2])))
                .is_some_and(|s| {
                    cfg.smoothable_with(&s.name, |k| s.properties.get(k).map(String::as_str))
                })
        },
        |f: FluidState| f.kind() == FluidKind::Empty,
        |f: FluidState| f.is_source(),
        |q| {
            scene
                .fluids
                .fluid(scene.block(BlockPos::new(q[0], q[1], q[2])))
        },
    )
}
