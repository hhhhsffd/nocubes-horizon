use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
#[doc = " `FluidFlowWorld` over the mob's scene: the neighbours a flow vector reads."]
struct FlowScene<'a, 'w>(&'a MobWorld<'w>);
impl FluidFlowWorld for FlowScene<'_, '_> {
    type Error = std::convert::Infallible;
    type Block = BlockStateId;
    fn fluid(&mut self, p: BlockPos) -> Result<FluidState, Self::Error> {
        Ok(nocubes_extended(self.0, [p.x, p.y, p.z]))
    }
    fn block(&mut self, p: BlockPos) -> Result<BlockStateId, Self::Error> {
        Ok(self.0.world.block(p.x, p.y, p.z))
    }
    fn blocks_fluid_flow(&self, s: BlockStateId) -> bool {
        self.0.fluid_tables.blocks_flow(s)
    }
    fn is_ice_class(&self, s: BlockStateId) -> bool {
        self.0.fluid_tables.ice(s)
    }
    fn is_face_sturdy(
        &mut self,
        s: BlockStateId,
        _: BlockPos,
        d: horizon_core::geometry::Direction,
    ) -> Result<bool, Self::Error> {
        Ok(self.0.fluid_tables.sturdy(s, d))
    }
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_41(&self, bounds: Aabb, eye_y: f64) -> Fluids {
        let b = bounds.deflate(Vec3::new(0.001, 0.001, 0.001));
        let (lo, hi) = (b.min_position(), b.max_position());
        let mut out = Fluids::default();
        let (ex, ez) = (
            ((lo.x + hi.x) / 2.0).floor() as i32,
            ((lo.z + hi.z) / 2.0).floor() as i32,
        );
        let cells = (
            [
                (lo.x.floor() as i32).min(ex),
                lo.y.floor() as i32,
                (lo.z.floor() as i32).min(ez),
            ],
            [
                (hi.x.ceil() as i32 - 1).max(ex),
                hi.y.ceil() as i32 - 1,
                (hi.z.ceil() as i32 - 1).max(ez),
            ],
        );
        if !(horizon_core::nocubes::config().extend_fluids_range > 0
            || self
                .world
                .may_hold(cells.0, cells.1, |s| self.facts.get(s).fluid != Fluid::None))
        {
            return out;
        }
        for x in lo.x.floor() as i32..=(hi.x.ceil() as i32 - 1) {
            for y in lo.y.floor() as i32..=(hi.y.ceil() as i32 - 1) {
                for z in lo.z.floor() as i32..=(hi.z.ceil() as i32 - 1) {
                    let (fluid, h) = self.fluid_height(x, y, z);
                    if fluid == Fluid::None {
                        continue;
                    }
                    let top = f64::from(y) + h;
                    if top >= lo.y {
                        let depth = top - lo.y;
                        match fluid {
                            Fluid::Water => {
                                out.water = out.water.max(depth);
                                out.in_water = true;
                            }
                            Fluid::Lava => {
                                out.lava = out.lava.max(depth);
                                out.in_lava = true;
                            }
                            Fluid::None => {}
                        }
                    }
                }
            }
        }
        for y in lo.y.floor() as i32..=(hi.y.ceil() as i32 - 1) {
            let (fluid, h) = self.fluid_height(ex, y, ez);
            if fluid != Fluid::Water || f64::from(y) + h < lo.y || eye_y < f64::from(y) {
                continue;
            }
            let state = self.world.block(ex, y, ez);
            let source = self
                .registry
                .state(state)
                .and_then(|s| s.properties.get("level"))
                .is_none_or(|l| l == "0");
            let above = self.facts.get(self.world.block(ex, y + 1, ez));
            let camera = if source && above.sturdy_full & 1 != 0 {
                1.0
            } else {
                h
            };
            if eye_y <= f64::from(y) + camera {
                out.eye_in_water = true;
            }
        }
        out
    }
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_42(
        &self,
        bounds: Aabb,
    ) -> (CurrentAccumulator, CurrentAccumulator) {
        let base_y = bounds.min_position().y;
        let b = bounds.deflate(Vec3::new(0.001, 0.001, 0.001));
        let (lo, hi) = (b.min_position(), b.max_position());
        let mut scene = FlowScene(self);
        let (mut water, mut lava) = (CurrentAccumulator::default(), CurrentAccumulator::default());
        for x in lo.x.floor() as i32..=(hi.x.ceil() as i32 - 1) {
            for y in lo.y.floor() as i32..=(hi.y.ceil() as i32 - 1) {
                for z in lo.z.floor() as i32..=(hi.z.ceil() as i32 - 1) {
                    let p = BlockPos::new(x, y, z);
                    let state = nocubes_extended(self, [x, y, z]);
                    let current = match state.kind() {
                        FluidKind::Empty => continue,
                        FluidKind::Water | FluidKind::FlowingWater => &mut water,
                        FluidKind::Lava | FluidKind::FlowingLava => &mut lava,
                    };
                    let surface = f64::from(y) + self.fluid_height(x, y, z).1;
                    if surface < lo.y {
                        continue;
                    }
                    current.height = current.height.max(surface - base_y);
                    let mut flow = flow_at(&mut scene, state, p).unwrap_or(Vec3::ZERO);
                    if current.height < 0.4 {
                        flow = flow.scale(current.height);
                    }
                    current.accumulate(flow);
                }
            }
        }
        (water, lava)
    }
}
impl MobWorld<'_> {
    pub(super) fn nocubes_query_43(&self, x: i32, y: i32, z: i32) -> Vec<Aabb> {
        let mut scene = SceneCollisions {
            world: self.world,
            binding: self.collision,
            subject: None,
        };
        let state = self.world.block(x, y, z);
        let at = horizon_core::position::BlockPos::new(x, y, z);
        let context = horizon_world::collision_context::CollisionContext::empty(false);
        let offset = Vec3::new(f64::from(x), f64::from(y), f64::from(z));
        horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
            self.collision
                .nocubes_collision_shape_with_context(&mut scene, state, at, &context)
        })
        .map(|shape| shape.boxes().iter().map(|b| b.move_by(offset)).collect())
        .unwrap_or_default()
    }
}
impl Mob {
    pub(super) fn nocubes_query_44(&mut self, w: &MobWorld, events: &mut Vec<MobEvent>) {
        let mut wanted = self.velocity;
        if self.stuck_multiplier.length_squared() > 1.0e-7 {
            wanted = wanted.multiply(self.stuck_multiplier);
            self.stuck_multiplier = Vec3::ZERO;
            self.velocity = Vec3::ZERO;
        }
        let step = self.kind.step_height as f32;
        let scene = SceneCollisions {
            world: w.world,
            binding: w.collision,
            subject: self.kind.flags.lava_walker.then(|| PowderSnowSubject {
                fall_distance: self.fall_distance,
                lava_walker: true,
                ..Default::default()
            }),
        };
        let border = w.border.map(|b| (b, [self.position.x, self.position.z]));
        let moved = horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
            scene.resolve_in_border(self.bounds(), wanted, step, self.on_ground, border)
        })
        .unwrap_or(Vec3::ZERO);
        let length = moved.x * moved.x + moved.y * moved.y + moved.z * moved.z;
        let wanted_length = wanted.x * wanted.x + wanted.y * wanted.y + wanted.z * wanted.z;
        if length > 1.0e-7 || wanted_length - length < 1.0e-7 {
            self.position = Vec3::new(
                self.position.x + moved.x,
                self.position.y + moved.y,
                self.position.z + moved.z,
            );
        }
        let differs = |a: f64, b: f64| (b - a).abs() >= f64::from(1.0e-5_f32);
        let x_hit = differs(wanted.x, moved.x);
        let z_hit = differs(wanted.z, moved.z);
        self.horizontal_collision = x_hit || z_hit;
        let moved_vertically = wanted.y.abs() > 0.0;
        let vertical = wanted.y != moved.y;
        self.vertical_collision_below = vertical && wanted.y < 0.0;
        self.on_ground = self.vertical_collision_below;
        if self.kind.flags.lava_walker && self.fluids.in_lava {
            self.fall_distance = 0.0;
        } else if !matches!(
            self.kind.species,
            super::super::Species::Bat | super::super::Species::Bee
        ) {
            self.check_fall_damage(w, moved, events);
        }
        if moved_vertically && vertical || self.horizontal_collision {
            let on = (
                self.position.x.floor() as i32,
                (self.position.y - f64::from(0.2_f32)).floor() as i32,
                self.position.z.floor() as i32,
            );
            let input = RestitutionInput {
                block: w.world.block(on.0, on.1, on.2),
                collided_x: x_hit,
                collided_z: z_hit,
                vertical_collision: vertical,
                collision_below: self.vertical_collision_below,
                living: true,
                resolved_movement: moved,
                fall_distance: self.fall_distance,
            };
            let _ = restitute_movement(
                &mut MobRestitution {
                    mob: self,
                    world: w,
                },
                input,
            );
        }
        self.supporting = self.supporting_block(w, moved);
        self.movement_emission(w, moved, events);
        let factor = self.block_speed_factor(w);
        self.velocity = horizon_world::entity_move::scale_by_block_speed(self.velocity, factor);
    }
}
fn nocubes_extended(world: &MobWorld<'_>, p: [i32; 3]) -> FluidState {
    let cfg = horizon_core::nocubes::config();
    horizon_core::nocubes::fluids::extended(
        p,
        cfg.extend_fluids_range,
        |p| horizon_core::nocubes::kinds::smoothable(&cfg, world.world.block(p[0], p[1], p[2]).0),
        |f: FluidState| f.kind() == FluidKind::Empty,
        |f: FluidState| f.is_source(),
        |p| {
            world
                .fluid_tables
                .fluid(world.world.block(p[0], p[1], p[2]))
        },
    )
}
