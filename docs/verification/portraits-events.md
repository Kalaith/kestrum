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

## First Kestrum implementation checkpoint

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

Project formatting, strict shared-pool Clippy with `--all-targets -D warnings`,
and whitespace checks passed. The source gate covers 432 Rust files; the largest
is 794 physical lines. No assets or UI captures are part of this checkpoint.

## Verification boundaries

All rendering checks must remain headless, use the supported 1920×1080 canvas,
and write stable captures directly into this directory. No visible window,
fullscreen interaction, display takeover, or physical input is authorized.
Automated checks do not establish human portrait recognition, human playtesting,
or physical-touch verification.

Implementation and integrated verification are in progress; this document does
not yet claim feature acceptance, rendered artwork, or a successful integrated
test run.
