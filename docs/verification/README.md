# Verification evidence

[Documentation index](../README.md) · [Current project status](../../README.md)

## How to use these records

Verification reports record a dated change, checkout, test result and review
scope. They are historical evidence, not an implementation queue or proof of
current playability. An older report's unresolved issue may have a later fix;
an older passing suite does not override a later failure. Follow the current
project status and active plan before treating a historical finding as work.

Preserve the dated reports and supplied references. Record new work in its
own relevant verification report, with links to the earlier evidence it
supersedes. Keep this index concise rather than duplicating every test count.
Screenshots use stable filenames and are replaced for the same screen/state;
an older report's image link may consequently show a newer capture. Its written
result still describes the recorded run.

## Latest relevant records

[M01 kingdom overview](kingdom-overview.md) is the latest implementation and
runtime record. [Map plan documentation verification](map-plan.md) records the
preceding reconciliation at `3073c18`; it claims no runtime validation.

The subsequent [spatial scale plan](../map-playability-plan.md#spatial-scale-review)
adds M01A as the next milestone and records the user's sole 1920×1080 spec.
It establishes no new runtime results. Earlier 720p checks remain historical
evidence; current acceptance follows the corrected resolution contract.

| Record | What it establishes |
| --- | --- |
| [M01 kingdom overview — 2026-10-01](kingdom-overview.md) | Known political/control layers, symbols/labels/banners, actual accounts and attention; full-suite result, final focused checks, nine native states at both sizes, publishing and browser gameplay/reload evidence |
| [Movement plans — 2026-10-01](movement-plans.md) | Latest immediate destination taps, queued continuation, selected-unit End Turn and native action-harness checks; distinguishes the final focused checks from the preceding engine-change full suite and browser review |
| [World army orders — 2026-10-01](world-army-orders.md) | World-level selection and regional entrance/departure orders; its Confirm Move flow is superseded by movement plans |
| [Map fog — 2026-10-01](map-fog.md) | Frontier clearing and outgoing road hints, with focused tests and captures; it does not establish useful political borders or strategic map readability |
| [Developed midgame campaign — 2026-10-01](midgame-campaign.md) | Reproducible developed native save, people/formation coverage, camera framing and limited browser smoke checks |
| [Battle review — 2026-09-30](battle-review.md) | Correctness fixes, the production-victory failure and remaining battle presentation findings |
| [Home-region tutorial — 2026-09-28](home-region-tutorial.md) | Revision-3 regional starts and reachable navigation lessons; navigation coverage does not establish an engaging opening campaign |
| [K18 closeout — 2026-09-28](k18-integrated.md) | Foundation closure under amended scope and its earlier integrated evidence |
| [K18 physical touch scope](k18-touch.md) | The user's recorded waiver and the unexecuted device checklist; no physical-touch acceptance is claimed |

The K01–K18 and B01–B07 reports remain useful regression evidence. Their original
package order and provisional assumptions do not require repeating completed
work before improving the map. [Documentation verification](documentation.md)
and [implementation-plan verification](implementation-plan.md) describe the
2026-09-26 planning baseline, including blockers that later work superseded.

## Inherited limitations, last recorded 2026-10-01

M01 reran the full suite and affected focused checks, refreshed native visual
evidence, published, and exercised browser gameplay. These limits remain:

- The M01 full suite again has one failing
  target: `k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`.
  It does not reach victory within 240 rounds. The 400-round continuity target
  passes in that run; one release profiling test remains intentionally ignored.
- M01's published browser checks cover attention focus, immediate travel,
  drag/zoom, selected-unit End Turn, actual receipts, world warnings and reload.
  The final focused suite and captures cover the last display refinements;
  the full suite ran immediately before those refinements.
- Minimum native screenshots were reviewed at 1280 × 720. The in-app browser's
  minimum-size override again produced a smaller upper-left image and black remainder;
  minimum browser acceptance remains unverified. Native captures and normal
  browser clicks do not establish that missing result.
- Physical-touch testing was waived for K18. No device or pinch result has since
  been established by these records. Keep touch-accessible controls and no-hover
  interaction in the implementation requirements; report the device limitation
  without reopening the completed waiver as a pending user request.
- [Battle review](battle-review.md#remaining-presentation-findings) retains
  multi-army clipping, target-priority editing, small tactic/leader controls and
  aggregate-morale findings. They are existing issues to preserve or address
  when relevant, not prerequisites for every map milestone.
- M01's review establishes specific rendered states and browser interactions.
  It does not claim a first-time human playtest or the broader M05 early/developed
  campaign acceptance; M01A and M02–M05 remain unstarted.
