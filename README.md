# Kestrum

Kestrum is a generational strategy game about kingdoms, armies, people shaped by service, and places changed by decades of war and peace.

## Current milestone

The first campaign shell is implemented in Rust, Macroquad, and Macroquad Toolkit.
The game opens on a Kestrum title screen with Continue, New Game, Settings, How to
Play, Credits, and native Quit Game. New Game unfolds an empty illustrated atlas.
As explicitly requested, towns, roads, ownership, regions, and armies are deferred.
There are no demo actions, energy, grid, debug panels, or economy controls.

The map fills the entire logical canvas. Ordinary play keeps only the map name,
season/turn, Menu, zoom/recenter controls, compass, and End Turn visible. End Turn
advances a season and saves the campaign. The menu contains manual save/load,
settings, help, and the route back to the title. Continue restores the current
campaign or its disk/browser save. Starting over requires confirmation.

This milestone uses one campaign save. The full design's named save slots,
faction rounds, armies, economy, and nested regions remain future work. Geography
labels describe terrain; they are not selectable strategic locations.

## Screen brief

| Question | Current answer |
| --- | --- |
| Current decision | Survey the empty land that will host the campaign. |
| Dominant focus | The continuous, full-bleed illustrated atlas. |
| Primary action | Pan/zoom to inspect geography; End Turn advances the chronology. |
| Supporting information | Current season, year, and turn at the top edge. |
| Deferred information | Settings, saves, help, and credits appear only when opened. No unimplemented system gets a panel. |
| Layout and camera | 1280 × 720 logical canvas, scaled to 1920 × 1080; minimum supported landscape canvas is 1280 × 720. Camera stays within the atlas at 1–3× zoom. |
| Input and feedback | Drag with one finger or the left mouse button; pinch, wheel, or visible + / -; Recenter restores the atlas. All controls are at least 48 logical pixels tall. Help names the visible controls and gestures. |

Approximately 80% of the minimum canvas sits between the shallow edge controls;
the terrain continues beneath them. No framed central map widget or persistent
sidebar exists. Removing the terrain leaves only a handful of game controls.

## Development

Kestrum is a registered member of the real shared Cargo workspace. The toolkit
path is `../macroquad-toolkit`; Macroquad remains pinned to `=0.4.16`.

```powershell
..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test -p kestrum --all-features
cargo fmt -p kestrum -- --check
.\scripts\capture_ui.ps1 -Fullscreen
.\scripts\capture_ui.ps1 -Scenes title_minimum,gameplay_minimum,help_minimum -WindowWidth 1280 -WindowHeight 720
.\publish.ps1
```

The capture harness isolates verification from campaign saves, writes directly to
`docs/verification/`, and exits its hidden game process. Native full-screen
captures use the actual monitor dimensions. The root thumbnail is the title
capture, with no additional image processing.

## Code and artwork

- `src/data.rs`: toolkit-loaded JSON schemas and semantic validation.
- `src/state.rs`: title/campaign transitions and saved chronology.
- `src/navigation.rs`: bounded map camera, reused by mouse and touch.
- `src/game.rs`: input routing, action dispatch, assets, and toolkit persistence.
- `src/ui/`: atlas, menus, typography, and toolkit plaque rendering.
- `assets/data/game_config.json`: presentation copy and terrain labels.
- `assets/art/kestrum_atlas.png`: generated original atlas, OpenAI ImageGen.
- `assets/fonts/`: Cinzel (SIL OFL) and DejaVu Sans (Bitstream Vera license).
- `assets/data/economy.json`: retained design data, not active simulation.

## Documentation and verification

Start with the [design index](docs/README.md). The founding drafts remain preserved
under `docs/reference/`. Historical references describe the earlier documentation
milestone; this README and the [UI verification record](docs/verification/initial-map.md)
describe the implementation now present.

For future development, use the [implementation plan](docs/implementation-plan.md).
It supplies 18 ordered work packages, concrete provisional rules, state/data/save
contracts, behavioral acceptance cases, a complete system coverage ledger, and a
reusable prompt for implementing one package at a time. These are planned systems;
the title and empty atlas remain the current playable milestone.

Follow [AGENTS.md](AGENTS.md), [CODE_STANDARDS.md](CODE_STANDARDS.md), and
[UI_STYLE.md](UI_STYLE.md). Shared guidance remains owned by `rust_management/docs/`.
