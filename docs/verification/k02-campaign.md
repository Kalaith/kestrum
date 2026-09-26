# K02 — Campaign ownership and faction phases

Date: 2026-09-26. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `6d23e99` (K01). No pre-existing project changes were present.

## Scope

K02 adds authoritative strategic state, atomic commands, persisted named RNG
streams, public/own-state projection and player-first faction rounds. NPCs pass
until K12. UI provides Pause, Step and Resume between NPC actions. Shell-v1 saves
remain a separate read-only variant and retain their original bytes.

The full-release task continues through K18. This record is a package checkpoint,
not a claim that armies, the recoverable catalogue or later gameplay exists.

## Validation status

The actual checkout passed formatting, locked all-target/all-feature Clippy, all
23 tests (including the source-size gate), and Windows/WebGL Preview publishing.
Project Roost tracking and catalog refresh also succeeded. A browser reload exposed
an outer save-envelope deserialization bug. Explicit schema dispatch fixes Serde's
untagged numeric-map-key buffering while preserving the existing v1/v2 bytes.
Regressions now use the actual toolkit envelope and full-width u64 values. All
checks and the no-argument publisher passed again after that fix. The previously
failing browser save then restored Autumn, Year 1, Round 3 successfully.

Native captures at 1920 × 1080 fullscreen and 1280 × 720 cover gameplay, menu,
Help, legacy, paused NPCs, NPC menu, a long faction name, new-game confirmation
and save errors. The shared wrapper wrote each directly to its stable
`ui_<scene>.png` filename here. Relevant states were visually inspected: the map
remains dominant, NPC controls have readable dark backing, long labels fit, and
touch controls remain at least 48 logical pixels. All capture processes exited.

The published WebGL build was exercised through a loopback server serving the
actual Preview deployment. End Turn moved from Rose to Oak without changing the
season; automatic rival passes advanced exactly one season. Pause held the phase,
Step advanced one rival, Menu disabled saving with a reason, Help/Back preserved
the phase, and Resume completed the round. Manual save and a complete browser
reload/Continue restored the same round. Fast clicks initially missed their
press origin when press/release arrived within one frame. Input now records the
press edge while preserving an existing multi-touch gesture's origin.

The browser's DOM reports a 1280 × 720 fullscreen canvas, but after interactions
the in-app browser capture displays the canvas shrunken into the upper-left while
input coordinates retain the DOM dimensions. This reproduces the inherited
fullscreen/display-scaling limitation; it remains open for K18 investigation.
Physical touch, pinch and platform soft-keyboard behavior remain unverified.

K02 is committed under the subject `Kestrum's kingdoms share one seasonal round
(K02 campaign phases)`. K03 is the next required package; K18 retains the platform
limitations above. No later gameplay or full-release acceptance is claimed here.

## Next dependency

K03 requires shared storage work. Inspection found no indexed catalogue, a
Windows remove-before-rename gap in toolkit `persistence/files.rs`, and browser
read/delete errors concealed by `rust_management/web/storage.js`. Fix the shared
primitives and add failure-injected journal tests before claiming recoverable
catalogue behavior. Toolkit was clean at `c0340a8` during inspection. K02's
deterministic scenario identity must become an externally allocated unique
campaign ID when K03 introduces multiple campaigns; never consume simulation RNG
for catalogue allocation.
