# World army orders — 2026-10-01

Army selection on the world map keeps the atlas open. Army Details opens the
selected army's roster, and Back returns to the same world view. Recenter keeps
the current scope. Next cycles armies represented by a region's world banner,
including armies at different sites; Move Group still requires physical
colocation. Region entry remains an explicit action.

A world destination resolves to a physical site, or to the cheapest accessible
known entrance of a region. Movement follows the existing discovered route
graph. It charges internal travel to the departure gate before crossing a world
edge, preserves per-member budgets and stops at the last affordable site. It
does not bypass peaceful borders, encounters, sieges or discovery restrictions.
The order card shows the total cost and included departure travel; Review Route
lists each physical edge. Its drawing responsibilities are separated into small
functions. Long roster names leave room for the Previous control.

## Validation in the real checkout

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Pass |
| Strict pooled Clippy, all targets and features | Pass |
| Focused world-army, movement, exploration, world-navigation and source-gate targets | 18 tests pass |
| Final world-army and source-gate rerun | 7 tests pass |
| Pooled full suite, all features, `--no-fail-fast` | One existing production-victory test fails; all other targets pass; one release profiling test remains ignored |
| Source-size gate | Pass; every Rust file is within 800 physical lines |
| No-argument `publish.ps1` | Windows and WebGL releases deploy to Preview; Project Roost records the publish |

The six new regression cases cover known regional entrances, full departure
costs for formations and people, partial travel stopping at the gate, peaceful
borders and hidden destinations, scope-preserving recenter/banner geometry, and
cycling regional armies while rejecting groups at different physical sites.

The full suite's failing target is
`k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`.
It reaches round 240 without production victory. This test's victory-deadline
problem was previously recorded in [review fixes](review-fixes.md); its current
deadline is longer. The 400-round continuity target passes in this run. These
world-order changes do not alter AI planning or the existing movement executor.
The final presentation cleanup is covered by strict Clippy, the focused rerun,
fresh captures and another no-argument publish.

## Visual and interaction review

The hidden shared capture wrapper writes directly to the stable files below.
Requested normal size is 1920 × 1080 (1920 × 1061 native client image); minimum
size is 1280 × 720. Final capture processes 4060 and 31708 exited, and no Kestrum
capture process remains running.

| State | Normal | Minimum |
| --- | --- | --- |
| Interior army departing a region | [Exit](ui_world_move_exit.png) | [Exit](ui_world_move_exit_minimum.png) |
| Region selected as destination | [Entrance](ui_world_move_entry.png) | [Entrance](ui_world_move_entry_minimum.png) |
| Partial movement and exhausted gate arrival | [Stop](ui_world_move_partial.png) | [Stop](ui_world_move_partial_minimum.png) |
| Armies at separate regional sites and a long name | [Banner](ui_world_move_stack.png) | [Banner](ui_world_move_stack_minimum.png) |
| Selected army's optional roster | [Details](ui_world_army_details.png) | [Details](ui_world_army_details_minimum.png) |
| Internal edge and world edge in the route review | [Route](ui_world_move_review.png) | [Route](ui_world_move_review_minimum.png) |
| Updated movement help | [Help](ui_help_movement.png) | [Help](ui_help_movement_minimum.png) |

Review confirms the atlas remains dominant during orders, with one supporting
card opposite the army. Costs sit beside Confirm Move. Next, Army Details,
Review Route, dismissal and Recenter remain visible and use the existing
48-logical-pixel controls. Long names stay clear of controls; departure costs,
route rows, stopped movement and help fit at both sizes. Management and full
route detail appear only on request. Arrival remains visible through the army's
physical site and allowance after feedback expires.

The harness presses/releases the actual projected world army banner without
hover, dispatches real destination and confirmation actions, and asserts that
selection, cycling, recenter, details and partial arrival retain world scope.
Its stack and roster fixtures use a legal split and actual physical movement.

Browser review at `http://127.0.0.1/games/kestrum/` uses the published WebGL build
and the visible full-screen control. A default Rose campaign starts at Riverfold
Green. World Map returns to the atlas; tapping its Army banner keeps Riverfold's
world marker visible and opens the Rose Host order card. Army Details, Back,
Recenter and card dismissal preserve that world view. No browser warnings or
errors were observed.

The in-app browser's temporary 1280 × 720 override again displays a smaller
upper-left image and black remainder despite reporting a 1280 × 720 canvas.
Resetting the override restores the normal image. This matches the presentation
limitation in [map movement verification](map-movement-exploration.md); minimum
browser acceptance remains unverified. Physical touch and pinch hardware were
not exercised. Native minimum-size review, no-hover harness input and browser
click equivalents do not establish physical-touch acceptance.
