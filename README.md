# Kestrum

Kestrum is a generational strategy game about kingdoms, armies, people shaped by service, and places changed by decades of war and peace.

## Current milestone

The campaign foundation is implemented in Rust, Macroquad, and Macroquad Toolkit.
The game opens on a Kestrum title screen with Continue, New Game, Settings, How to
Play, Credits, and native Quit Game. New Game starts Rosemarch's strategic state
over the illustrated atlas. Selectable markers arrive in K04.
There are no demo actions, energy, grid, debug panels, or economy controls.

The map fills the entire logical canvas. Ordinary play keeps only the map name,
season/round, active faction, Menu, zoom/recenter controls, compass, and End Turn
visible. End Turn passes to the next faction; each rival currently passes. Pause,
Step and Resume control rival progression between atomic actions. A full round
advances one season and saves the campaign. The menu contains manual save/load,
settings, help, and the route back to the title. Continue restores the current
campaign or its disk/browser save. Starting over requires confirmation.

The save catalogue keeps each round checkpoint and each new named save. It offers
explicit overwrite/deletion and retries failed writes without replaying the round.
Armies, economy, and selectable nested regions remain later packages. Geography
labels describe terrain; they are not selectable strategic locations.

### K01 — Typed content and Rosemarch

K01 is complete. Startup and the graphics-free `GameData::load()` library API load
and validate presentation, the existing economy, setup rules, and the authored
Rosemarch scenario through toolkit JSON APIs. The scenario has five major markers,
four headquarters, ten regional sites, twelve internal edges and four external
routes with reciprocal entrance mappings. City and High Fort plus either supplied
entrance form its anchor expression. Faction identity, local control, founding
grants, diplomacy and normalized positions are durable JSON content.

The economy's balance values are unchanged. All existing fields are typed and
validated, including unsupported policies, duplicate table IDs and invalid costs.
Scenario resource grants reference its 500 Gold / 200 Wood / 150 Stone starting
table; full formation grants reference its troop capacities. K02 instantiates
factions and the graph. The original `kestrum_campaign_v1` bytes remain readable
through Open Old Atlas and are never overwritten by a strategic campaign.

[K01 verification](docs/verification/k01-content.md) records the five new behavioral
tests, 13 preserved regressions, native captures and Windows/WebGL Preview publish.
Project Roost tracking was unavailable for K01 and succeeded during K02 publishing.
Browser fullscreen and physical touch limitations remain open.

### K02 — Campaign ownership and seasonal phases

Strategic state owns stable graph/faction IDs, founding resources, four persisted
toolkit RNG streams, the current faction, round order, acted set and command
sequence. Accepted commands commit atomically; rejection preserves state. Player
observation exposes public places and the player's own faction resources.

The season stays fixed while the player and each eligible rival take their turns.
Four rounds make one year. Rivals currently pass without income or fabricated
events from future systems. Menu overlays and errors hold automatic progression.
Manual saving is available during the player's phase. Old atlas campaigns are
read-only, with no invented strategic history.

K02 passes 23 tests, formatting, strict Clippy, both-size native visual review and
Windows/WebGL Preview publishing. Browser checks cover phase controls and save
restoration after reload. [K02 evidence](docs/verification/k02-campaign.md) records
the remaining fullscreen/physical-touch limitations. K04–K18 remain required.

### K03 — Recoverable campaign saves

Saved Campaigns opens from the title or Menu. The paged list has no game-imposed
slot limit. New Save keeps a separate named copy; Overwrite and Delete require
confirmation. Naming uses a shared touch keyboard with optional physical typing.
Automatic saves retain each completed round. Retry uses the exact failed snapshot;
Continue Unsaved returns to the current campaign without repeating its effects.

The shared toolkit writes fresh payloads and publishes catalogue references through
a recoverable journal. Native file locks and browser Web Locks serialize writers.
Storage errors remain visible. A second game window can browse committed entries
and retry for writer ownership. Campaign IDs and save identities do not use game RNG.
Open Old Atlas preserves the shell; Import Earlier Campaign copies the K02 save
into the catalogue with a new identity. Both source slots stay intact.

K03 passes all 28 game tests, formatting, strict Clippy, both-size native review,
browser catalogue/reload/recovery checks and Windows/WebGL Preview publishing.
See [save verification](docs/verification/k03-saves.md) for shared commits, failure
coverage and the remaining platform limitations.

## Screen brief

| Question | Current answer |
| --- | --- |
| Current decision | Survey the land, finish the player's orders, and follow each faction's phase. |
| Dominant focus | The continuous, full-bleed illustrated atlas. |
| Primary action | End Turn passes to the next faction; Pause, Resume and Step control the visible NPC sequence. |
| Supporting information | Season, year, round and active faction at the top edge; the season changes only after all eligible factions finish. |
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

- `src/data.rs`, `src/data/`: combined content, typed schemas and semantic validation.
- `src/state.rs`, `src/state/`: title/campaign transitions and authoritative state.
- `src/engine.rs`, `src/engine/`: atomic commands, projections and faction rounds.
- `src/navigation.rs`: bounded map camera, reused by mouse and touch.
- `src/game.rs`: input routing, action dispatch, assets, and toolkit persistence.
- `src/ui/`: atlas, menus, typography, and toolkit plaque rendering.
- `assets/data/game_config.json`: presentation copy and terrain labels.
- `assets/art/kestrum_atlas.png`: generated original atlas, OpenAI ImageGen.
- `assets/fonts/`: Cinzel (SIL OFL) and DejaVu Sans (Bitstream Vera license).
- `assets/data/economy.json`: loaded and validated economy defaults; simulation starts in K05.
- `assets/data/campaign_rules.json`: supported setup policy and eight botanical emblems.
- `assets/data/scenarios/rosemarch.json`: versioned small scenario, topology and founding grants.

## Documentation and verification

Start with the [design index](docs/README.md). The founding drafts remain preserved
under `docs/reference/`. Historical references describe the earlier documentation
milestone; this README and the [UI verification record](docs/verification/initial-map.md)
describe the implementation now present.

For future development, use the [implementation plan](docs/implementation-plan.md).
It supplies 18 ordered work packages, concrete provisional rules, state/data/save
contracts, behavioral acceptance cases, a complete system coverage ledger, and a
reusable prompt for implementing one package at a time. The current assignment
continues automatically through all remaining packages, with validation and a
commit at each completed package.

Follow [AGENTS.md](AGENTS.md), [CODE_STANDARDS.md](CODE_STANDARDS.md), and
[UI_STYLE.md](UI_STYLE.md). Shared guidance remains owned by `rust_management/docs/`.
