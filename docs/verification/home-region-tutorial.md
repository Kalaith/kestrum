# A reachable home-region tutorial — 2026-09-28

The discovery change exposed a tutorial assumption: the first successful move
advanced to Enter Region even though production headquarters were standalone
villages several routes from any visible region. New production headquarters
now occupy the Green within one of the eight authored regions. The world marker
therefore represents the home region, and the camera opens inside it at 1.5×.
World Map returns to its already-known marker at 2.5×. Only the home site and
adjacent physical sites are revealed, not all ten regional places.

One adjacent regional route stays free for the first movement lesson. The two
local threats remain nearby: one on the other adjacent site and one two routes
away. Headquarters keep their Village tier, horse access, Training Ground,
starting resources and three formations. All eight regions provide equivalent
starting topology and resource access.

After movement, the guide names World Map, the home region marker and Enter
Region. It reacts to the actual map scope; entering the region and returning to
the world advance the existing receipts. Older standalone-start saves retain
their locations and discoveries. If none of their regions is visible, the guide
explains exploring to an entrance instead of asking for a hidden marker.

Atlas revision 3 adds horse access at the eight reserved regional sites. Revision
1 and 2 saves retain their original terrain tags and geometry; loading does not
move headquarters, armies or people. The authored route graph and marker/site
IDs have not changed. The revised starting choices apply to new campaigns.

## Validation

All commands ran in the actual project checkout through the shared build tools.

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Pass |
| `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Pass |
| `..\rust_management\cargo.ps1 test -p kestrum --all-features --no-fail-fast` | All targets except the two campaign simulations below passed; the unbounded victory test was stopped so the remaining suite could finish |
| `..\rust_management\cargo.ps1 test -p kestrum --test k18_production_battle --test regional_starts --test code_standards --no-fail-fast` | Five regional-start cases and source-size gate pass; victory script fails with the bounded diagnostic below |
| `git diff --check` | Pass |
| `.\publish.ps1` | Pass: Windows release, WebGL release, five packaged assets and deployment to Preview |

The five new cases cover distinct home regions for 4–8 factions with local-only
discovery; a visible, affordable, threat-free first move across five seeds and
four/eight-faction setups; camera framing and picking through the discovered
world/region projection; revision-2 save continuity; and invalid regional-start
content or current-revision terrain. Existing generation, atlas revision,
movement, tutorial, knowledge, navigation and persistence tests also pass.

Two previously failing campaign checks remain unresolved. The continuity run
now ends at round 265 instead of its required round 400 (previously 268). The
seed-88 victory script stopped advancing rounds under the new starts. Its old
unbounded run was terminated; a new 256-actions-per-round assertion reports the
failure at round 31, PlayerTurn, sequence 770, after 320 player and 419 NPC
orders. It remains a failing check, not a skipped or weakened victory assertion.
One pre-existing release-only profiling test remains ignored.

## Visual and interaction evidence

The shared hidden-window capture wrapper exercised headquarters, first move,
World Map, selection of the visible home marker, Enter Region, World Map again,
and the remaining tutorial actions through completion. It explicitly asserts
that the selected home region exists in the discovered projection. Movement
remains open during the region lesson, covering the actual state after arrival.

Seven scenes were captured at each requested size, 1920 × 1080 and 1280 × 720.
Actual normal images are 1920 × 1061 after window framing; minimum images are
1280 × 720. Processes 12016 and 24632 exited cleanly. A standalone minimum-size
region capture (process 32000, also exited) confirms the complete World Map
instruction is readable.

| State | Normal | Minimum |
| --- | --- | --- |
| Revealed regional home | [Home](ui_tutorial_headquarters.png) | [Home](ui_tutorial_headquarters_minimum.png) |
| First journey preview | [Move](ui_move_preview.png) | [Move](ui_move_preview_minimum.png) |
| Instruction after arrival | [Return](ui_tutorial_region_return.png) | [Return](ui_tutorial_region_return_minimum.png) |
| Available home marker and Enter Region | [Select](ui_tutorial_region_select.png) | [Select](ui_tutorial_region_select_minimum.png) |
| Region lesson completed | [Region](ui_tutorial_region.png) | [Region](ui_tutorial_region_minimum.png) |
| Full guide completed | [Complete](ui_tutorial_complete.png) | [Complete](ui_tutorial_complete_minimum.png) |
| Multiple local armies with long name | [Stack](ui_move_stack.png) | [Stack](ui_move_stack_minimum.png) |

The local army and destination dominate the opening view. The guide occupies the
existing header, while the supporting order card keeps costs and confirmation
together. World Map, Enter Region, Next and dismissal remain visible touch-sized
controls. Long army names stay within their card. Regional navigation preserves
fog and never reveals the rest of the region merely by entering it.

Physical-touch hardware is unavailable. Native minimum-size captures establish
layout and the real action-dispatch checks establish the tutorial sequence;
neither is a claim of physical pinch/touch-device verification. The previously
documented in-app browser viewport-emulation issue is still a platform limitation.

The published browser build loaded, but a separate test tab could not create a
campaign while the existing campaign held the save lock. That test tab was
closed. The existing tab then became unavailable to the browser session before
a separate save or reload could be completed. No save copy was created and the
existing campaign was not intentionally reloaded or overwritten. Consequently,
the new tutorial sequence is verified by native captures and action tests, but
not by an end-to-end interaction with the published WebGL build.
