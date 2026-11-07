use super::*;
use std::sync::{Mutex, OnceLock};
#[derive(Default)]
pub struct ScreenState {
    pub page: usize,
    pub edit: Option<(String, String)>,
    pub error: Option<String>,
}
pub fn screen() -> &'static Mutex<ScreenState> {
    static S: OnceLock<Mutex<ScreenState>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(ScreenState::default()))
}
pub fn text(key: &str) -> String {
    static EN: OnceLock<std::collections::BTreeMap<String, String>> = OnceLock::new();
    EN.get_or_init(|| {
        serde_json::from_str(include_str!("../data/en_us.json")).expect("NoCubes locale")
    })
    .get(key)
    .cloned()
    .unwrap_or_else(|| {
        key.strip_prefix("nocubes.config.")
            .unwrap_or(key)
            .replace('_', " ")
    })
}
pub fn server_field(key: &str) -> bool {
    matches!(
        key,
        "state_overrides"
            | "collisions"
            | "mesher"
            | "use_defaults"
            | "whitelist"
            | "blacklist"
            | "roughness"
            | "old_slopes"
            | "old_in_fluids"
            | "sampled_collisions"
            | "collision_resolution"
            | "old_collision_enhancement"
            | "only_old_collisions"
            | "player_collisions_only"
            | "force_visuals"
            | "extend_fluids_range"
    )
}
pub fn set_setting(key: &str, value: serde_json::Value) -> Result<(), String> {
    let mut value_all = serde_json::to_value(&*local_config()).map_err(|e| e.to_string())?;
    let obj = value_all.as_object_mut().ok_or("configuration object")?;
    if !obj.contains_key(key) {
        return Err("unknown setting".into());
    }
    obj.insert(key.into(), value);
    let next: Config = serde_json::from_value(value_all).map_err(|e| e.to_string())?;
    next.validate()?;
    update_local(|c| *c = next)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SmoothEdit {
    pub selector: String,
    pub smooth: bool,
}
pub fn validate_selector(s: &str) -> Result<(), String> {
    if s.len() > 2048 || s.is_empty() || s.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err("invalid block selector".into());
    }
    let (name, props) = s.split_once('[').unwrap_or((s, ""));
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_:/.-".contains(c))
    {
        return Err("invalid block name".into());
    }
    if !props.is_empty()
        && !props.strip_suffix(']').is_some_and(|p| {
            p.split(',').all(|kv| {
                kv.split_once('=')
                    .is_some_and(|(k, v)| !k.is_empty() && !v.is_empty())
            })
        })
    {
        return Err("invalid block properties".into());
    }
    Ok(())
}
pub fn edit_from_bytes(data: &[u8]) -> Result<SmoothEdit, String> {
    if data.len() > 4096 {
        return Err("smoothability request too large".into());
    }
    let e: SmoothEdit = serde_json::from_slice(data).map_err(|e| e.to_string())?;
    validate_selector(&e.selector)?;
    Ok(e)
}
pub fn binding_matches(binding: &str, key: &str, modifiers: u32) -> bool {
    let mut parts = binding.split('+').collect::<Vec<_>>();
    let Some(name) = parts.pop() else {
        return false;
    };
    let mut expected = 0;
    for p in parts {
        match p.to_ascii_lowercase().as_str() {
            "shift" => expected |= 1,
            "ctrl" | "control" => expected |= 2,
            "alt" => expected |= 4,
            "super" | "meta" => expected |= 8,
            _ => return false,
        }
    }
    expected == modifiers & 15 && name.eq_ignore_ascii_case(key)
}
pub fn edit_authorized(permission: u8, owner: bool) -> bool {
    owner || permission >= 2
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selectors_reject_invalid_network_input() {
        for s in ["", "stone[bad]", "stone[a=]", "stone\n", "Stone"] {
            assert!(validate_selector(s).is_err());
        }
        assert!(validate_selector("minecraft:stone[axis=x]").is_ok());
        assert!(edit_from_bytes(&vec![0; 4097]).is_err());
    }
}
