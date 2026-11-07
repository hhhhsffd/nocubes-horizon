use super::Config;
use std::{
    path::PathBuf,
    sync::{
        Arc, OnceLock, RwLock,
        atomic::{AtomicU64, Ordering},
    },
};
static REVISION: AtomicU64 = AtomicU64::new(1);
struct Settings {
    local: Arc<Config>,
    remote: Option<Config>,
    effective: Arc<Config>,
    disk: Option<Vec<u8>>,
    visual: Option<Config>,
}
fn path() -> PathBuf {
    std::env::var_os("NOCUBES_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| "config/nocubes.json".into())
}
fn parse(bytes: &[u8]) -> Result<Config, String> {
    let c: Config = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    c.validate()?;
    Ok(c)
}
fn store() -> &'static RwLock<Settings> {
    static STATE: OnceLock<RwLock<Settings>> = OnceLock::new();
    STATE.get_or_init(|| {
        let disk = std::fs::read(path()).ok();
        let c = disk
            .as_deref()
            .map(parse)
            .transpose()
            .unwrap_or_else(|e| {
                eprintln!("[NoCubes] configuration: {e}");
                Some(Config {
                    render: false,
                    collisions: false,
                    ..Config::default()
                })
            })
            .unwrap_or_default();
        let local = Arc::new(c);
        RwLock::new(Settings {
            local: local.clone(),
            effective: local,
            remote: None,
            disk,
            visual: None,
        })
    })
}
fn merged(local: &Config, remote: Option<&Config>) -> Config {
    let Some(server) = remote else {
        return local.clone();
    };
    let mut result = local.clone();
    result.mesher = server.mesher;
    result.use_defaults = server.use_defaults;
    result.whitelist = server.whitelist.clone();
    result.blacklist = server.blacklist.clone();
    result.state_overrides = server.state_overrides.clone();
    result.collisions = server.collisions;
    result.sampled_collisions = server.sampled_collisions;
    result.collision_resolution = server.collision_resolution;
    result.old_collision_enhancement = server.old_collision_enhancement;
    result.only_old_collisions = server.only_old_collisions;
    result.player_collisions_only = server.player_collisions_only;
    result.force_visuals = server.force_visuals;
    result.render |= server.force_visuals;
    result.extend_fluids_range = server.extend_fluids_range;
    result.old_in_fluids = server.old_in_fluids;
    result.roughness = server.roughness;
    result.old_slopes = server.old_slopes;
    result
}
fn refresh(s: &mut Settings) {
    let mut effective = merged(&s.local, s.remote.as_ref());
    if s.remote.is_none()
        && let Some(v) = &s.visual
    {
        effective.whitelist = v.whitelist.clone();
        effective.blacklist = v.blacklist.clone();
        effective.state_overrides = v.state_overrides.clone();
    }
    s.effective = Arc::new(effective);
    REVISION.fetch_add(1, Ordering::Release);
}
pub fn revision() -> u64 {
    REVISION.load(Ordering::Acquire)
}
pub fn config() -> Arc<Config> {
    let c = store()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .effective
        .clone();
    if enabled() {
        if CLIENT_CONNECTED.load(Ordering::Acquire) && !has_server_config() {
            Arc::new(Config {
                collisions: false,
                ..(*c).clone()
            })
        } else {
            c
        }
    } else {
        Arc::new(Config {
            render: false,
            collisions: false,
            extend_fluids_range: 0,
            debug: false,
            ..(*c).clone()
        })
    }
}

pub fn local_config() -> Arc<Config> {
    store()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .local
        .clone()
}
pub fn has_server_config() -> bool {
    store()
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .remote
        .is_some()
}
pub fn server_config_bytes() -> Result<Vec<u8>, String> {
    use std::io::Write;
    let json = serde_json::to_vec(&*local_config()).map_err(|e| e.to_string())?;
    if json.len() < 16000 {
        return Ok(json);
    }
    let mut out = flate2::write::ZlibEncoder::new(b"NCZ1".to_vec(), flate2::Compression::fast());
    out.write_all(&json).map_err(|e| e.to_string())?;
    let bytes = out.finish().map_err(|e| e.to_string())?;
    if bytes.len() > 30000 {
        return Err("configuration exceeds native message capacity after compression".into());
    }
    Ok(bytes)
}
pub fn apply_server_config(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() > 30000 {
        return Err("server configuration exceeds 30000 bytes".into());
    }
    let decoded;
    let bytes = if let Some(payload) = bytes.strip_prefix(b"NCZ1") {
        use std::io::Read;
        let mut value = Vec::new();
        flate2::read::ZlibDecoder::new(payload)
            .take(2_000_001)
            .read_to_end(&mut value)
            .map_err(|e| e.to_string())?;
        if value.len() > 2_000_000 {
            return Err("decompressed configuration exceeds limit".into());
        }
        decoded = value;
        decoded.as_slice()
    } else {
        bytes
    };
    let c = parse(bytes)?;
    let mut s = store().write().unwrap_or_else(|e| e.into_inner());
    s.remote = Some(c);
    s.visual = None;
    refresh(&mut s);
    Ok(())
}
pub fn clear_server_config() {
    let mut s = store().write().unwrap_or_else(|e| e.into_inner());
    s.remote = None;
    s.visual = None;
    CLIENT_CONNECTED.store(false, Ordering::Release);
    refresh(&mut s);
}
pub fn reload_local() -> Result<bool, String> {
    let disk = match std::fs::read(path()) {
        Ok(b) => Some(b),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.to_string()),
    };
    let mut s = store().write().unwrap_or_else(|e| e.into_inner());
    if s.disk == disk {
        return Ok(false);
    }
    let c = disk.as_deref().map(parse).transpose()?.unwrap_or_default();
    s.local = Arc::new(c);
    s.disk = disk;
    refresh(&mut s);
    Ok(true)
}
fn persist(destination: &std::path::Path, c: &Config) -> Result<Vec<u8>, String> {
    c.validate()?;
    let bytes = serde_json::to_vec_pretty(c).map_err(|e| e.to_string())?;
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let temp = destination.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&temp, &bytes)
        .and_then(|()| std::fs::rename(&temp, destination))
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}
pub fn update_local(update: impl FnOnce(&mut Config)) -> Result<(), String> {
    let mut s = store().write().unwrap_or_else(|e| e.into_inner());
    let mut c = (*s.local).clone();
    update(&mut c);
    c.validate()?;
    if c == *s.local {
        return Ok(());
    }
    let bytes = persist(&path(), &c)?;
    s.local = Arc::new(c);
    s.disk = Some(bytes);
    refresh(&mut s);
    Ok(())
}
/// Update one state selector, removing contradictory entries in either list.
fn apply_rule(c: &mut Config, selector: &str, value: bool) {
    if selector.contains('[') {
        c.state_overrides.insert(selector.into(), value);
    } else {
        c.state_overrides
            .retain(|k, _| k.split('[').next() != Some(selector));
        c.whitelist
            .retain(|k| k.split('[').next() != Some(selector));
        c.blacklist
            .retain(|k| k.split('[').next() != Some(selector));
        if value {
            c.whitelist.push(selector.into());
        } else {
            c.blacklist.push(selector.into());
        }
    }
}
pub fn set_smoothable(selector: &str, value: bool) -> Result<(), String> {
    super::ui::validate_selector(selector)?;
    update_local(|c| apply_rule(c, selector, value))
}
pub fn set_visual_smoothable(selector: &str, value: bool) -> Result<(), String> {
    super::ui::validate_selector(selector)?;
    let mut s = store().write().unwrap_or_else(|e| e.into_inner());
    let mut visual = (*s.effective).clone();
    apply_rule(&mut visual, selector, value);
    s.visual = Some(visual);
    refresh(&mut s);
    Ok(())
}
static CLIENT_CONNECTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn set_client_connected() {
    CLIENT_CONNECTED.store(true, Ordering::Release);
    REVISION.fetch_add(1, Ordering::Release);
}

static ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
pub fn set_enabled(enabled: bool) {
    if ENABLED.swap(enabled, Ordering::AcqRel) != enabled {
        REVISION.fetch_add(1, Ordering::Release);
    }
}
pub fn enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persisted_settings_round_trip_and_invalid_edits_preserve_file() {
        let root = std::env::temp_dir().join(format!("nocubes-config-test-{}", std::process::id()));
        let file = root.join("config/nocubes.json");
        let cfg = Config {
            extend_fluids_range: 2,
            grass_tufts: true,
            ..Config::default()
        };
        let bytes = persist(&file, &cfg).unwrap();
        assert_eq!(parse(&std::fs::read(&file).unwrap()).unwrap(), cfg);
        assert!(
            persist(
                &file,
                &Config {
                    old_collision_enhancement: 5,
                    ..cfg
                }
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&file).unwrap(), bytes);
        std::fs::remove_dir_all(root).unwrap();
    }
}
