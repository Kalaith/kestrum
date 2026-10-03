# Development workflow and resumed TODO evidence

## Scope (2026-10-03)

Resumed from clean `master` at `8da9e44`. Daniel authorized continued TODO work,
proportionate checks, prompt feature-slice commits and Luna xhigh workers.
Portrait artwork and external publishing are excluded from this pass.
The [project instructions](../../PROJECT_AGENTS.md) record the authorized path.
Strict Clippy remains required for each Rust slice after Daniel's clarification.
Shared guidance corrections/resync belong to the separate management task.
Daniel further clarified that prototype save compatibility/migrations are
optional: unsupported saves must fail clearly and recoverably, without crashing.
Existing working coverage is retained, not expanded merely to preserve old saves.

## Configuration evidence

The inspected local Codex configuration defaults to `gpt-6-astra` with `high`
reasoning; a separate named profile has other settings. Worker spawn requests
explicitly set `model: gpt-6-luna`, `reasoning_effort: xhigh`, and independent
bounded work orders. The orchestration tool accepted these requests. No separate
server/runtime model attestation is exposed, so configuration is the extent of
the model-identity claim. The coordinator plans/reviews; workers implement and
execute checks with one build owner at a time.

## Cost audit and decisions

- 77 root integration-test targets and 393 test attributes were counted by the
  audit, including support sources. Counts alone do not establish redundancy.
- `campaign_scenarios` covers six distinct cases, including the four/eight
  faction 400-round replay, exact continuation through round-200 reload,
  a peaceful long campaign and a 24-entry save catalogue. Preserve these checks.
- The current warm baseline compilation took 0.95s; `campaign_scenarios` passed
  in 165.40s. Older records report 149.43s and, earlier, 552.03s. These are
  different runs, not an apples-to-apples optimization benchmark.
- Requiring the full suite after every UI-only slice repeated this expensive
  coverage without a corresponding simulation change. The new path avoids that
  repetition; no tests, victory caps or save assertions were removed/weakened.
- The last checkpoint's final full-suite invocation misplaced `--no-fail-fast`
  after the harness separator, paying 4m09s compilation without running tests.
  The resumed baseline uses Cargo's flag before any separator.
- The last visual batch captured 13 native scenes, despite only two outstanding
  polish findings. The UI cleanup needs three affected native scenes, including
  the existing notification action audit; unchanged screens need no recapture.
- Do not separately build release/native, recapture every screen, publish, and
  then rebuild WASM for each small slice. Use the capture wrapper's actual
  binary build and one fresh WASM build when browser review needs it.

## Execution record

Workflow-only commits `7ecd88c` and `db08342` passed documentation diff,
command/reference and staged-whitespace review. They claim no runtime coverage.

The single overdue checkpoint baseline ran with
`..\rust_management\cargo.ps1 test -p kestrum --locked --no-fail-fast`.
It completed in **292.508s wall time**, with 0.95s reported compilation.
All targets completed; only
`k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`
failed at its unchanged round-240 cap (line 531). One release profiling test
remains intentionally ignored. The source-size gate passed. This validates the
checkpoint's simulation/rendering API test baseline; the later UI-only drawing
edits have their separate focused checks below. It is not a full-suite pass.

The first sandbox attempt failed before Cargo because the shared pool lease
directory was read-only; the authorized escalated retry acquired slot 1.
Avoiding an unchanged full-suite rerun saves approximately this run's 4m53s per
UI slice on this checkout; no claim of a measured hour-to-minutes improvement.

## Notification settings and People spacing slice

`menus.rs` suppresses only the generic overlay Back while global notification
settings draw their own `CloseGlobalSettings` control. People rows remove the
separator crossing the following portrait, and move metadata/status baselines
up 3/4px. Four rows, portrait artwork, full movement/status information and
48px action hitboxes remain intact.

- Formatting passed.
- Pooled `test -p kestrum --locked --test code_standards` passed: compilation
  32.39s, gate 0.06s.
- Pooled `clippy -p kestrum --locked --all-targets --all-features '--' -D warnings`
  passed in 3.49s.
- `& .\scripts\capture_ui.ps1 -Scenes notifications,notification_global_settings,portraits_dense -WindowWidth 1920 -WindowHeight 1080 -SkipBuild`
  passed using the current debug build. PID 26384 exited. The notification
  scene also runs the existing pointer/action audit, including Back with the
  map card both open and closed.
- Coordinator inspected the refreshed 1920x1080 global settings and dense People
  PNGs: one legible Back label, no divider through portraits, readable metadata
  and unchanged action targets. Browser interaction remains separate work.

The first capture invocation used the shared wrapper directly without explicit
window dimensions and therefore retained a 1920x1061 client; the game's assertion
correctly rejected it. A worker then issued a prohibited `-Fullscreen` retry;
that call was interrupted after 9.5s with no output. No Kestrum process remained
in the subsequent process check. Whether that call reached fullscreen could
not be verified. The successful replacement used explicit dimensions and no
fullscreen/visible flags. This invocation mistake is not a toolkit regression,
and the interrupted retry is not claimed as acceptance. The three stable
screenshots deleted by the failed wrapper were replaced by the successful run.

No repeated simulation suite, release build or publication was needed for these
drawing-only changes. The earlier baseline failure remains an explicit exception.

## Headless import diagnostics slice

The current Rust checkout at `3d5f652` built for WASM release successfully in
76.631s (Cargo reported 1m16s). The exact pooled artifact was
`target/pool/slot-1/wasm32-unknown-unknown/release/kestrum.wasm`, 9,002,391 bytes,
SHA-256 `2ee67580ec295e515df38a060d457da5eb24511b925cb2d8aecefa06cec4b7ff`.
No publisher, deployment tracker or remote push was invoked.

The browser harness now waits for the app's held save-writer Web Lock rather
than assuming WASM exports imply completed asynchronous asset startup. Clicks
map through canvas bounds and span rendered frames with a bounded wait. Import
must produce an actual Imported catalogue entry before downstream interactions
can run. Failure captures the corresponding stable profile image and reports
storage, lock and canvas/input diagnostics. This is a diagnostic improvement,
not completed browser acceptance.

Three headless notification attempts used the same fresh WASM. The measured
viewport, canvas CSS and canvas pixels were all 1920x1080, DPR 1, with no
fullscreen element. The valid fixture key and app writer lock were present.
The current diagnostic shows `elementFromPoint(1680,850)` resolving to the
focused canvas; mouse handlers were installed and trusted pointer/mouse down/up
events reached that canvas. Despite this, the title remained visible and no
`mq-indexed:*` save keys were written. The captured Import control is enabled.
Neither coordinates, missing fixture, startup readiness nor a missed brief
press has been established as the root cause. No recovery overlay was observed.
An intermediate attempt exposed an out-of-scope diagnostic variable in the new
failure handler; its signature/call sites were corrected before handoff.

Blocking the host's external Ko-fi script also produces a separately identified
`kofiWidgetOverlay is not defined` page error. That host dependency failure is
not evidence of an import or persistence defect. Other game page errors remain
fatal. The root browser input/import cause still needs diagnosis; receipt,
preference/reload, portrait context and WebGL metrics acceptance remain unrun.
The stable `ui_web_notifications.png` is a title-screen diagnostic, not campaign
acceptance. No profile success or human/physical-touch verification is claimed.

Final `node --check scripts/verify_portraits_events.cjs` and `git diff --check`
passed. The final cleanup (profile-specific failure path and host-error
classification) was syntax/review checked without a fourth browser rerun.
The browser's `finally` closes it after failure. The two task-generated
Chromium `debug.log` additions were removed by restoring its exact original
tracked bytes, preserving the pre-existing diagnostic line.

## Seed-88 bounded diagnosis

The test player's recruitment ceiling is six formations across all armies
(`tests/k18_production_battle.rs:211`), while the configured AI can fill two
six-formation armies. A focused experiment increased only that script ceiling
to the configured army count times six. The source-size gate passed, but the
production battle test still failed at round 240 in 78.94s (6.65s compilation).
Command: `..\rust_management\cargo.ps1 test -p kestrum --locked --test k18_production_battle --test code_standards`.
The initial sandbox attempt could not acquire a pool lease and ran no Cargo;
the authorized retry produced these results.

The final trace still had factions 1/2/3 independent and faction 4 eliminated;
only faction 2 had live armies, both full. Late player moves repeatedly changed
targets around sites 1071/1072/1073 while attempting to reach site 1078. The
script ranks enemy holding count ahead of route cost (lines 284, 320 and 359).
This suggests investigating target selection and route commitment, but does
not prove a game-rule defect or an adequate strategy fix.

The unsuccessful recruitment experiment was removed completely: the test's
HEAD, index and worktree blob hashes all matched
`75c8b4635cf50ac7d7d79728ac3fda31828fdfcd`. No cap, assertion or production
balance was changed. The original baseline exception remains outstanding;
no full-suite rerun was needed after restoring the tested original source.
