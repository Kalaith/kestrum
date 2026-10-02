# Macroquad Toolkit Game Template

This is a working starter crate for new Rust + Macroquad games in this workspace.
It intentionally uses `macroquad-toolkit` heavily so new projects begin with the
same shared patterns as the existing games.

Read [UI_STYLE.md](UI_STYLE.md) before adapting the screens. This starter
demonstrates toolkit integration, not a finished layout for every game. Its
demo panels, permanent help, technical labels, and save/debug controls must
be reconsidered around the new game's current decision and dominant play
area. Keep useful infrastructure and choose what deserves to remain visible.

## Toolkit Features Already Wired

- `AssetManager` with a texture manifest at `assets/data/texture_manifest.json`
  and an exact runtime asset list in `asset_registry.json`
- `DataRegistry` and embedded JSON loading for data-driven actions
- `save_to_slot_with_version`, `load_from_slot_with_migration`, `delete_slot`, and `get_save_slots`
- `NotificationManager` with toolkit toast rendering
- `VirtualUi`, `SurfaceStyle`, `TextStyle`, `GridLayout`, meters, badges, bounded text fitting
- `FlatGrid`, `FogState`, `TilePos`, line-of-sight visibility, and flood-fill reachability
- `CameraTransform` shared by map drawing and picking, with bounded pan,
  cursor-anchored wheel zoom, right-mouse drag in logical coordinates, and
  visible Left/Right/Up/Down/Zoom/Reset controls
- `Pointer` for DPI-correct mouse/touch activation and toolkit button rendering
- `ScrollArea` with wheel/drag scrolling, fling, scrollbar, and suppression of
  action activation after a scroll gesture; extra JSON actions stay inside the panel
- `GameSettings` persistence for the visible Show FPS/Hide FPS control, backed
  by `DebugOverlay`
- Pause/Resume controls (Escape shortcut) that stop energy regeneration and
  actions while leaving map exploration and save controls available
- `EventBus<UiAction>` so UI returns intents and game logic applies them
- Rust 2018 module layout using `data.rs`, `state.rs`, and `ui.rs` parent
  files instead of `mod.rs`

The template avoids browser-incompatible filesystem access. Static data is
embedded with `include_str!()`, runtime browser assets go through Macroquad or
toolkit async loaders, and save data uses macroquad-toolkit persistence.
Shared UI math, such as grid layout and mouse selection, is kept in helper
types so rendering and input do not duplicate coordinate calculations.

## Run

From `rust_management/`:

```powershell
.\cargo.ps1 run --manifest-path template/Cargo.toml
```

## Test

```powershell
.\cargo.ps1 test --manifest-path template/Cargo.toml
```

## Rename For A New Game

1. Copy `template/` to your new game folder.
2. Rename the package in `Cargo.toml`.
3. Change the `macroquad-toolkit` dependency path in `Cargo.toml` from
   `../../macroquad-toolkit` to `../macroquad-toolkit`. The former is correct
   only while the template remains nested inside `rust_management/`.
4. Read `UI_STYLE.md` and record its screen brief in the game's GDD or README:
   current decision, dominant focus, primary action, supporting/deferred
   information, viewport/camera plan, and touch interaction. Recompose the
   demo screen before expanding content; relocate utilities and remove demo
   copy and unused surfaces.
5. Update `assets/data/game_config.json` and put player-facing copy in
   `assets/data/game_text.json`.
6. Replace `actions.json` with your game data.
7. Add externally loaded textures to both
   `assets/data/texture_manifest.json` and `asset_registry.json`. Keep embedded
   JSON data out of the registry. If the game adds other runtime-loaded assets,
   list each exact `assets/...` path in the registry as well.
8. Update `game_page.json` with the title, WASM/package name, page copy, and
   controls. The publisher derives the Project Roost slug globally as
   `rust_<your_game_dir>` and generates `index.html` from this file and the
   shared web template; do not create a hand-maintained game `index.html`.
9. Add a root-level 16:9 `catalog_thumbnail.png` showing the title/menu screen.
10. Update the capture prefix/scenes in `scripts/capture_ui.ps1` if the package
    name and environment-variable prefix differ.
11. If the game has an itch.io page, update `itch.json` with its owner/game
    target and stable `html5`/`windows` channels. Run `publish-itch.ps1` from
    the project directory after the ordinary `publish.ps1`; use `-DryRun` and
    `-Preview` before the first upload.
12. Complete the `UI_STYLE.md` visual review at normal and minimum supported
    sizes, including relevant dense states and touch interactions. Inspect
    focus and readability as well as clipping and control placement.
13. Run `cargo fmt`, `cargo test`, `cargo clippy --all-targets --all-features --
    -D warnings`, then `./publish.ps1` from the new game folder.

For the complete setup, Git, architecture, and publishing checklist, read the
management repository's `docs/onboarding/README.md`.

## Starter interaction patterns

The UI returns intents through `EventBus`; `Game` owns simulation, settings,
scroll state, and camera input. Use the explicit logical-coordinate button
renderer in a virtual frame and let `Pointer` decide activation. When using
cards, keep each label in a separate text box; reflow or disclose content before
shrinking it. Scroll rows are culled with
`is_fully_visible`, and `absorbs_press` prevents releasing a drag from running
an action. Save controls remain outside the scroll region.

Arrow keys also select tiles, Space runs the first data action, S/L save/load,
and +/- zoom. Every core action has a visible tap target. FPS preference is
stored separately from the game save. No audio or network dependency is added
solely for a demonstration.

The capture wrapper works both here and after copying the template to a
workspace-level game directory. Run `./scripts/capture_ui.ps1` to refresh
the gameplay, paused, scrolled, and zoomed scenes in `docs/verification/`.

Register each new game explicitly in `rust_management/workspace/Cargo.toml`, then
run `python rust_management/sync-workspace.py`. Keep the exact Macroquad version
pin and use the shared build pool; see `rust_management/docs/CARGO_WORKSPACE.md`.
