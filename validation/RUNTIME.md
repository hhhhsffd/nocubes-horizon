# NoCubes 0.2.2 validation

The subsequent grass texture fix is recorded in [GRASS-0.2.3.md](GRASS-0.2.3.md), and distant-terrain smoothing in [LOD-0.2.4.md](LOD-0.2.4.md). The measurements below remain the historical 0.2.2 results.

The release is `dist/nocubes-0.2.2.hnmod`, built and tested on Windows with an AMD Ryzen 5 5600X, 16 GB RAM and an NVIDIA GeForce RTX 2080. The game used Vulkan at 1280 × 720. The isolated launcher instance is **NoCubes Port Validation**, running `native-41f822b-1f1c688859ad`. The player's main instance was not changed.

## Performance

The comparison uses Horizon's normal terrain and collision paths with NoCubes disabled in the same executable. Native adapter hooks remain installed. The enabled configuration uses default Surface Nets with smooth collisions. These results compare Horizon paths; Java Minecraft was not benchmarked.

Each comparison ran disabled, enabled, enabled, disabled, with 10 seconds standing still followed by four forward/back walking laps per sample. Camera, world, resolution and simulation distance were held constant. VSync and the AFK frame cap were disabled. Measurements began after terrain work settled, with no compiler running. Reported FPS combines frame counts and measurement durations across the two runs.

| Render distance | Workload | Horizon baseline FPS | NoCubes FPS | FPS reduction | Added average frame time |
| --- | --- | ---: | ---: | ---: | ---: |
| 8 chunks | Standing | 1,507 | 1,437 | 4.6% | 0.032 ms |
| 8 chunks | Walking | 1,505 | 1,400 | 7.0% | 0.050 ms |
| 24 chunks | Standing | 1,031 | 895 | 13.2% | 0.147 ms |
| 24 chunks | Walking | 1,058 | 877 | 17.1% | 0.195 ms |

At a 120 FPS limit and 24 chunks, both configurations averaged approximately 120 FPS. Across the capped samples, baseline p95 frame times were 8.62–8.66 ms and NoCubes p95 frame times were 8.65–8.74 ms. Uncapped overhead remains, especially at longer render distances.

The isolated CPU workload compared the original 0.2.1 engine at commit `78492df` with 0.2.2, using the same benchmark, toolchain and locked dependencies, with the game closed. Meshing was 1.6–10.9 times faster across the nine modes; default Surface Nets improved from 1.358 ms to 0.393 ms per section. Face and collision-box counts matched. Uncached collision timings vary by mode; gameplay also reuses validated collision shapes.

Raw measurements are in `matched-runtime-8.json`, `matched-runtime-24.json`, `matched-runtime-24-capped120.json`, and `controlled-cpu-*.txt`. `performance-summary.json` and `controlled-cpu-summary.json` contain aggregates. The earlier `baseline-*.json` files are exploratory measurements, separate from the matched final comparison.

## Changes

- Resolve block rules into immutable registry/configuration snapshots, including stable disabled and visual-only configurations.
- Use contiguous sampling, density and Surface Nets vertex caches in bounded neighborhoods.
- Cache collision shapes with validation of every sampled block, configuration and registry; join boxes before optimizing the final shape.
- Reuse bounded local meshes for picking, selection outlines and damage overlays, validating sampled blocks before reuse.
- Skip unused meshing passes, delegate unaffected sections to Horizon, and suppress fully enclosed smooth opaque sections. Restore conservative section occlusion with a two-cell surface halo.
- Ignore duplicate server configuration snapshots without incrementing the revision or remeshing.
- Light grass tufts from outside the smooth surface instead of leaving them black.

## Checks and gameplay

All 37 engine tests and three tests in the transformed Horizon renderer passed. The native tests check collision cache invalidation, equivalence of batched shape unions, and the conservative occlusion halo. Formatting, workspace Clippy with warnings denied, the full optimized native client build, and the official archive validator passed. All 161 archive files match the workspace; the compiled engine source also matches.

The final in-game smoke test ran every mesher and recorded zero pending chunks for each. A warmed ray hit disappeared after removing its block and returned after restoration with the player stationary. Switching to Cubic changed the hit surface. Removing the floor released the player from a cached collision shape, from Y 81.0 to Y 71.8. See `runtime-smoke.json` and `screenshots/all-meshers.png`.

Additional manual checks covered terrain/snow/plant/water rendering, the grass-tuft lighting fix, smooth selection, mining and replacing a stone block, walking, a persisted settings-screen boolean edit, and live file reload. Remote multiplayer and exhaustive gameplay/visual parity remain unverified. Alternative meshers and enhanced/sampled collision settings retain their algorithmic costs. No finite benchmark establishes that every performance issue is eliminated.

## Reproduction

After launching the validation world under Horizon agent control, use the launcher's current `horizon-ctl.exe` with `tools/runtime-smoke.ps1` and `tools/runtime-benchmark.ps1`. The scripts default to the isolated validation instance and restore its NoCubes configuration after each run. The benchmark accepts `-Distance 8`, `-Distance 24`, and `-MaxFps 120`. Its control argument must point to a current CLI supporting `wait-ready`.

See `runtime-report.json` for artifact hashes, source pins, exact coverage and limitations. `report.json` is the historical 0.2.1 report.
