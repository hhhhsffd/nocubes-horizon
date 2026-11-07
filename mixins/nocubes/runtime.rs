use super::*;
use horizon_client::text::Text;
use horizon_core::nocubes;
use std::sync::atomic::{AtomicU64, Ordering};
static REGISTRY_SEEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static CLIENT_SEEN: AtomicU64 = AtomicU64::new(0);
static SERVER_SEEN: AtomicU64 = AtomicU64::new(0);
static LAST_RELOAD: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
fn translated(key: &str) -> String {
    let value = horizon_client::lang::get(key);
    if value == key {
        nocubes::ui::text(key)
    } else {
        value
    }
}
fn rows() -> Vec<(String, serde_json::Value)> {
    let mut rows: Vec<_> = serde_json::to_value(&*nocubes::config())
        .expect("finite configuration")
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    rows.sort_by_key(|(key, _)| key != "block_toggle_key");
    rows
}
impl Game<'_> {
    fn nocubes_settings_open(&self) -> bool {
        if self.native_mod_screen() == Some("nocubes") {
            return true;
        }
        if self.screen != Screen::Options {
            return false;
        }
        let horizon_client::options_ui::Page::NativeMod(index) = self.ui.options.page() else {
            return false;
        };
        self.settings
            .mods
            .installed
            .get(index)
            .is_some_and(|m| m.id == "nocubes")
    }
    pub(super) fn nocubes_remesh(&mut self) {
        if let Some(lod) = &mut self.lod {
            lod.nocubes_sync();
        }
        for (&(x, z), c) in &self.world.chunks {
            for sy in 0..c.sections.len() {
                self.world.dirty_sections.insert((x, sy as i32, z));
            }
        }
    }
    pub(super) fn nocubes_tick(&mut self) {
        let registry = self.pack.assets.registry as *const _ as usize;
        if REGISTRY_SEEN.swap(registry, Ordering::AcqRel) != registry {
            horizon_client::mesher::nocubes_registry::install(self.res.facts(), &self.pack.assets);
        }
        nocubes::set_enabled(self.native_on("nocubes"));
        let now = std::time::Instant::now();
        let mut last = LAST_RELOAD.lock().unwrap_or_else(|e| e.into_inner());
        if last.is_none_or(|t| now.duration_since(t).as_secs() >= 1) {
            *last = Some(now);
            if let Err(e) = nocubes::reload_local() {
                eprintln!("[NoCubes] {e}");
            }
        }
        drop(last);
        let revision = nocubes::revision();
        if CLIENT_SEEN.swap(revision, Ordering::AcqRel) != revision {
            self.nocubes_remesh();
        }
        if self.remote.is_some() {
            if self.native_on("nocubes") && !nocubes::has_server_config() {
                let mut last = HELLO.lock().unwrap_or_else(|e| e.into_inner());
                if last.is_none_or(|t| now.duration_since(t).as_secs() >= 5) {
                    *last = Some(now);
                    let _ = self.native_net_send_to_server("nocubes:hello", &[]);
                }
            }
        } else if SERVER_SEEN.swap(revision, Ordering::AcqRel) != revision {
            if let Ok(data) = nocubes::server_config_bytes() {
                for p in self.native_server_players() {
                    let _ = self.native_net_send_to_player(p.uuid, "nocubes:config", &data);
                }
            }
        }
    }
    pub(super) fn nocubes_key(&mut self, key: &str, pressed: bool) {
        if !self.native_on("nocubes") || !pressed || self.screen != Screen::Playing {
            return;
        }
        let Some(key) = horizon_client::keybinds::key_code(key) else {
            return;
        };
        let cfg = nocubes::config();
        let key = format!("{key:?}");
        if nocubes::ui::binding_matches(&cfg.block_toggle_key, &key, glfw_modifiers(self.modifiers))
        {
            self.nocubes_toggle_block();
        }
    }
    pub(super) fn nocubes_toggle_block(&mut self) {
        let Some(hit) = self.pick() else {
            return;
        };
        let id = self.world.block(hit.pos[0], hit.pos[1], hit.pos[2]);
        let Some(state) = self.pack.assets.registry.state(id) else {
            return;
        };
        let cfg = nocubes::config();
        let smooth = !nocubes::kinds::smoothable(&cfg, id.0);
        let selector = state.name.clone();
        if self.remote.is_none() {
            if let Err(e) = nocubes::set_smoothable(&selector, smooth) {
                eprintln!("[NoCubes] {e}");
            }
        } else if nocubes::has_server_config() {
            let request = nocubes::ui::SmoothEdit { selector, smooth };
            if let Ok(bytes) = serde_json::to_vec(&request) {
                let _ = self.native_net_send_to_server("nocubes:edit", &bytes);
            }
        } else if let Err(e) = nocubes::set_visual_smoothable(&selector, smooth) {
            eprintln!("[NoCubes] {e}");
        }
    }
    pub(super) fn nocubes_from_client(&mut self, player: u128, channel: &str, data: &[u8]) {
        if !self.native_on("nocubes")
            || !matches!(
                channel,
                "nocubes:hello" | "nocubes:edit" | "nocubes:setting"
            )
        {
            return;
        }
        if channel == "nocubes:hello" {
            if let Ok(data) = nocubes::server_config_bytes() {
                let _ = self.native_net_send_to_player(player, "nocubes:config", &data);
            }
            return;
        }
        let owner = self
            .lan
            .as_ref()
            .is_some_and(|h| h.hub.singleplayer_owner == Some(player));
        let level = self
            .native_server_players()
            .into_iter()
            .find(|p| p.uuid == player)
            .map_or(0, |p| p.permission);
        let permission = nocubes::ui::edit_authorized(level, owner);
        if !permission {
            let _ = self.native_tell(
                player,
                Text::translate_with_fallback(
                    "nocubes.command.changeSmoothableNoPermission",
                    &translated("nocubes.command.changeSmoothableNoPermission"),
                    &[],
                ),
            );
            return;
        }
        let result = if channel == "nocubes:edit" {
            nocubes::ui::edit_from_bytes(data).and_then(|edit| {
                // Refuse selectors for blocks absent from this server's registry.
                let name = edit.selector.split('[').next().unwrap_or("");
                if self.pack.assets.registry.default_state(name).is_none() {
                    return Err("unknown block".into());
                }
                nocubes::set_smoothable(&edit.selector, edit.smooth)
            })
        } else {
            if data.len() > 30000 {
                return;
            }
            serde_json::from_slice::<(String, serde_json::Value)>(data)
                .map_err(|e| e.to_string())
                .and_then(|(key, value)| {
                    if !nocubes::ui::server_field(&key) {
                        return Err("client-only setting".into());
                    }
                    nocubes::ui::set_setting(&key, value)
                })
        };
        if let Err(e) = result {
            eprintln!("[NoCubes] refused setting: {e}");
            return;
        }
        if let Ok(bytes) = nocubes::server_config_bytes() {
            for p in self.native_server_players() {
                let _ = self.native_net_send_to_player(p.uuid, "nocubes:config", &bytes);
            }
        }
    }
    pub(super) fn nocubes_screen(
        &self,
        batch: &mut HudBatch,
        screen: &crate::mod_api::NativeScreenView<'_>,
    ) {
        if !self.nocubes_settings_open() {
            return;
        }
        let state = nocubes::ui::screen()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let [x, y, w, h] = screen.rect;
        self.native_fill(batch, [x, y, w, h], 0xee18202a);
        self.native_text(
            batch,
            &translated("nocubes.ui.title"),
            [x + 12., y + 10.],
            0xffffff,
            true,
        );
        let rows = rows();
        let size = ((h - 70.) / 15.).max(1.) as usize;
        let start = state.page * size;
        for (i, (key, value)) in rows.iter().skip(start).take(size).enumerate() {
            let label = translated(&format!("nocubes.config.{key}"));
            let display = if key == "block_toggle_key" {
                if state.binding {
                    translated("nocubes.ui.binding_wait")
                } else if value.as_str().is_none_or(str::is_empty) {
                    translated("nocubes.ui.unbound")
                } else {
                    value.as_str().unwrap().to_owned()
                }
            } else {
                value.to_string()
            };
            let line = format!("{label}: {display}");
            let line = if line.chars().count() > 80 {
                format!("{}…", line.chars().take(79).collect::<String>())
            } else {
                line
            };
            self.native_text(
                batch,
                &line,
                [x + 12., y + 30. + i as f32 * 15.],
                if nocubes::ui::server_field(key) {
                    0xaaccff
                } else {
                    0xffffff
                },
                false,
            );
        }
        let footer = if state.binding {
            translated("nocubes.ui.binding_help")
        } else if let Some((key, value)) = &state.edit {
            format!("{key}: {value}")
        } else {
            translated("nocubes.ui.help")
        };
        self.native_text(batch, &footer, [x + 12., y + h - 30.], 0xcccccc, false);
        if let Some(error) = &state.error {
            self.native_text(batch, error, [x + 12., y + h - 16.], 0xff6666, false);
        }
    }
    pub(super) fn nocubes_screen_click(
        &self,
        screen: &crate::mod_api::NativeScreenView<'_>,
        button: i32,
        canceled: &mut bool,
    ) {
        if !self.nocubes_settings_open() {
            return;
        }
        *canceled = true;
        let mut state = nocubes::ui::screen()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let [x, y, w, h] = screen.rect;
        let [mx, my] = screen.mouse;
        if mx < x || mx > x + w {
            return;
        }
        let size = ((h - 70.) / 15.).max(1.) as usize;
        let rows = rows();
        if my > y + h - 40. {
            if button == 1 {
                state.page = state.page.saturating_sub(1);
            } else {
                state.page = (state.page + 1) % rows.len().div_ceil(size);
            }
            return;
        }
        let index = ((my - y - 30.) / 15.).floor() as isize;
        if index < 0 || index as usize >= size {
            return;
        }
        let Some((key, value)) = rows.get(state.page * size + index as usize) else {
            return;
        };
        state.error = None;
        state.binding = key == "block_toggle_key";
        if state.binding {
            state.edit = None;
            return;
        }
        state.edit = Some((
            key.clone(),
            if let Some(b) = value.as_bool() {
                (!b).to_string()
            } else {
                value.to_string()
            },
        ));
        // Booleans can be changed with a single click; text/numeric fields use Enter.
        if value.is_boolean() {
            drop(state);
            self.nocubes_commit_setting();
        }
    }
    pub(super) fn nocubes_commit_setting(&self) {
        let edit = nocubes::ui::screen()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .edit
            .take();
        let Some((key, text)) = edit else {
            return;
        };
        let result = serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|e| e.to_string())
            .and_then(|value| {
                if nocubes::has_server_config() && nocubes::ui::server_field(&key) {
                    // Shared read-only input hooks queue requests for the next mutable tick.
                    PENDING
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push((key, value));
                    Ok(())
                } else {
                    nocubes::ui::set_setting(&key, value)
                }
            });
        nocubes::ui::screen()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .error = result.err();
    }
    pub(super) fn nocubes_screen_key(
        &self,
        key: &str,
        repeat: bool,
        modifiers: u32,
        canceled: &mut bool,
    ) {
        if !self.nocubes_settings_open() {
            return;
        }
        let mut state = nocubes::ui::screen()
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.binding {
            *canceled = true;
            if repeat {
                return;
            }
            if key == "key.keyboard.escape" {
                state.binding = false;
                return;
            }
            let binding = if matches!(key, "key.keyboard.backspace" | "key.keyboard.delete") {
                Some(String::new())
            } else {
                horizon_client::keybinds::key_code(key)
                    .and_then(|code| nocubes::ui::binding_chord(&format!("{code:?}"), modifiers))
            };
            if let Some(binding) = binding {
                state.binding = false;
                drop(state);
                let result = nocubes::ui::set_setting("block_toggle_key", binding.into());
                nocubes::ui::screen()
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .error = result.err();
            }
            return;
        }
        drop(state);
        if key == "key.keyboard.enter" {
            self.nocubes_commit_setting();
            *canceled = true;
        } else if key == "key.keyboard.backspace" {
            if let Some((_, s)) = &mut nocubes::ui::screen()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .edit
            {
                s.pop();
                *canceled = true;
            }
        }
    }
    pub(super) fn nocubes_char(&self, c: char, canceled: &mut bool) {
        if !self.nocubes_settings_open() {
            return;
        }
        if nocubes::ui::screen()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .binding
        {
            *canceled = true;
            return;
        }
        if let Some((_, s)) = &mut nocubes::ui::screen()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .edit
        {
            if s.len() < 30000 && !c.is_control() {
                s.push(c);
            }
            *canceled = true;
        }
    }
    pub(super) fn nocubes_pending(&mut self) {
        let requests = std::mem::take(&mut *PENDING.lock().unwrap_or_else(|e| e.into_inner()));
        for request in requests {
            if let Ok(bytes) = serde_json::to_vec(&request) {
                let _ = self.native_net_send_to_server("nocubes:setting", &bytes);
            }
        }
    }
}
static PENDING: std::sync::Mutex<Vec<(String, serde_json::Value)>> =
    std::sync::Mutex::new(Vec::new());

static HELLO: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);
