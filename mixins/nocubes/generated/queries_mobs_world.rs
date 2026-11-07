use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
#[doc = " Level.clip with VISUAL shapes and no fluids: the block the segment first"]
#[doc = " hits, None inside when it hits nothing and outside when the segment is"]
#[doc = " too long to walk. Glass and copper grates (`TransparentBlock`) and bars and panes"]
#[doc = " (`IronBarsBlock`) have no visual shape; mud, soul sand and powder snow"]
#[doc = " look like full blocks."]
pub fn visual_clip<W: DynamicCollisionWorld>(
    facts: &BlockFacts,
    collision: &BlockCollisionRegistry,
    scene: &mut W,
    state: &dyn Fn(i32, i32, i32) -> BlockStateId,
    from: Vec3,
    to: Vec3,
) -> Option<Option<[i32; 3]>>
where
    W: horizon_client::collisions::nocubes_collision::QueryWorld,
{
    let full = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 1.0));
    let result = traverse_blocks(from, to, 512, |p: BlockPos| {
        let id = state(p.x, p.y, p.z);
        if facts.is_air(id) {
            return None;
        }
        let name = facts.get(id).name.trim_start_matches("minecraft:");
        if name.ends_with("glass")
            || name.ends_with("glass_pane")
            || name.ends_with("_bars")
            || name.ends_with("copper_grate")
        {
            return None;
        }
        if matches!(name, "mud" | "soul_sand" | "powder_snow") {
            return full
                .move_by(Vec3::new(f64::from(p.x), f64::from(p.y), f64::from(p.z)))
                .clip(from, to)
                .map(|_| [p.x, p.y, p.z]);
        }
        let shape = collision
            .nocubes_collision_shape(scene, id, [p.x, p.y, p.z])
            .ok()?;
        shape.clip(from, to, p).map(|_| [p.x, p.y, p.z])
    });
    result.ok()
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_48(&self, x: i32, y: i32, z: i32) -> f64 {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        let mut shape = |y: i32| {
            horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
                self.collision.nocubes_collision_shape(
                    &mut scene,
                    self.world.block(x, y, z),
                    [x, y, z],
                )
            })
            .unwrap_or_else(|_| VoxelShape::empty())
        };
        let here = shape(y);
        let below = shape(y - 1);
        block_floor_height(&here, || &below)
    }
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_49(
        &self,
        feet_y: Option<f64>,
        from: Vec3,
        to: Vec3,
        fluids_block: bool,
    ) -> bool {
        let context = feet_y.map(horizon_world::collision_context::CollisionContext::position);
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        let result = traverse_blocks(from, to, 512, |p: BlockPos| {
            let state = self.world.block(p.x, p.y, p.z);
            if fluids_block && self.facts.get(state).fluid != Fluid::None {
                return Some(());
            }
            if self.facts.is_air(state) {
                return None;
            }
            let shape = match &context {
                Some(context) => {
                    horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
                        self.collision
                            .nocubes_collision_shape_with_context(&mut scene, state, p, context)
                    })
                }
                None => horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
                    self.collision
                        .nocubes_collision_shape(&mut scene, state, [p.x, p.y, p.z])
                }),
            }
            .ok()?;
            shape.clip(from, to, p).map(|_| ())
        });
        matches!(result, Ok(None))
    }
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_50(&self, from: Vec3, to: Vec3) -> Option<Option<[i32; 3]>> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        let state = |x, y, z| self.world.block(x, y, z);
        visual_clip(self.facts, self.collision, &mut scene, &state, from, to)
    }
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_51(&self, from: Vec3, to: Vec3) -> Option<Vec3> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        let result = traverse_blocks(from, to, 512, |p: BlockPos| {
            let state = self.world.block(p.x, p.y, p.z);
            if self.facts.is_air(state) {
                return None;
            }
            let shape = horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
                self.collision
                    .nocubes_collision_shape(&mut scene, state, [p.x, p.y, p.z])
            })
            .ok()?;
            shape.clip(from, to, p).map(|hit| hit.location)
        });
        result.ok().flatten()
    }
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_52(&self, from: Vec3, to: Vec3) -> Option<[i32; 3]> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        let result = traverse_blocks(from, to, 512, |p: BlockPos| {
            let state = self.world.block(p.x, p.y, p.z);
            if self.facts.is_air(state) {
                return None;
            }
            let shape = horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
                self.collision
                    .nocubes_collision_shape(&mut scene, state, [p.x, p.y, p.z])
            })
            .ok()?;
            shape.clip(from, to, p).map(|_| [p.x, p.y, p.z])
        });
        result.ok().flatten()
    }
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_53(
        &self,
        state: horizon_world::registry::BlockStateId,
        p: [i32; 3],
    ) -> Option<horizon_core::geometry::Aabb> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
            self.collision.nocubes_collision_shape(&mut scene, state, p)
        })
        .ok()?
        .bounds()
    }
}
