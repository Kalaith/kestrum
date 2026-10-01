# M01A spatial scale and navigation

[Evidence index](README.md) · [Map plan](../map-playability-plan.md#m01a-spatial-scale-and-map-navigation)

Date: 2026-10-02. Checkout: `master`, M01A working changes over `09017fa`.
**M01A complete under the recorded scope.** Engineering checks,
automated native actions and background browser gameplay are recorded below.
The user requested headless completion; the final native checks used the hidden
capture wrapper. This report supersedes M01's presentation-scale results where changed.

## Implemented scope

- The sole design and acceptance canvas is **1920×1080**. The map HUD, title,
  accounts and inspector use those coordinates directly, without enlarging the
  previous composition. Management sheets center their existing pixel-sized
  content with matching drawing and pointer translation.
- The world is 5280×2970 and each region is 3360×1890 at working zoom 1: 2.75 and
  1.75 screen widths. Authored normalized positions, graph identities, travel
  costs and save schemas are unchanged.
- Overview, campaign and detail bands control visible information. Separate
  entry/exit thresholds prevent flicker: overview 0.68/0.78, detail 1.45/1.30;
  maximum zoom is 2.4. Presentation data validates these settings.
- New and loaded campaigns focus home, opening its region when needed. Explicit
  Overview fits known land; Return View restores the previous camera. Recenter
  focuses a selected force or home. World and individual regions retain their
  own session cameras across selection and management visits.
- Compact place and force marks retain 48-pixel targets. Crowded places and
  nearby forces offer group focus before precise selection; regional counts
  distinguish regional forces from armies sharing one site. Important places
  remain visible. Full force details remain available on selection.
- Map Key explains scale bands, counts and symbols. Help returns to its opening
  context; Map Key Back returns to the map, while Menu Help Back returns to Menu.

## Engineering results

All commands ran against the actual project checkout and registered workspace.

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Passed after final native pointer assertions |
| `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Passed after final native pointer assertions |
| `..\rust_management\cargo.ps1 test -p kestrum --all-features --no-fail-fast` | One failing target; all other targets passed; one release profiling test intentionally ignored |
| `tests/code_standards.rs` source-size check | Passed in the full suite; every Rust file remains within 800 total lines |
| No-argument `publish.ps1` | Final run passed (exit 0): Windows and WebGL release builds, preview deployment and ProjectRoost tracking |

The sole failing target remains
`k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`:
the production campaign does not reach victory within 240 rounds. The full suite
is **not passing**. This inherited balance failure was not suppressed or weakened.
The separate 400-round continuity target passed.

The full suite preceded the final toast-position and first-entry regional-focus
corrections. Their focused rerun passed all 27 tests: navigation (5), world
navigation (2), map overview navigation (3), world army orders (6), atlas revision
(5), movement plans (5), and source size (1). The final native batch then passed
the pointer-frame assertions and replaced all 14 captures below.

Relevant passing coverage includes camera bounds and anchored gestures, five
navigation cases covering extents/bands/context/data validation, crowded-place
and compact-force grouping, projection and picking, hidden-information rules,
world/region transitions, direct and queued movement, and save compatibility.
Camera changes do not introduce persisted camera fields or alter movement rules.

## Native capture and action evidence

The final hidden shared capture batch completed all 14 scenes with exit code 0,
using fullscreen to obtain the actual client size. The launched game process
(`20136`) was verified exited.
Every listed PNG was measured at **1920×1080**, and each was visually reviewed.
No smaller resolution is an acceptance target.

| Capture | Review scope |
| --- | --- |
| [Early home](ui_production_world.png) | Regional home at campaign scale, early fog, compact army and connected land beyond the viewport |
| [Developed campaign](ui_midgame_map.png) | Working neighborhood, regional force counts, accounts and readable header |
| [Developed frontier](ui_midgame_frontier.png) | Selected place, detail scale, inspector and local terrain |
| [Explicit overview](ui_spatial_overview.png) | Quiet minor places, important labels, known realm and Return View |
| [Navigation return](ui_spatial_navigation.png) | Restored working view and selected capital, with no stray menu overlay |
| [Regional detail](ui_overview_region.png) | Compact sites, control/danger cues, offscreen route continuation |
| [Urgent dense realm](ui_overview_urgent.png) | Eight-faction overview, long names, warnings and expanded Attention |
| [Partial travel](ui_world_move_partial.png) | Saved route, depleted movement, route controls and feedback |
| [Selected force stack](ui_world_move_stack.png) | Regional count, cycling selection and compact movement inspector |
| [Help](ui_help.png) | Centered sheet, navigation instructions and visible footer controls |
| [Title](ui_title.png) | Direct 1920 composition and readable primary actions |
| [Setup](ui_production_setup.png) | Centered sheet, readable choices and complete footer |
| [Army management](ui_midgame_army.png) | Full roster, costs and reachable management actions |
| [Dense battlefield](ui_battle_campaign_editor_dense.png) | Centered original-size battlefield; inherited small-control limits remain |

The reviewed states leave the map dominant and keep required map controls
reachable. Label priority and truncation limit dense overlap. Routes and terrain
continue beyond the working viewport; the existing atlas remains recognizable
at campaign and detail scales. The refreshed partial-travel capture confirms
feedback sits above the accounts without covering them.

The `spatial_overview` and `spatial_navigation` scenes also execute application
actions with assertions: selection/dismissal and Help retain the working view;
Overview returns to the exact camera; world and region cameras restore
independently; Help preserves both return paths. Campaign equality is checked
after accounting for existing region/world tutorial visit receipts. These are
automated application checks, not manual native interaction.

## Interaction evidence

The first published build was exercised in a background browser with both canvas
backing dimensions and CSS dimensions measured at **1920×1080**:

- An older Rose campaign loaded at round 2. Overview/Return View restored the
  working view, and Map Key Back worked at screen position `(960, 752)`.
- A `(250, -40)` drag panned without issuing an order. Selecting the force and
  moving from Bridge to Green charged exactly 3 movement points, from 6 to 3.
- End Turn reached round 3 with resources 560 gold, 246 wood and 178 stone,
  retaining the camera. Recenter restored the working scale.
- Three visible zoom-in taps reached Local detail. World Map and Enter Region
  returned to that regional detail camera.
- Save-name keyboard clicks used Clear, `m`, `a`, `p` to enter `map`; Cancel
  returned safely. The older Founding and round-2 entries remained in the
  catalogue alongside the round-3 checkpoint.

After final publishing, reload and Continue restored Rose in Autumn, round 3,
with 560 gold, 246 wood and 178 stone. The army remained at Green with movement
refreshed to 6; selecting it worked and Bridge still showed cost 3. Browser
warning/error logs were empty. Canvas backing, CSS bounds `(0, 0, 1920, 1080)`
and viewport again measured 1920×1080. The final
[browser capture](ui_spatial_browser.jpg) also measures **1920×1080** and was
visually reviewed with its complete bottom toolbar and padding. Full-page capture
with an explicit canvas-sized clip produced the complete export.

The hidden native batch passed assertions through the actual `ui::draw` path
using synthetic screen-coordinate pointer frames. These cover Menu/Settings
Back, press/release origin guards and the visible setup keyboard's
lowercase/uppercase/numeric pages, Clear, Backspace, Done and Back. Both spatial
scenes assert the actual 1920×1080 client size. This tests drawing and action
dispatch, not operating-system input timing or physical touch. An earlier live
native check also loaded the older developed Briarhold round-62 campaign.

## Scoped outcome

M01A is complete: the map has independent navigable extents, three information
bands, compact marks, explicit Overview/Return View and retained working views.
Native action assertions, background browser gameplay, publishing and reload
were verified at actual 1920×1080. M02 remains the next unimplemented milestone.

## Limits retained

- The existing regional chain topology remains for M03; M01A changes its
  presentation space. M02 orders/consequences and M04 opening guidance remain
  unimplemented.
- Battlefield content retains its original pixel sizes. Existing small
  tactic/leader/playback controls, dense placement, target-priority editing and
  aggregate-morale findings remain in the [battle review](battle-review.md#remaining-presentation-findings).
- The [physical-touch waiver](k18-touch.md) remains in force. Synthetic gesture
  and mouse checks do not establish device touch or hardware pinch acceptance.
- Native operating-system rapid-click timing was not independently established;
  the final headless review validates pointer frames and dispatched actions.
- Historical 1280×720 browser scaling failures remain historical evidence;
  they do not create a second acceptance resolution.
- This work does not establish a first-time human playtest, settled campaign
  balance or the broader M05 early/developed campaign acceptance.
