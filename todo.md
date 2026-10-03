# Outstanding agent work

This queue records unfinished portrait/event work at the user's wrap-up request.
Use the [portrait plan](docs/hero-portrait-generator-plan.md),
[notification plan](docs/notification-plan.md) and
[verification record](docs/verification/portraits-events.md) for contracts and
actual evidence. Keep this file outstanding-only. Other map milestones remain
in their existing roadmap.

## Portrait catalog expansion (G03, deferred artwork scope)

Portrait artwork is serviceable; Daniel excludes improving it in this pass.
The expansion/export tasks below remain deferred. Earlier compatibility details
are optional prototype design choices, not a mandate to preserve old saves;
unsupported formats must instead fail clearly and recoverably.

- [ ] Implement canonical catalog/allocation revision 2, supporting saved revisions
  1 and 2 without changing descriptor schema 1. Add
  `introduced_catalog_revision` (default 1) to feature and palette records;
  reject a revision-1 descriptor using a revision-2 feature in both allocation
  validation and runtime manifest resolution. Preserve frozen migration data,
  existing IDs, visual-equivalence keys, palette ramps, rig/anchors, 24 original
  geometry tuples and all 111 existing exported PNG hashes.
- [ ] Append `face_square`, `face_round`, `nose_aquiline`, `eyes_round`, and
  `hair_wavy`, `hair_braided`, `hair_topknot`, `hair_long`, `hair_undercut`.
  Append compatible geometry to reach four faces, three noses, three eye styles
  and eight hair silhouettes including bald: 288 tuples. Add the ramps below;
  do not recolor an existing palette ID in place.
- [ ] Transition a saved registry atomically on the first new person allocated
  with revision 2. Validate/transition a cloned registry, allocate successfully,
  then replace the original. Preserve existing descriptors and deceased
  reservations; use unused expanded space before recycling a deceased face.
  Failure must not alter reservations, gameplay RNG or accepted person IDs.
- [ ] Add focused expansion tests for immutable old metadata/assets, defaulted
  introduction revisions and forged old descriptors, exhausted tiny-v1 to v2
  allocation, invalid-transition rollback, and deterministic portable fixtures.
  Update ordinary campaign test helpers to use the current catalog; genuine
  migration fixtures must continue using `PortraitCatalog::load_frozen()`.

Agreed new ramps, listed as shadow / base / highlight RGB:

| Palette ID | Shadow | Base | Highlight |
| --- | --- | --- | --- |
| `skin_porcelain` | 145,99,91 | 230,187,168 | 255,226,208 |
| `skin_sand` | 113,77,47 | 188,140,93 | 235,195,139 |
| `skin_deep` | 23,18,22 | 66,43,40 | 123,84,69 |
| `hair_silver` | 74,79,94 | 180,186,198 | 241,244,249 |
| `hair_indigo` | 20,24,47 | 54,66,114 | 120,139,181 |
| `hair_chestnut` | 46,29,25 | 105,67,45 | 172,121,80 |
| `eyes_blue` | 23,42,68 | 65,116,161 | 150,204,226 |
| `eyes_gray` | 46,52,62 | 126,140,151 | 213,224,225 |

## Release artwork and runtime review (G03/G04)

- [ ] After the actual revision-2 catalog exists, run
  `scripts/export_illustrated_portraits.ps1` against it. The 26 registered masters
  and exporter are ready in `assets/art-source/portraits/illustrated/`; preserve
  the cleaner anime direction. Do not create an alternate review catalog.
  Expect 509 runtime PNGs and 210 source/mask pairs. Verify all 111 frozen hashes
  before and after; correct only new fits if registration or occlusion fails.
- [ ] Inspect the contact sheet at 40/64/128px, all 288 geometries at 64px, and
  skin/hair palette extremes on light/dark backgrounds. The harness rasterizes
  864 cases and counts 37,152 legal descriptors; do not describe that as 37,152
  rasterized or perceptually unique faces. Retain at least 45% visible authored
  iris coverage beneath front hair. Record observed readability limits.
- [ ] Register exactly the 509 runtime assets in `asset_registry.json`, excluding
  masters and parity fixtures. Preserve the v1 Rust/C# 128/256px parity checks.
  Update `docs/portrait-art.md`, manifests and verification with real results.
- [ ] Complete the remaining G04 context matrix: formation members, households
  and children, known/unknown enemies, deceased memories, battle reports and
  observer return. Check identity across promotion, transfer, reload and cache
  eviction, plus the revision-2 allocation transition. Capture relevant dense
  states with the shared hidden wrapper at 1920x1080.
- [ ] Measure revision-2 cold/warm composition and bounded source/GPU cache use
  in native and WebGL builds; record registry costs for representative older
  campaigns. Distinguish estimated cache bytes, WASM linear memory and measured
  frame timings. Automation does not establish human recognition or playtesting.

## Event and platform acceptance

- [ ] Resolve the headless browser input/import blocker in
  `scripts/verify_portraits_events.cjs`. Fresh WASM built successfully; the
  fail-fast harness confirms fixture, writer lock, correct 1920x1080 canvas and
  trusted clicks delivered to the Import control, but the title remains and no
  catalogue entry is written. See [current evidence](docs/verification/development-workflow.md).
  Diagnose the runtime/input path before later assertions; replace stable failed
  `ui_web_*` diagnostics only with reached states. Never invoke fullscreen.
- [ ] Collect current native/WebGL portrait cache and frame-time metrics. The
  successful capture wrapper output did not expose the expected metrics lines;
  no runtime cache measurements were collected in the final run.
- [ ] Finish any interaction coverage left unverified in the current scoped
  record: grouped/overflow navigation, last-page delivery settings, muting and
  re-enabling, save/reload acknowledgements, NPC progression with an open card,
  pan/zoom outside it, and return from History/Career/battle without stray orders.
  Use `scripts/verify_portraits_events.cjs` with a fresh actual-checkout WASM
  build; inspect resulting states rather than treating dispatched clicks as
  proof. Keep every browser run headless and never invoke fullscreen.
- [ ] Investigate the pre-existing seed-88 round-240 victory-cap failure in
  `tests/k18_production_battle.rs`. Preserve meaningful victory/save-roundtrip
  assertions; do not merely raise the cap or weaken the test. The user allowed
  this documented baseline exception for current commits, not a claimed pass.
