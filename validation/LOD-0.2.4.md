# NoCubes 0.2.4 distant-terrain validation

The release is `dist/nocubes-0.2.4.hnmod` (173468 bytes), SHA-256 `88a6fcfa2c1226409771fe09b54c411a8717a95786b0f9216cba7e5a46ab0a3e`. The bundle contains 167 files matching the workspace and passes the official archive validator (`bundle-check-0.2.4.json`).

The optimized native build is `native-41f822b-2cfd14b1ada5`, selected in **NoCubes Port Validation**. Validation ran on 2026-11-07 using the RTX 2080, Vulkan, at 1280 x 720. The main player instance was not modified.

## LOD behavior

- Smooth the existing reduced column meshes with a lightweight Surface Nets vertex calculation at each native LOD cell width. The smoother uses eight column-density samples per vertex, independent of distance. It does not generate a block-resolution volume or add chunks to the simulation.
- Preserve Horizon's quadtree, distance levels, scheduling, data sources, column summaries, heightmap reduction and terrain cutouts. Water and protected non-smooth blocks retain their native geometry.
- Retain merged faces when their geometry is unchanged. Subdivide changed faces at the existing coarse lattice, preserving material, biome tint, packed light and tiled UVs. Upward-facing grass slopes use one tinted grass top instead of duplicate side overlays.
- Pin boundaries against near/finer terrain, unknown neighbors and mixed LOD widths. Same-width neighbors share the density samples used for their common edge.
- Limit output to twice the source vertex count, clamped to 4096–32768 vertices per tile, with additional subdivision/cache work limits. If a tile exceeds its budget, retain its original reduced mesh. Complex tiles can therefore remain cubic. This bounds additional detail; smoothing is not free.
- Cubic NoCubes modes keep cubic LOD. Other near-terrain mesher choices use this dedicated lightweight LOD smoother.
- Observe configuration revisions independently for each LOD owner. A weak-owner registry releases dead entries and avoids the integrated server consuming the client's revision marker. Live toggles remesh the existing cached columns rather than resampling terrain.

## Checks

All 43 engine tests and eight native renderer/LOD tests passed (51 total). New checks cover scale-independent sampling, slopes, flat surfaces, preserved water vertices, material-run continuity, same-width seams, mixed-width/unknown neighbors, clipped footprints, coarse cell counts, output budgets, grass layers, independent configuration observers and reuse of cached columns. Formatting and workspace Clippy passed. The complete optimized native client built successfully; the final builder's LOD source files match the workspace.

The in-game smoke test used the isolated `NoCubesValidation` world, a fixed camera, four near chunks and 64-chunk LOD. All three captures had 117 loaded chunks and 2529 active LOD tiles. The resident LOD quad counts were 6752575 enabled, 4644975 disabled, and 6752575 re-enabled. The paired change confirms that live settings rebuild distant geometry, including already cached tiles. These are geometry counts, not FPS benchmarks.

The wider check ran 256-chunk LOD with 4261 active tiles across five distance levels. The original 24-chunk near distance and 1024-chunk LOD distance were restored afterwards (`lod-final-status.json`), with multiple coarse levels active and zero pending near chunks at capture time. Far terrain continues refining in the background.

Reviewed captures are `screenshots/lod-enabled.png`, `lod-disabled.png`, `lod-reenabled.png` and `lod-wide.png`. They show smoothed distant hills, correct grass surfaces and flat water. `lod-smoke.json` contains the raw statuses; `tools/lod-smoke.ps1` reproduces the checks and restores the settings it starts with. The initial cached-toggle failure was caught in-game and fixed before the final build; its historical capture is `lod-smoke-initial-cache-issue.json`.

Remote multiplayer, arbitrary resource packs and every possible terrain seam were not retested. No matched FPS benchmark was run for this release. Historical performance measurements in `RUNTIME.md` describe 0.2.2.
