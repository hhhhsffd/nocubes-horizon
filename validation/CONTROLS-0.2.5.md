# NoCubes 0.2.5 defaults and controls validation

The release is `dist/nocubes-0.2.5.hnmod` (175257 bytes), SHA-256 `71aea1accaf4a619d214487c921efdfaa8b99b42ab0f4f40c283bc0f47c93718`. All 168 archive files match the workspace; the official archive validator passes (`bundle-check-0.2.5.json`).

The optimized native client is `native-41f822b-4ed4fb958e03`, selected in **NoCubes Port Validation**. It built successfully on the pinned Horizon revision. All 167 mod-source files matched the build snapshot; the transformed engine's 21 files also matched the workspace. Validation used Windows, the RTX 2080 and Vulkan at 1280 × 720. The main player instance was not modified.

## Default block policy

- Classify each registered state from Horizon's full-collision/model metadata, rather than limiting defaults to a named terrain list. Keep existing terrain exceptions such as snow and paths. Exclude plants, vines and air from automatic smoothing, and retain ordinary non-full shapes and fluid rendering.
- Resolve the same effective rules for terrain, LOD, physical queries, picking, highlights and extended-fluid consumers. Explicit whitelist, blacklist and state rules still take precedence. Disabling defaults still supports a whitelist-only configuration.
- `classification-smoke.json` records paired in-game ray queries with rendering on and off, using an empty configuration rather than the validation instance's old block overrides. Glass, leaves and double slabs changed to smooth surfaces. Half slabs, short grass, torches and fences retained identical picked surfaces. All seven fixtures passed and their original blocks were restored.

## Controls and settings

- Register NoCubes in **Options → Mods**, with a settings entry usable in a world and from the title screen. The first settings row is **Toggle targeted block smoothness**, initially unbound. Clicking it captures a physical key/chord; Backspace/Delete clears it and Escape cancels capture.
- The key applies a block-type selector: it toggles every block of that type, clearing conflicting type-specific state overrides. It does not edit only the selected placed block. Multiplayer authority rules are unchanged; remote gameplay edits still require permission.
- Remove command registration/handling, the former visual/settings/state shortcuts and login hints. Old JSON shortcut fields are accepted but ignored, hidden from the settings screen and omitted on save. They cannot reactivate a binding. The new key remains client-local even when server gameplay settings are applied.
- `controls-smoke.json` records seven successful in-game checks: the old default key is inactive; two separate oak-plank blocks change together while stone bricks remain unchanged; a second press restores the type; repeats/releases do not toggle twice; modifier capture and cancellation work; clearing disables the chosen key; no shortcut hints appear in chat.
- The two plank surfaces moved from approximately Z `0.33333` to cubic Z `0.0` after one press. The stone-brick query remained unchanged. This checks actual geometry, not just configuration text.
- Title-screen binding/clearing also passed. The control CLI refuses keyboard events without an open world, so this additional test used keyboard messages addressed to the game's own Windows window. Reviewed captures include `screenshots/controls-unbound.png`, `controls-title-bound.png` and `controls-title-unbound.png`.
- Fixtures were restored, the validation camera returned to its initial position, and the game was saved and closed. The validation instance retains its other block/settings overrides, with the new key explicitly empty and obsolete fields removed. Near distance is 24 chunks; LOD was returned to its 512-chunk startup value (`controls-final-state.json`).

## Automated checks and limitations

All **49 engine tests** and **11 transformed native renderer tests** passed (60 total). Workspace formatting and Clippy with warnings denied passed; edited native adapters pass standalone formatting checks. `controls-tests.log` and `controls-native-tests.log` contain the test results.

Remote multiplayer, arbitrary modded registries and exhaustive visual parity were not retested. No new FPS benchmark was run for this change; older performance measurements remain historical. A source-only build snapshot outside the repository was used because the launcher's source-digest scan otherwise reads the repository's multi-gigabyte ignored build outputs before compilation.
