use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
#[doc = " `Level.clip` with `ClipContext.Block.VISUAL` and no fluids:"]
#[doc = " `BlockBehaviour.getVisualShape` is the collision shape except for the"]
#[doc = " blocks that override it."]
struct VisualRays<'a, 'r> {
    scene: SceneCollisions<'a, 'r>,
    facts: &'a BlockFacts,
    context: CollisionContext<'static>,
}
impl RayWorld for VisualRays<'_, '_> {
    type Error = io::Error;
    type Block = BlockStateId;
    type Fluid = ();
    fn block_state(&mut self, p: BlockPos) -> io::Result<BlockStateId> {
        Ok(self.scene.world.block(p.x, p.y, p.z))
    }
    fn fluid_state(&mut self, _: BlockPos) -> io::Result<()> {
        Ok(())
    }
    fn block_shape(
        &mut self,
        state: BlockStateId,
        p: BlockPos,
        _: RayBlockMode,
    ) -> io::Result<VoxelShape> {
        let facts = self.facts.get(state);
        let name = &*facts.name;
        if name.ends_with("glass")
            || name.ends_with("glass_pane")
            || name.ends_with("_bars")
            || name.ends_with("copper_grate")
            || name == "minecraft:powder_snow"
        {
            return Ok(VoxelShape::empty());
        }
        if name == "minecraft:soul_sand" || name == "minecraft:mud" {
            return Ok(VoxelShape::block());
        }
        if name == "minecraft:snow" || name.ends_with("_fence") {
            let mut shape = VoxelShape::empty();
            for b in facts.outline.iter() {
                let b = b.map(f64::from);
                let part = VoxelShape::from_box(Aabb::new(
                    Vec3::new(b[0], b[1], b[2]),
                    Vec3::new(b[3], b[4], b[5]),
                ))
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                shape = shape
                    .join(&part, ShapeOp::OR)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
            }
            return Ok(shape);
        }
        let binding = self.scene.binding;
        binding
            .nocubes_collision_shape_with_context(&mut self.scene, state, p, &self.context)
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
    fn interaction_shape(&mut self, _: BlockStateId, _: BlockPos) -> io::Result<VoxelShape> {
        Ok(VoxelShape::empty())
    }
    fn fluid_is_source(&mut self, _: ()) -> io::Result<bool> {
        Ok(false)
    }
    fn fluid_is_empty(&mut self, _: ()) -> io::Result<bool> {
        Ok(true)
    }
    fn fluid_is_water(&mut self, _: ()) -> io::Result<bool> {
        Ok(false)
    }
    fn fluid_shape(&mut self, _: (), _: BlockPos) -> io::Result<VoxelShape> {
        Ok(VoxelShape::empty())
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_24(&mut self, partial: f32) -> Option<(glam::DVec3, f32, f32)> {
        let (entity, descending) = match self.camera_mob() {
            Some(m) => (super::super::remote_camera::mob_camera_entity(m), false),
            None => {
                let mut entity = self.player_camera_entity()?;
                if let Some(position) = self.remote_minecart_rider_position(partial) {
                    entity.position = position;
                    entity.previous = position;
                }
                (entity, self.player.as_ref()?.shift_key_down)
            }
        };
        let mut view = self.view;
        let pull = self.emote_camera_pull(partial);
        let kind = match self.camera_type {
            CameraType::FirstPerson => horizon_client::camera::CameraType::FirstPerson,
            CameraType::ThirdPersonBack => horizon_client::camera::CameraType::ThirdPersonBack,
            CameraType::ThirdPersonFront => horizon_client::camera::CameraType::ThirdPersonFront,
        };
        let context = CollisionContext::for_entity(
            CollisionEntity {
                identity: EntityIdentity(0),
                y: entity.position[1],
                descending,
                main_hand: Some(CollisionItem::EMPTY),
            },
            false,
        );
        let mut rays = VisualRays {
            scene: SceneCollisions {
                world: &self.world,
                binding: self.collision,
                subject: None,
            },
            facts: self.res.facts(),
            context,
        };
        view.align_with_entity_distance(
            &entity,
            partial,
            kind,
            |distance| {
                let mut distance = distance * pull;
                self.native_camera_distance(partial, kind.is_mirrored(), &entity, &mut distance);
                distance
            },
            |from, to| {
                let hit = clip_world(
                    &mut rays,
                    Vec3::new(from[0], from[1], from[2]),
                    Vec3::new(to[0], to[1], to[2]),
                    RayBlockMode::Visual,
                    RayFluidMode::None,
                    4096,
                )
                .ok()?;
                (!hit.miss).then_some([hit.hit.location.x, hit.hit.location.y, hit.hit.location.z])
            },
        );
        self.view = view;
        Some((
            glam::DVec3::from_array(view.position),
            view.y_rot,
            view.x_rot,
        ))
    }
}
