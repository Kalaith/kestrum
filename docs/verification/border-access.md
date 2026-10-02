# Regional army orders and diplomatic clarity — 2026-10-02

## Reported campaign

Read-only inspection of the native Continue checkpoint matched the supplied
Summer, Year 16 screenshot (61 completed rounds, Briarhold, production seed 88).
Briarhold was already at war with Orren Frostmarch and Bera Pike. Rose Host,
Army 9 and Army 10 occupied Briarhold-controlled sites, but enemy sites broke
their supply routes to the headquarters at Westmere Green. The region's
political claim remained foreign, which made the occupied sites look foreign
at world scale. No native save was edited or copied.

Rose Host could legally advance from Alder Vale Market to enemy-controlled,
undefended Alder Vale Hall for four movement. The world-region interaction hid
that choice: tapping the army's own region cleared its preview, and Enter Region
was absent until a region was selected and could disappear behind a saved route.
The regional camera could also restore a view with the army offscreen.

## Correction

- A selected world army exposes Enter Region, including during route review and
  queued travel. Tapping its current world region opens the regional map.
- Entry retains the army and queued order. A useful remembered regional camera
  is retained; a view hiding the selected army is focused on that army. World
  camera restoration is unchanged.
- Local order instructions describe movement and attacks. Existing movement
  commands still capture undefended hostile sites or trigger battles/sieges.
- Supply attention says No supply; the army card explains that a missing
  friendly route to HQ prevents recovery. Supply does not prohibit movement.
- Foreign control and political claims show Peace/War. Foreign region
  inspectors expose Kingdom, and blocked peaceful routes open the kingdom
  controlling the blocking site. Declaring war still requires the existing
  explicit review and confirmation.

## Validation scope

Checks run against the actual checkout and registered workspace. The README's
sole supported normal/minimum acceptance canvas is 1920×1080. There is no new
smaller-screen layout. Native capture scenes use the shared hidden-window
wrapper and write directly to stable files here.

The new capture harness exercises five behavior groups: region entry and
camera restoration, queued-order retention, hostile advance, peaceful-border
decisions, and control/claim inspection. It uses synthetic no-hover pointer
press/release frames through the real UI and map picking. Press-only, missing
origin and dragged-release frames cannot activate Enter Region. These are
application interaction checks; they do not establish physical-device touch
or operating-system input timing.

## Results

- `cargo fmt -p kestrum -- --check`: passed.
- `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings`:
  passed against the final source.
- `..\rust_management\cargo.ps1 test -p kestrum --all-features --no-fail-fast`:
  336 passed, one failed and one intentionally ignored. The inherited
  `k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`
  still fails to reach victory within 240 rounds. The 400-round continuity
  target passed. This change does not claim a fully passing suite.
- The final focused rerun of `code_standards`, `world_army_orders`, `movement`,
  `movement_plans`, `diplomacy` and `diplomacy_integrity` passed all 26 tests.
  The source-size gate confirms every Rust file remains within 800 total lines.
- `git diff --check`: passed.
- `.\publish.ps1` without parameters: passed; Windows and WebGL release builds
  were published to Preview at `http://127.0.0.1/games/kestrum/`, with assets and
  the catalog thumbnail. Project Roost and the catalog were refreshed.

### Native visual and interaction evidence

All 13 screenshots below were visually reviewed at an actual 1920×1080 canvas,
the normal and minimum supported size. The map remains the dominant play area;
local attack directions and the region-entry action fit without clipping.
The long-name stack, blocked world route and dense political-claim inspector
retain their controls. Foreign political claims and local ownership are shown
separately, and the attack destination is visible after regional entry.

| State | Evidence |
| --- | --- |
| Unsupplied world army with Enter Region | [World order card](ui_border_cutoff_world.png) |
| Regional entry repairs an offscreen army camera | [Regional order card](ui_border_cutoff_region.png) |
| Hostile advance from an isolated site succeeds | [Captured hostile site](ui_border_hostile_attack.png) |
| Entering a region retains queued travel | [Queued regional order](ui_border_region_queued.png) |
| Peace blocks movement and offers Kingdom | [Blocked peaceful route](ui_border_peace_blocked.png) |
| The blocking kingdom opens for review | [Kingdom review](ui_border_peace_kingdom.png) |
| War requires explicit confirmation | [War confirmation](ui_border_peace_confirm.png) |
| Confirmed war permits advance and restores supply | [Advance after war](ui_border_peace_entered.png) |
| Foreign region exposes its relation and Kingdom | [Foreign region inspector](ui_border_foreign_region.png) |
| Owned site inside a foreign political claim | [Occupied site inspector](ui_border_owned_occupation.png) |
| World route across a peaceful border | [World route](ui_world_move_peace.png) |
| Dense army stack with a long name | [Army stack](ui_world_move_stack.png) |
| Updated movement tutorial | [Movement help](ui_help_movement.png) |

Captures used `scripts/capture_ui.ps1` with `-Fullscreen` and its default hidden
window. A windowed attempt correctly failed the strict size assertion at
1920×1061. Two inspector-fixture failures were corrected by explicitly creating
and revealing the foreign regional claim. The final fixture uses normal visible
campaign projection. Each scene then passed its assertions and replaced its
stable screenshot. Failure logs created by these attempts were inspected and
removed; no launched game process remained.

### Published browser smoke check

The published WebGL build loaded the existing browser campaign. Selecting Rose
Host, returning to the world map, tapping Enter Region, and returning to the
world map retained the selected army and its six remaining movement. The new
local attack instructions and persistent Enter Region control were visible;
no browser warnings or errors were recorded. No move or End Turn was issued.
See [published browser evidence](ui_border_browser.jpg).

The browser canvas was 1200×675 inside the page. Fullscreen briefly measured
1920×1080 but exited during the next automated interaction, so this is a
browser loading/navigation smoke check, not full-size browser acceptance.
Native evidence establishes the supported-size visual review. The temporary
browser viewport override was reset and the verification tab was closed.
Physical-device touch and operating-system rapid-tap timing remain unverified;
this report does not extend the recorded K18 touch waiver or close M02–M05.
