use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl Game<'_> {
    fn nocubes_push_for_piston(
        &mut self,
        (pos, piston, new_progress): Push,
        accumulators: &mut std::collections::HashMap<Target, PistonMovement>,
    ) {
        let delta_progress = f64::from(new_progress - piston.progress);
        if delta_progress <= 0.0 {
            return;
        }
        let Some(bindings) = self.collision.pistons() else {
            return;
        };
        let Ok(state) = bindings.collision_related_state(piston.shape_state()) else {
            return;
        };
        let mut scene = SceneCollisions {
            world: &self.world,
            binding: self.collision,
            subject: None,
        };
        let Ok(shape) =
            self.collision
                .nocubes_collision_shape(&mut scene, state, [pos.x, pos.y, pos.z])
        else {
            return;
        };
        if shape.is_empty() {
            return;
        }
        let movement = piston.movement_direction();
        let boxes: Vec<Aabb> = shape.boxes().to_vec();
        let facts = self.res.facts();
        let slime = &*facts.get(piston.moved_state).name == "minecraft:slime_block";
        let honey = &*facts.get(piston.moved_state).name == "minecraft:honey_block";
        let sticky_top = if honey {
            let mut scene = SceneCollisions {
                world: &self.world,
                binding: self.collision,
                subject: None,
            };
            self.collision
                .nocubes_collision_shape(&mut scene, piston.moved_state, [pos.x, pos.y, pos.z])
                .ok()
                .and_then(|s| s.bounds())
                .map_or(0.0, |b| b.max_position().y)
        } else {
            0.0
        };
        let steps = self.push_step_heights();
        let search = shape.bounds().map(|bounds| {
            piston_push::movement_search_area(
                piston_push::move_by_position_and_progress(pos, bounds, piston.shape_state()),
                movement,
                delta_progress,
            )
        });
        self.push_targets().for_each(|target| {
            let Some(body) = self.push_body(target, pos, &steps) else {
                return;
            };
            let amount = piston_push::collided_amount(
                &boxes,
                pos,
                piston.shape_state(),
                new_progress,
                movement,
                body.bounds,
            );
            if slime && search.is_some_and(|area| area.intersects(body.bounds)) {
                self.bounce(target, movement);
            }
            if amount > 0.0 {
                let step = piston_push::step_vector(movement);
                let before = body.bounds;
                self.move_pushed(
                    target,
                    before,
                    [body.position.x, body.position.z],
                    body.step_height,
                    body.on_ground,
                    step.scale(amount),
                    movement,
                    accumulators,
                );
                if !piston.extending
                    && piston.source
                    && let Some(after) = self.push_body(target, pos, &steps)
                    && let Some(fix) = piston_push::fix_within_piston_base(
                        pos,
                        after.bounds,
                        movement,
                        delta_progress,
                    )
                {
                    let back = piston_push::step_vector(movement.opposite());
                    self.move_pushed(
                        target,
                        after.bounds,
                        [after.position.x, after.position.z],
                        after.step_height,
                        after.on_ground,
                        back.scale(fix),
                        movement,
                        accumulators,
                    );
                }
            }
        });
        if honey && movement.normal()[1] == 0 {
            let area = piston_push::sticky_area(pos, piston.shape_state(), sticky_top);
            for target in self.push_targets() {
                let Some(body) = self.push_body(target, pos, &steps) else {
                    continue;
                };
                if piston_push::sticky_entity_selected(
                    area,
                    body.bounds,
                    horizon_world::level::data::PushReaction::PushPull,
                    body.on_ground,
                    body.supported,
                    body.position,
                ) {
                    self.move_pushed(
                        target,
                        body.bounds,
                        [body.position.x, body.position.z],
                        body.step_height,
                        body.on_ground,
                        piston_push::step_vector(movement).scale(delta_progress),
                        movement,
                        accumulators,
                    );
                }
            }
        }
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_25(&mut self, pushes: Vec<Push>) {
        let mut accumulators = std::collections::HashMap::new();
        for push in pushes {
            self.nocubes_push_for_piston(push, &mut accumulators);
        }
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_26(
        &mut self,
        pos: BlockPos,
        old: horizon_world::registry::BlockStateId,
    ) {
        let new = self.world.block(pos.x, pos.y, pos.z);
        if new == old {
            return;
        }
        let at = [pos.x, pos.y, pos.z];
        let shape = |state| {
            let mut scene = SceneCollisions {
                world: &self.world,
                binding: self.collision,
                subject: None,
            };
            self.collision
                .nocubes_collision_shape(&mut scene, state, at)
                .ok()
        };
        let (Some(old_shape), Some(new_shape)) = (shape(old), shape(new)) else {
            return;
        };
        let Ok(added) = old_shape.join_unoptimized(&new_shape, ShapeOp::ONLY_SECOND) else {
            return;
        };
        let added = added.moved(Vec3::new(
            f64::from(pos.x),
            f64::from(pos.y),
            f64::from(pos.z),
        ));
        let Some(bounds) = added.bounds() else {
            return;
        };
        let steps = self.push_step_heights();
        for target in self.push_targets() {
            let Some(body) = self.push_body(target, pos, &steps) else {
                continue;
            };
            if !body.bounds.intersects(bounds) {
                continue;
            }
            let dy = added.collide(Axis::Y, body.bounds.move_by(Vec3::new(0.0, 1.0, 0.0)), -1.0);
            self.apply_push(target, Vec3::new(0.0, 1.0 + dy, 0.0));
        }
    }
}
