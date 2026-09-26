# K04 — Selectable world and regional geography

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `efef921` (K03). No pre-existing Kestrum changes were present.

## Delivered behavior

The atlas now renders the authored five major markers and their routes. A regional
marker opens ten physical sites, internal routes and explicit gates. Tap selection,
Enter Region, World Map, Close, zoom and Recenter use visible controls. Each map
keeps its own transient camera; new/load resets presentation state.

Inspectors separate local controller, political claim and contested state. A claim
requires secure anchors plus a supplied entrance. Losing one keeps the previous
claim as contested, without capturing or erasing internal sites. Supply queries
follow physical routes through secure friendly sites and expose only the player's
derived reachability in the UI. The region marker has no duplicate occupiable site.
I03 documents additive old-v2 migration and deterministic ownership ties.

## Automated validation

Formatting, locked all-target/all-feature strict Clippy and all **33 tests** pass
in the real checkout. The test suite includes the 800-line source-size gate.
The five K04 cases cover:

1. All four external routes resolve their entrance and reciprocal exit, with no
   direct headquarters-to-city or gate-to-gate shortcut.
2. Marker/site namespaces remain separate; invalid graph/control/claim candidates
   cannot replace live play or alter UI overlays.
3. Secure supplied anchors transfer political ownership while hostile pockets
   retain their controller; foreign/neutral/contested sites and a lost HQ block
   supply. Road changes do not invent supply access.
4. Losing an anchor or supply preserves the historical claim and real controllers.
   Current saves round-trip; actual old-v2 toolkit envelopes and indexed catalogue
   payloads load without changing source bytes or metadata. Partial/malformed new
   fields fail validation.
5. Draw/pick coordinates agree across world/region, zoom, resize, letterboxes and
   1/1.5/2 display scales. Targets remain 48 pixels, ties resolve stably, cameras
   restore exactly and invalid selection cannot mutate the campaign. A release
   crossing targets or moving more than six pixels does not select.

The five earlier camera tests and all campaign/storage regressions remain intact.
Independent source review found no additional K04 semantic/persistence defect.

## Visual and interaction review

The shared hidden capture wrapper produced 1920 × 1080 fullscreen and 1280 × 720
captures directly in this directory. All launched game processes exited. Reviewed:

- `ui_gameplay[ _minimum].png`: world topology without selection.
- `ui_world_selected[ _minimum].png`: claim, controller counts, anchors and Enter Region.
- `ui_region[ _minimum].png`: ten sites, gates and breadcrumb.
- `ui_region_selected[ _minimum].png`: East Gate and external connections.
- `ui_region_partial[ _minimum].png`: Rose claim with a surviving Hawthorn pocket.
- `ui_region_contested[ _minimum].png`: contested High Fort, retained claim, lost supply.
- `ui_region_long_name[ _minimum].png`: three-line title and bounded map label.
- `ui_help[ _minimum].png`: current selection, navigation and phase instructions.

The bracket notation means the normal filename and its `_minimum` counterpart.
The map remains dominant; one dismissible inspector leaves the selected target
visible. Owner initials/shapes and explicit text supplement color. Title wrapping,
body text and 48-pixel controls fit both sizes. Review fixed a label clipping at
the inspector boundary. Gestures are blocked beneath inspectors, visible feedback
and letterbox margins. A final-frame short drag cannot become a selection.

Actual published WebGL review loaded the earlier K03 Round 5 campaign, selected
Rosemarch, entered the region, inspected High Fort, closed its inspector, zoomed,
panned and returned to the world. Reentering retained the regional camera.
A short drag from the fort left selection closed; an ordinary tap selected it.
The final published document also loaded at a forced 1280 × 720 input canvas and
selected Rosemarch and East Gate. Error/warning logs were empty.

The browser's normal fullscreen canvas measured 1936 × 1048 and displayed correctly.
The inherited forced-720p mismatch persists: DOM/backing/input dimensions report
1280 × 720 while the captured image shrinks toward the upper-left. Therefore this
is functional input evidence, not a passing minimum-browser display review.
Physical touch/pinch remains unverified. Both limitations stay open for K18.

## Publication and continuation

The no-argument publisher passed Windows and WebGL release builds, packaging,
Preview deployment, catalogue update and Project Roost recording. It passed again
after the final label fix. The host page describes the new map controls.

K04 commit subject: `Kestrum reveals the gates beneath its borders (K04 geography)`.
Next: K05 six-slot armies, recruitment and round economy. K05–K18 remain required;
this checkpoint is not full-release completion.
