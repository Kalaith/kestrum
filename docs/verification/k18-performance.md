# K18 performance follow-up

## Planner diagnosis and changes

Release measurements on the AMD Ryzen 7 5800X / RTX 4080 SUPER workstation
separated production planning from command application and state validation.
The 40-round seed-180018 baseline used 512 NPC actions at four factions and
1,217 at eight. Planning p95 was 30.97 / 31.54 ms; application p95 was
3.76 / 6.01 ms. A slow End Turn proposal spent 44–54 ms considering expansion.
Late construction proposals spent up to 32 ms rejecting work orders.

The planner now caches paths from each observed origin for one decision,
including the existing lexicographic path tie break. Hostile/contested sites
remain attack destinations only; threats and neutral foreign territory retain
their original restrictions. Five regressions compare the batched paths against
the previous single-destination search, including ties and rebuilding after a
territory change.

Identical failed commands are checked once per decision. Shared personnel,
recruitment, construction and focus validators reject impossible candidates
before transactional preview clones the campaign. Accepted candidates still
pass the original preview and application boundaries. No command ordering,
random draws, rule values or information visibility changes.

After route caching and personnel/recruitment/construction prechecks, planning
p95 fell to 6.14 / 9.48 ms in the same 40-round runs, with the same 512 / 1,217
actions. Application p95 was 3.48 / 5.71 ms. Focus prechecks were subsequently
extracted from the same authoritative command validator.

## Long release replay before the final focus precheck

Runs were serial, with no concurrent Rust compilation or simulation tests.
These measurements are complementary to rendered gameplay, not frame rates.

| Factions / round | Elapsed | Phase ms p50 / p95 / max | Action ms p50 / p95 / max | Checkpoint / roundtrip |
| --- | ---: | ---: | ---: | ---: |
| 4 / 200 | 21.60 s | 25.58 / 108.68 / 179.43 | 9.48 / 28.00 / 40.51 | 564,673 B / 17.79 ms |
| 4 / 400 | 65.99 s | 35.23 / 177.68 / 512.20 | 20.71 / 46.60 / 57.10 | 453,239 B / 13.42 ms |
| 8 / 200 | 67.37 s | 27.48 / 130.16 / 581.49 | 14.20 / 58.89 / 101.60 | 669,321 B / 19.91 ms |
| 8 / 400 | 131.74 s | 26.37 / 114.63 / 581.49 | 16.31 / 57.21 / 101.60 | 619,129 B / 16.00 ms |

The eight-faction phase p95 is below the provisional 250 ms target. Individual
late actions still exceed a 16.7 ms frame budget; these runs do not establish
continuous 60 Hz. Exact replay after the round-200 reload passed for both sizes.
At round 400 the four/eight-faction states retained 62/97 people, 12/22
households, 73/122 family rows, 7/15 armies, 39/87 formations and 211/338 detailed
events. No orphaned family rows remained. Eight-faction peaks were 111 people,
49 households, 1,513 detailed events and 92,312 modeled inhabitants.

## Reproducible rendered measurements

F3 starts a bounded console-only frame profile in the ordinary native or WebGL
game; F3 again prints and stops it. Nothing is added to gameplay screens.
Reports distinguish game CPU submission, time until the next rendered frame,
and pointer-release samples until that next frame. The latter starts when the
game receives input, so it excludes OS/browser event delivery and display scanout.
The first 60 frames warm caches; each report covers at most 600 frames.
WebGL also reports allocated linear memory, which includes assets and reserved
space rather than just live campaign objects.

Native scripted runs use the same executable, rendering loop and command paths,
with player saves/preferences disabled. Set `KESTRUM_PROFILE_FRAMES`, optionally
`KESTRUM_PROFILE_FACTIONS` (default 8), `KESTRUM_PROFILE_ROUND` (default 0), and
`KESTRUM_HEADLESS=1`, then use the shared Cargo run wrapper with `--release`.
The run generates the requested production state, emits `K18_PROFILE_READY`,
passes the player's turn and advances NPC commands periodically, then exits.
Generation is excluded from frame samples. Hidden rendering cannot measure
physical display or touch latency. This mode writes no screenshots or saves.

## Closeout — 2026-09-28

The user ended further testing and balance work before rendered profiling was
run. No native/WebGL frame-time, input-latency or steady-state memory result is
claimed. The opt-in profiler remains available for later development.

Formatting and strict all-target/all-feature Clippy passed. Five path regressions
passed in debug and release. The subsequent locked all-target/all-feature suite
passed its 4/8-faction 400-round replay and source-size gate, then failed
`four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`:
the seed-88 scripted policy did not reach victory by round 120. This is an
unresolved result after the AI changes, not evidence that victory is impossible.
The test and its assertion remain intact; later test binaries were not reached.
There was no further balance investigation or rerun after the user's stop.

The no-argument publisher subsequently built Windows and WebGL release packages,
deployed Preview and recorded the publication in Project Roost successfully.
