use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
impl GameLevel<'_> {
    pub(super) fn nocubes_query_31(&self, pos: BlockPos, state: BlockStateId) -> bool {
        let player = self.player_collision.as_ref();
        let context = player.map_or_else(
            || CollisionContext::empty(false),
            |player| {
                CollisionContext::for_entity(
                    CollisionEntity {
                        identity: EntityIdentity(0),
                        y: player.y,
                        descending: player.descending,
                        main_hand: Some(CollisionItem {
                            item: &player.main_hand_item,
                            count: player.main_hand_count,
                        }),
                    },
                    false,
                )
            },
        );
        let mut scene = collisions::SceneCollisions {
            world: &*self.world,
            binding: self.collision,
            subject: player.map(|player| horizon_client::collisions::PowderSnowSubject {
                fall_distance: player.fall_distance,
                falling_block: false,
                leather_boots: player.leather_boots,
                walkable_type: false,
                lava_walker: false,
            }),
        };
        let Ok(shape) = self
            .collision
            .nocubes_collision_shape_with_context(&mut scene, state, pos, &context)
        else {
            return false;
        };
        if shape.is_empty() {
            return true;
        }
        let shape = shape.moved(horizon_core::geometry::Vec3::new(
            f64::from(pos.x),
            f64::from(pos.y),
            f64::from(pos.z),
        ));
        horizon_world::entity_obstruction::is_unobstructed(
            &mut BuildingObstruction(self.building_blocker_boxes()),
            None,
            &shape,
        )
        .unwrap_or(false)
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_32(&self) -> [Option<f64>; 2] {
        match self.player.as_ref() {
            Some(p) if !p.spectator && self.riding.is_none() => {
                closest_space_nudge(p, &self.world, self.res.facts())
            }
            _ => [None; 2],
        }
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_33(
        &mut self,
        camera: glam::DVec3,
        alpha: f64,
        fov_degrees: f32,
        mut out: Vec<Vertex>,
    ) -> (
        Vec<Vertex>,
        Arc<[StaticSectionSnapshot]>,
        Vec<[i32; 3]>,
        StaticCacheFrameStats,
    ) {
        let mut laps = horizon_client::spans::laps();
        let mut static_block_entities = std::sync::Arc::from([]);
        let mut visible_static_block_entity_sections = Vec::new();
        let mut static_block_entity_stats = StaticCacheFrameStats::default();
        self.entity_spans.borrow_mut().clear();
        self.entity_motion.borrow_mut().clear();
        self.drain_transient_block_queue();
        self.prepare_entity_culling();
        self.prepare_occlusion_culling(camera);
        self.prepare_trims();
        self.begin_skinning();
        self.mark_entity(out.len(), "minecraft:entity_flame");
        let cracks = self.cracks(camera);
        let fire_yaw = if self.camera_type == perspective::CameraType::ThirdPersonFront {
            self.yaw + 180.0
        } else {
            self.yaw
        };
        let fire_view = self.view_frustum.as_ref().filter(|_| {
            self.pack_ids.is_none()
                && self.native_shaders.picked.is_empty()
                && !self.settings.horizon.taa
        });
        let fire_visible = |position, width, height| {
            fire_view.is_none_or(|view| {
                horizon_client::fire_render::in_view(view, camera, position, width, height)
            })
        };
        laps.lap("eg_prep");
        for m in &self.mobs {
            let blazing = m.species() == horizon_client::mobs::Species::Blaze && m.extra.flag;
            if (m.is_on_fire() || blazing) && !m.removed {
                let at = m.previous.lerp(m.position, alpha);
                if !fire_visible([at.x, at.y, at.z], m.width(), m.height()) {
                    continue;
                }
                horizon_client::fire_render::flames(
                    [at.x, at.y, at.z],
                    m.width(),
                    m.height(),
                    camera,
                    fire_yaw,
                    &self.gui.atlas.sprites,
                    self.world.light(
                        at.x.floor() as i32,
                        at.y.floor() as i32,
                        at.z.floor() as i32,
                    ),
                    &mut out,
                );
            }
        }
        if self.own_body_in_view()
            && let Some(p) = &self.player
            && p.fire_ticks > 0
            && !p.spectator
        {
            let feet = self
                .previous_position
                .lerp(p.position, self.clock.camera_fraction());
            if fire_visible(
                [feet.x, feet.y, feet.z],
                p.dimensions.width,
                p.dimensions.height,
            ) {
                horizon_client::fire_render::flames(
                    [feet.x, feet.y, feet.z],
                    p.dimensions.width,
                    p.dimensions.height,
                    camera,
                    fire_yaw,
                    &self.gui.atlas.sprites,
                    self.world.light(
                        feet.x.floor() as i32,
                        feet.y.floor() as i32,
                        feet.z.floor() as i32,
                    ),
                    &mut out,
                );
            }
        }
        laps.lap("eg_fire_mobs");
        for item in &self.dropped {
            if item.fire_ticks > 0
                && !self
                    .res
                    .items()
                    .resists_fire(&item.stack, "minecraft:in_fire")
            {
                let at = item.previous.lerp(item.position, alpha);
                if !fire_visible([at.x, at.y, at.z], 0.25, 0.25) {
                    continue;
                }
                horizon_client::fire_render::flames(
                    [at.x, at.y, at.z],
                    0.25,
                    0.25,
                    camera,
                    fire_yaw,
                    &self.gui.atlas.sprites,
                    self.world.light(
                        at.x.floor() as i32,
                        at.y.floor() as i32,
                        at.z.floor() as i32,
                    ),
                    &mut out,
                );
            }
        }
        laps.lap("eg_fire_items");
        for stand in &self.armor_stands {
            if stand.fire_ticks > 0 && !stand.marker {
                let scale = if stand.small { 0.5 } else { 1.0 };
                let at = stand.render_position(alpha);
                if !fire_visible([at.x, at.y, at.z], 0.5 * scale, 1.975 * scale) {
                    continue;
                }
                horizon_client::fire_render::flames(
                    [at.x, at.y, at.z],
                    0.5 * scale,
                    1.975 * scale,
                    camera,
                    fire_yaw,
                    &self.gui.atlas.sprites,
                    self.world.light(
                        at.x.floor() as i32,
                        at.y.floor() as i32,
                        at.z.floor() as i32,
                    ),
                    &mut out,
                );
            }
        }
        laps.lap("eg_fire_stands");
        self.mark_entity(out.len(), "minecraft:item");
        self.dropped_item_geometry(camera, alpha, &mut out);
        self.mark_motion(out.len(), 0);
        self.mark_entity(out.len(), "");
        self.vault_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "");
        self.campfire_geometry(camera, &mut out);
        self.brushable_geometry(camera, &mut out);
        self.shelf_geometry(camera, &mut out);
        self.mark_entity(out.len(), "minecraft:ominous_item_spawner");
        self.ominous_item_spawner_geometry(camera, alpha, &mut out);
        laps.lap("eg_items_misc");
        self.mob_geometry(camera, alpha, &mut out);
        self.mark_motion(out.len(), 0);
        laps.lap("eg_mobs");
        self.mark_player(out.len(), !self.own_body_in_view());
        let player_start = out.len();
        self.player_model_geometry(camera, self.clock.camera_fraction(), &mut out);
        if self.pack_ids.is_none() && !self.own_body_in_view() {
            self.shadow_only_vertices
                .borrow_mut()
                .extend(out.drain(player_start..));
        }
        self.cosmetics_fx_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "minecraft:evoker_fangs");
        self.fangs_geometry(camera, alpha, &mut out);
        self.block_item_display_geometry(camera, alpha, &mut out);
        laps.lap("eg_players");
        let mut see_through = std::mem::take(&mut self.see_through_vertices);
        see_through.clear();
        self.name_tag_geometry(camera, alpha, &mut out, &mut see_through);
        self.test_instance_marker_geometry(camera, &mut out, &mut see_through);
        self.debug_renderer_geometry(camera, &mut out);
        self.text_display_geometry(camera, alpha, &mut out, &mut see_through);
        self.see_through_vertices = see_through;
        laps.lap("eg_name_tags");
        self.mark_translucent_entity(out.len(), "minecraft:entity_shadow");
        let pack_shadows = self.renderer.as_ref().is_some_and(|r| {
            r.pack_entity_shadows() || (self.dim == Dim::Overworld && r.native_shadow())
        });
        let shadow_sprite = (self.settings.entity_shadows && !pack_shadows)
            .then(|| self.gui.atlas.sprites.get("misc/shadow").copied())
            .flatten();
        if let Some(sprite) = shadow_sprite {
            let ambient = match self.dim {
                Dim::Nether => 0.1,
                Dim::End => 0.25,
                Dim::Overworld => 0.0,
            };
            let darken = self.sky_darken();
            let a = alpha;
            let lerp = |p: Vec3, q: Vec3| {
                glam::DVec3::new(
                    p.x + (q.x - p.x) * a,
                    p.y + (q.y - p.y) * a,
                    p.z + (q.z - p.z) * a,
                )
            };
            let mut casters: Vec<(glam::DVec3, f32, f32)> =
                horizon_client::shadows::mob_casters(&self.mobs, a).collect();
            let item = horizon_client::shadows::shadow_params("minecraft:item");
            casters.extend(
                self.dropped
                    .iter()
                    .map(|i| (lerp(i.previous, i.position), item.0, item.1)),
            );
            let orb = horizon_client::shadows::shadow_params("minecraft:experience_orb");
            casters.extend(
                self.orbs
                    .iter()
                    .map(|o| (lerp(o.previous, o.position), orb.0, orb.1)),
            );
            casters.extend(self.text_displays.values().filter_map(|d| d.shadow(a)));
            let shadow_view = self
                .view_frustum
                .as_ref()
                .filter(|_| self.pack_ids.is_none() && self.native_shaders.picked.is_empty());
            for (pos, radius, strength) in casters {
                if let Some(view) = shadow_view
                    && !horizon_client::shadows::in_view(view, camera, pos, radius, strength)
                {
                    continue;
                }
                horizon_client::shadows::shadow(
                    &self.world,
                    self.res.facts(),
                    &|state| {
                        self.res
                            .collision()
                            .cached_shapes()
                            .cached_shape(state)
                            .ok()
                            .flatten()
                            .and_then(|shape| shape.bounds())
                    },
                    sprite,
                    camera,
                    pos,
                    radius,
                    strength,
                    ambient,
                    darken,
                    &mut out,
                );
            }
        }
        laps.lap("eg_shadows");
        if let Some(r) = &self.renderer {
            let view = self.view_frustum;
            let off_screen = |state: u32| {
                let name = &*self.res.facts().get(BlockStateId(state)).name;
                matches!(
                    name,
                    "minecraft:beacon"
                        | "minecraft:end_gateway"
                        | "minecraft:structure_block"
                        | "minecraft:test_instance_block"
                )
            };
            let long_range = |state: u32| {
                &*self.res.facts().get(BlockStateId(state)).name == "minecraft:end_gateway"
            };
            let in_view = |p: &[i32; 3]| {
                view.is_none_or(|v| {
                    let o = (glam::DVec3::new(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]))
                        - camera)
                        .as_vec3();
                    v.intersects(o - glam::Vec3::ONE, o + glam::Vec3::splat(2.0))
                })
            };
            let camera_list: Vec<&([i32; 3], u32)> = r
                .visible_block_entities_in_frustum(
                    self.settings.chunk_fade,
                    camera,
                    view,
                    &off_screen,
                    long_range,
                )
                .filter(|(block, section_inside)| {
                    let (pos, state) = **block;
                    self.res.block_entity_render().renders(state)
                        && (off_screen(state) || *section_inside || in_view(&pos))
                })
                .map(|(block, _)| block)
                .collect();
            let shadow_only: Vec<&([i32; 3], u32)> =
                if self.pack_ids.is_some() && self.pack_shadow_casters.0 {
                    let listed: HashSet<[i32; 3]> = camera_list.iter().map(|b| b.0).collect();
                    r.shadow_pass_block_entities(camera)
                        .filter(|(pos, state)| {
                            !listed.contains(pos) && self.res.block_entity_render().renders(*state)
                        })
                        .collect()
                } else {
                    Vec::new()
                };
            let shadow_only_at: HashSet<[i32; 3]> = shadow_only.iter().map(|b| b.0).collect();
            let dynamic_visible = |pos: [i32; 3], state: u32, lo: glam::DVec3, hi: glam::DVec3| {
                if off_screen(state) || shadow_only_at.contains(&pos) {
                    return true;
                }
                if let Some(view) = view
                    && !view.intersects(
                        (lo - camera - glam::DVec3::splat(0.5)).as_vec3(),
                        (hi - camera + glam::DVec3::splat(0.5)).as_vec3(),
                    )
                {
                    return false;
                }
                let name = &*self.res.facts().get(BlockStateId(state)).name;
                if self.settings.horizon.entity_culling == EntityCullingMode::Off
                    || self.pack_shadow_casters.0
                    || name == "minecraft:end_portal"
                    || name == "minecraft:end_gateway"
                    || hi.y - lo.y > 2.0
                {
                    return true;
                }
                let bounds = Aabb::new(
                    Vec3::new(lo.x + 0.06, lo.y + 0.06, lo.z + 0.06),
                    Vec3::new(hi.x - 0.06, hi.y - 0.06, hi.z - 0.06),
                );
                !self.block_entity_occluded(bounds)
            };
            let lid = |p: [i32; 3]| self.chest_lid(p, alpha as f32);
            let anim = |p: [i32; 3]| self.block_anim(p, alpha as f32);
            let scene = BlockEntityScene {
                camera,
                orientation: self.camera_orientation(),
                anim: &anim,
                texts: None,
                partial: alpha as f32,
                game_time: self.world.game_time,
                max_y: self.world.shape.max_y() - 1,
                world: &self.world,
                sprites: &self.gui.atlas.sprites,
                emissive: &self.gui.atlas.emissive,
                translucent_sprites: &self.gui.atlas.translucent,
                atlas_revision: self.gui.atlas.revision,
                nether: self.dim == Dim::Nether,
                lid: &lid,
                sign: &|p| match self.world.block_entities.get(&(p[0], p[1], p[2])) {
                    Some(BlockEntity::Sign(sign)) => Some(sign),
                    _ => None,
                },
                banner: &|p| self.banner_layers_at(p),
                skull: &|p| self.skull_skin_at(p),
                glyph: &|c| self.gui.atlas.glyph_cell(c),
                glyphs: self.gui.atlas.font.epoch(),
                dynamic_visible: Some(&dynamic_visible),
            };
            let visible = camera_list.into_iter().chain(shadow_only.iter().copied());
            let block_entity_cache = CacheOptions {
                shulker_boxes: self.settings.horizon.cache_shulker_boxes,
                signs: self.settings.horizon.cache_signs,
                decorated_pots: self.settings.horizon.cache_decorated_pots,
                bells: self.settings.horizon.cache_bells,
                chests: self.settings.horizon.cache_chests,
                banners: self.settings.horizon.cache_banners,
                copper_golem_statues: self.settings.horizon.cache_copper_golem_statues,
            };
            if self.pack_ids.is_some() {
                let section_revision = |pos| r.block_entity_section_revision(pos);
                static_block_entity_stats = self
                    .res
                    .block_entity_render()
                    .render_with_cache_options_and_callbacks(
                        &scene,
                        visible,
                        block_entity_cache,
                        r.block_entity_revision(),
                        Some(&section_revision),
                        &mut out,
                        |_, state, start| self.mark_block_entity(start, state),
                        |pos, _, start, out| {
                            if shadow_only_at.contains(&pos) {
                                self.hide_spans_from(start, out.len());
                            }
                        },
                    );
            } else {
                let cache_candidates = r
                    .visible_block_entities_in_frustum(
                        self.settings.chunk_fade,
                        camera,
                        None,
                        |_| false,
                        |_| false,
                    )
                    .filter(|(block, _)| self.res.block_entity_render().renders(block.1))
                    .map(|(block, _)| block);
                (
                    static_block_entities,
                    visible_static_block_entity_sections,
                    static_block_entity_stats,
                ) = self
                    .res
                    .block_entity_render()
                    .render_with_gpu_cache_options(
                        &scene,
                        visible,
                        cache_candidates,
                        block_entity_cache,
                        r.block_entity_revision(),
                        |pos| r.block_entity_section_revision(pos),
                        &mut out,
                    );
            }
            let mods = self
                .world
                .block_entities
                .has_mods()
                .then(|| {
                    r.visible_block_entities(self.settings.chunk_fade)
                        .filter(|(_, state)| !self.res.block_entity_render().renders(*state))
                })
                .into_iter()
                .flatten();
            let untemplated = self.untemplated_mod_block_entities(camera);
            self.mod_block_entity_geometry(
                mods.chain(untemplated.iter()),
                camera,
                alpha as f32,
                &mut out,
            );
            self.mark_stage(out.len(), "DESTROY");
            for &(pos, stage) in &cracks {
                let state = self.world.block(pos[0], pos[1], pos[2]);
                self.res
                    .block_entity_render()
                    .render_breaking(&scene, pos, state.0, stage, &mut out);
            }
        }
        laps.lap("eg_block_entities");
        self.mark_entity(out.len(), "minecraft:experience_orb");
        if let Some(r) = self
            .gui
            .atlas
            .sprites
            .get("entity/experience/experience_orb")
            .copied()
        {
            let (yaw, pitch) = (self.yaw.to_radians(), self.pitch.to_radians());
            let forward = glam::Vec3::new(
                -yaw.sin() * pitch.cos(),
                -pitch.sin(),
                yaw.cos() * pitch.cos(),
            );
            let right = forward.cross(glam::Vec3::Y).normalize_or_zero();
            let up = right.cross(forward).normalize_or_zero();
            let live = self.orbs.iter().map(|o| {
                let at = [
                    o.previous.x + (o.position.x - o.previous.x) * alpha,
                    o.previous.y + (o.position.y - o.previous.y) * alpha,
                    o.previous.z + (o.position.z - o.previous.z) * alpha,
                ];
                (o, at, alpha as f32)
            });
            let taken = self.pickups.live.iter().filter_map(|p| match &p.taken {
                pickups::Taken::Orb(o) => Some((&**o, pickups::position(p, alpha), 1.0)),
                _ => None,
            });
            for (o, at, partial) in live.chain(taken) {
                let p = glam::DVec3::new(at[0], at[1] + 0.1, at[2]) - camera;
                let icon = o.icon();
                let (du, dv) = ((r.u1 - r.u0) / 4.0, (r.v1 - r.v0) / 4.0);
                let (u0, v0) = (r.u0 + (icon % 4) as f32 * du, r.v0 + (icon / 4) as f32 * dv);
                let t = (o.age as f32 + partial) / 2.0;
                let red = ((t.sin() + 1.0) * 0.5 * 255.0) as u32;
                let blue = (((t + 4.188_79).sin() + 1.0) * 0.1 * 255.0) as u32;
                let color = 0x8000_0000 | blue << 16 | 255 << 8 | red;
                let light = self.world.light(
                    o.position.x.floor() as i32,
                    o.light_probe_y().floor() as i32,
                    o.position.z.floor() as i32,
                );
                let packed = o.render_light(i32::from(light));
                let c = p.as_vec3();
                let s = horizon_client::entities::ORB_QUAD_SCALE;
                for (k, uv) in horizon_client::entities::ORB_QUAD.iter().zip([
                    [u0, v0 + dv],
                    [u0 + du, v0 + dv],
                    [u0 + du, v0],
                    [u0, v0],
                ]) {
                    let v = c + right * (k[0] * s) + up * (k[1] * s);
                    out.push(Vertex {
                        normal: [0; 4],
                        pos: v.to_array(),
                        uv,
                        layer: horizon_client::gfx::OIT_ENTITY
                            | u32::from(horizon_client::mesher::VERY_BRIGHT) << 16,
                        color,
                        light: packed,
                        glint_uv: [0.0, 0.0],
                    });
                }
            }
        }
        laps.lap("eg_orbs");
        self.mark_entity(out.len(), "minecraft:falling_block");
        for f in &self.falling {
            let (x, y, z) = (
                f.position.x.floor() as i32,
                f.position.y.floor() as i32,
                f.position.z.floor() as i32,
            );
            if !falling_block_visible(f.state, self.world.block(x, y, z)) {
                continue;
            }
            let Some((_, tint)) = self.block_face_sprite(f.state) else {
                continue;
            };
            let tables = &self.pack.context.tables;
            let p = f.model_origin(alpha, |state, x, z| tables.offset(state, x, z)) - camera;
            let o = p.as_vec3();
            let light = self.world.light(x, y + 1, z);
            let packed = (u32::from(light >> 4) * 16) | ((u32::from(light & 15) * 16) << 8);
            let start = out.len();
            self.block_model_vertices(
                f.state,
                horizon_core::random::position_seed(f.start_pos),
                glam::Mat4::from_translation(o),
                packed,
                false,
                &mut out,
            );
            let tint = horizon_client::mobs::render::abgr(tint);
            for vertex in &mut out[start..] {
                vertex.color = horizon_client::hud_layout::multiply(vertex.color, tint);
            }
        }
        self.mark_entity(out.len(), "");
        self.moving_piston_geometry(camera, alpha, &mut out);
        self.transient_block_geometry(camera, &mut out);
        self.mark_entity(out.len(), "minecraft:tnt");
        self.tnt_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "minecraft:painting");
        self.painting_geometry(camera, &mut out);
        self.mark_entity(out.len(), "minecraft:item_frame");
        self.frame_geometry(camera, &mut out);
        self.mark_entity(out.len(), "minecraft:armor_stand");
        self.armor_stand_geometry(camera, alpha, &mut out);
        self.cushion_geometry(camera, &mut out);
        self.mark_entity(out.len(), "minecraft:oak_boat");
        self.boat_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "minecraft:minecart");
        self.cart_geometry(camera, alpha, &mut out);
        self.mark_stage(out.len(), "NONE");
        self.beacon_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "minecraft:fishing_bobber");
        self.fishing_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "minecraft:leash_knot");
        self.leash_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "minecraft:trident");
        self.trident_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "minecraft:firework_rocket");
        self.rocket_geometry(camera, alpha, &mut out);
        self.mark_entity(out.len(), "minecraft:splash_potion");
        self.thrown_potion_geometry(camera, alpha, &mut out);
        laps.lap("eg_objects");
        self.mark_stage(out.len(), "PARTICLES");
        let world = &self.world;
        let particle_mode = self.settings.horizon.particle_culling;
        if particle_mode == horizon_client::graphics::ParticleCullingMode::Off {
            self.particles.render(
                camera,
                self.yaw,
                self.pitch,
                alpha as f32,
                &|b| world.light(b[0], b[1], b[2]),
                &mut out,
            );
        } else {
            let frustum = self.view_frustum;
            let pixel_focal_length = self.renderer.as_ref().map_or(0.0, |renderer| {
                let tangent = (f64::from(fov_degrees).to_radians() * 0.5).tan();
                if tangent.is_finite() && tangent > 0.0 {
                    (f64::from(renderer.config.height.max(1)) / (2.0 * tangent)) as f32
                } else {
                    0.0
                }
            });
            {
                let mut particle_culling = self.particle_occlusion_context(camera);
                self.particles.render_culled(
                    camera,
                    self.yaw,
                    self.pitch,
                    alpha as f32,
                    pixel_focal_length,
                    true,
                    frustum.as_ref(),
                    &mut |min, max| particle_culling.hidden(min, max),
                    &|b| world.light(b[0], b[1], b[2]),
                    &mut out,
                );
            }
            self.finish_async_particle_culling();
        }
        laps.lap("eg_particles");
        self.mark_stage(out.len(), "DESTROY");
        for &(pos, stage) in &cracks {
            let o = (glam::DVec3::new(f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2]))
                - camera)
                .as_vec3();
            let state = self.world.block(pos[0], pos[1], pos[2]);
            if self.res.facts().get(state).render_shape
                == horizon_client::blocks::RenderShape::Model
                && let Ok(models) = self
                    .pack
                    .render
                    .model_catalog
                    .collect(state, BlockPos::new(pos[0], pos[1], pos[2]))
            {
                let offset = self
                    .pack
                    .context
                    .tables
                    .offset(state, pos[0], pos[2])
                    .unwrap_or([0.0; 3]);
                let origin = o + glam::DVec3::from(offset).as_vec3();
                let translucent = models
                    .iter()
                    .any(|(_, model)| model.faces.iter().any(|f| f.translucent));
                for (_, model) in models {
                    self.nocubes_crack_geometry(pos, model, origin, translucent, stage, &mut out);
                }
            }
        }
        laps.lap("eg_cracks");
        self.finish_async_occlusion_culling();
        laps.lap("eg_culling_finish");
        self.native_world_geometry(camera, alpha as f32, &mut out);
        (
            out,
            static_block_entities,
            visible_static_block_entity_sections,
            static_block_entity_stats,
        )
    }
}
impl Game<'_> {
    pub(super) fn nocubes_query_34(&mut self, pos: [i32; 3]) {
        let Some((position, yaw)) = self.player.as_ref().map(|p| (p.position, p.yaw)) else {
            return;
        };
        let state = self.world.block(pos[0], pos[1], pos[2]);
        let props = self
            .pack
            .assets
            .registry
            .state(state)
            .map(|s| s.properties.clone())
            .unwrap_or_default();
        let d = facing_offset(props.get("facing").map_or("north", String::as_str));
        let (head, foot) = if props.get("part").map(String::as_str) == Some("foot") {
            ([pos[0] + d[0], pos[1], pos[2] + d[2]], pos)
        } else {
            (pos, [pos[0] - d[0], pos[1], pos[2] - d[2]])
        };
        let head_state = self.world.block(head[0], head[1], head[2]);
        let occupied = self
            .pack
            .assets
            .registry
            .state(head_state)
            .is_some_and(|s| s.properties.get("occupied").is_some_and(|v| v == "true"));
        if occupied {
            if !self.kick_villager_out_of_bed(head) {
                self.tell_sleeper("block.minecraft.bed.occupied", "This bed is occupied", true);
            }
            return;
        }
        let reachable = |bed: [i32; 3]| {
            (position.x - f64::from(bed[0]) - 0.5).abs() <= 3.0
                && (position.y - f64::from(bed[1])).abs() <= 2.0
                && (position.z - f64::from(bed[2]) - 0.5).abs() <= 3.0
        };
        let mut problem: Option<&'static str> = if !reachable(head) && !reachable(foot) {
            Some("too_far_away")
        } else {
            None
        };
        let blocked = problem.is_none()
            && [head, foot].iter().any(|bed| {
                let _f = self
                    .res
                    .facts()
                    .get(self.world.block(bed[0], bed[1] + 1, bed[2]));
                crate::game::nocubes_suffocation::occupied(
                    &self.world,
                    self.res.facts(),
                    [bed[0], bed[1] + 1, bed[2]],
                    self.player.as_ref().unwrap().bounds(),
                )
            });
        if blocked {
            problem = Some("obstructed");
        }
        if problem.is_none() && self.bed_sets_spawn(state) {
            let config = respawn_anchor::RespawnConfig::wrapped(self.dim, head, yaw, false);
            let moved = self
                .respawn_config
                .is_none_or(|c| (c.dim, c.pos) != (config.dim, config.pos));
            let mut cancel = false;
            if moved
                && let (Some(player), Some(name)) =
                    (self.lan_acting_uuid, self.lan_acting_name.clone())
            {
                self.native_player_spawn_point_set(
                    player,
                    &name,
                    self.dim.id(),
                    head,
                    "bed",
                    &mut cancel,
                );
            }
            if !cancel {
                self.respawn_config = Some(config);
                if moved {
                    self.tell_sleeper("block.minecraft.set_spawn", "Respawn point set", false);
                }
                self.spawn = if self.lan_acting.is_some() {
                    None
                } else {
                    self.level
                        .world_spawn
                        .map(|[x, y, z]| {
                            Vec3::new(f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5)
                        })
                        .or(self.spawn)
                };
            }
        }
        if problem.is_none() && !(self.dim == Dim::Overworld && self.sky_darken() >= 4) {
            problem = Some("not_possible_now");
        }
        if problem.is_none()
            && !self.creative
            && self.mobs.iter().any(|m| {
                horizon_client::mobs::ai::is_monster(m.species())
                    && (m.species() != horizon_client::mobs::Species::ZombifiedPiglin
                        || (m.extra.anger > 0 && m.ai.target.is_some()))
                    && m.alive()
                    && (m.position.x - position.x).abs() < 8.0
                    && (m.position.y - position.y).abs() < 5.0
                    && (m.position.z - position.z).abs() < 8.0
            })
        {
            problem = Some("not_safe");
        }
        let mut problem = problem.map(str::to_owned);
        if let (Some(player), Some(name)) = (self.lan_acting_uuid, self.lan_acting_name.clone()) {
            self.native_can_player_sleep(player, &name, head, &mut problem);
        }
        if let Some(problem) = problem {
            match problem.as_str() {
                "too_far_away" => self.tell_sleeper(
                    "block.minecraft.bed.too_far_away",
                    "You may not rest now; the bed is too far away",
                    true,
                ),
                "obstructed" => self.tell_sleeper(
                    "block.minecraft.bed.obstructed",
                    "This bed is obstructed",
                    true,
                ),
                "not_safe" => self.tell_sleeper(
                    "block.minecraft.bed.not_safe",
                    "You may not rest now; there are monsters nearby",
                    true,
                ),
                "not_possible_now" => self.tell_sleeper(
                    "block.minecraft.bed.no_sleep",
                    "You can sleep only at night or during thunderstorms",
                    true,
                ),
                _ => {}
            }
            return;
        }
        if self.ui.rules.int("players_sleeping_percentage") > 100 {
            self.tell_sleeper(
                "sleep.not_possible",
                "No amount of rest can pass this night",
                true,
            );
        }
        if let (Some(player), Some(name)) = (self.lan_acting_uuid, self.lan_acting_name.clone()) {
            let mut cancel = false;
            self.native_player_bed_enter(player, &name, head, &mut cancel);
            if cancel {
                return;
            }
        }
        let Some(at) = self.sleeping_position(head) else {
            return;
        };
        self.stop_riding();
        self.stop_mining();
        self.left = false;
        self.right = false;
        self.using = None;
        self.sleeping = Some(head);
        self.sleep_timer = 0;
        if let Some(p) = self.player.as_mut() {
            p.position = at;
            p.velocity = Vec3::ZERO;
            p.pitch = 0.0;
            p.teleported = true;
        }
        self.pitch = 0.0;
        if self.lan_acting.is_none() {
            self.previous_position = at;
            self.snap_player_model_pitch();
        }
        self.occupy_bed(head);
        self.award_custom_stat("sleep_in_bed", 1);
        self.trigger_player("minecraft:slept_in_bed");
        self.stats.reset("custom", "minecraft:time_since_rest");
    }
}
pub(super) fn nocubes_query_35(
    p: &PlayerMotion,
    world: &World,
    facts: &BlockFacts,
) -> [Option<f64>; 2] {
    use horizon_core::geometry::Axis;
    let mut out = [None; 2];
    let body = p.bounds();
    let (min_y, max_y) = (body.min(Axis::Y) + 1.0e-7, body.max(Axis::Y) - 1.0e-7);
    let suffocates_at = |x: i32, z: i32| {
        ((min_y.floor() as i32 - 1)..=(max_y.ceil() as i32 + 1)).any(|y| {
            f64::from(y) < max_y && f64::from(y + 1) > min_y && {
                let _f = facts.get(world.block(x, y, z));
                crate::game::nocubes_suffocation::occupied(world, facts, [x, y, z], body)
            }
        })
    };
    let side = f64::from(p.dimensions.width) * 0.35;
    let (px, pz) = (p.position.x, p.position.z);
    for (x, z) in [
        (px - side, pz + side),
        (px - side, pz - side),
        (px + side, pz - side),
        (px + side, pz + side),
    ] {
        let (bx, bz) = (x.floor() as i32, z.floor() as i32);
        if !suffocates_at(bx, bz) {
            continue;
        }
        let (xd, zd) = (x - f64::from(bx), z - f64::from(bz));
        let mut closest = 1.797_693_134_862_315_7e308;
        let mut chosen = None;
        for (axis, step, distance) in [(0, -1, xd), (0, 1, 1.0 - xd), (1, -1, zd), (1, 1, 1.0 - zd)]
        {
            let (nx, nz) = if axis == 0 {
                (bx + step, bz)
            } else {
                (bx, bz + step)
            };
            if distance < closest && !suffocates_at(nx, nz) {
                closest = distance;
                chosen = Some((axis, step));
            }
        }
        if let Some((axis, step)) = chosen {
            out[axis] = Some(0.1 * f64::from(step));
        }
    }
    out
}
