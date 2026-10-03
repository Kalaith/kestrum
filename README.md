# Kestrum

Kestrum is a generational strategy game about kingdoms, armies, people shaped by
service, and places changed by decades of war and peace.

## Current milestone

M01A of the [map playability plan](docs/map-playability-plan.md) is complete under
its [recorded scope](docs/verification/spatial-scale.md): spatial scale and map
navigation use the sole 1920×1080 spec. Engineering checks, 14 native captures,
automated native actions, background browser gameplay, publishing and reload
are recorded. The full suite retains one inherited production-victory failure.
It builds on M01's [kingdom overview](docs/verification/kingdom-overview.md).
M02's map orders and seasonal consequences follow, then regional geography and
the opening guide in M03/M04.
The [notification feature plan](docs/notification-plan.md) details M02's
top event bar, compact clickable details, hero and controlled-place updates,
advance warnings and per-type mute controls. Durable event foundations are
implemented; presentation and lifecycle acceptance remain in progress.
The [hero portrait generator plan](docs/hero-portrait-generator-plan.md) adds a
graphics workstream: persistent per-person features, layered human art
and a first People/Career portrait slice. Its outstanding G01-G04 TODOs preserve
the existing M02-M05 sequence. Persistent identity and migration foundations are
implemented; artwork and portrait rendering remain under development. The
[implementation record](docs/verification/portraits-events.md) distinguishes
completed checkpoints from pending runtime and visual acceptance.
The [design index](docs/README.md) identifies the owning documents and historical
evidence.

K01–K18 and B01–B07 are complete under their recorded scope. This describes
implemented systems and earlier checks; it does not establish satisfying map
play, settled campaign balance or full platform acceptance. The current source
and the latest scoped evidence take precedence over old milestone prose.

## Implemented campaign

- Observe opens a separate AI-only session from the title menu. Watch the full
  map without fog, inspect every kingdom, and use Pause, Step or 1×/2×/4× speed.
  Leaving restores the human campaign. See the [Observer screen brief](#observer)
  and [verification record](docs/verification/observer.md).
- Production setup creates a seeded human kingdom with a name, botanical
  emblem, 4–8 factions and 80 major markers. The current authored world has
  eight ten-site regions and 152 physical sites. Rosemarch is a separate test
  scenario.
- Seasonal faction rounds use deterministic commands and state-owned randomness.
  Rivals recruit, move, build, fight and progress through the campaign rules.
  Pause, Step and Resume control their atomic actions.
- Physical control, political claims, contested regions, secure supply,
  exploration and observed enemy knowledge are persistent.
- Six-slot armies support recruitment, transfers, supplied recovery, service,
  careers and specialization. Each formation holds one named member plus troops,
  or troops alone; commander appointment is separate.
- Construction, roads, development, damage, occupation, refugees, local threats,
  capitals and headquarters affect places through time.
- War, peace, truces, annexation/submission and saved victory/defeat are available.
  Submission is a narrow inactive defeat outcome, with no tributary simulation.
- Battle preparation, deterministic formation combat, tactics, doctrines, leaders,
  battlefield playback, lasting casualties, wounds, retreat and sieges are implemented.
- Aging, mentorship, households, succession, heirlooms and bounded histories
  preserve continuity. Hidden enemy details remain restricted.
- The save catalogue supports named saves and round checkpoints without a
  game-imposed slot cap, explicit overwrite/deletion, and storage recovery.

## Map controls

Tap an Army marker, then a destination to move immediately. Long orders use the
available allowance and save their remaining physical route. End Turn refreshes
movement and continues those orders; encounters and changed access can pause them.
A destination can also be queued when no movement remains. Cancel Route stops
it; another destination replaces it. Review Route inspects the saved order.

For orders inside a region, select the army and tap **Enter Region**. Tapping
its current region also opens the local sites. These navigation actions retain
the selected army and any queued route. Tap a hostile site to advance: an
undefended site is captured and defenders trigger battle or a siege. A peaceful
border names the kingdom blocking passage and offers **Kingdom** to review
**Declare War**. Place inspectors show Peace or War beside foreign control.

An army's site can be yours while the surrounding political claim belongs to
a rival. **No supply** means there is no secure friendly route to headquarters;
it prevents normal recovery, not movement or attacks.

Tap a place to inspect it. Enter Region and World Map retain their respective
camera contexts. Drag to pan; pinch, the mouse wheel, or visible +/− controls
zoom. Overview fits known land and changes to Return View, which restores the
working camera. Recenter focuses the selected army or headquarters at campaign
scale. Close dismisses selection. Selection, End Turn and returning from a sheet
retain the camera; End Turn remains reachable during movement selection.

The overview shows known political claims separately from local occupation and
contested control, settlement silhouettes and the owned capital. Labels respond
to zoom, importance and available space. Compact owned force marks expose route,
siege, idle and supply state; selection opens full counts and order details.
Overview aggregates nearby forces. Tap a crowded group to focus it before
choosing an exact place or army. Regional force groups and armies sharing one
physical site remain distinct. Map Key explains these scales and symbols.

The map has independent presentation bounds: 5280×2970 for the world and
3360×1890 for regions. At normal zoom 1, they span 2.75 and 1.75 screen widths.
Kingdom overview, campaign and local detail bands reveal information as space
allows. Different entry and exit thresholds keep labels stable near a boundary.
These dimensions change the drawing; the saved physical graph and travel costs
remain authoritative.

Gold, Wood and Stone stay in a lower strip with the last completed season's
actual income, upkeep paid/due and recovery spending. Attention starts collapsed
beside it, opens upward to three known conditions per page and collapses during
selection. Tap Attention to close the current selection and reopen the list;
tap an entry to focus its exact observed place or owned army. World-region warnings aggregate only known
internal threats, participant sieges and hostile contact cues.

Army Details opens composition, recruitment and people. Manage opens a place's
construction, roads, focus and local actions. Menu contains Kingdom, Records,
save/load, settings and help. These are the **current** paths; the map plan brings
common decisions and context closer to the world.

The current introduction walks through headquarters, movement, regional
navigation, careers, household review, the first turn and Records. It can be
dismissed or resumed from How to Play. Its replacement is M04 of the map plan.

## Current map limitations

M01A provides spatial navigation; common management actions and full seasonal
consequences still need M02's contextual inspector and outcome work. Saved routes
retain their existing selection-based detail; an unselected force mark reports a
queued route without displaying its full destination/path. All eight regional
graphs still repeat the same ten-site chain, and their local view reuses the
continental background. M03 owns that geography change.

These are known design problems addressed by the active plan. The map's full
canvas and unclipped controls alone do not establish strategic readability.

## Screen briefs

### Observer

Observer opens separately from the title menu. Choose an AI kingdom count and
world seed, then Start observing. Every kingdom uses the AI, and the world and
regional maps reveal all places, borders and armies without fog.

| Question | Observer screen |
| --- | --- |
| Current decision | Where to look and how quickly the simulation advances. |
| Dominant focus | The full atlas, with regional detail available on selection. |
| Primary action | Pause or resume; advance one AI action while paused; choose 1×, 2× or 4× speed. |
| Supporting information | Season, current AI kingdom, playback status and speed. |
| Deferred information | All AI kingdoms in a separate roster; local detail on selection; help in the menu. |
| Layout and camera | Full map at the sole supported 1920×1080 canvas; compact controls along the lower edge. |
| Input and feedback | Visible tap controls, drag/pinch and zoom buttons; selected speed and paused status remain visible. |

Opening a sheet suspends automatic turns. The simulation handles battles and
diplomatic decisions without human orders. Observer sessions are separate from
campaign saves; returning to the title restores any campaign already in memory.

### Strategic map

The [interface chapter](docs/10-interface-and-accessibility.md) owns the target
screen composition; the [map plan](docs/map-playability-plan.md#target-screen-brief)
owns its delivery. The map is the dominant play area. Calendar, active faction,
balances and actual seasonal accounts support it, with a compact attention list
that gives way to selection. Political fill, local control marks, owned forces,
settlement symbols and importance-based labels explain the known world. Camera
controls, Menu and End Turn remain reachable; the selected-place inspector and
movement card retain direct destination-tap movement. M01A establishes useful
working and overview scales with smaller unselected symbols and fewer full
cards. M02 brings common actions and seasonal consequences into that context.
Known political borders show each kingdom's color on its own side of a dark
outline. They retain the existing fog rules and world atlas shoreline mask.

The border-access correction keeps the selected army's next decision beside
the map: Enter Region exposes local attack targets without losing its route.
Supply warnings explain recovery, foreign control shows the diplomatic state,
and a closed peaceful border opens that kingdom's war review. The existing
confirmation remains the point where war is declared.

**1920×1080 is the sole design and acceptance resolution**, including logical
UI coordinates, per the user's 2026-10-01 correction. Native and browser checks
must measure the actual usable canvas. Smaller hosts can scale/letterbox the
same composition; there is no separate 1280×720 layout or acceptance requirement.
M01A composes the map directly in these coordinates. Its 48–60 pixel controls
retain their intended size, leaving most of the canvas for navigable terrain.
Resources and actual receipts sit above the bottom camera controls; Attention
opens upward at the right. A selected place uses one narrow side inspector.
Overview deliberately frames known land, Return View restores the working
camera, and Recenter focuses a selected army or home. Map Key explains scale
and symbols. Tutorial prompts share the quiet top strip and disappear when
completed or dismissed.

### Management and setup sheets

The current decision is the selected army, place, record, save or setup choice.
That sheet is the dominant focus; costs, eligibility and confirmation remain
next to its primary action. The map is dimmed behind it and secondary systems
stay in their own sheets. Existing sheet content keeps its readable pixel sizes
in a centered 1280×720 content region of the 1920×1080 canvas, without enlargement.
The toolkit viewport handles the host boundary; sheet drawing and pointer input
receive the same translation. Back, confirmations and on-screen text entry
remain visible touch controls. Sheet navigation preserves map camera context.

### Portrait and notification integration brief

This integration is in progress. People and Career keep their existing decisions:
choose a person, inspect their service, and act on available career or transfer
options. A 64px roster portrait and 128px Career portrait support recognition;
names, status, costs and visible actions remain readable beside them. Children
and unknown identities use generic silhouettes. Dated memories render only their
saved appearance. Portraits add no gameplay action or permanent map panel.

Seasonal notifications support deciding whether a recent change needs attention
before End Turn. The atlas remains dominant at the supported 1920x1080 canvas.
A compact rail beneath the header opens one narrow, dismissible card; the card
temporarily replaces the inspector/expanded Attention region. End Turn and map
pan/zoom remain available outside it. Details, Recent and per-type delivery
settings are deferred until selected. Visible Open, Close, Show on Map and paging
controls support taps without hover; opening a receipt alone does not move the
camera or issue an order. Counts, navigation and portraits use observer-safe facts.

### Battle screens

Preparation shows opposing formations in one landscape. Select a friendly group
to edit its slots, ordered tactics and leader, then Start Battle. Execution
offers play/pause, step, speed and skip; those controls only present the resolved
receipt. Aftermath shows surviving groups and Continue returns to campaign play.
Detailed reports remain retrievable.

M01A centers the existing battlefield content at its original pixel size within
1920×1080. Its inherited small tactic and playback controls, dense placement and
other recorded battle findings remain outside this map milestone's redesign.

The [battle plan](docs/battle-system-implementation-plan.md) records the completed
delivery and the [battle review](docs/verification/battle-review.md) records later
fixes and remaining presentation issues.

## Known validation limits

The [evidence index](docs/verification/README.md) owns the pointers to current
records and inherited limitations. M01A passed formatting, strict Clippy, the
800-line source limit and its final 14-scene native capture batch, all at actual
1920×1080. Its full suite passed all targets except the production-victory target;
one profiling test remains intentionally ignored. Automated native actions,
background browser gameplay, publishing and reload passed, with a complete
1920×1080 browser capture in [spatial-scale verification](docs/verification/spatial-scale.md).
These limits remain:

- The unchanged seed-88 production-victory test fails at its 240-round cap after
  corrected combat reactions/routs. The failure remains a balance issue.
- Historical minimum-size WebGL acceptance remained unverified: the viewport
  override produced a smaller rendered image with black remainder after interaction.
- Dense battlefield placement, target-priority editing, small tactic controls
  and aggregate morale have recorded presentation findings.
- Physical-touch testing was waived on 2026-09-28; further platform, performance
  and balance work was deferred under that scope. The waiver does not make those
  checks pass or remove visible touch-control requirements.
- One release profiling test is intentionally ignored.
- Native operating-system rapid-click timing remains unverified; the final
  headless checks exercise synthetic pointer frames and dispatched actions.

These inherited limits remain until superseded by scoped verification evidence.
Rerun affected checks for implementation changes and report exact remaining
failures. Do not reopen completed packages or weaken their regression assertions.

## Native development saves

Native debug startup resumes the catalogue's Continue entry once storage is
ready. Release and browser builds open the title. Use
`..\rust_management\cargo.ps1 run -p kestrum '--' --title` to open the native
title explicitly.

`examples/prepare_midgame.rs` creates a separate **Briarhold - Midgame Battle
Review** save through production commands: Year 16, 60 completed rounds, broad
discovery, developed holdings, multiple armies and ten named adults. It preserves
existing catalogue entries. The catalogue number and Continue entry depend on
local saves; they are not stable project configuration.

With the native game closed:

```powershell
..\rust_management\cargo.ps1 run -p kestrum --example prepare_midgame
```

Native catalogue storage lives under `%LOCALAPPDATA%\kestrum`. The generator is
tracked; the user's catalogue is not a fixture to overwrite. Loaded campaigns
focus home at normal campaign scale. Overview explicitly fits known geography;
camera positions are transient and do not change the saved campaign schema.

## Development

Use the actual registered workspace. Macroquad is pinned to `=0.4.16`; toolkit
path is `../macroquad-toolkit`. Follow [AGENTS.md](AGENTS.md),
[CODE_STANDARDS.md](CODE_STANDARDS.md) and [UI_STYLE.md](UI_STYLE.md).
Shared guidance is maintained in `rust_management/docs/`.

```powershell
cargo fmt -p kestrum -- --check
..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test -p kestrum --all-features
.\publish.ps1
```

After meaningful game changes, no-parameter publishing is required. For UI
changes, use supported scenes in `scripts/capture_ui.ps1`, write directly to
`docs/verification/`, replace equivalent captures and confirm launched games
exit. Use an actual 1920×1080 canvas plus browser interactions. See
[delivery and validation](docs/12-delivery-and-validation.md) for completion rules.

## Code and artwork

- `src/data/`: typed content and semantic validation.
- `src/state/`: authoritative campaign state, invariants and compatibility.
- `src/engine/`: commands, simulation, observer projection and faction rounds.
- `src/navigation/`: map targets, cameras and exploration.
- `src/game/`: input/action coordination, projection, storage and capture.
- `src/ui/`: rendering and explicit player intents.
- `assets/data/game_config.json`: save identity, asset paths, timing and atlas label placement.
- `assets/data/game_text.json`: title copy, season names, geography names and interface text moved from `game_config.json`.
- Other files in `assets/data/`: setup, balance, world layout and scenarios.
- `assets/art/kestrum_atlas.png`: original atlas generated with OpenAI ImageGen.
- `assets/fonts/`: Cinzel (SIL OFL) and DejaVu Sans (Bitstream Vera license).
- `catalog_thumbnail.png`: stable publishing thumbnail.

Current new campaigns use atlas layout revision 3. Existing saves retain their
supported geography. The map plan requires versioned topology compatibility
before new regional connections are introduced.
