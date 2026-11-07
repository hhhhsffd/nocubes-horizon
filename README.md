# NoCubes for Horizon

A Rust port of [Cadiboo/NoCubes](https://github.com/Cadiboo/NoCubes) for Horizon native mods. The installable bundle is [`dist/nocubes-0.2.1.hnmod`](dist/nocubes-0.2.1.hnmod). It contains Rust source which Horizon compiles into the instance executable.

This release implements all nine upstream mesher modes, terrain rendering, smooth collisions and picking, fluid extension, grass and snow features, outlines and debug overlays, live configuration, block/state controls, and multiplayer configuration/edit synchronization. [PORTING.md](PORTING.md) maps the upstream features to their Rust implementations and explains the Horizon-specific integration.

Version 0.2.1 corrects legacy source-fluid solidity, partial-snow face culling, Wulferis transparency-pass barriers, special-block backfaces, and rolling profiling.

No in-game testing or GPU testing was performed, as requested. CPU tests, source transformation, compilation checks, and bundle validation are recorded in [`validation`](validation). Runtime appearance and gameplay still require validation on a machine that can run Horizon.

The native adapters require these exact revisions:

| Component | Revision |
| --- | --- |
| Horizon, Minecraft 26.3 | `41f822b659789780f58d32c63ddfb6887ea1b8bf` |
| horizon-mixins | `5a64af938ec86b00d53d16a408aabc0038fb3e57` |
| Original NoCubes | `38904f71b818a1f4770c5253b9a1dd522201babc` |

Install the `.hnmod` through the launcher's native mod flow and allow it to compile the instance. For a source checkout, Horizon also supports:

```sh
horizon-launcher native-build ./mixins/nocubes --instance <instance> --trust-native-code
```

Use the mod on the server and clients for smooth physical collisions. The server supplies gameplay settings; client visual preferences remain local. On servers without NoCubes, terrain smoothing and temporary visual block edits work locally, with custom physical collisions disabled. Disconnecting restores local settings.

Default controls while playing:

| Input | Action |
| --- | --- |
| `N` | Toggle smoothing for the targeted block type |
| `Shift+N` | Toggle smoothing for only the targeted block state |
| `Ctrl+V` | Toggle terrain visuals, unless forced by the server |
| `Ctrl+O` or `/nocubes` | Open settings |

The settings screen is also available through Horizon's native mod screen. Click a boolean to toggle it. Other values use JSON notation; Backspace edits and Enter applies. Click the footer for the next page, or right-click it for the previous page. Strings, including key bindings, need JSON quotes. Gameplay edits on a remote server require operator permission level 2 or the integrated server owner's identity. The server broadcasts accepted changes and clients remesh automatically.

Settings are saved to `config/nocubes.json` in the game's working directory. `NOCUBES_CONFIG` overrides the path. A missing file uses defaults. Changes made outside the game are reloaded every second; invalid updates preserve the last valid configuration. Invalid initial configuration reports an error and disables rendering/collisions until corrected. [`nocubes.example.json`](mixins/nocubes/data/nocubes.example.json) contains every setting and its default.

Block selectors accept namespaced IDs and optional property filters, for example `minecraft:stone` or `minecraft:grass_block[snowy=true]`. Omitted namespaces mean `minecraft`. Blacklists override whitelists/defaults; explicit `state_overrides` take precedence over those lists. Default rules include upstream vanilla and supported mod block IDs, with absent IDs harmlessly ignored. Key chords use physical Rust/winit names such as `Ctrl+KeyV` and `Shift+KeyN`.

All original collision enhancement levels are supported (`old_collision_enhancement` 0–4, `only_old_collisions`, `player_collisions_only`). An additional opt-in `sampled_collisions` mode uses `collision_resolution` 2–16; the default uses the ported upstream face colliders. Fluid extension range is 0–2, with source-fluid precedence and read-only extension; it does not place fluid blocks in the world.

Build and validate without launching the game:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p nocubes-engine --example default_config
cargo run -p nocubes-bundle
```

With the pinned Horizon and horizon-mixins checkouts:

```sh
cargo run -p nocubes-adapters -- /path/to/horizon
# The generator updates the generated section of horizon-mixin.toml.
horizon-mixins apply --workspace /path/to/horizon --mods ./mixins --out /tmp/nocubes-stage
CARGO_TARGET_DIR=/tmp/nocubes-target cargo check --manifest-path /tmp/nocubes-stage/Cargo.toml \
  --locked -p horizon-client --lib --no-default-features
cargo horizon-mod check ./dist/nocubes-0.2.1.hnmod --json
```

The compile check uses Horizon's headless/server feature set, which avoids system audio/controller development dependencies. It creates no window and requests no GPU. Keep the Cargo target directory outside the disposable mixin stage.

The engine and tooling are entirely Rust. No Java, JNI, WASM, or game assets are required by the port. LGPL-3.0-or-later; see `LICENSE`, `mixins/nocubes/NOTICE`, and `mixins/nocubes/LICENSE.Horizon` for attribution and the MIT-licensed Horizon integration. This is an unofficial port.
