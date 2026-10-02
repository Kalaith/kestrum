# Observer mode — 2026-10-02

[Verification index](README.md) · [Screen brief](../../README.md#observer)

## Scope

The title menu's Observe action opens a separate setup for 4–8 AI kingdoms and
a generated world seed. Every faction, including the former player slot, uses
the existing AI. The world and regional maps reveal all places, armies, borders
and active threats without altering faction exploration records. AI planning
continues to use each faction's private projection.

Automatic play has Pause/Resume, one-action Step while paused, and 1×/2×/4×
speeds. Menus suspend progression. Battles and diplomatic decisions resolve
automatically; the session continues after any individual faction is eliminated
and stops when only one independent kingdom remains. Kingdom rows focus their
headquarters, and map inspectors show local forces without order controls.

Observer sessions stay in memory. Entering retains the current human campaign
and camera; leaving restores them. Observer does not write campaign checkpoints
or expose save/load controls. Existing saves without the new mode flag remain
ordinary human campaigns.

## Regression coverage

Eight core tests cover constructors and normal-play compatibility, rejection of
human commands through both state and engine entry points, deterministic AI
steps, round boundaries, former-player elimination and a different winner,
automatic battles, older serialized saves, and playback timing. Six map tests
cover full visibility, unchanged knowledge, private AI views, military/supply
summaries, world/regional army targeting, and ordinary fog boundaries.

The two suites exceed the five-case target because compatibility, elimination,
battle resolution, pacing and information boundaries need independent failure
signals. They exercise existing campaign rules rather than mirror UI drawing.

The native capture scenes also use the real UI draw/action path with synthetic
pointer frames. Assertions cover title/setup entry, press/release origin guards,
automatic progress, menu suspension, Pause, Step, 4× selection, all-faction map
projection, and restoration of the prior human campaign through Continue.

## Engineering results

All commands used the actual checkout and its shared workspace. Build, test,
Clippy and capture commands used the shared three-slot pool.

- `cargo fmt -p kestrum --check` and `git diff --check`: passed.
- `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings`:
  passed.
- `..\rust_management\cargo.ps1 test -p kestrum --all-features --no-fail-fast`:
  every target passed except the inherited
  `k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`.
  It again reached round 240 without victory. The 400-round continuity test
  passed; the release profiling test remains intentionally ignored.
- After the final founding-faction validation guard, the Observer, visibility,
  persistence, strategic-campaign, diplomacy-integrity and source-size targets
  were rerun: **32 passed**. The save compatibility test also rejects an unknown
  founding faction, including in Observer mode.
- The source gate passed with no exceptions; the largest Rust file has 794 lines.

## Native visual review

`scripts/capture_ui.ps1 -Scenes title,observer_setup,observer_world,observer_kingdoms,observer_region,observer_developed,observer_help -Fullscreen`
completed with exit code 0. Its hidden game process (`17752`) was verified exited.
Every capture measures **1920×1080** and was visually inspected.

| Capture | Review scope |
| --- | --- |
| [Title](ui_title.png) | Observe is a distinct action beneath New Game; catalog thumbnail refreshed |
| [Setup](ui_observer_setup.png) | Eight AI kingdoms, seed choice, session explanation and tap instructions |
| [Full world](ui_observer_world.png) | Every region and army marker visible, no fog, quiet header and complete playback toolbar |
| [Kingdom roster](ui_observer_kingdoms.png) | All eight rows, faction colors, current AI, territory counts and reachable Back |
| [Regional inspection](ui_observer_region.png) | Foreign kingdom army name and troop count, no order controls |
| [Developed inspection](ui_observer_developed.png) | Three completed rounds, two armies at one site, wrapped force details clear of buttons |
| [Help](ui_observer_help.png) | Observer-specific instructions and visible dismissal |

The map remains dominant; playback occupies one lower control strip and details
appear on demand. The roster fits the maximum eight factions. Inspector text
wraps within its reserved area, with forces ahead of secondary place details.
No reviewed text clips into controls. The capture assertions establish pointer
handling and application actions; they do not establish hardware touch timing.

## Published browser check

`publish.ps1` completed the Windows, WASM, Preview and Roost publish steps. The
published game page loaded and showed the Observe entry alongside the existing
Continue save. The browser canvas reported **1920×1080**. After an attempted
coordinate click on Continue, the browser had left fullscreen without opening
the save; the exact exit trigger was not established. Therefore the
remaining published-browser flow—Observer setup, playback, roster and return to
Continue—was not verified. Browser interaction stopped at that point.

Source inspection confirmed that the page's Full Screen button invokes
`requestFullscreen()`. On exit, `fullscreenchange` removes its `game-playing`
class and resizes the canvas back into the page. Invoking the button took over
the user's display despite the tab starting hidden. The test tab was closed and
the temporary viewport override reset. Future checks must remain headless, as
recorded in [project guidance](../../PROJECT_AGENTS.md).

## Review limits

The sole supported and acceptance canvas remains **1920×1080**. Earlier 720p
records do not define another target. Synthetic pointer frames and browser mouse
interactions do not establish physical touchscreen or native rapid-click timing;
the existing [physical-touch waiver](k18-touch.md) remains in force.

Observer uses existing AI and campaign balance. It does not establish that every
seed reaches a winner within a fixed number of rounds, or complete the separate
M02–M05 map playability milestones.
