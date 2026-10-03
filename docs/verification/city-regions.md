# City-created regions

Date: 2026-10-03–04. Scope: the requested region-free opening, deliberate city
investment, city spacing, local countryside views and capital tutorial.

## Design and ownership

New production campaigns retain all 152 physical sites and their routes, with
one country-map marker per site. No region is unlocked at the start. The old
eight-cluster layout supplies authored geography; Rosemarch remains a separate
regression scenario. Derived geography is validated against its deterministic
source rather than permitting arbitrary saved topology.

A Village or Town may receive paid city development. The shared command and
projection enforce ownership, safety, supply, usable conditions, affordability,
and no directly neighboring City or Major City. Investment preserves population.
Natural growth cannot bypass the paid City transition. The AI uses the same
command and retains its upkeep reserve.

A functioning city unlocks a local view containing the center and connected
countryside. These are existing physical identities with unchanged ownership,
income and movement costs. The region closes if the center falls below City or
becomes ruined; no sites or saved references are removed. The local view must
use observer-visible state and must not disclose hidden neighbors.

The opening tutorial must use the ordinary paid action on the capital, then
enter that specific region and return to the country map. Management costs and
blocked reasons remain visible, and completing the guide dismisses its prompts.

The controls expose Develop City from both Manage's Overview and Local Actions.
Opening a review remains possible when blocked; only Confirm Action is disabled.
The review uses the same cost and conditions as the engine. Show Capital selects
the current administrative capital so its Manage control is available. A paid
success stays in Manage and the guide directs the player back to the country map
to enter the newly unlocked region. Resume recognizes an existing functioning city;
the region lesson checks the current capital and gives recovery instructions if
that region is unavailable.

Country and local cameras remain independent. City regions project existing
neighbors into local positions for places, armies, routes, picking, territory
and fog. Fields, woods and buildings use only revealed sites and their observed
development. Legacy authored regions retain their existing coordinates and
backdrop. Inspectors begin below the notification rail.

## Validation

The first slice, `50caecb`, covers production generation, save validation and
city-development engine behavior. The second covers controls, local rendering
and tutorial integration. The full suite ran at the engine integration boundary;
the UI slice reuses unaffected campaign results and adds focused navigation,
tutorial and hidden visual acceptance under the project validation override.

The initial full integration run used
`..\rust_management\cargo.ps1 test -p kestrum --locked --all-features --no-fail-fast`.
Wall time was 564.21 seconds, including a 3 minute 56 second cold-slot compilation.
The long `campaign_scenarios` target passed in 174.42 seconds. City development
(5), region membership/projection (3), generation (5), regional starts (5), and
the source-size gate (1) passed. Strict all-target/all-feature Clippy passed.

The full run retained the known seed-88 victory failure and found stale region
fixtures in `world_navigation` and `observer_visibility`, plus two midgame setup
failures. The former tests now retain their authored-region checks on Rosemarch.
The midgame diagnostic confirmed a real opening battle whose detailed report
expired under the existing 40-round retention rule. It also exposed a scripted
patrol grouping learners from different locations. Focused verification of these
fixture corrections then passed using
`..\rust_management\cargo.ps1 test -p kestrum --locked --all-features --test midgame --test world_navigation --test observer_visibility --test code_standards --no-fail-fast`:
midgame 5/5, navigation 2/2, observer visibility 6/6 and source size 1/1.
Wall time was 45.29 seconds (29.81 seconds compilation; midgame 14.66 seconds).
The repaired patrol uses each learner army's actual route evidence and ordinary
movement; the round-60 save still needs ten adults, eight trained heroes and
positive persistent battle/meaningful-encounter evidence. No game rule or test
round limit was relaxed. Formatting, diff checks and strict Clippy passed again
(Clippy wall time 5.37 seconds). Unaffected full-suite results were reused.

`scripts/capture_ui.ps1 -Scenes production_world,move_arrived -WindowWidth 1920 -WindowHeight 1080`
passed through the hidden shared wrapper in 2.55 seconds; its process exited.
The [opening country map](ui_production_world.png) shows separate physical nodes
with no unlocked region; the [arrival state](ui_move_arrived.png) preserves the
movement receipt and newly discovered neighbors. Both 1920×1080 images were
inspected. This first review identified an arrival heading that overlapped the
notification strip. The second slice moves both place and movement inspectors
below that rail; the refreshed arrival capture verifies the correction.

### City controls, countryside and tutorial

Focused verification passed for `world_navigation` (4/4), `tutorial` (8/8),
`city_regions` (3/3), and `code_standards` (1/1). These cover independent cameras,
local picking and army positions, current-capital investment receipts, rejected
and repeated payments, tutorial save/reopen behavior, changed capitals, and
region closure after decline or ruin. Distinct cases retain engine, navigation
and guide contracts rather than duplicating their implementation.

Each target ran separately with
`..\rust_management\cargo.ps1 test -p kestrum --locked --all-features --test <target>`.
The initial tutorial run passed 7/8; the same command passed 8/8 after the
fixture repair below.

The changed-capital tutorial fixture initially lacked a safe, supplied path.
It now authors and validates a two-edge path and uses ordinary Move Capital and
Develop City commands. Capture fixtures were also updated for physical markers,
paid city development, explicit region entry and the corresponding tutorial
receipt. These repairs preserve the simulation rules and immutable-focus
assertions. A Clippy conditional warning was corrected without changing behavior.

After the guide layout fixes, formatting, `git diff --check`, the source-size target
and strict all-target/all-feature Clippy passed again. Hidden native captures
show both entry points, affordable and blocked reviews, payment success, city
spacing, dense countryside, observer navigation and the capital tutorial's
return to the country map. The tutorial route and completion scenes execute the
remaining movement, career, household, first-turn and Records actions, and
assert successful completion. All accepted captures use an actual 1920×1080
canvas. The first batch emitted six valid images (`production_world`,
`move_arrived`, `production_region`, `production_region_map`, `city_dense`, and
`city_spacing_blocked`) before failing at the later overview fixture. Those
images were reviewed and reused because subsequent repairs did not affect their
rendered state. The corrected overview capture, subsequent observer/tutorial
batches, and final tutorial recapture all exited successfully. Every launched
capture process has exited.

Code and guide validation commands (before the final setup-copy correction):

```powershell
cargo fmt -p kestrum -- --check
..\rust_management\cargo.ps1 test -p kestrum --locked --all-features --test code_standards
..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 build -p kestrum --locked --all-features
.\scripts\capture_ui.ps1 -Scenes @('tutorial_city_overview','tutorial_city_actions','tutorial_city_review','tutorial_city_blocked','tutorial_city_success','tutorial_city_region','tutorial_region_return','tutorial_region_select','tutorial_region','tutorial_world','tutorial_route','tutorial_complete') -WindowWidth 1920 -WindowHeight 1080 -Frames 12 -SkipBuild
..\rust_management\cargo.ps1 build -p kestrum --locked --all-features --release --target wasm32-unknown-unknown
```

The source-size target's fresh test-profile compilation took 1m22s; its test
body took 0.06s. Clippy reported 16.92s. The native build reported 0.27s with
0.51s wrapper wall time. The final 12-scene tutorial batch exited 0. The initial
WASM release build reported 1m41s and measured 101.62s wrapper wall time; its
9,115,950-byte output came from pool slot 1. These measurements separate Cargo's
reported times from wrapper wall time where both were collected.

Reviewed evidence:

- [City investment on Overview](ui_tutorial_city_overview.png),
  [Local Actions](ui_tutorial_city_actions.png),
  [cost review](ui_tutorial_city_review.png),
  [unaffordable review](ui_tutorial_city_blocked.png), and
  [successful payment](ui_tutorial_region_return.png).
- [Spacing blocker](ui_city_spacing_blocked.png),
  [developed countryside](ui_city_dense.png),
  [observer region](ui_observer_region.png), and
  [known regional pressure](ui_overview_region.png).
- [Capital region](ui_tutorial_region.png),
  [country return](ui_tutorial_world.png),
  [tutorial route](ui_tutorial_route.png), and
  [completed guide](ui_tutorial_complete.png).

Visual review also corrected the Overview cost/readiness lines and widened
Show Capital so its label fits. All affected tutorial images were replaced
using the refreshed native build. The new `tutorial_city_region` and
`tutorial_city_success` capture aliases produced byte-identical images to the
existing `tutorial_region` and `tutorial_region_return` evidence. SHA-256
comparison confirmed the duplicates; only the established filenames were kept.
The local terrain depicts existing observed
development; broader route variety and geographic art remain separate map-plan
work.

### Browser acceptance

The first fresh WASM output was served locally with current checkout assets
and the existing WebGL host shell/shared runtime. Headless Chromium used an
actual 1920×1080 CSS and backing canvas at device-pixel ratio 1; no fullscreen
control was invoked. The visible-control flow was New Game → Show Capital →
Manage → Develop City → cost review → Confirm Action → Back → select Riverfold
Green → Enter Region → Country Map → select Rose Host. The guide reached its
movement destination prompt (5/9). Full guide completion was verified natively
as described above, not repeated in the browser.

The capital started as a Village with population 250/750 and no Enter Region
control. The review showed resources 500 Gold / 200 Wood / 150 Stone and cost
250 / 100 / 75. Confirmation showed remaining resources 250 / 100 / 75, City
population 250/1500, and the city-investment success receipt. Region entry and
country return were observed before capturing the
[browser movement state](ui_city_browser.png). Normal game persistence created
the campaign-catalogue keys without injected campaign state.

Startup took approximately 5.1 seconds. The interaction/review run took about
4m08s, including inspection pauses. Console errors, page errors, failed requests
and local-server misses were all zero. The browser and local server were closed
after verification. This establishes the changed browser flow with mouse input;
it does not establish physical touch-device coverage.

The same browser review found that the expanded New Game description clipped
at the right edge of its single-line layout. The copy was shortened to a complete
95-character line. Game text is embedded through `include_json!`, so this
specific visual repair required refreshed native and WASM outputs. The native
rebuild reported 1m26s (86.39s wrapper wall time); the second WASM build reported
1m38s (98.71s wall time) and produced 9,115,870 bytes in the same pooled path.
The same build commands above were used. The affected scene was recaptured with
`scripts/capture_ui.ps1 -Scenes @('production_setup') -WindowWidth 1920 -WindowHeight 1080 -Frames 12 -SkipBuild`;
it exited 0 and the [New Game description](ui_production_setup.png) fits on one
line. No Rust code changed for this correction, so behavioral and Clippy
results were reused; JSON parsing and native startup validation passed. The earlier
payment/navigation acceptance remains valid because only that setup sentence
changed. The reported vertical color boundary in the city view matches the
existing physical-control fill and border in native captures; it is not a
separate landscape defect.

The focused browser recheck used the refreshed WASM at 1920×1080 / DPR 1.
New Game opened in 6.5 seconds and the entire corrected line fit inside its
dialog. Console, page, request and local-server errors remained zero. The
setup image was inspected in memory; no duplicate file was written. The
browser and server were closed again after this check.

### Retained diagnostics and limits

The two diagnosed failed capture batches left four shared-wrapper logs:
`.capture_stdout_32012.log`, `.capture_stderr_32012.log`,
`.capture_stdout_404.log`, and `.capture_stderr_404.log`. Their contents and
timestamps identify them as this task's failure diagnostics, and the failed
processes have exited. Automatic approval review rejected exact-path cleanup
with only `blocked by policy`; the logs remain untracked and are excluded from
the feature commit. No alternate deletion method was attempted.

The Kestrum-specific workflow in [PROJECT_AGENTS.md](../../PROJECT_AGENTS.md)
requires local pooled builds and hidden verification; external publication is
deliberately unrun. The inherited seed-88 round-240 production-victory failure
is an authorized baseline exception, not a passing check. The existing physical
touch waiver remains in force; synthetic input does not establish device touch.
