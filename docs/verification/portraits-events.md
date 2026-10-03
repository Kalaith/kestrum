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
tactical-RPG portrait reference. Its experimental source and exports are preserved
in place and are not production-approved. A built-in imagegen concept demonstrates
a richer illustrated direction; it is not an interchangeable production layer
set. An explicitly configured Astra high review recommends detailed near-front
art on the existing rig, with real aligned face/eye/nose/hair assets and mask QA.
No exposed control selects the built-in imagegen model, so no image-model upgrade
is claimed. Runtime integration and the release artwork remain pending.

## Verification boundaries

All rendering checks must remain headless, use the supported 1920×1080 canvas,
and write stable captures directly into this directory. No visible window,
fullscreen interaction, display takeover, or physical input is authorized.
Automated checks do not establish human portrait recognition, human playtesting,
or physical-touch verification.

Implementation and integrated verification are in progress. This document does
not yet claim feature acceptance, rendered artwork, or a full-suite pass.
