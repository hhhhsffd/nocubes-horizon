use super::*;
use horizon_client::collisions::nocubes_collision::RegistryExt;
pub(super) fn nocubes_query_37(m: &mut Mob, w: &MobWorld, events: &mut Vec<MobEvent>) {
    let spawn = if m.species() == Species::IronGolem {
        let horizontal = m.velocity.x * m.velocity.x + m.velocity.z * m.velocity.z;
        horizontal > f64::from(2.500_000_3e-7_f32) && m.random.next_int_bound(5).unwrap_or(1) == 0
    } else {
        m.extra.sprinting && !m.fluids.in_water && !m.fluids.in_lava && m.alive()
    };
    if spawn {
        let at = m.block_on_legacy(w);
        let state = w.world.block(at[0], at[1], at[2]);
        if w.facts.get(state).render_shape != crate::blocks::RenderShape::Invisible {
            let _ = (m.random.next_double(), m.random.next_double());
        }
    }
    m.was_eye_in_water = m.fluids.eye_in_water;
    m.update_fluids(w, events);
    if m.species() == Species::Drowned {
        super::super::undead::drowned_update_swimming(m, w);
    }
    let fire = crate::motion::environment::fire_contact(w.world, w.registry, m.bounds());
    if fire > 0.0 {
        m.ticks_frozen = 0;
    }
    if m.fire_ticks > 0 && (m.fluids.in_water || in_rain(m, w)) {
        m.fire_ticks = 0;
        events.push(MobEvent::Sound(
            "entity.generic.extinguish_fire",
            m.center(),
            0.7,
            1.6 + (m.random.next_float() - m.random.next_float()) * 0.4,
        ));
    }
    if m.fire_ticks > 0 {
        if m.kind.flags.fire_immune {
            m.fire_ticks = 0;
        } else {
            if m.fire_ticks % 20 == 0 && !m.fluids.in_lava {
                m.damage(1.0, Damage::of(DamageKind::OnFire));
            }
            m.fire_ticks -= 1;
        }
    }
    if m.position.y < f64::from(w.world.bottom_y() - 64) {
        m.damage(4.0, Damage::of(DamageKind::OutOfWorld));
    }
    m.first_tick = false;
    if m.alive() {
        let mut scene = crate::collisions::SceneCollisions {
            world: w.world,
            binding: w.collision,
            subject: None,
        };
        if m.species() != Species::Vex
            && in_wall(
                m.width(),
                [m.position.x, m.eye_y(), m.position.z],
                |x, y, z, eye_bounds| {
                    let p = [x, y, z];
                    let state = w.world.block(x, y, z);
                    if !w
                        .suffocation
                        .is_suffocating(&mut scene, state, p)
                        .unwrap_or(false)
                    {
                        return false;
                    }
                    horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
                        w.collision.nocubes_collision_shape(&mut scene, state, p)
                    })
                    .ok()
                    .is_some_and(|shape| {
                        let Ok(eye_shape) = VoxelShape::from_box(*eye_bounds) else {
                            return false;
                        };
                        shape
                            .moved(Vec3::new(f64::from(x), f64::from(y), f64::from(z)))
                            .join_is_not_empty(&eye_shape, ShapeOp::AND)
                            .unwrap_or(false)
                    })
                },
            )
        {
            m.damage(1.0, Damage::of(DamageKind::InWall));
        }
        if m.species() == Species::Axolotl {
            axolotl_air_supply(m, w);
        } else if m.kind.flags.needs_water {
            if !m.fluids.in_water {
                m.air -= 1;
                if m.air == -20 {
                    m.air = 0;
                    m.damage(2.0, Damage::of(DamageKind::Drown));
                }
            } else {
                m.air = 300;
            }
        } else {
            let drowns = m.fluids.eye_in_water && !m.kind.flags.breathes_water;
            let vanilla = Breath {
                can_breathe: !drowns,
                consume: 1,
                refill: if drowns { 0 } else { 4 },
            };
            let breath = breathe(m.id, &m.kind.id, vanilla);
            if let Some(drowning) = drown(m.id, &m.kind.id, &mut m.air, breath, true) {
                events.push(MobEvent::Drown {
                    mob: m.id,
                    at: [m.position.x, m.position.y, m.position.z],
                });
                if let Some(damage) = drowning.damage {
                    m.damage(damage, Damage::of(DamageKind::Drown));
                }
            }
        }
        freeze_tick(m, w);
    }
    if m.species() == Species::Illusioner {
        super::super::illusioner::tick(m, events);
    }
    if matches!(m.species(), Species::Evoker | Species::Illusioner) {
        super::super::illager::cast_particles(m, events);
    }
    if m.hurt_time > 0 {
        m.hurt_time -= 1;
    }
    if m.invulnerable > 0 {
        m.invulnerable -= 1;
    }
    tick_effects(m, w.mod_effects);
    block_contact_damage(m, w, fire);
    enderman_hurt_teleports(m, w, events);
    if !m.alive() {
        m.death_time += 1;
    }
    events.append(&mut m.outbox);
    if m.alive() {
        let roll = m.random.next_int_bound(1000).unwrap_or(999);
        let t = m.ambient_timer;
        m.ambient_timer += 1;
        if roll < t {
            m.ambient_timer = -m.ambient_interval();
            play_ambient(m, w, events);
        }
    }
    super::super::leash::tick(m, w, events);
}
