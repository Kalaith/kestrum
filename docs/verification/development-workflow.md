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
