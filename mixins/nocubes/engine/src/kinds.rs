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
    pub air: bool,
    pub plant: bool,
    pub short_plant: bool,
    pub snow_layer: bool,
    pub source_liquid_block: bool,
    pub replaceable: bool,
    pub vine: bool,
}
impl StateKind {
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
fn resolver() -> &'static RwLock<Option<Resolver>> {
    static R: OnceLock<RwLock<Option<Resolver>>> = OnceLock::new();
    R.get_or_init(|| RwLock::new(None))
}
pub fn install_resolver(
    resolve: impl Fn(&super::Config, u32) -> super::Sample + Send + Sync + 'static,
) {
    *resolver().write().unwrap_or_else(|e| e.into_inner()) = Some(std::sync::Arc::new(resolve));
}
pub fn sample(c: &super::Config, id: u32) -> super::Sample {
    resolver()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map_or(super::Sample::AIR, |r| r(c, id))
}

pub fn smoothable(c: &super::Config, id: u32) -> bool {
    sample(c, id).smooth
}
