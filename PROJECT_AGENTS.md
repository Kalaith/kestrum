# Kestrum collaboration preferences

These project-specific instructions supplement the shared RustGames checklist.

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
  requirements. Preserve existing work and include all current project changes
  at each appropriate commit boundary.
