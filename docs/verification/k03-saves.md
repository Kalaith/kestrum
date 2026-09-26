# K03 — Recoverable campaign catalogue

Date: 2026-09-26–27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `131f3bb` (K02). No pre-existing Kestrum changes were present.

## Scope and decisions

K03 replaces the interim strategic slot with an indexed catalogue. The toolkit
owns monotonic identities, fresh payload keys, pending-operation recovery, atomic
publication, checked cleanup and native/browser writer ownership. Kestrum owns
metadata, compatibility, candidate validation and the last successful Continue
selection. I02 records branch-safe checkpoint identities and naming decisions.

The UI opens a paged save list on demand, offers new named copies, confirms
overwrite/deletion, and retains exact failed snapshots for Retry. Continue Unsaved
does not replay a completed season. Old shell and interim strategic bytes remain
preserved through read-only opening or explicit import.

## Validation status

Shared prerequisites are committed on master: toolkit `92d629c` (storage),
`4f4751a` (text entry), `0bf8eea` (font preparation), and `rust_management`
`e8c8dfb` (browser bridge).
Toolkit checks passed formatting, strict Clippy, 428 existing unit tests,
five storage and five text-entry integration tests, 29 doc tests (nine
existing examples ignored), native examples, WASM analytics examples/library,
documentation and the 800-line gate. Native backend verification reopened eight
catalogue entries through actual files and cleaned only its test-owned keys.
Five bridge memory tests and the isolated Chromium protocol test passed, including
12 payloads after reload and Web Locks ownership across document closure.
See the toolkit's `docs/verification/indexed-storage.md` and `docs/INDEXED_SAVES.md`.

The actual game checkout passed formatting, locked all-target/all-feature strict
Clippy and all 28 tests, including the source-size gate. The five persistence
cases exercise 24 entries, each failed write stage, overwrite/delete/retry,
supported and rejected versions, and branched/resumed rounds. Shared review fixed
malformed-journal and namespace-alias hazards. Game review fixed off-screen
selection, recovery of Continue without undoing deliberate older-save loads,
stale overwrites, delayed obsolete requests, and valid loading when the Continue
write fails. These fixes have regression coverage. No game RNG changes for storage.

The no-argument publisher built and packaged Windows and WebGL, deployed Preview,
updated the catalogue and successfully recorded Project Roost tracking. All game
checks and publishing passed again after the font-atlas fix described below.

Native captures cover save list, naming, symbols, busy storage, invalid load,
deletion confirmation, failed-save recovery, title, Help and new-game confirmation
at 1920 × 1080 fullscreen and 1280 × 720. The shared wrapper wrote directly to
stable `ui_<scene>.png` and `_minimum` files here; all launched game processes
exited. Maximum names, a 17-entry list and Round 4004 remain readable. Review
fixed indistinguishable keyboard case labels by using the body font and shortened
Help to remove a paragraph overlap. The repeated captures show clear case/symbol
keys, reachable actions, bounded errors and no overlap in these scenes.

## Published browser interaction

The loopback server served the actual Preview deployment. Interactive checks:

- Imported the K02 campaign at Autumn Year 1, Round 3 with the source preserved.
- Entered `Qa1` using only visible Clear, case-page and character/number keys.
  Empty naming disabled Create Save. Created four further independent copies.
- Reloaded and reached every one of the six entries across two pages. Completing
  a round added a distinct Winter Year 1 checkpoint as the seventh entry.
- Overwrote only the chosen verification copy through Confirm Overwrite; its ID
  remained the same and its date changed. Opened Delete confirmation and cancelled.
  Actual deletion/idempotence is exercised in the failure-store tests.
- Loaded the older `Qa1` entry, reloaded, and Continue restored Autumn Year 1
  despite newer Winter saves. Open Old Atlas restored the old Summer atlas with
  no strategic End Turn control.
- A second real window discovered entries while the first held the writer lease.
  It loaded Winter successfully and warned that Continue could not be updated.
  Advancing to Spring Year 2 produced the expected save-recovery sheet. Closing
  the first window and retrying saved the same Round 5 without replaying it.
  The immediate first retry still saw the closing document's lock; a subsequent
  retry after release succeeded.

The new 1920 × 1080 browser window retained correct fullscreen composition.
The earlier 1280 × 720 overridden window still reproduced the inherited
upper-left shrinking/display mismatch while DOM/input dimensions stayed correct.
Three `glBindTexture ... deleted texture ID 28` messages initially appeared on
first dense UI use. Source review identified lazy font-atlas growth while earlier
glyphs were already batched. Toolkit explicit-font preparation now caches the
current Unicode samples at the correct DPI before uploading the atlas. Kestrum
prepares fixed sizes at startup and only visible changing text each frame.
A fresh published browser document then passed title, first list, naming, all
keyboard pages, the Unicode name `Été Ω`, saved list and deletion preview with
an empty error/warning log. Refreshed native list/name/symbol captures still fit
both supported sizes. No storage exception occurred in successful save flows.

Native interactive restart, physical touch/pinch and real device soft input have
not been verified in this package. The inherited browser fullscreen/display
scaling issue remains open for K18.

K03 is committed under `Kestrum keeps each turning of its history (K03 recoverable
saves)`. The next required package is K04 selectable geography; no later gameplay
or full-release acceptance is claimed here.
