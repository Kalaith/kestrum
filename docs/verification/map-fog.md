# Map fog and frontier connections — 2026-10-01

Fog now follows the boundary between the nearest discovered and undiscovered
locations, with a soft transition. Known locations reveal their surrounding land
out to the map edges. Holding the northern band therefore clears the empty land
above it; complete discovery removes all exploration fog. The same rule applies
to physical sites in regional views, using atlas coordinates so panning and
zooming preserve the boundary.

Outgoing connections retain their authored bends and fade into the fog. These
are drawing hints only. The filtered campaign projection, map picking, movement
preview, saved exploration and discovery rules remain unchanged. Roads wholly
between hidden locations are not drawn, and hidden destinations gain no labels
or interactive targets.

## Validation in the actual checkout

Formatting, Clippy with all targets/features and warnings denied, and the
800-physical-line source gate pass. The focused map-fog, map-exploration,
movement, world-navigation and code-standards suites pass all 17 tests. Five
fog tests cover northern ownership, soft/complete/empty discovery, authored
connection geometry and hidden picking, regional isolation, and camera changes.
The existing five exploration cases continue to verify travel, persistence,
read-only previews and rejected orders. Broader simulation/balance suites were
not rerun for this presentation change; existing limitations remain in README.

The final `.\publish.ps1` run without parameters passed, building and packaging
both Windows and WebGL releases and deploying them to Preview at
`http://127.0.0.1/games/kestrum/`. Project Roost recorded the publish. Formatting
and `git diff --check` also passed after the final source review.

## Visual and interaction review

The shared capture wrapper produced seven scenes at each requested size:
1920 × 1080 (actual native framebuffer 1920 × 1061 after window framing), and
the minimum supported 1280 × 720. Its processes 6092 and 27784 exited normally.
Captures replaced their stable files directly in this directory.

| State | Normal | Minimum |
| --- | --- | --- |
| Northern band owned | [North](ui_fog_north.png) | [North](ui_fog_north_minimum.png) |
| Entire map revealed | [Complete](ui_fog_revealed.png) | [Complete](ui_fog_revealed_minimum.png) |
| Starting home region and outgoing road hints | [Home](ui_production_world.png) | [Home](ui_production_world_minimum.png) |
| Regional map | [Region](ui_production_region_map.png) | [Region](ui_production_region_map_minimum.png) |
| Movement preview and confirmation | [Preview](ui_move_preview.png) | [Preview](ui_move_preview_minimum.png) |
| Arrival and updated discovery | [Arrival](ui_move_arrived.png) | [Arrival](ui_move_arrived_minimum.png) |
| Dense developed kingdom | [Midgame](ui_midgame_map.png) | [Midgame](ui_midgame_map_minimum.png) |

The starting production-world and regional-map scenes both use the game's
automatic home-region framing. Northern, complete and midgame captures exercise
the world map. Every distinct view was inspected at both sizes. The terrain
remains dominant, the northern border is continuously clear, and thin road hints
extend into the soft boundary without disclosing hidden place labels. The dense
map keeps its existing ownership markers and army controls. Selected movement
uses one supporting card with readable costs and a reachable Confirm Move;
arrival exposes the new terrain and remaining movement. No controls were added.

The movement harness presses/releases the actual projected Army banner with
hover disabled, then uses the ordinary selection and confirmation actions. It
asserts the movement card and arrival remain on the map. Fog geometry tests also
exercise changed cameras, and existing navigation tests cover resized picking.
Physical-touch hardware and pinch gestures remain untested, as already waived
in README.

The published WebGL build was exercised through the in-app browser at a
1920 × 1080 page viewport. A fresh Rose campaign showed a road fading west from
Riverfold Bridge into fog. Clicking Army, Bridge and Confirm Move spent three
movement, left three remaining, and revealed Riverfold Orchard and its
surrounding terrain. Card dismissal, visible + zoom, destination picking after
zoom and World Map were checked without keyboard shortcuts. The world view
retained a fading westward road hint and a partial-region inspector without
disclosing undiscovered site records. No browser warnings or errors were logged.
The [browser evidence](ui_fog_browser.jpg) retains the page context.

The 1280 × 720 fullscreen browser override reproduced the previously recorded
in-app WebGL scaling problem: although the canvas and DOM viewport both reported
1280 × 720, the game image occupied only the upper-left portion with black
remainder. This is a browser-review limitation, not a minimum-size acceptance
pass; [evidence](ui_fog_browser_minimum.jpg) is retained. The temporary viewport
override was reset. Minimum-size native review passes. Physical touch remains
outside the checked scope.
