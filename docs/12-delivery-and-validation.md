# 12 Delivery and validation

[Documentation index](README.md) · [Active map plan](map-playability-plan.md) ·
[Evidence index](verification/README.md)

## Current delivery

M01 and M01A are complete under their recorded scope. M01A passed formatting,
strict Clippy, source-size checks and the final 14-scene native capture batch,
as recorded in [verification](verification/spatial-scale.md). The full
suite retains the single inherited production-victory failure and one ignored
profiling test. Automated native actions, background browser gameplay,
publishing and reload passed, with native/browser captures at actual 1920×1080.
M02–M05 remain unimplemented. Complete, validate and commit each independently
useful major change before beginning the next.

K01–K18 and B01–B07 are completed delivery records under their recorded scope.
Their packages must not be reopened because a historical chapter says a system
is future work. Preserve their useful mechanical contracts and regression tests.

## Scope and acceptance

The map work is accepted by observable player behavior: identify the realm and
useful places, understand forces and orders, act without losing map context,
and explain the consequences of a season. The plan provides milestone-specific
behavioral cases and supported-size visual scenarios.

The generational campaign, human/ordinary content, six-slot armies, 4–8 factions,
80 major markers and seasonal turns remain. More races, extensive diplomacy,
multiplayer, additional world locations and new combat mechanics are deferred.
M01A expands navigable presentation space and composes the map at the sole
1920×1080 canvas. Its 5280×2970 world and 3360×1890 regional bounds use normal
zoom 1, three information bands and explicit Overview/Return View navigation.
A seamless transition between world/regional scopes remains deferred.

## Required engineering checks

The user-authorized [project override](../PROJECT_AGENTS.md) supersedes the
shared per-slice full-suite/publish requirement for Kestrum. Before each slice,
record its affected behavior and focused checks. Review, validate and commit it
before starting the next independent feature. Run from the actual checkout:

```powershell
cargo fmt -p kestrum -- --check
..\rust_management\cargo.ps1 test -p kestrum --locked --test code_standards
# Add relevant --test <target> arguments to the same focused invocation.
# At a justified integration boundary (not automatically for every slice):
..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test -p kestrum --locked --all-features --no-fail-fast
```

Choose focused behavioral suites while developing; run the relevant integration
coverage before completion. Preserve useful regressions and strongly target five
cases per major feature. The source-size check belongs to `tests/code_standards.rs`.
Every Rust file must stay within 800 total lines, with no exemptions.

Use the shared launcher for build/check/test/Clippy/run. Formatting may use
ordinary Cargo. Do not change workspace membership, create alternate manifests,
copy the project, fabricate placeholder crates, override target directories or
clean shared caches to bypass a failure. Investigate and report the real path.

Do not run `publish.ps1` during this task: it has external deployment/tracker
effects. Pooled native/WASM builds and local hidden/headless checks are the
authorized substitute. Report external publishing as unrun. Documentation-only
changes use document checks without gameplay builds.

## Visual and interaction review

Read `UI_STYLE.md`. The user's 2026-10-01 correction sets 1920×1080 as the sole
design and acceptance spec. Review that actual usable canvas in native and
published browser gameplay; measure client/canvas dimensions separately from
window or page dimensions. Historical 720p checks do not create another target.
Include relevant early, selected,
dense, long-name, multi-army, urgent, blocked and post-season states. Check that
the current decision, action, cost and consequence are understandable.

Exercise visible tap/click controls, selection/dismissal, drag suppression,
zoom/recenter, Overview/Return View, group focus, regional transitions, immediate
movement, route continuation and cancellation, common contextual orders and
seasonal feedback. Include reload
when changed state persists. Check that both rendering and picking remain correct
after viewport changes.

M01A review also checks that working cameras survive selection, dismissal, End
Turn and management return, and that world and regional contexts stay independent.
Verify the three bands across their separate entry/exit thresholds. Inspect the
centered management sheets at their original pixel size and exercise translated
pointer input, confirmation and on-screen text entry. The inherited small
battlefield tactic/playback controls remain an explicit limitation.

Use `scripts/capture_ui.ps1` with supported scenes and the hidden-window default.
Write directly to stable files in `docs/verification/`, replace equivalent
captures, wait for completion and confirm the launched game exits. Do not create
scratch captures, backup directories or alternate capture pipelines.

Physical-touch testing was waived on 2026-09-28; further platform, performance
and balance work was deferred under that scope. Preserve that authorization.
Visible touch-capable controls and the sole 1920×1080 canvas remain requirements.
Record click equivalents, actual hardware touch and untested interactions
separately. Do not claim a deferred test passed.

## Existing limitations

The [evidence index](verification/README.md) links the latest recorded limitations:
the seed-88 victory assertion at its 240-round cap, historical 720p browser scaling and
dense battlefield presentation. Historical failures that have since been fixed
are not a new task queue. Conversely, an earlier full-suite pass does not erase
a later failure.

Rerun relevant checks after changes and report actual results. Do not weaken an
assertion or suppress a failure to close a milestone. An unrelated inherited
failure can be reported while independently useful map work proceeds; final
acceptance must state its practical limits. Keep old 720p results as historical
evidence; evaluate host scaling issues against the current 1920×1080 canvas.

## Save and world validation

Before changing topology, establish the compatibility boundary in
[the map plan](map-playability-plan.md#save-and-topology-compatibility).
Cover old and new worlds, queued orders, roads, partial control, pending battles,
sieges, knowledge and deterministic continuation. Do not use the user's save
catalogue as disposable test data.

Use the existing developed-campaign generator and capture harness when suitable.
Its native catalogue entry number is machine-local; verify the loaded campaign
rather than treating a number from an old report as a permanent identifier.

## Playability review

A source review and automated walkthrough establish behavior; first-time human
play establishes whether the intended decisions are understandable. M05 records
both what was observed and what remains untested. Keep the opening's 10–15 minute
target as a measured playtest goal rather than a promise.

The ordinary end-of-season view should answer what changed and where. A player
should be able to recall a meaningful place or person from actual events.
Compile success, a feature checklist, or screenshots without overlap cannot
stand in for these checks.

Use the [M05 campaign cases](map-playability-plan.md#campaign-review-cases) to
review decisions after discovery, preparation feedback, interrupted orders,
retrievable information and continuity. Record both a meaningful action and
deliberate waiting, including the player's reason and expected consequence.
Distinguish a missing rule, an existing rule the interface fails to explain,
and a balance/pacing question before proposing work.

Report engineering correctness, interaction coverage, player comprehension and
campaign pacing separately. Passing one is not evidence that the others passed.
The [early Stellaris review](stellaris-release-lessons.md) supplies rationale;
it adds no automatic requirement for new systems or a full campaign playtest
to every map milestone. Existing deferred checks and the physical-touch waiver
retain their scope. Record unexecuted cases as unverified.

## Documentation maintenance

The project README owns current behavior, the map plan owns next delivery, the
system chapters own design, and the decision register owns fixed constraints.
Update those documents in the same commit as behavior. Replace superseded active
instructions; preserve dated records and original source material as history.

For documentation-only changes, validate relative file links and heading anchors,
source-reference hashes, status consistency, commands and whitespace. Inspect the
final diff for accidental source/data changes. Do not claim runtime validation
from a documentation check.

## Commits

Work on `master` unless explicitly asked otherwise. Follow
`rust_management/docs/COMMIT_STYLE.md`: a clear subject in the game's voice with
a parenthetical tag, an honest body and AI co-authorship.

After validation stage the slice's owned files, including required evidence
and documentation. Preserve unrelated/pre-existing work without staging it.
Before finishing, verify clean `git status --short` and report the commit and
validation results. If something blocks this, state the blocker.
