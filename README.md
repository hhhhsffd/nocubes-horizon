# NoCubes for Horizon

A Rust port of [Cadiboo/NoCubes](https://github.com/Cadiboo/NoCubes) for Horizon native mods. The installable bundle is [`dist/nocubes-0.2.6.hnmod`](dist/nocubes-0.2.6.hnmod). It contains Rust source which Horizon compiles into the instance executable.

This release implements all nine upstream mesher modes, terrain rendering, smooth collisions and picking, fluid extension, grass and snow features, outlines and debug overlays, live configuration, block/state controls, and multiplayer configuration/edit synchronization. [PORTING.md](PORTING.md) maps the upstream features to their Rust implementations and explains the Horizon-specific integration.

Version 0.2.6 adds an optional **NoCubes player model**, off by default. Enable it in mod settings to round player body parts, skin layers and first-person arms using each player's existing skin UVs and textures. Classic/slim shapes, part poses and animations are retained; hitboxes, armour and held items are unchanged. The client-side toggle does not rebuild terrain or LOD meshes. Rounded model geometry is cached (32 source models maximum); curved faces use Horizon's existing CPU model path, so this option adds rendering work. Rounded outer layers take precedence over Horizon's pixel-extruded 3D skin layers while enabled. Custom non-box parts and custom entity-model replacements are left unchanged.

Version 0.2.5 smooths full blocks by default, removes built-in shortcuts and chat hints, and adds an initially unbound block-type toggle in mod settings. All 60 automated tests and the in-game control/classification checks passed; see [validation details](validation/CONTROLS-0.2.5.md).

Version 0.2.4 smooths Horizon far-terrain LOD using Surface Nets at each existing LOD cell width. It preserves the native distance levels, column summaries, water, biome colors, lighting and terrain cutouts. Flat faces remain merged; tile mesh budgets retain the original reduced geometry for unusually complex tiles. Cubic modes retain cubic LOD. Visual and smoothability changes remesh cached LOD tiles without resampling them.

Version 0.2.3 fixes rotated terrain textures and grass side overlays on ground and hills. UV orientation follows the baked model and surface geometry across meshers. Upward-facing grass slopes use the grass top; vertical sides retain their side texture unless better grass is enabled.

Version 0.2.2 adds cached registry classification, contiguous mesher sampling and vertex caches, dependency-validated collision shape reuse, batched collision unions, reusable picking/outline/crack meshes, conservative terrain occlusion, stable configuration snapshots, and duplicate network snapshot suppression. Disabling visuals restores the normal Horizon mesher.

GPU gameplay testing now runs on Windows with an NVIDIA GeForce RTX 2080 (Vulkan, 1280 x 720) in an isolated launcher instance. CPU checks and measured runtime coverage are recorded in `validation/runtime-report.json`; benchmark inputs and raw measurements are included in `validation`. These checks do not establish exhaustive multiplayer or visual parity.

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

Open settings through Horizon's native mod screen. The first row, **Toggle targeted block smoothness**, is unbound by default. Click it and press the desired key (modifier chords are supported). Backspace/Delete clears the binding; Escape cancels capture. While playing, the assigned key toggles smoothing for the targeted block type, leaving other types unchanged. There are no built-in NoCubes shortcuts, chat commands or login hints. Legacy shortcut fields in existing configuration files are ignored and omitted on the next save.

Click a boolean to toggle it. Other values use JSON notation; Backspace edits and Enter applies. Click the footer for the next page, or right-click it for the previous page. Gameplay edits on a remote server require operator permission level 2 or the integrated server owner's identity. The server broadcasts accepted changes and clients remesh automatically.

Settings are saved to `config/nocubes.json` in the game's working directory. `NOCUBES_CONFIG` overrides the path. A missing file uses defaults. Changes made outside the game are reloaded every second; invalid updates preserve the last valid configuration. Invalid initial configuration reports an error and disables rendering/collisions until corrected. [`nocubes.example.json`](mixins/nocubes/data/nocubes.example.json) contains every setting and its default.

Block selectors accept namespaced IDs and optional property filters, for example `minecraft:stone` or `minecraft:grass_block[snowy=true]`. Omitted namespaces mean `minecraft`. Blacklists override whitelists/defaults; explicit `state_overrides` take precedence over those lists. All full-collision model blocks (including building materials, logs, glass and modded cubes) now smooth by default. Plants, vines, fluids, air and non-full shapes keep their normal geometry; the existing terrain exceptions such as snow and paths remain supported. Classification is per state, so double slabs can smooth without changing half slabs. Turning off `use_defaults` leaves only explicit whitelist/state rules.

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
cargo horizon-mod check ./dist/nocubes-0.2.6.hnmod --json
```

The compile check uses Horizon's headless/server feature set, which avoids system audio/controller development dependencies. It creates no window and requests no GPU. Keep the Cargo target directory outside the disposable mixin stage.

The engine and tooling are entirely Rust. No Java, JNI, WASM, or game assets are required by the port. LGPL-3.0-or-later; see `LICENSE`, `mixins/nocubes/NOTICE`, and `mixins/nocubes/LICENSE.Horizon` for attribution and the MIT-licensed Horizon integration. This is an unofficial port.
