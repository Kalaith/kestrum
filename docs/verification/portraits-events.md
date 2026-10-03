# Portrait and campaign-event verification

Implementation follows [the portrait plan](../hero-portrait-generator-plan.md)
and [the campaign notification plan](../notification-plan.md), including G01–G04
and M02-N1–N3. Verification uses the actual checkout and its shared workspace.

## Pre-change baseline

Baseline revision: `bf843aa` on `master`, with a clean working tree.

- Formatting and shared-pool Clippy with `-D warnings` passed.
- The source-size gate passed; no Rust source exceeded 800 physical lines.
- The shared-pool test run passed the reported targets up to
  `k18_production_battle`, including the 400-round replay coverage. Cargo stopped
  at `four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`
  (`tests/k18_production_battle.rs:531`): seed 88 remained at war after round 240,
  with factions 1 and 2 independent and factions 3 and 4 eliminated. Later test
  binaries were not run by that baseline command. This failure predates the
  portrait and notification changes.

## Publishing constraint

The required no-argument `publish.ps1` was inspected but not run. Its shared
publisher deploys to a local preview tree and can write a Project Roost deployment
record externally. The task explicitly prohibits remote publication and tracker
writes. Build/test results must be reported separately and do not constitute a
passing publishing gate. Publishing or security configuration is not altered.

## Shared image primitives

Dependency checkout: `D:\WebHatchery\RustGames\macroquad-toolkit`, commit
`806b50f` (`CPU image composition`). Its working tree was clean after the local
commit. Changes are limited to asset decoding, a CPU composition module, its
export, and two integration-test files. No dependency or compiler version changed.

Owned decoded images support game-owned eviction without uploading source layers.
The shared functions implement straight-alpha over, mask tinting and area
downsampling in premultiplied-alpha space, followed by a final texture upload.

Toolkit verification passed with source frozen and Cargo offline: formatting,
strict Clippy, source size, all-feature tests (428 unit, 15 integration and 29
doc-tests passed; 9 doc-tests ignored), native example checks, WASM library and
example checks with analytics, and API documentation. The five new image cases
cover PNG/JPEG/pack decoding, blending, mask coverage, downsampling and transparent
edges against light/dark surfaces. These pixel tests do not verify portrait art.

Toolkit follow-up `3ff2a1e` adds owned encoded-byte loading with the same pack
precedence as decoded-image loading. This supports a game-owned compressed source
cache without retaining every decoded layer. Its full all-feature tests, strict
all-target/all-feature Clippy, formatting, source gate, native/WASM example checks
and documentation passed. Six asset/image integration cases passed; loose-file
loading still needs verification through the actual game runtime.

## First Kestrum implementation checkpoint

Committed as `9d6173d` on `master`, with a clean tree after the commit.

Required appearance descriptors now accompany people and retained encounter/battle
snapshots. A seed-derived allocator owns campaign reservations, and wholly legacy
saves receive frozen-catalog migration. Existing fixture creation paths allocate
identities instead of copying another person's appearance. The proof catalog
contains identity metadata only; portrait art and UI acceptance remain pending.

The event schema and transaction-owned collectors are a foundation. The compact
rail/card, settings integration, exact shared construction/development forecasts,
presentation copy refinement, and broader lifecycle regressions remain pending.
Current UI continues using its existing labelled presentation.

Focused checkpoint regressions passed: 47 tests across `portraits` (8),
`knowledge` (5), `continuity_feedback` (2), `legacy` (11), `lifecycle` (6),
`succession` (14), and the official `code_standards` source-size gate (1).
This is scoped evidence; it does not replace the complete regression run or
the portrait migration and event lifecycle cases still being added.

After the final snapshot-layout and fixture corrections, the changed-target pass
also passed all 21 cases in `ai_review`, `campaign`, `code_standards`,
`continuity_feedback`, and `portraits`. Across these passes, 57 distinct cases in
nine targets passed. Long campaign scenarios and the complete suite remain for
the integrated verification pass.

Project formatting, strict shared-pool Clippy with `--all-targets -D warnings`,
and whitespace checks passed. The source gate covers 432 Rust files; the largest
is 794 physical lines. No assets or UI captures are part of this checkpoint.

## Migration regression checkpoint

The subsequent complete `..\rust_management\cargo.ps1 test --locked
--no-fail-fast` run reached every target. The 400-round replay and actual version-2
portrait migration/round-trip cases passed. Five targets failed: the event module
source-size gate, the pre-person military migration fixture, a portrait golden
value, two progression fixtures that reused an issued person ID, and the existing
seed-88 production victory-cap case. The first four are regressions being fixed;
the victory-cap test remains a failing required check, not a pass or waiver.

The victory test still uses seed 88 and a 240-round cap. Its current outcome has
factions 1 and 2 independent, factions 3 and 4 eliminated, and phase PlayerTurn,
matching the baseline failure category. It took 277.21 seconds in this run;
performance and the event query costs remain under review. No AI or victory-test
behavior has been changed to mask this result.

After correcting the source split, frozen golden expectation and historical
fixtures, the full `test --locked --no-fail-fast` rerun passed every target except
the unchanged seed-88 victory-cap case (258.29 seconds). Portrait tests passed
15/15, progression 10/10, and the economy migration case passed. The 400-round
replay passed in 180.76 seconds. The additional portrait cases cover distinct
legacy/modern migration, optional notification snapshots and canonical allocation
regressions rather than replacing useful existing coverage to meet a test count.

Strict all-target/all-feature Clippy, project formatting and whitespace checks
passed. The canonical source gate passed; a direct scan found 433 Rust files,
maximum 793 physical lines, with none over 800. Fifty local documentation links
and anchors across the edited feature documents and art notes passed review.

The user explicitly authorized committing validated portrait/event slices with
the documented pre-existing seed-88 failure retained. That exception does not
cover new regressions or claim a full-suite pass. Publishing remains prohibited
under the separate task constraint above. Unfinished event corrections and
artwork remain unstaged at this portrait migration commit boundary.

## Revised artwork direction

The user rejected the flat vector proof in favor of the supplied detailed
tactical-RPG portrait reference. Its experimental vector source is retained, while
the illustrated exporter replaces the rejected runtime exports. A built-in imagegen concept demonstrates
a richer illustrated direction; it is not an interchangeable production layer
set. An explicitly configured Astra high review recommends detailed near-front
art on the existing rig, with real aligned face/eye/nose/hair assets and mask QA.
No exposed control selects the built-in imagegen model, so no image-model upgrade
is claimed. The user then selected a cleaner anime treatment closer to the supplied
reference. The [current concept](portrait_direction.png) uses crisp linework,
restrained cel shading and front-facing original characters. Its complete
[generation prompt and provenance](../../assets/art-source/portraits/concept.json)
are retained with the artwork sources. This concept is a reference for component
production; it does not prove aligned layers, thumbnail recognition or runtime
rendering. Runtime integration and the release artwork remain pending.

## Illustrated component proof

The cleaner anime direction is now represented by real transparent components,
not just the concept sheet. The G02 export review composited all 336 legal color
combinations across its 24 geometry tuples. It validated 111 runtime PNGs and 44
aligned source/mask pairs. The reviewed [40/64/128px contact sheet](portrait_contact_sheet.png)
is 880x1814 pixels; its G02 SHA256 is
`F229E7D013FE52D09CB53F9FE8B578893CA69146B0A02B69D9BDFD6FB5D15439`.
The illustrated exporter replaces the rejected vector exports; retained source
code is not an approval of that earlier treatment.

The lead accepted the crisp contours, illustrated hair and restrained shading as
the production direction. The near pair differing only in face silhouette is
subtle at 40px, and dark hair/skin on the dark surface is subdued at 64px. These
are observed limits, not evidence of human recognition or campaign-wide perceptual
uniqueness. The bounded G03 expansion adds rounder features and varied hair
silhouettes while preserving the approved G02 exported bytes. Runtime compositor
parity, integrated UI captures and the full release matrix remain pending.

## Event and rendering integration review

The integrated full `test --locked --no-fail-fast` run reached every target.
All targets passed except the documented seed-88 victory-cap case and two new
notification fixture failures. The seed-88 outcome remained round 240 with
factions 1 and 2 independent and factions 3 and 4 eliminated (69.23 seconds).
The 400-round replay passed in 160.96 seconds. These timings are observations,
not a controlled performance comparison.

The notification fixture review corrected suppression-cutoff expectations and
used an actual blocking threat to exercise facility forecast clearance. It also
exposed a production issue: an undelivered construction warning could lose its
episode counter while its order remained open. Retention now preserves that
counter until the order closes, with explicit clear/recur assertions for both
delivered and capacity-omitted warnings. Validation of this correction and hidden
runtime captures are in progress; the earlier full run does not validate this
later change.

After that correction, formatting, strict all-target/all-feature Clippy, the
source-size gate and all 19 focused event tests passed. The first hidden native
capture attempt used the shared wrapper with the release binary, `-SkipBuild`,
`-Frames 20`, `-WindowWidth 1920`, and `-WindowHeight 1080` for `notifications`,
`notification_details`, `notification_warning`, `notification_settings`, and
`notifications_dense`. No `-Visible` or `-Fullscreen` option was passed.
Kestrum's capture assertion at `src/game/spatial_capture.rs:171` measured a
1920x1061 framebuffer, 19 pixels shorter than the requested 1920x1080. The audit
stopped before writing any of the five screenshots. Game PID 24052 and wrapper
PID 24160 both exited; the wrapper removed its temporary scene manifest. Its
failed-run diagnostic files remain at `.capture_stdout_24160.log` (0 bytes) and
`.capture_stderr_24160.log` (247 bytes) until the sizing repair is proven. No
fullscreen or visible-window retry was attempted. An exact hidden-client resize
fix is being made in the shared capture tool, and the captures remain
unverified.

The illustrated source/export checkpoint is committed as `36cea85`. Its 111
frozen layer files are not yet enabled by the runtime catalog. The source set
also includes the authored expansion masters; release-matrix validation remains
pending. The reference compositor produced the durable 128px/256px fixtures
documented in [the art notes](../portrait-art.md).

## Current event-engine checkpoint

The current focused run passes all 20 notification tests: six shared-boundary
forecast cases plus receipt/lifecycle, Attention, migration, movement,
transaction and warning-retention regressions. The added decline/ruin case
establishes a real owned observer at the subject site, checks the conditional
receipt, then compares an unchanged seasonal boundary with recovery before that
boundary. The authored habitation sequence is Village to Hamlet. Early fixture
failures omitted observation or expected Camp; they did not justify changing the
game's visibility or development rules.
Separate cases retain independent coverage for save migration, observer boundaries,
continued movement, construction ordering and omitted-warning recurrence; these
failure modes warrant more than the five-case feature target.

The 11 current CPU rendering cases pass. The Rust compositor matches every
reference alpha and premultiplied RGB byte at both 128px and 256px (maximum
delta zero). Metadata-only tests explicitly use the frozen legacy catalog now
that the current v1 catalog enables real artwork. Formatting, strict Clippy and
the source-size gate pass. The mandatory post-fixture full rerun completed with
exactly the documented seed-88 round-240 victory-cap failure; all other targets
passed, including all 15 portrait identity tests and the 400-round replay
(149.43 seconds). The user explicitly allowed this documented baseline exception
for validated slices. This is not a full-suite pass. The no-argument publisher
remains unrun because it can update an external deployment record and the user
prohibited publication and external tracker writes.

The first resize repair still measured 1920x1061, before any scene rendered.
The second repair preserves visibility, focus and stacking order while skipping
the hidden window's default size-clamping message. Its targeted toolkit test
and strict Clippy pass. The subsequent hidden run produced eight verified
1920x1080 PNGs before an inherited battle-capture fixture failed in
portraits_known. A separate hidden run produced missing-source, life-history
and Career captures at the same dimensions. Both game processes exited.
The runtime reports a 1920x1080 canvas at DPI 1. No visible or fullscreen
retry was used. The remaining known-enemy scene needs its real-battle fixture
repaired; the first visual review also found rail/settings label clipping.

Before the rendering code is committed, its tests will be moved onto a public
production rendering API. The current private test-module declarations under
src/game/portraits/ do not satisfy the shared test-placement standard. This
correction is separate from the event-engine checkpoint.

## Verification boundaries

All rendering checks must remain headless, use the supported 1920×1080 canvas,
and write stable captures directly into this directory. No visible window,
fullscreen interaction, display takeover, or physical input is authorized.
Automated checks do not establish human portrait recognition, human playtesting,
or physical-touch verification.

Implementation and integrated verification are in progress. This document does
not yet claim feature acceptance or a full-suite pass.
