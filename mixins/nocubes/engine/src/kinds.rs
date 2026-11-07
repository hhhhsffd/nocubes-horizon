use std::{
    collections::HashMap,
    sync::{Arc, OnceLock, RwLock},
};
#[derive(Clone, Copy, Debug, Default)]
pub struct BlockKind {
    pub plant: bool,
    pub short_plant: bool,
    pub replaceable: bool,
    pub snow: bool,
    pub vine: bool,
    pub single_sided: bool,
}
/// State facts used by the original OldNoCubes and Wulferis predicates.
#[derive(Clone, Copy, Debug, Default)]
pub struct StateKind {
    pub full_block: bool,
    pub air: bool,
    pub plant: bool,
    pub short_plant: bool,
    pub snow_layer: bool,
    pub source_liquid_block: bool,
    pub replaceable: bool,
    pub vine: bool,
}
impl StateKind {
    pub fn default_smooth(self, legacy_default: bool) -> bool {
        !self.air && !self.plant && !self.vine && (self.full_block || legacy_default)
    }
    pub fn legacy_solid(self, in_fluids: bool) -> bool {
        !(self.air || self.plant || self.snow_layer || in_fluids && self.source_liquid_block)
    }
    pub fn wulferis_barrier(self, globally_smooth: bool) -> bool {
        (!self.replaceable && !globally_smooth && !self.short_plant && !self.air) || self.vine
    }
}
fn table() -> &'static RwLock<Arc<HashMap<String, BlockKind>>> {
    static T: OnceLock<RwLock<Arc<HashMap<String, BlockKind>>>> = OnceLock::new();
    T.get_or_init(|| RwLock::new(Arc::new(HashMap::new())))
}
pub fn install(kinds: HashMap<String, BlockKind>) {
    *table().write().unwrap_or_else(|e| e.into_inner()) = Arc::new(kinds);
}
pub fn snapshot() -> Arc<HashMap<String, BlockKind>> {
    table().read().unwrap_or_else(|e| e.into_inner()).clone()
}
type Resolver = std::sync::Arc<dyn Fn(&super::Config, u32) -> super::Sample + Send + Sync>;
#[derive(Clone, Default)]
pub struct Samples(Arc<[super::Sample]>, Arc<[bool]>);
impl Samples {
    pub fn visual_smooth(&self) -> &[bool] {
        &self.1
    }
    pub fn same_registry(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn get(&self, id: u32) -> super::Sample {
        self.0
            .get(id as usize)
            .copied()
            .unwrap_or(super::Sample::AIR)
    }
}
struct StateResolver {
    resolve: Resolver,
    count: usize,
    cached: Option<(Arc<super::Config>, Samples)>,
}
impl StateResolver {
    fn matches(&self, c: &super::Config) -> bool {
        self.cached
            .as_ref()
            .is_some_and(|(old, _)| std::ptr::eq(old.as_ref(), c))
    }
    fn samples(&mut self, c: &Arc<super::Config>) -> Samples {
        if !self.matches(c) {
            let values: Arc<[super::Sample]> = (0..self.count)
                .map(|id| (self.resolve)(c, id as u32))
                .collect();
            let smooth = values.iter().map(|s| c.render && s.smooth).collect();
            self.cached = Some((c.clone(), Samples(values, smooth)));
        }
        self.cached.as_ref().unwrap().1.clone()
    }
}
fn resolver() -> &'static RwLock<Option<StateResolver>> {
    static R: OnceLock<RwLock<Option<StateResolver>>> = OnceLock::new();
    R.get_or_init(|| RwLock::new(None))
}
pub fn install_resolver(
    count: usize,
    resolve: impl Fn(&super::Config, u32) -> super::Sample + Send + Sync + 'static,
) {
    *resolver().write().unwrap_or_else(|e| e.into_inner()) = Some(StateResolver {
        resolve: Arc::new(resolve),
        count,
        cached: None,
    });
}
/// Resolve block selectors once per registry/configuration, then sample without locks.
pub fn samples(c: &Arc<super::Config>) -> Samples {
    {
        let r = resolver().read().unwrap_or_else(|e| e.into_inner());
        if let Some(r) = r.as_ref() {
            if r.matches(c) {
                return r.cached.as_ref().unwrap().1.clone();
            }
        } else {
            return Samples::default();
        }
    }
    resolver()
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .as_mut()
        .map_or_else(Samples::default, |r| r.samples(c))
}
pub fn sample(c: &super::Config, id: u32) -> super::Sample {
    resolver()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map_or(super::Sample::AIR, |r| {
            if r.matches(c) {
                r.cached.as_ref().unwrap().1.get(id)
            } else {
                (r.resolve)(c, id)
            }
        })
}

pub fn smoothable(c: &super::Config, id: u32) -> bool {
    sample(c, id).smooth
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_table_reuses_snapshot_and_rebuilds_after_configuration_change() {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let count = calls.clone();
        let mut resolver = StateResolver {
            count: 3,
            cached: None,
            resolve: Arc::new(move |c, id| {
                count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                super::super::Sample {
                    state: id,
                    smooth: c.use_defaults,
                    density: if c.use_defaults { 1. } else { -1. },
                    solid: false,
                }
            }),
        };
        let config = Arc::new(super::super::Config::default());
        let first = resolver.samples(&config);
        let second = resolver.samples(&config.clone());
        assert!(first.same_registry(&second));
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 3);
        assert!(first.get(1).smooth);
        assert!(!first.get(999).smooth);
        let changed = Arc::new(super::super::Config {
            use_defaults: false,
            ..(*config).clone()
        });
        let third = resolver.samples(&changed);
        assert!(!first.same_registry(&third));
        assert!(!third.get(1).smooth);
        assert!(first.get(1).smooth);
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 6);
        let hidden = Arc::new(super::super::Config {
            render: false,
            ..(*config).clone()
        });
        let hidden = resolver.samples(&hidden);
        assert!(hidden.get(1).smooth);
        assert!(!hidden.visual_smooth()[1]);
    }
}
