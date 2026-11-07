# NoCubes 0.2.5

- Full-collision model blocks now smooth by default, including building blocks, logs, glass, leaves and modded cubes. Plants, vines, air, liquids and non-full shapes keep their ordinary geometry. Existing terrain-specific support, including snow and paths, remains intact.
- The targeted-block toggle changes **every block of that type**, not a single placed block or only one state. Explicit block/state rules remain supported.
- Open **Options → Mods → NoCubes → Settings** and click **Toggle targeted block smoothness** to assign a key. It is **unbound by default**. Backspace/Delete clears it; Escape cancels capture. Modifier chords are supported.
- Removed the old built-in shortcuts, chat commands and login hints. Old configuration files are accepted, but their legacy shortcut fields cannot reactivate those controls.
- Rendering, picking, selection, collisions, fluids and distant terrain share the same registry-based block rules. This release also includes the previous terrain performance, grass texture and LOD updates.

Install the `.hnmod` using Horizon's native mod installation/build flow. Native source adapters remain pinned to Horizon commit `41f822b659789780f58d32c63ddfb6887ea1b8bf` (Minecraft 26.3).

Validated with 60 automated tests, an optimized native build, archive verification, and in-game block-type toggles, key capture/clearing and protected-shape checks on Windows/RTX 2080. See `validation/CONTROLS-0.2.5.md` for scope and limitations.
