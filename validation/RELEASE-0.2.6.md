# NoCubes 0.2.6

Adds an optional **NoCubes player model**, using each player's existing skin textures.

- Open **Options → Mods → NoCubes → Settings** and enable **NoCubes player model**. It is off by default and requires no key binding.
- Rounds the head, body, arms, legs and outer skin layers, including first-person arms, while preserving the classic/slim skin layout, UVs, part poses and animations.
- Uses Horizon's existing per-player skin selection and atlas; no replacement skin or bundled player texture is introduced.
- Purely client-side appearance: hitboxes, armour, held items and gameplay remain unchanged. Custom non-box parts and custom entity-model replacements remain intact.
- Cached, bounded model generation; toggling player appearance does not remesh terrain or distant LODs. Rounded faces use the existing CPU model-rendering fallback and add rendering work.
- Rounded outer layers take precedence over Horizon's pixel-extruded 3D skin layers while enabled. Disabling this option restores the original model/skin-layer path.

The installable native source bundle remains pinned to Horizon commit `41f822b659789780f58d32c63ddfb6887ea1b8bf` (Minecraft 26.3).

Validated with 66 automated tests, prepared classic/slim model checks, and in-game third-person, inventory, first-person, movement and live settings-toggle checks. The player-only toggle did not rebuild terrain. See `validation/PLAYER-0.2.6.md` for scope and limitations.
