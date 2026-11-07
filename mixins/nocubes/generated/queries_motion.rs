use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl MotionWorld<'_, '_, '_> {
    pub(super) fn nocubes_query_56(
        &mut self,
        from: Vec3,
        to: Vec3,
        visited: &mut crate::fast_hash::FxHashSet<i64>,
        entered: &mut Vec<(i32, BlockPos)>,
        max_iterations: i32,
        damage_only: bool,
    ) -> Result<i32> {
        let deflated = self.player.dimensions.bounds(to).deflate(Vec3::new(
            f64::from(1.0e-5_f32),
            f64::from(1.0e-5_f32),
            f64::from(1.0e-5_f32),
        ));
        let far = 0.999_990_000_000_252_6_f64;
        let moved_far = from.distance_squared(to) > far * far;
        let mut hits = Vec::new();
        let mut iterations = 0;
        let alive = !self.player.health.dead_or_dying();
        let (scene, registry) = (&*self.scene, self.assets.registry);
        horizon_core::geometry::sweep_intersected_blocks(
            from,
            to,
            deflated,
            1_000_000,
            |p, iteration| {
                if !alive || iteration >= max_iterations {
                    return false;
                }
                iterations = iteration;
                let state = scene.block(p.x, p.y, p.z);
                let air = registry.state(state).is_none_or(|d| {
                    matches!(
                        d.name.as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    )
                });
                if !air {
                    hits.push((iteration, p, state));
                }
                true
            },
        )
        .map_err(|e| format!("block sweep: {e:?}"))?;
        for (step, p, state) in hits {
            let Some(definition) = self.assets.registry.state(state) else {
                continue;
            };
            let name = definition.name.as_str();
            let powder_snow = name == "minecraft:powder_snow";
            let content_top = match name {
                "minecraft:lava_cauldron" => Some(15.0 / 16.0),
                "minecraft:water_cauldron" | "minecraft:powder_snow_cauldron" => {
                    let level = self
                        .assets
                        .registry
                        .state(state)
                        .and_then(|s| s.properties.get("level"))
                        .and_then(|l| l.parse::<u8>().ok())
                        .unwrap_or(1);
                    Some((6.0 + 3.0 * f64::from(level)) / 16.0)
                }
                _ => None,
            };
            if matches!(name, "minecraft:end_portal" | "minecraft:tripwire") {
                continue;
            }
            let inside = !(powder_snow || content_top.is_some()) || {
                let context = self.context();
                let binding = self.collision_binding;
                let shape = binding.nocubes_collision_shape_with_context(
                    &mut self.collisions(),
                    state,
                    p,
                    &context,
                )?;
                powder_snow && (shape.is_empty() || shape.boxes() == VoxelShape::block().boxes())
                    || {
                        let offset = Vec3::new(f64::from(p.x), f64::from(p.y), f64::from(p.z));
                        let mut boxes: Vec<_> = shape.moved(offset).boxes().to_vec();
                        if let Some(top) = content_top {
                            boxes.push(Aabb::new(
                                Vec3::new(2.0 / 16.0, 4.0 / 16.0, 2.0 / 16.0).add_vector(offset),
                                Vec3::new(14.0 / 16.0, top, 14.0 / 16.0).add_vector(offset),
                            ));
                        }
                        self.player
                            .dimensions
                            .bounds(from)
                            .collided_along_vector(to.subtract(from), &boxes)
                    }
            };
            if inside && visited.insert(p.pack()) {
                crate::block_contact::emit(crate::block_contact::Event::Inside(
                    crate::block_contact::Contact {
                        entity: self.player.entity_id,
                        kind: "minecraft:player",
                        block: name,
                        state,
                        position: p,
                    },
                ));
                match name {
                    "minecraft:cactus" => self.environment_hurt("minecraft:cactus", 1.0)?,
                    "minecraft:lava_cauldron" => self.lava_cauldron_inside()?,
                    "minecraft:water_cauldron" | "minecraft:powder_snow_cauldron" => {
                        self.layered_cauldron_inside(p);
                    }
                    "minecraft:wither_rose" => {
                        self.player.events.push(MotionEvent::InsideWitherRose);
                    }
                    "minecraft:campfire" | "minecraft:soul_campfire" => {
                        let lit = self
                            .assets
                            .registry
                            .state(state)
                            .and_then(|s| s.properties.get("lit"))
                            .is_some_and(|v| v == "true");
                        let damage = if name == "minecraft:campfire" {
                            1.0
                        } else {
                            2.0
                        };
                        if lit {
                            self.environment_hurt("minecraft:campfire", damage)?;
                        }
                    }
                    "minecraft:sweet_berry_bush" if damage_only => {
                        self.sweet_berry_bush_damage(state)?;
                    }
                    "minecraft:honey_block" => self.honey_block_inside(p)?,
                    "minecraft:powder_snow" if damage_only => {
                        self.powder_snow_snowflake(p);
                        entered.push((step, p));
                    }
                    _ => {}
                }
                self.player.events.push(MotionEvent::EnterBlock { state });
                if damage_only {
                    continue;
                }
                match name {
                    "minecraft:powder_snow" => {
                        self.powder_snow_inside(p)?;
                        entered.push((step, p));
                    }
                    "minecraft:cobweb" => self.web_inside(),
                    "minecraft:sweet_berry_bush" => self.sweet_berry_bush_inside(state)?,
                    "minecraft:bubble_column" => {
                        let precise = moved_far || deflated.intersects(Aabb::block(p));
                        self.bubble_column_inside(state, p, precise)?
                    }
                    "minecraft:nether_portal" => self
                        .player
                        .events
                        .push(MotionEvent::InsidePortal { pos: p }),
                    _ => {}
                }
            }
        }
        Ok(iterations + 1)
    }
}
impl MotionWorld<'_, '_, '_> {
    pub(super) fn nocubes_query_57(
        &mut self,
        p: BlockPos,
    ) -> Result<horizon_core::geometry::VoxelShape> {
        let context = self.context();
        let state = self.block(p);
        let binding = self.collision_binding;
        Ok(binding.nocubes_collision_shape_with_context(
            &mut self.collisions(),
            state,
            p,
            &context,
        )?)
    }
}
