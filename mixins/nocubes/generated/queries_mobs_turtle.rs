use super::*;
impl LayEgg {
    pub(super) fn nocubes_query_47(&mut self, m: &mut Mob, cx: &mut Cx) {
        self.0.tick(m, cx.w);
        if m.fluids.in_water || !self.0.reached {
            return;
        }
        if m.extra.counter < 1 {
            set_laying_egg(m, true);
        } else if m.extra.counter > 200 {
            let at = [m.position.x, m.position.y, m.position.z];
            let pitch = 0.9 + m.random.next_float() * 0.2;
            cx.events
                .push(MobEvent::Sound("entity.turtle.lay_egg", at, 0.3, pitch));
            let eggs = m.random.next_int_bound(4).unwrap_or(0) + 1;
            let registry = cx.w.registry;
            let state = registry
                .default_state("minecraft:turtle_egg")
                .and_then(|d| registry.state(d).cloned())
                .and_then(|mut s| {
                    s.properties.insert("eggs".into(), eggs.to_string());
                    horizon_client::collisions::nocubes_collision::with_nonplayer(|| {
                        registry.resolve(&s)
                    })
                    .ok()
                });
            let b = self.0.block;
            if let Some(state) = state {
                cx.events.push(MobEvent::SetBlock {
                    pos: [b[0], b[1] + 1, b[2]],
                    state,
                });
            }
            m.extra.flag = false;
            set_laying_egg(m, false);
            m.extra.in_love = 600;
        }
        if m.extra.flag2 {
            m.extra.counter += 1;
        }
    }
}
