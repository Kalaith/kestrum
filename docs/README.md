# Kestrum design documentation

Kestrum is a generational strategy game in which connected places create campaigns, campaigns shape people, and people and places carry that history forward. This documentation consolidates the three founding design drafts and subsequent discussion clarifications into a navigable game design document (GDD).

**Project state:** design documentation and the supplied WebHatchery template. The systems described here are intended behavior, not implemented features. No Kestrum gameplay is claimed by this initial documentation milestone.

## Reading order

| Document | Contents |
| --- | --- |
| [01 — Vision and experience](01-vision-and-experience.md) | Identity, pillars, inspirations, player role, loops, setting, success criteria |
| [02 — World, time, and control](02-world-time-and-control.md) | Node graphs, nested regions, entrances, seasons, faction turns, territorial anchors |
| [03 — Kingdoms and economy](03-kingdoms-and-economy.md) | Setup, victory, resources, development focuses, infrastructure, diplomacy, AI, intelligence |
| [04 — Armies and logistics](04-armies-and-logistics.md) | Six formation slots, headcounts, characters, veterancy, movement, roads, supply, outposts |
| [05 — Battles and sieges](05-battles-and-sieges.md) | Automatic combat, encounters, casualties, retreat, siege states, relief, local threats |
| [06 — Character development](06-character-development.md) | Emergence, evidence, recognition, traits, classes, mentorship, rivalries |
| [07 — Living places](07-living-places.md) | Layered node state, growth, decline, occupation, ruins, refugees, changing capitals |
| [08 — Generations and succession](08-generations-and-succession.md) | Age, injury, retirement, families, heirs, recruitment, legacy, simulation depth |
| [09 — History and content](09-history-and-content.md) | Chronology, historical eras, biographies, content guidance, worked campaign examples |
| [10 — Interface and accessibility](10-interface-and-accessibility.md) | Screen briefs, touch flows, information hierarchy, map presentation, visual review |
| [11 — Simulation and data](11-simulation-and-data.md) | Proposed state ownership, actions, system contracts, data validation, saves, determinism |
| [12 — Delivery and validation](12-delivery-and-validation.md) | Prototype, first release, later scope, acceptance scenarios, template handoff, checks |
| [13 — Decisions and open questions](13-decisions-and-open-questions.md) | Confirmed decisions, agreed directions needing mechanics/tuning, unanswered questions |
| [Glossary](glossary.md) | Consistent terminology across chapters |
| [Source coverage](source-coverage.md) | Section-by-section provenance and source preservation checks |
| [Documentation verification](verification/documentation.md) | Checks and limitations of this documentation milestone |

Read chapters 01–03 for the game overview, 04–09 for system design, and 10–13 before implementation. Cross-references between chapters describe dependencies rather than separate games or optional competing designs.

## Design status and authority

These labels distinguish the origin and maturity of a rule:

- **Source design:** stated in the supplied drafts. It may still be a tentative direction or an example; original qualifiers remain important.
- **Confirmed decision:** settled by the subsequent discussion and recorded in chapter 13. Implement this rule without reopening it because related tuning is unfinished.
- **Agreed direction:** the discussion establishes the feature's behavior or philosophy; the register names the remaining mechanics and tuning.
- **Working interpretation:** a documented way to reconcile source ambiguity. It is a baseline for planning, not a claim that the author settled the question.
- **Proposal:** new detail added in this consolidation. It must be evaluated during implementation and playtesting.
- **Open:** a choice that needs a decision before the dependent feature can be accepted.
- **Future scope:** preserved design that is outside the first functional version.

Unlabelled descriptive system sections restate source design. The discussion clarification supplied on 2026-09-26 takes precedence where it settles an earlier ambiguity. Newly specified algorithms, contracts, acceptance criteria, screen plans, and development sequencing are proposals unless stated otherwise. Numbers remain tunable when the sources or discussion call them approximate.

The [decision register](13-decisions-and-open-questions.md) owns reconciliation across chapters. Shared engineering documents at the project root remain authoritative for code and workflow. Do not alter those shared documents to express Kestrum-specific design.

The latest clarification confirms six formation slots, emergence from abstract troops, loss of formation history on destruction, fixed world and regional graphs, named-character army contributions, and 4–8 total factions including the player. The register separates these from remaining mechanics and unanswered questions. **O16, how reinforcements affect veteran formations, is the next design priority.** Vassalisation as a defeat outcome remains a proposal to reconcile the victory objective with early War/Peace diplomacy.

## Complete source preservation

The following files are permanent source references, copied byte for byte from the supplied drafts:

- [Original game concept](reference/kestrum_game_concept.md)
- [Original kingdom, army, and war systems](reference/kestrum_kingdom_army_war_systems.md)
- [Original living-world and generational design](reference/kestrum_living_world_generational_design.md)

All original examples, diagrams, wording, repetitions, uncertainties, and competing directions remain there. The topical chapters make the material usable as one design; the references preserve every detail. Neither layer depends on the three root-level drafts. Those root copies can be deleted later without losing content from this documentation set. They are retained in the initial commit because deletion was not requested.

The [coverage ledger](source-coverage.md) records a destination for every original heading and SHA-256 checksums for exact preservation. When changing the design, update the relevant chapter and decision register; keep these founding references unchanged.

The [original template README](reference/template_readme.md) is also retained as provenance. Its paths and instructions describe the starter template and are not a statement that this checkout has completed onboarding.

## Maintaining the design

1. Keep one primary chapter for each rule and link to it from related systems.
2. Record why a working interpretation becomes a confirmed decision or is replaced.
3. Keep illustrative characters and campaign dates separate from fixed scenario content.
4. Update milestone acceptance criteria when scope changes.
5. Report actual implementation and verification status separately from intended design.

The existing images in `docs/verification/` belong to the supplied template. They do not demonstrate Kestrum screens.
