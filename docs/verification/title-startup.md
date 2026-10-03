# Title startup — 2026-10-03

Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`, starting with
no working-tree changes. The branch was already eight commits ahead of
`origin/master`.

## Change

Native debug startup no longer loads the selected campaign when catalogue
storage becomes ready. Debug, release and browser startup therefore stay on the
title screen until the player chooses Continue. The existing Continue action
and explicit save/load paths are unchanged. The native midgame-save example and
README now describe the title-first flow, and the completed startup item was
removed from `todo.md`.

## Validation

- `cargo fmt -p kestrum --check` and `git diff --check`: passed.
- `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings`: passed.
- `..\rust_management\cargo.ps1 test -p kestrum --all-features --test code_standards --test campaign --test persistence`: passed; source-size gate 1, campaign 5, persistence 8 tests.
- `..\rust_management\cargo.ps1 build -p kestrum`: native debug build passed.
- `scripts/capture_ui.ps1 -Scenes title -Frames 12 -WindowWidth 1920 -WindowHeight 1080 -SkipBuild`: passed and wrote [the title capture](ui_title.png). The hidden wrapper verified the game process exited.
- `scripts/capture_ui.ps1 -Scenes observer_world -Frames 12 -WindowWidth 1920 -WindowHeight 1080 -SkipBuild`: passed and wrote [the Observer capture](ui_observer_world.png). Its existing action assertions open Main Menu, select Continue, and restore the in-memory human campaign.

Running both capture scenes in one process exited 101 after the title image was
written, with a miniquad `graphics.rs:335` integer-overflow panic while moving
to the second scene. Each scene passed in its own wrapper run, so the combined
capture was not used as acceptance evidence. No harness changes were made.

Capture mode skips storage initialization and polling. These captures therefore
do not prove native startup against an existing on-disk catalogue. The source
change removes the only storage-ready auto-load path; the focused checks cover
title campaign state, persistence behavior, and the explicit Continue action.
No save catalogue entries were created or modified. Publishing was not run per
the project validation override.
