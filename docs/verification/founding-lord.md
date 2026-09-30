# Founding lord verification — 2026-09-30

New production campaigns give each kingdom exactly one generated, named lord,
aged 18–24, attached to Warriors and commanding its starting army. The player's
lord founded the chosen kingdom; rival founders receive the same grant. Noble
status is separate from Officer profession, earned recognition and traits.
Names use the existing human-name data. A separate seeded age stream preserves
the geography, naming and threat rolls for the same setup.

The lord adds 50 permille (five percentage points) to army leadership while fit,
non-retired, attached to a surviving formation and appointed commander. The
ordinary Officer bonus still applies, giving the starting army 98.3% leadership.
Save/reload preserves the founder. Earlier saves receive no new title or people;
the authored Rosemarch regression scenario retains its original grants.

## Automated checks and publishing

All commands ran against the actual checkout and shared Cargo workspace.

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Passed. |
| `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets '--' -D warnings` | Passed. |
| `..\rust_management\cargo.ps1 test -p kestrum --test founding_lord --test generation --test code_standards` | All 11 tests passed: five feature cases, five generation regressions and the 800-line source gate. |
| `..\rust_management\cargo.ps1 test -p kestrum --all-targets` | Failed the existing 400-round campaign continuity case; the original four-faction seed ended at round 188. |
| Broader `--all-targets '--' --no-fail-fast` run | Found two failing targets: campaign continuity and the existing scripted production victory deadline. Other targets completed successfully; the profiling case remains ignored. |
| `.\publish.ps1` with no arguments, after the final UI change | Passed Windows and WebGL release builds, packaging, Preview deployment, Project Roost recording and catalogue update. |

The five new tests cover exactly one generated young commander per faction,
seed replay and all seven possible ages, the bonus's eligibility conditions,
save compatibility, and rejection of invalid founder data.

The two long-campaign failures remain validation blockers. Diagnostics temporarily
restored age-24 founders without the new title or bonus in those tests, using the
same actual checkout: the continuity case still ended before round 400, at round
265; the seed-88 conquest script still stalled at round 31. Trying an alternate
continuity seed also failed. All diagnostic test edits were removed, preserving
the original tests. This feature does not claim a passing full suite or a fix for
those simulation problems. Earlier results are recorded in
[review-fix verification](review-fixes.md).

Preview: [Kestrum](http://127.0.0.1/games/kestrum/).

## Native visual review

The shared hidden-window capture wrapper wrote directly to the stable evidence
paths below. Normal captures requested a 1920 × 1080 window and produced a
1920 × 1061 client image; minimum captures are exactly 1280 × 720. All captures
completed successfully, and no Kestrum game process remained after capture.

| State | Normal | Minimum |
| --- | --- | --- |
| Production setup | [Setup](ui_production_setup.png) | [Setup](ui_production_setup_minimum.png) |
| Starting army | [Army](ui_founder_army.png) | [Army](ui_founder_army_minimum.png) |
| One named person | [People](ui_founder_people.png) | [People](ui_founder_people_minimum.png) |
| Lord's career | [Career](ui_founder_career.png) | [Career](ui_founder_career_minimum.png) |
| Six named people | [Dense list](ui_founder_dense.png) | [Dense list](ui_founder_dense_minimum.png) |
| Maximum-length kingdom name | [Long name](ui_founder_long_name.png) | [Long name](ui_founder_long_name_minimum.png) |

Review followed `UI_STYLE.md`. Setup keeps Create Campaign as its primary action
and explains the starting lord in one quiet line. Army leadership and commander
stay beside the roster; identity, age and founding role stay in the existing
career view. No extra panel is needed. Text remains legible without clipping or
overlap at both sizes. A 32-character wide-letter kingdom name wraps the founding
description into two lines above the age. The dense list pages four rows at a
time; companions do not inherit the Lord title. Existing visible Army, Details,
Orders, People, Career, Back and paging controls remain the input route.

## Published browser review and limits

At 1920 × 1080, the published WebGL game was exercised through New Game,
production setup, Create Campaign, Army, Army Details, Orders, People and Career.
The default seed produced Lord Catrin Cairn, aged 24, as the one named starting
commander, with 98.3% army leadership and the five-point founding bonus shown in
Career. Reloading the final publication and using Continue restored that campaign
with the same titled commander and leadership.

At 1280 × 720, the inherited WebGL scaling problem remains: the canvas reports
the correct dimensions, but the game renders into a smaller upper-left area with
black space, and pointer locations do not reliably match the visible controls.
Minimum native layout passes; minimum browser interaction cannot be signed off.
Physical-touch input was not exercised. The existing visible controls and touch
keyboard are retained. Browser viewport overrides were reset after review.
