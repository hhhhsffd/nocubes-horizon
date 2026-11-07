# NoCubes 0.2.6 player model validation

The release is `dist/nocubes-0.2.6.hnmod` (182667 bytes), SHA-256 `9c4baa14791aade654c4c6d90db6d6f4a46e2c56d32f68fe67323ae602637197`. The bundle builder verified all 173 files against the workspace; the official archive validator passed (`bundle-check-0.2.6.json`). The native build is `native-41f822b-02fb060fdc7e`, selected in **NoCubes Port Validation**, on the same pinned Horizon revision as 0.2.5.

## Implementation

- `player_model` is a client-local boolean, false by default, shown as **NoCubes player model** immediately after the unbound block-toggle setting. It is independent of terrain rendering and does not change player hitboxes or physics.
- Round the standard player's twelve body/outer-layer cuboids in their original local coordinates. Each source face becomes nine bounded quads, producing 648 quads per complete classic/slim model. Preserve every part's name, pose, hierarchy, winding, culling flag and original skin UV rectangle. Clamp interpolated UVs to that rectangle to avoid numerical overshoot into other skin regions.
- Keep Horizon's existing per-player skin/default-skin selection, skin downloads, texture atlas, slim metadata and animations. Geometry is shared by source model, not by skin, so different players retain their individual texture selection. No replacement player texture or exported Minecraft model data is shipped.
- First-person arms use the same rounded, correctly sized model parts with the original hand transforms and the player's current skin sprite. Armour, held items, capes and other non-body attachments retain their original paths. Custom non-cuboid parts are not rewritten; custom entity-model body replacements retain renderer priority.
- Cache at most 32 rounded source models, with weak source ownership and stable output Arcs. Rounded outer layers take precedence over Horizon's pixel-extruded 3D skin layer option while enabled; disabling restores the original path.
- Curved polygon normals use Horizon's existing CPU model fallback. This option increases player geometry/rendering work; it does not claim zero performance cost. It is off by default.
- Separate appearance/input revisions from terrain revisions. Changing only the player model or key binding does not invalidate terrain/LOD meshes. Gameplay/terrain changes still invalidate them.

## Automated and prepared-data checks

All 54 engine tests, 11 transformed native renderer tests and the native player-model cache test passed (66 total). New checks cover classic/slim/head/body bounds, finite geometry and normals, closed shared edges, rotated/mirrored skin UVs, degenerate/custom geometry fallback, opt-in/client-local settings, server merge behavior, terrain equality and cached pose/name/hierarchy/source preservation. Formatting and workspace Clippy with warnings denied passed.

The `player_models` example also read the locally prepared `entity-models.json`: all twelve parts of both `minecraft:player#main` and `minecraft:player_slim#main` were successfully rounded, with 648 polygons each (`player-prepared-models.json`). The prepared model data itself is not included in the repository.

## In-game checks

Validation ran on Windows with an RTX 2080, Vulkan, at 1280 × 720 in the isolated `NoCubesValidation` world. The main player instance was not modified.

- A settings-screen click persisted the option on, and a second click restored it off. No key binding was added.
- Reviewed paired front-view and survival-inventory captures show a rounded head, body and limbs while retaining the same face, hair, jacket and trouser skin textures. The first-person arm is rounded with the same sleeve/skin texture. A walking capture shows the existing posed limb animation.
- The live player-only toggle left the terrain's `meshed_sections` counter unchanged. `player-smoke.json` records the statuses and assertions; captures are under `screenshots/player-0.2.6/`.
- The seven block-control checks and seven protected-shape fixtures were rerun after the revision change. Both passed (`player-controls-regression.json`, `player-classification-regression.json`). The block-type toggle remains initially unbound, and plants/non-full shapes retain normal geometry.
- The player option was left off, the block key empty, and the validation camera, game mode and selected slot were restored. Render distance was returned to 24 chunks; the running validation session's 64-chunk LOD setting was not changed. The game was saved and closed after testing.

Remote multiplayer, downloaded/custom-skin changes, all armour/cosmetic combinations, custom resource-pack models and full 3D skin-layer compatibility were not exhaustively retested. No matched FPS benchmark was run for this optional model feature. The appearance captures and prepared-data checks do not establish exhaustive parity.
