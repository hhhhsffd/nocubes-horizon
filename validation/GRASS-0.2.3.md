# NoCubes 0.2.3 grass texture regression

Built and visually checked on 2026-11-07 with the RTX 2080, Vulkan, at 1280 x 720. The **NoCubes Port Validation** instance now selects `native-41f822b-e16970bbcb59`. The player's main instance was not changed.

The release is `dist/nocubes-0.2.3.hnmod` (163386 bytes), SHA-256 `ebcfd862609642584bb94223fd93df57cd4d67fc76983eacefcb31f6e1072d37`.

## Fix

The old renderer applied fixed UV permutations to surface vertices whose starting corner varies with mesher, direction and triangle winding. A floating stepped grass fixture reproduced sideways grass fringes on its walls (`screenshots/grass-before.png`). UVs now follow projected vertex positions and the selected baked model, preserving the model's own UV rotations. The helper supports repeated vertices in triangular faces.

Upward-facing grass slopes now use the top model even when their horizontal normal exceeds the vertical component. Vertical walls keep the grass side model, except with better grass enabled; downward faces keep dirt. Model layers must match the selected face direction, so unrelated unculled side overlays cannot cover a top face.

## Validation

- All 40 engine tests passed, including three new regressions covering all six baked face directions, vertex permutations, sloping sides, baked texture rotations and repeated triangle vertices.
- Workspace formatting, standalone renderer formatting and workspace Clippy with warnings denied passed.
- The optimized native client built successfully. Its renderer snapshot and texture helper were checked against the workspace before installation.
- The bundle builder verified that all 162 archive files match the workspace. The official native bundle validator passed (`bundle-check-0.2.3.json`).
- In-game captures settled with zero pending chunks for all nine meshers at 8 chunks (`grass-texture-smoke.json`). All nine images were visually reviewed in `screenshots/grass-0.2.3/all-meshers.png`.
- Additional Surface Nets views checked north/east/south/west walls, the dirt underside, the existing snow fixture, and natural terrain. Images are under `screenshots/grass-0.2.3/`; `natural-ground.png` records the natural terrain check.

The initial 24-chunk sweep completed Surface Nets, Surface Nets 2x, Marching Cubes, Marching Cubes 2x and Culling Cubic. StupidCubic exceeded the 60-second readiness limit and filled video memory because it deliberately emits internal faces. The full sweep was rerun successfully at 8 chunks. This remains an expensive mesher mode; this release does not claim to fix its high-distance memory cost. The original render distance (24) and NoCubes configuration were restored after the sweep.

Run `tools/grass-texture-smoke.ps1 -Control <horizon-ctl.exe>` with the isolated `NoCubesValidation` world open to reproduce the screenshots. The script leaves its floating grass fixture in that validation world. Runtime captures require visual review; their successful completion alone does not assert pixel correctness.

The performance results in `RUNTIME.md` and `runtime-report.json` describe 0.2.2. Performance was not rebenchmarked for this texture fix. Remote multiplayer and arbitrary resource packs were not retested.
