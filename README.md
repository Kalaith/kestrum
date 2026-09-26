# Kestrum

Kestrum is a generational strategy game about kingdoms, armies, people shaped by service, and places changed by decades of war and peace.

## Current milestone

The campaign foundation is implemented in Rust, Macroquad, and Macroquad Toolkit.
The game opens on a Kestrum title screen with Continue, New Game, Settings, How to
Play, Credits, and native Quit Game. New Game starts Rosemarch's strategic state
over the illustrated atlas, with selectable headquarters and a regional marker.
Select your headquarters and open Armies to inspect or recruit your forces.

The map fills the entire logical canvas. Ordinary play keeps only the map name,
season/round, active faction, Menu, zoom/recenter controls, compass, and End Turn
visible. End Turn passes to the next faction; each rival currently passes. Pause,
Step and Resume control rival progression between atomic actions. A full round
advances one season and saves the campaign. The menu contains manual save/load,
settings, help, and the route back to the title. Continue restores the current
campaign or its disk/browser save. Starting over requires confirmation.

The save catalogue keeps each round checkpoint and each new named save. It offers
explicit overwrite/deletion and retries failed writes without replaying the round.
Enter Region opens Rosemarch's ten connected sites. Six-slot armies, recruitment,
disbanding, income, upkeep, movement, transfers and supplied recovery are playable.
Hostile field encounters resolve automatically, with lasting casualties, retreat,
person wounds and recorded reports. Fortified encounters follow in K10.
Terrain labels are distinct from selectable place markers.

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
Four rounds make one year. Rivals currently pass; all active factions receive
income and pay upkeep at the common boundary. Menu overlays and errors hold automatic progression.
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

### K04 — Selectable geography and territorial claims

Tap a world marker or regional site to inspect its local controller, political
claim, contested state, supply and connections. Enter Region and World Map retain
the respective cameras; Close dismisses the inspector. Gates name their external
headquarters connections. The world marker summarizes the same physical sites.

Secure anchors and a supplied entrance determine a region's political owner.
Losing an anchor retains the previous claim as contested, while every physical
site keeps its actual controller. Supply follows secure friendly physical routes.
Earlier v2 saves gain these derived fields without changing their original bytes.

All 33 tests, formatting, strict Clippy, native review at both supported sizes and
Windows/WebGL Preview publishing pass. [K04 evidence](docs/verification/k04-geography.md)
records browser navigation checks and the inherited minimum-browser display issue.
K05 extends this foundation below. K06–K18 remain required before the full release.

### K05 — Armies, recruitment and round economy

New campaigns begin with four authored armies, twelve formations and four Officer
founders. Each army has six slots. Armies shows headcounts, leadership, commander,
supply, upkeep and available resources. Recruit or an empty slot opens six troop
choices with costs and missing requirements. Confirm Recruit creates a full new
formation that becomes ready next round. New Army creates a separate roster at
the same site. Previous/Next pages through co-located armies.

Disband requires confirmation, gives no refund and preserves named people at the
site or in another friendly formation. Removing the last formation removes the
empty army. A full round grants secure local income and one eligible HQ bonus,
then pays full formation upkeep. A shortfall clamps Gold to zero and blocks
recruitment until a later boundary pays upkeep in full. The roster shows the
actual last-round income, paid/due upkeep and shortfall.

Earlier strategic saves retain their date, resources and RNG without invented
troops or founders; legal recruitment starts their army roster. The five K05
behavioral cases and all 38 game tests pass. [K05 evidence](docs/verification/k05-armies.md)
records visual, browser, save and Windows/WebGL Preview verification and limitations.

### K06 — Movement, composition and connected recovery

Orders opens group movement, formation transfers, local People and disbanding.
Choose co-located armies, select a physical destination, review the route and
Confirm Move. Every member pays actual edge costs; the group stops at its last
legal site when a later edge is blocked. Route details can close without cancelling
the order. Fortified hostile entry remains unavailable until K10; K07 adds field combat.

Transfer whole formations or people between local friendly rosters, or split a
formation into a new army. Spent movement is preserved. Opening Armies pauses a
rival phase so these transfers remain available between rival actions.

After income and upkeep, supplied surviving formations recover up to 20% of their
capacity, limited by missing troops and affordable Gold. Orders shows the current
forecast and actual last-round recovery. Cut supply and unpaid upkeep block it.
All 48 tests, strict Clippy, formatting, source-size checks, native visual review,
browser movement/transfer/reload and Windows/WebGL Preview publishing pass.
[K06 evidence](docs/verification/k06-logistics.md) records the checks and remaining
platform limitations. K07–K18 remain required.

### K07 — Automatic battles and recorded consequences

Move a selected army group into a hostile unfortified force to fight every enemy
army at that site. Up to eight simultaneous exchanges account for surviving troop
counts, named leadership, troop counters and defensive terrain. Participants spend
their remaining movement. Defeated survivors retreat through legal adjacent routes;
trapped forces are destroyed. Inhabited battlefields and hostile captures retain
structural damage, with occupation recorded after capture.

Battle Reports opens after an encounter and remains available through Menu.
Outcome, Forces, People and Factors show the recorded result, separate formation
losses, destinations, wounds, command succession and the strength factors used.
Previous/Next pages through long rosters; Older/Newer changes encounters. Reports
preserve what the participating faction witnessed and do not rerun the battle.

People from destroyed formations can die or escape wounded through the documented
combat rolls. A surviving commander can also be wounded; another fit adult in the
same army takes command when available. Wounded people cannot contribute leadership
and recover after two supplied seasonal boundaries. Earlier saves gain no invented
encounters, injuries or occupation. All 58 tests, strict Clippy, formatting,
source-size checks, both-size native review, published browser battle/recovery/
reload and Windows/WebGL Preview publishing pass. [K07 evidence](docs/verification/k07-combat.md)
records the checks and remaining platform limitations. K08–K18 remain required.

## Screen brief

| Question | Current answer |
| --- | --- |
| Current decision | Inspect a place, manage its armies, or select and confirm a costed route for a travelling group. |
| Dominant focus | The connected strategic map over the full-bleed illustrated atlas. |
| Primary action | Enter Region opens a regional marker; World Map returns to the previous world camera. End Turn remains separate from selection. |
| Supporting information | A dismissible inspector shows local control, regional claim and anchor requirements. The top edge shows season, round and active faction. |
| Deferred information | Settings, saves, help, and credits appear only when opened. No unimplemented system gets a panel. |
| Layout and camera | 1280 × 720 logical canvas, scaled to 1920 × 1080; minimum supported landscape canvas is 1280 × 720. Camera stays within the atlas at 1–3× zoom. |
| Input and feedback | Tap markers to inspect; Close dismisses selection. Drag to pan; pinch, wheel, or visible + / - to zoom; Recenter restores the current map. Targets and controls are at least 48 logical pixels. Help names the controls. |

Approximately 80% of the minimum canvas sits between the shallow edge controls;
the terrain continues beneath them. No framed central map widget or persistent
sidebar exists. A selected place adds one inspector opposite its map position.

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
- `assets/data/economy.json`: validated recruitment, upkeep, income and recovery defaults; K05 consumes recruitment and seasonal economy.
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
