# Kestrum documentation

Start with the [map playability plan](map-playability-plan.md) for new work and
the [project README](../README.md) for implemented behavior and development
commands. The current priority is a readable kingdom map, useful spatial
decisions and visible consequences. M01's kingdom overview and M01A's spatial
scale/navigation are complete under their recorded scope. M01A's
[verification](verification/spatial-scale.md) records engineering checks, actual
1920×1080 native/browser captures, automated native actions, browser gameplay,
publishing and reload, with explicit inherited limits. M02 is next.
The [hero portrait plan](hero-portrait-generator-plan.md) adds a companion
graphics workstream; the [agent queue](../todo.md) records its unfinished work
and remaining event acceptance. It does not reorder M02-M05.
K01–K18 and B01–B07 describe completed
implementation work under their recorded scope.

## Reading order

1. Read the active map plan and choose its next unfinished milestone.
2. Read the relevant system chapter and its current source.
3. Use the decision register for fixed constraints and the delivery chapter for
   validation. Consult earlier packets only when their mechanical detail is needed.

There is no requirement to reread every founding draft, repeat completed packages,
or obtain new approval for ordinary implementation choices already delegated.
Historical speculative wording does not override the current request or plan.

## Current documents

| Document | Owns |
| --- | --- |
| [Map playability plan](map-playability-plan.md) | Active milestones, dependencies, map scope, compatibility and acceptance |
| [Early Stellaris lessons](stellaris-release-lessons.md) | Source-grounded design review and rationale for campaign acceptance; no gameplay changes or separate delivery queue |
| [Campaign notifications](notification-plan.md) | Implemented event rail, durable receipts, controlled-place warnings, delivery settings and acceptance contract |
| [Hero portrait generator](hero-portrait-generator-plan.md) | Persistent appearance, layered human art, duplicate policy and G03/G04 continuation contracts |
| [01 Vision and experience](01-vision-and-experience.md) | Player role, core loops and generational identity |
| [02 World, time, and control](02-world-time-and-control.md) | Physical graph, regions, ownership and seasonal turns |
| [03 Kingdoms and economy](03-kingdoms-and-economy.md) | Setup, resources, development, diplomacy and AI |
| [04 Armies and logistics](04-armies-and-logistics.md) | Formation membership, movement, orders, supply and transfers |
| [05 Battles and sieges](05-battles-and-sieges.md) | Current encounters, preparation, resolution and lasting results |
| [06 Character development](06-character-development.md) | Service, emergence, careers and mentorship opportunities |
| [07 Living places](07-living-places.md) | Growth, decline, occupation, refugees and local roles |
| [08 Generations and succession](08-generations-and-succession.md) | Aging, recovery, households, heirs and continuity |
| [09 History and content](09-history-and-content.md) | Knowledge, history retention, legacy and content scope |
| [10 Interface and accessibility](10-interface-and-accessibility.md) | Implemented 1920×1080 composition, spatial navigation and remaining interface work |
| [11 Simulation and data](11-simulation-and-data.md) | State ownership, commands, projections, content and saves |
| [12 Delivery and validation](12-delivery-and-validation.md) | Required checks, evidence, publishing and completion |
| [13 Decisions and open questions](13-decisions-and-open-questions.md) | Fixed constraints, current defaults and bounded remaining design choices |
| [Glossary](glossary.md) | Shared terminology |
| [Evidence index](verification/README.md) | Latest scoped records, historical evidence and remaining limitations |

## Document authority

The user's current request and explicit earlier decisions govern scope. Shared
engineering documents govern implementation and workflow; edit their canonical
copies in `rust_management/docs/` for any shared change.

The map plan owns the next delivery sequence. System chapters own current design
rules and distinguish implemented behavior from planned changes. The decision
register records fixed constraints and adjustable defaults. Source and tests
establish what the checkout actually does; if source conflicts with the intended
design, record the gap rather than claiming the intention is implemented.

Use these status meanings consistently:

- **Implemented:** present in the current source. It does not imply every
  platform or playability check passed.
- **Planned:** specified next work, with acceptance in the map plan.
- **Tunable default:** an existing or recommended implementation choice that can
  be adjusted with relevant validation; it is not an unanswered approval gate.
- **Deferred:** outside the current delivery or waived testing scope.
- **Historical:** a record of an earlier design or check, not current instructions.

## Completed delivery and provenance

| Reference | How to use it |
| --- | --- |
| [K01–K18 plan](implementation-plan.md) and [packets](implementation/work-packages.md) | Completed delivery history and detailed mechanical contracts |
| [Mechanical rules](implementation/strategic-rules.md), [combat rules](implementation/combat-rules.md), [people and places](implementation/people-and-places.md) | Baseline detail; later implemented changes and the map plan take precedence |
| [Battle delivery](battle-system-implementation-plan.md) | Completed B01–B07 scope and combat integration reference |
| [Implementation review](implementation-review.md) | Dated findings and their dispositions |
| [Acceptance ledger](implementation/acceptance.md) | Recorded package completion with its actual testing scope |
| [Verification records](verification/README.md) | Evidence tied to the named version, scene, date and platform |
| [Source coverage](source-coverage.md) | Founding-draft provenance and preservation hashes |

Historical pass counts, old blockers, screenshots and machine-local save numbers
must not be copied into current status as fresh validation. The evidence index
points to later records where earlier results were superseded. Original evidence
is preserved rather than rewritten to claim checks that never happened.

The original drafts in `docs/reference/` and their three root copies are source
material. Their unresolved questions, examples and template instructions have no
authority over the current plan. Preserve their bytes and provenance:
[concept](reference/kestrum_game_concept.md),
[kingdom systems](reference/kestrum_kingdom_army_war_systems.md),
[generational design](reference/kestrum_living_world_generational_design.md).
The [starter README](reference/template_readme.md) is also historical.

## Maintenance

Update the owning chapter and active milestone in the same commit as behavior.
Remove superseded instructions from active text instead of appending another
contradictory status paragraph. Keep validation results in their scoped evidence
record and link them. Future work should need one current plan, not a new
reconciliation of completed milestones.
