# Kestrum

Kestrum is a generational strategy game about kingdoms, armies, people shaped by
service, and places changed by decades of war and peace.

## Current milestone

M01 of the [map playability plan](docs/map-playability-plan.md) is implemented:
a readable kingdom overview. [M01 verification](docs/verification/kingdom-overview.md)
records its checks and retained limitations. M01A, spatial scale and map
navigation at the sole 1920×1080 spec, is next and has not started. M02's map
orders and seasonal consequences follow, then regional geography and the opening
guide in M03/M04. This plan does not change the running game yet.
The [design index](docs/README.md) identifies the owning documents and historical
evidence.

K01–K18 and B01–B07 are complete under their recorded scope. This describes
implemented systems and earlier checks; it does not establish satisfying map
play, settled campaign balance or full platform acceptance. The current source
and the latest scoped evidence take precedence over old milestone prose.

## Implemented campaign

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

Tap an Army banner, then a destination to move immediately. Long orders use the
available allowance and save their remaining physical route. End Turn refreshes
movement and continues those orders; encounters and changed access can pause them.
A destination can also be queued when no movement remains. Cancel Route stops
it; another destination replaces it. Review Route inspects the saved order.

Tap a place to inspect it. Enter Region and World Map retain their respective
camera contexts. Drag to pan; pinch, the mouse wheel, or visible +/− controls
zoom. Recenter returns to the selected army or headquarters. Close dismisses
selection. End Turn remains reachable during movement selection.

The overview shows known political claims separately from local occupation and
contested control, settlement silhouettes and the owned capital. Labels respond
to zoom, importance and available space. Owned banners show identity, troop
strength and route, siege, idle or supply state; world stacks aggregate owned
forces without treating their internal sites as one physical location.

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

The current logical UI and atlas both use 1280×720, with large fixed-size
symbols and banners. Developed saves fit discovered geography into that view.
M01A migrates to the sole 1920×1080 canvas, separates world extent from UI size,
adds scale-dependent map detail and makes whole-kingdom framing an explicit
Overview action. Ordinary play will frame a useful part of a larger landscape.

M01 provides the overview; common management actions and full seasonal
consequences still need M02's contextual inspector and outcome work. Saved routes
retain their existing selection-based detail; an unselected banner reports a
queued route without displaying its full destination/path. All eight regional
graphs still repeat the same ten-site chain, and their local view reuses the
continental background. M03 owns that geography change.

These are known design problems addressed by the active plan. The map's full
canvas and unclipped controls alone do not establish strategic readability.

## Screen briefs

### Strategic map

The [interface chapter](docs/10-interface-and-accessibility.md) owns the target
screen composition; the [map plan](docs/map-playability-plan.md#target-screen-brief)
owns its delivery. The map is the dominant play area. Calendar, active faction,
balances and actual seasonal accounts support it, with a compact attention list
that gives way to selection. Political fill, local control marks, owned banners,
settlement symbols and importance-based labels explain the known world. Camera
controls, Menu and End Turn remain reachable; the selected-place inspector and
movement card retain direct destination-tap movement. M01A establishes useful
working and overview scales with smaller unselected symbols and fewer full
cards. M02 brings common actions and seasonal consequences into that context.

**1920×1080 is the sole design and acceptance resolution**, including logical
UI coordinates, per the user's 2026-10-01 correction. Native and browser checks
must measure the actual usable canvas. Smaller hosts can scale/letterbox the
same composition; there is no separate 1280×720 layout or acceptance requirement.
The current implementation still uses the old logical canvas until M01A lands.

### Battle screens

Preparation shows opposing formations in one landscape. Select a friendly group
to edit its slots, ordered tactics and leader, then Start Battle. Execution
offers play/pause, step, speed and skip; those controls only present the resolved
receipt. Aftermath shows surviving groups and Continue returns to campaign play.
Detailed reports remain retrievable.

The [battle plan](docs/battle-system-implementation-plan.md) records the completed
delivery and the [battle review](docs/verification/battle-review.md) records later
fixes and remaining presentation issues.

## Known validation limits

The [evidence index](docs/verification/README.md) owns the pointers to current
records and inherited limitations. M01 passed formatting, strict Clippy, the
800-line source limit, 23 final focused tests, native normal/minimum visual
review, publishing and the recorded browser gameplay/reload checks. Its full
suite and browser-size review retain these limits:

- The unchanged seed-88 production-victory test fails at its 240-round cap after
  corrected combat reactions/routs. The failure remains a balance issue.
- Minimum-size WebGL acceptance remains unverified: the viewport override again
  produced a smaller rendered image with black remainder after interaction.
- Dense battlefield placement, target-priority editing, small tactic controls
  and aggregate morale have recorded presentation findings.
- Physical-touch testing was waived on 2026-09-28; further platform, performance
  and balance work was deferred under that scope. The waiver does not make those
  checks pass or remove visible touch-control requirements.
- One release profiling test is intentionally ignored.

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
tracked; the user's catalogue is not a fixture to overwrite. Loaded developed
campaigns frame discovered geography, while early campaigns focus home.

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
- `assets/data/`: setup, balance, world layout, scenarios and presentation text.
- `assets/art/kestrum_atlas.png`: original atlas generated with OpenAI ImageGen.
- `assets/fonts/`: Cinzel (SIL OFL) and DejaVu Sans (Bitstream Vera license).
- `catalog_thumbnail.png`: stable publishing thumbnail.

Current new campaigns use atlas layout revision 3. Existing saves retain their
supported geography. The map plan requires versioned topology compatibility
before new regional connections are introduced.
