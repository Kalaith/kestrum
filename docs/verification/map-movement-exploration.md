# Direct map movement and exploration — 2026-09-28

The current decision is where the selected army should travel. The map remains
the dominant area; one compact card contains remaining movement, destination,
route cost, relevant risk/supply and Confirm Move. Army Details, Review Route and
Move Group remain optional. Colocated armies can be selected with Next. A normal
move ends on the map with its updated army and allowance; combat still opens its
report when an encounter needs attention.

New campaigns focus the camera on headquarters and reveal owned/occupied sites
plus their immediate physical-route neighbors. Army travel records each visited
site and its exits. Unknown places, routes and kingdoms are absent from the map
projection and cannot be picked or used by its route preview. Discoveries persist
after departure and across saves. Previewing or rejecting an order reveals
nothing. Old saves without exploration fields start from their present holdings
and armies; missing historical journeys cannot be reconstructed. The existing
authoritative simulation and AI keep their full graph.

## Checks in the actual checkout

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Pass |
| `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Pass |
| `..\rust_management\cargo.ps1 test -p kestrum --all-features --no-fail-fast` | Two pre-existing simulation failures remain after the migration regression fix below; all other test targets pass |
| `..\rust_management\cargo.ps1 test -p kestrum --test knowledge --test map_exploration --test code_standards` | Pass, 11 tests after the legacy-knowledge assertion was updated |
| Existing movement and world-navigation test targets | Pass in the focused and full runs |
| Source-size gate | Pass; every Rust source file is within 800 physical lines |
| `git diff --check` | Pass |
| `.\publish.ps1` without parameters | Pass; final Windows and WebGL releases deployed to Preview, Project Roost recorded the publish |

The five new exploration tests cover initial visibility/picking, read-only route
previews, persisted travel and observer separation, rejected orders/legacy-save
migration/invalid IDs, and projected army-banner geometry at changed camera
scales and scopes. Existing legacy knowledge coverage now compares witnessed
people separately from exploration: a legacy save cannot retain journeys that
its schema never recorded. The final focused rerun verifies that correction.

The full suite reproduces the two failures already recorded in README and
`review-fixes.md`: the four/eight-faction continuity scenario ends at round 268
before the required round 400, and seed 88 does not reach victory by round 120.
These remain unresolved simulation/balance checks. One existing release-only
profiling test remains ignored. No alternate project copy or fabricated workspace
was used to validate the change.

## Visual and pointer review

The shared `scripts/capture_ui.ps1` wrapper captured the following real scenes at
requested 1920 × 1080 and 1280 × 720 sizes, directly to the stable paths below.
Normal native images are 1920 × 1061 after window framing; minimum images are
1280 × 720. Wrapper-launched processes 32548, 18816 and 22188 exited cleanly.

| State | Normal | Minimum |
| --- | --- | --- |
| Home area and discovery fog | [Home](ui_production_world.png) | [Home](ui_production_world_minimum.png) |
| Army selected, remaining movement and nearby costs | [Selection](ui_move_map.png) | [Selection](ui_move_map_minimum.png) |
| Route preview and on-map confirmation | [Preview](ui_move_preview.png) | [Preview](ui_move_preview_minimum.png) |
| Arrival and newly revealed exits | [Arrival](ui_move_arrived.png) | [Arrival](ui_move_arrived_minimum.png) |
| Blocked route with Clear Threat | [Blocked](ui_move_blocked.png) | [Blocked](ui_move_blocked_minimum.png) |
| Exhausted movement and recovery instructions | [Exhausted](ui_move_exhausted.png) | [Exhausted](ui_move_exhausted_minimum.png) |
| Multiple armies and a long army name | [Stack](ui_move_stack.png) | [Stack](ui_move_stack_minimum.png) |
| Optional group selection | [Group](ui_move_group_dense.png) | [Group](ui_move_group_dense_minimum.png) |
| First-use Army instruction | [Tutorial](ui_tutorial_headquarters.png) | [Tutorial](ui_tutorial_headquarters_minimum.png) |

The map movement capture harness presses/releases the actual projected army
banner with hovering disabled, asserts BeginMove, and checks that the order card
has no full-screen overlay. It dispatches destination/confirmation through the
real game actions and asserts arrival while the map remains open. Stack captures
split the founding army through a legal command and select its other army.

Visual review confirms a dominant local map, a single supporting order card,
readable costs next to Confirm Move, visible 48-logical-pixel controls, and no
clipped primary actions or competing management panels. The card sits opposite
the selected army, and map drawing/picking respect its occupied area. Long names
truncate within the card without covering Next or dismissal. Movement exhaustion
explains closing the card and ending the turn; arrival remains evident through
the new army position and allowance after its temporary notice fades.

The final published WebGL build was exercised at
`http://127.0.0.1/games/kestrum/`. A fresh default Rose campaign began at Riverfold
Green with only its local exits visible. Clicking Army showed 6 movement; clicking
Riverfold Hollow showed a cost of 2; Confirm Move stayed on the map, left 4
movement, and revealed Riverfold Gate and Field. The tutorial advanced with these
actions. Visible + zoom, destination picking after zoom, card dismissal and
Recenter were also exercised without keyboard shortcuts. Evidence:
[browser route preview](ui_move_browser.jpg) and
[browser arrival](ui_move_arrived_browser.jpg).

The browser canvas reported 1936 × 1000 at DPR 1 during interaction. Its temporary
1280 × 720 viewport override reproduced the previously documented in-app browser
WebGL presentation problem (a smaller upper-left image/black remainder), so the
override was reset before continuing. Minimum-size native review passes; this
does not establish minimum-size browser acceptance. No browser warnings/errors
were logged during that interruption. Physical-touch hardware and pinch gestures
were not tested; the harness checks no-hover pointer input, and browser review
uses click equivalents of the visible touch controls.
