# Kestrum collaboration preferences

These project-specific instructions supplement the shared RustGames checklist.

## Kestrum validation override (Daniel, 2026-10-03)

Daniel explicitly authorizes resuming development with proportionate validation.
For Kestrum this supersedes the shared full-suite/publish-per-slice gate;
do not edit shared policy or apply this exception to other games.

- Before a slice, name its owned files, behavior, relevant checks and evidence.
  Freeze those files for validation, review the complete diff, then commit the
  buildable slice before starting the next independent feature. Update `todo.md`
  with outstanding work only, including checks actually still missing.
- Each Rust slice: formatting, strict all-target/all-feature Clippy, source-size
  gate, and any retained test target that covers the changed area. Gameplay
  changes are validated by running and exercising the feature, not by tests.
  UI-only changes need an affected binary build and relevant hidden 1920x1080
  captures/interactions. Documentation uses diff/link/command review.
- Tests follow the shared testing policy (`CODE_STANDARDS.md` §11). The
  2026-10-09 review kept only the enforcement gates (`code_standards`,
  `asset_registry`), data integrity (`content`, `construction_data`,
  `persistence`, `atlas_revision`) and settled algorithms (`ai_routes`,
  `generation`, `portraits`, `portrait_rendering`). Do not write new tests or
  restore deleted gameplay, AI, UI or scenario suites; when changing behaviour
  a retained test asserts as experimental, delete that test.
- Run the full remaining suite when changing content loading, saves, world
  generation, route planning or portraits, and record why. Reuse unchanged
  results; rerun only after a relevant change or unresolved failure.
- Backward save compatibility and migrations are optional for this prototype
  (Daniel's clarification). Unsupported saves must show a clear recoverable
  error without crashing. Do not build migration machinery solely to satisfy
  earlier guidance or remove unrelated working code.
- Build WASM once when browser acceptance is needed; reuse that exact output
  across independent headless profiles. Capture only affected scenes and reuse
  the current build with the wrapper's `-SkipBuild` when valid. Never treat a
  dispatched click or a stale screenshot as acceptance.
- Do not run `publish.ps1`: its deployment/tracker effects are outside this
  task's authorization. Local pooled builds and headless verification are the
  authorized substitute; report external publication as deliberately unrun.
- Failures need diagnosis; do not weaken tests or claim unrun checks passed.
  Record blocked checks with their exact command.

## Delegation

- Use Luna (`gpt-6-luna`) at extra-high (`xhigh`) effort for implementation and
  validation where applicable. The primary agent owns feature planning,
  integration decisions and review of the results.
- Delegate test runs, Clippy, source-size checks, captures, publishing checks
  and evidence collection. Freeze relevant source edits before validation;
  coordinate shared build commands so tests do not race ongoing edits.

## Unobtrusive verification

- Keep verification headless. Never activate or take over the user's display
  for testing unless the user explicitly requests visible interaction.
- A hidden browser tab does not make its Full Screen button safe to invoke:
  `requestFullscreen()` can still take over the display. Do not invoke that
  button or the browser's native fullscreen controls during background checks.
- Use a headless browser viewport for canvas checks and the established hidden
  native capture wrapper. If a browser connection cannot remain unobtrusive,
  stop that interaction and report the unverified coverage.

## Commit cadence

- Commit each independently useful, validated part as it stabilizes, before
  starting the next part. Engine support, map projection and UI integration can
  be separate changes when each leaves a coherent, buildable project.
- Do not defer every commit until an entire feature or all visual review is
  finished. Keep genuinely exploratory or broken work uncommitted until its
  outcome is known, then finish and commit that part promptly.
- Continue following the shared master-branch, commit-message and clean-tree
  requirements. Preserve unrelated work; stage only files owned by the slice.
