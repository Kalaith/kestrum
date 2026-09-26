# Initial title and empty atlas verification

Date: 2026-09-26. Validated in the actual `kestrum` checkout on `master`.

## Scope

The user explicitly chose an empty illustrated landscape for New Game. No nodes,
armies, ownership, or recruitment were added. The presentation is ready for those
systems to be drawn in world space during later milestones.

## Automated checks

- Formatting: `cargo fmt -p kestrum -- --check` passed.
- Clippy: shared launcher, all targets/features, `-D warnings` passed.
- Tests: 13 passed (5 campaign, 5 navigation, 2 data/assets, 1 source-size gate).
- Every Rust file is below the 800 physical-line hard limit; tests live in `tests/`.
- Runtime source/configuration searches contain no old package/capture identity,
  action demo, energy system, grid, or debug controls.
- Canonical workspace registration and generated lockfile were installed with the
  management sync script; `sync-workspace.py --check` passed.

## Visual review

The shared hidden-window capture wrapper produced exact 1920 × 1080 full-screen
and 1280 × 720 windowed evidence. All launched game processes exited; a process
check returned zero remaining Kestrum processes.

| State | Evidence |
| --- | --- |
| Title | `ui_title.png`, `ui_title_minimum.png` |
| Empty atlas | `ui_gameplay.png`, `ui_gameplay_minimum.png` |
| Zoomed atlas | `ui_zoomed.png` |
| Open menu | `ui_menu.png`, `ui_menu_minimum.png` |
| Settings | `ui_settings.png`, `ui_settings_minimum.png` |
| Reopenable help | `ui_help.png`, `ui_help_minimum.png` |
| New-game confirmation | `ui_confirm_new.png`, `ui_confirm_new_minimum.png` |
| Save error | `ui_save_error.png`, `ui_save_error_minimum.png` |

Reviewed map hierarchy, typography, contrast, hit-target sizes, clipping, and
modal separation. The map fills the logical canvas and ordinary controls occupy
only the edges. At minimum size, controls are 48 pixels tall. Help does not appear
by default. Settings and save controls require opening a menu. Empty selection
inspectors and unimplemented systems have no UI. Dense armies/locations are not
supported yet and no screenshots claim to exercise them.

The original template pause/scroll evidence was removed; same-state map captures
were replaced in place. `catalog_thumbnail.png` is the 16:9 title capture.

## Publishing

The required no-argument `publish.ps1` succeeded: Windows and WASM release builds,
packaging, asset registration, preview deployment, and catalog update completed.
Preview files are under `\\wsl.localhost\Ubuntu\home\kalai\dev\games\kestrum`.
Project Roost tracking warned that `127.0.0.1:80` refused its connection; this did
not prevent building or deploying the game.

## Browser checks and limits

The actual published preview was served over a temporary loopback HTTP server for
browser verification because the normal preview HTTP service was unavailable.
Embedded WebGL play verified New Game, End Turn from Spring/Turn 1 to Summer/Turn
2, zoom, drag, Recenter, menu opening, manual save, loading, and Continue restoring
Summer/Turn 2 after a complete browser reload. Both geographic-name visibility and
high-contrast labels were exercised through the settings overlay.
No browser console warnings/errors were reported during initial checks.
The final embedded browser view is retained in `ui_browser.png`. The temporary
server and verification tab were closed, and the browser viewport override reset.

The in-app browser rendered the full-screen canvas at 720p and 1080p, but resizing
and input became unstable after entering browser fullscreen (one black render and
ignored campaign clicks). Leaving fullscreen restored embedded input. This is an
unresolved browser/fullscreen verification limitation; native full-screen captures
passed. Physical touchscreen taps and two-finger pinch could not be exercised with
the available browser control API. The toolkit gesture path and camera contracts
are covered by deterministic tests; this is not a substitute for device testing.
