# Portraits and campaign events

Current checkpoint: 2026-10-03. At the user's wrap-up request, the implemented
portrait proof/runtime and event system are committed together; the
release portrait expansion and remaining agent-run acceptance are in
[../../todo.md](../../todo.md). This record distinguishes implementation from
verified coverage and does not claim completion of G03/G04.

## Implementation and work split

Astra led planning, shared state/save integration, art direction and diff/visual
review. Luna workers were configured at extra-high effort for portrait and event
implementation, toolkit changes and validation. Backend runtime identity was not
exposed, so these are requested configurations rather than independently
verified model identities.

Persistent descriptors use stable feature IDs, explicit palettes and canonical
visual signatures. Deterministic allocation leaves gameplay RNG untouched;
legacy migration assigns once per retained person ID, including encounter and
pending/completed battle snapshots. Campaign reservations survive deceased
record pruning. Tiny-catalog tests cover finite exhaustion and explicit reuse.
A separate read-only review found no confirmed defect in current creation,
migration or reservation paths. Future callers of the low-level allocator must
continue supplying fresh monotonic person IDs.

The cleaner anime direction replaces the rejected vector proof. The current
catalog enables 111 real layered PNGs: two faces, two noses, two eye styles,
three hair silhouettes including bald, three skin/hair and two iris palettes.
All 336 legal color combinations across 24 geometries were raster reviewed.
The [contact sheet](portrait_contact_sheet.png) has SHA256
`F229E7D013FE52D09CB53F9FE8B578893CA69146B0A02B69D9BDFD6FB5D15439`.
The [art notes](../portrait-art.md) and retained source manifests record prompts,
registration, immutable hashes and composition rules. Image generation exposed
no model-tier selector; no image-model upgrade is claimed.

People/Career, formation, household, history, battle and notification call sites
use the bounded cache and authorized descriptors. Missing adult art uses the
neutral UI silhouette without rerolling identity. Child/unknown policy remains
separate. The public production compositor and request queue support external
integration tests; no test module is declared under portrait source files.
Budgets are 64 MiB encoded sources, 32 MiB decoded sources, 128 thumbnail textures,
32 detail textures and 128 queued jobs, with one composition per frame.

Campaign events collect observer-safe dated receipts at accepted transaction
boundaries, retain warning episodes, and share actual development/construction
rules for conditional forecasts. Muted risks remain in Attention. The compact
rail/card provides Recent, grouped details, safe subject links, read/dismiss and
local delivery settings. Read/dismiss changes are included in the next successful
full-campaign checkpoint or named save; clicking alone is not a durable write.

## Commits and dependencies

- `9d6173d`: persistent portrait identity and event receipt foundations.
- `fdf2fcd`: portrait compatibility corrections.
- `9834f70`: cleaner anime concept and provenance.
- `36cea85`: illustrated sources, 111 frozen runtime exports and reference images.
- `2d5fae2`: complete event-engine collection, forecasts and regressions.
- Shared macroquad-toolkit: `806b50f` CPU composition, `3ff2a1e` owned encoded asset
  bytes, and `72e4c669183a844a2d285cd57cdb98d3eb18ac5b` hidden capture client sizing.
  The toolkit's final tree is clean and its full validation matrix passed.

The final Kestrum integration commit is reported by the task's completion message
and Git history, avoiding a self-referential commit hash in this file.

## Final actual-checkout validation

| Check | Final result |
| --- | --- |
| Project formatting | Passed |
| Strict all-target/all-feature Clippy | Passed |
| Source limits | Passed: 461 Rust files, maximum 793 lines |
| Native release build | Passed, 2m06s |
| WASM release build | Passed, 1m45s; emitted an unused native-diagnostic-method warning. A native-only cfg guard was then added and native Clippy passed; WASM was not rebuilt after that guard. |
| Hidden native captures | All 13 passed at 1920x1080; game PID 8468 exited |
| Final full test command | Did not run tests: `cargo.ps1 test --locked '--' --no-fail-fast` incorrectly forwarded a Cargo option to the test harness. Compilation finished in 4m09s, then libtest rejected `no-fail-fast`. |
| Headless browser interactions | Failed harness navigation/import; reached title/How to Play instead of the campaign. Browser closed cleanly. |
| Publication | Not run under the user's no-publication constraint |

The previous correctly invoked full suite passed every target except the
established seed-88 round-240 victory-cap case; its 400-round replay passed in
149.43s. That run preceded the final rendering/API/UI corrections. The user
explicitly allowed the baseline exception, then requested committing all current
work with new findings deferred to todo.md and no more code changes. No final
full-suite pass is claimed, and the malformed command was not rerun after the
wrap-up instruction.

The five current external rendering cases cover full-frame C# reference parity
at 128/256px, complete/partial/legacy manifests, source reuse/eviction/reset,
missing/corrupt assets and age/unknown fallback policy, and bounded requests with
failure suppression/retry. Twenty event cases cover receipt ordering/lifecycle,
shared-boundary forecasts, observation, movement, migration and warning retention.
The additional event cases preserve distinct failure modes beyond the usual
five-case target. Fifteen portrait identity cases cover deterministic allocation,
compatibility and lifecycle behavior.

The full-suite baseline failure is
`k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`:
seed 88 reaches round 240 with factions 1 and 2 independent and 3 and 4 eliminated.
Do not treat the user's commit exception as a fix or weaken its victory assertion.

## Render and interaction evidence

Normal and minimum acceptance use the same supported **1920x1080** canvas.
Verification uses only the shared hidden native wrapper and headless Chromium;
no visible window, physical input, fullscreen button or Fullscreen API is used.

Initial hidden attempts exposed a 1920x1061 client despite requesting 1080.
The shared toolkit repair preserves focus, visibility and stacking order while
bypassing hidden-window size clamping. Subsequent captures measured both Win32
and Macroquad at 1920x1080, DPI 1. An old known-enemy fixture also failed because
it did not accept pending encounters; its repair uses the real StartPendingBattle
command and verifies the committed report. Failed processes exited, and only
the six verified wrapper-owned diagnostic logs were removed after recording
these facts. No alternate capture pipeline or scratch project was created.

The first visual pass inspected the event card, active warnings, dense People
and life history. The map remained dominant and End Turn reachable. Rail/settings
label clipping and a narrow adult fallback were corrected. Career now wraps the
full raw name, including a 64-character stress case, with bounded secondary text;
roster metadata sits below the action buttons. All 13 final native scenes succeeded at 1920x1080; the launched game (PID 8468)
exited. The lead inspected the full-name Career, dense roster, dated enemy
memory, corrected rail/settings and neutral fallback captures. The full name
fits two lines, settings labels fit their buttons and known-enemy text remains
explicitly dated. The dense-roster separators and duplicate global-settings Back label were
subsequently corrected in commit `3d5f652`.
Browser campaign interaction, preference persistence and portrait runtime
behavior remain unverified. The stopped review did not establish passing
browser acceptance.

## Measured costs and limits

Current native debug cache measurements and their limits are recorded in the
[development verification record](development-workflow.md#native-metric-output-slice).
Cache byte estimates are not measured process/GPU memory or release frame
percentiles; WebGL runtime measurements remain unverified.

At 40px the face-only oval/tapered near pair remains subtle; dark skin and hair
are subdued on charcoal at 64px. Automated art inspection is not evidence of
human recognition, perceptual uniqueness, physical-touch testing or human
playtesting. The larger G03 matrix is not exported or enabled: its registered
masters/export support are ready, with concrete continuation steps in todo.md.

The no-argument publisher remains unrun: the shared publisher can update an
external deployment record, and the user prohibited publication/tracker writes.
No remote push or publication is performed. Broader M02-M05 roadmap work remains
outside this portrait/event checkpoint.
