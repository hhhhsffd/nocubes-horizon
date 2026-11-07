# Port coverage

The port targets the pinned NoCubes source revision in `README.md`. Minecraft loader machinery is replaced with Horizon's native Rust source-mixin system. The bundle contains no Java compatibility layer.

| Upstream feature | Rust implementation |
| --- | --- |
| Surface Nets, averaged-density Surface Nets | `engine/src/lib.rs`: density grid, surface vertices, face winding, original offsets and negative-density snow layers |
| Marching Cubes, averaged-density Marching Cubes | `engine/src/lib.rs`, `marching.rs`: upstream 256-case table and interpolated edges |
| Culling Cubic, Stupid Cubic, Culling Chamfer | `engine/src/lib.rs`, `alternate.rs`: cube and original 18-face chamfer topology |
| OldNoCubes | `alternate.rs`: legacy slopes, fluid solidity, original roughness using world coordinates |
| Wulferis | `alternate.rs`: trilinear density sampling and gradient displacement, replaceable/plant/vine rules |
| Smoothability and default block lists | `registry.rs`, `defaults.rs`, `Config`, `settings.rs`: all full-collision model blocks plus upstream terrain exceptions, property filters and persisted per-state overrides |
| Terrain renderer | `render.rs`: ordinary-model suppression, separate opaque/see-through passes, material ownership, baked model layers, original UV orientation, tint, normal-sensitive vertex lighting and translucency |
| Horizon far-terrain LOD (port extension) | `lod_render.rs`, `lod_refresh.rs`, engine `lod.rs`: bounded smoothing of reduced column meshes, native tile levels and cutouts, preserved fluids, live mesh invalidation |
| Better grass, more snow, plant height, grass tufts | `render.rs`, engine material helpers and registry class metadata; snow material search and short-plant anchoring |
| Fluid extension and rendering | `engine/src/fluids.rs`, `fluid_render.rs`, fluid hooks and generated adapters: bounded non-recursive source lookup, above-first precedence, rendering, movement/current queries, mobs, boats, camera fog, projectiles and loose entities |
| Smooth physical collision | `engine/src/collision.rs`, `collision.rs`: inward face boxes, all four old enhancement levels, original collision switches, falling-block exemption and optional additional sampled mode |
| Entity-specific collision consumers | `generated`: adapters for movement, stepping, auto-jump, projectiles, explosions, camera rays, mob suffocation, placement, pistons, spawn/stand-up and geometry checks; boundary/palette shortcuts are disabled while custom collisions are active |
| Player suffocation, bed obstruction, push-out | `player_in_wall.rs`, `suffocation.rs`, generated query adapters |
| Picking and block selection | `picking.rs`: terrain triangle intersections with vanilla ordinary-block ray ordering; `overlays.rs` for configurable colored outlines |
| Block destruction rendering | `cracks.rs` and entity-geometry adapter: damage textures follow smooth surfaces and adjusted short plants |
| Debug options and profiling | `overlays.rs`, `render.rs`: smoothable bounds, signed densities, actual collision boxes, mesher collision boxes, nearby mesh, skip rendering and CPU mesh timings |
| Key mappings and configuration UI | `runtime.rs`, `engine/src/ui.rs`: initially unbound targeted-block toggle with click-to-bind capture, native mod settings screen, paged JSON field editing and immediate boolean changes; no built-in shortcuts, chat commands or login hints |
| Network and permissions | `runtime.rs`, `settings.rs` and native network hooks: login/handshake snapshots, bounded validated messages, permission-checked gameplay edits, remesh broadcasts and disconnect restoration |
| Live config and render invalidation | `settings.rs`, `dirty.rs`, `runtime.rs`: atomic saves, one-second reload, revision tracking and neighboring-section rebuilds |
| Text/colors | `engine/data/en_us.json`, `color.rs`, `named_colors.rs`: upstream text and setting descriptions, CSS hex/rgb/rgba/named color formats |

Paths without a prefix are relative to `mixins/nocubes`; engine paths are relative to its `engine` crate.

Horizon adaptations:

- Forge/Fabric registrations, Java Mixin interfaces, Cloth/Forge config screens, and Sodium/OptiFine hooks are replaced by Horizon's renderer, native input/screen hooks and source adapters. Those Java renderer plugins do not run in Horizon.
- Horizon loads source bundles, so the release contains portable Rust source rather than an architecture-specific shared library. Private source integrations are pinned to the exact Horizon revision.
- The Rust adapter generator copies the concrete pinned function bodies and redirects collision/fluid calls, preserving surrounding vanilla behavior. Generic trait helpers that cannot be native HEAD targets are copied into supporting Rust modules and their callers are redirected. `validation/query-adapters.txt` inventories these routes. No upstream checkout is required to consume the finished bundle.
- The global registry snapshot makes the same mesher-specific block classification available to all geometry consumers, including headless server queries. Runtime native enable changes trigger configuration revisions and remeshing.
- Persistent settings use JSON and Rust atomic file replacement. The settings screen exposes all supported options. Chords and some UI layout differ from the original Minecraft config framework.
- Physical compatibility requires the server mod. Vanilla-server edits are visual and temporary, and custom physical collision remains off until the server sends its configuration.

The original 0.2.1 validation report covers CPU behavior and headless compilation. The 0.2.2 follow-up adds a full native client build and GPU gameplay testing on Windows/RTX 2080. See `validation/runtime-report.json` for exact runtime coverage and limitations, and the raw benchmark files for timings.

## Follow-up parity audit (0.2.1)

The follow-up source review corrected the concrete differences found: legacy solidity now excludes only source `LiquidBlock` states (not flowing liquids or waterlogged solid blocks); legacy face culling checks full density rather than smoothability for neighboring snow layers; Wulferis treats globally smooth states in the other render pass as soft neighbors; snow layers, beacons and portals retain upstream single-sided rendering; profiling uses separate rolling 256-call chunk/block windows, includes rendering time, and logs on rollover. Grass tufts now follow the found material and the upstream non-plant-above rule rather than requiring air. Single-block outline timing is also included. Geometry consumers share registry state classification.

The CPU regressions establish the listed behaviors. Feature coverage, compilation, and archive validation do not by themselves prove exhaustive runtime parity. The validation report records the checks actually performed without an unconditional `full_port_complete` assertion. That report predates the user's request to enable in-game testing; the 0.2.2 runtime report records the follow-up checks.

## Runtime and performance follow-up (0.2.2)

Registry state rules are resolved once per immutable configuration snapshot. Mesher sampling and Surface Nets vertex caches use bounded contiguous storage, with sparse fallback outside the halo. Collision shapes are cached per thread with a bounded FIFO; every previously sampled block state is checked before reuse, along with the configuration and registry snapshots. Shape unions defer optimization until all boxes have been joined. This preserves immediate edits and avoids repeated union reconstruction. CPU parity checks compare cached and uncached geometry for all nine meshers, including snow and negative coordinates. Duplicate server snapshots preserve the configuration revision, avoiding unnecessary remeshing.

Picking, selection outlines and damage overlays reuse bounded local meshes. Every sampled block is validated before reuse, along with the configuration and registry snapshots. Terrain meshing skips unused passes and sections without nearby smooth states; fully enclosed opaque terrain produces no surfaces. Visibility uses a conservative two-cell interior mask rather than treating every section as open. Grass tufts sample light outside the smooth surface.
