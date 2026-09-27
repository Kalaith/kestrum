# Kestrum design documentation

Kestrum is a generational strategy game in which connected places create campaigns, campaigns shape people, and people and places carry that history forward. This documentation consolidates the three founding design drafts and subsequent discussion clarifications into a navigable game design document (GDD).

**Project state:** Kestrum has player-first faction rounds, deterministic commands, paused/stepped rival phases and recoverable named saves with separate round checkpoints. Old atlas saves remain read-only. Selectable world and regional maps show physical sites, gates, control, political claims and supply. Six-slot founding armies, Officers, recruitment/disbanding and round income/upkeep are playable. Costed group movement, free local transfers and supplied recovery are playable. Automatic field battles preserve casualties, retreats, wounds and recorded participant reports. These chapters describe intended behavior unless the current [implementation README](../README.md) says otherwise. Participation, veterancy, bounded histories and dated enemy observations are playable. Evidence-grounded people can emerge from service and earn ordinary careers and formation specializations; see [K13 verification](verification/k13-careers.md). See [K08 verification](verification/k08-service.md) and the earlier [UI verification record](verification/initial-map.md).

## Reading order

For implementation, begin with the [implementation plan](implementation-plan.md).
It turns these design chapters into 18 dependency-ordered work packages with
explicit contracts, provisional mechanics, acceptance cases, coverage and a
copyable agent handoff. The user requested concrete defaults for unresolved
mechanics on 2026-09-26; they are labelled **P** for review and are not newly
confirmed design decisions. K01–K13 are complete; K14–K18 remain required work.

Persistent construction, local facilities, roads and conserved Outpost settlers are
playable; see [K09 verification](verification/k09-construction.md) for validation and
the remaining platform limitations. Persistent sieges, escape and joint relief are
also playable; see [K10 verification](verification/k10-sieges.md). Living places,
conserved refugees, role relocation and ordinary threats are playable; see
[K11 verification](verification/k11-living-places.md). Legal rival orders,
War/Peace/truce, annexation/submission and saved kingdom endings are playable;
see [K12 verification](verification/k12-kingdoms.md). Grounded emergence and
ordinary careers are playable; see [K13 verification](verification/k13-careers.md).
The remaining generational features and production world belong to later packages.

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
| [13 — Decisions and open questions](13-decisions-and-open-questions.md) | Confirmed decisions, delegated defaults, remaining mechanics and scope questions |
| [Implementation plan](implementation-plan.md) | Current actionable sequence, precise provisional rules, agent handoff and acceptance ledger |
| [Plan verification](verification/implementation-plan.md) | Documentation checks and limits; no new gameplay implementation or balance validation |
| [Glossary](glossary.md) | Consistent terminology across chapters |
| [Source coverage](source-coverage.md) | Section-by-section provenance and source preservation checks |
| [Economy defaults](../assets/data/economy.json) | Provisional JSON costs, income, upkeep, recovery, and deficit/refund settings |
| [Documentation verification](verification/documentation.md) | Checks and limitations of this documentation milestone |

Read chapters 01–03 for the game overview, 04–09 for system design, and 10–13 before implementation. Cross-references between chapters describe dependencies rather than separate games or optional competing designs.

## Design status and authority

These labels distinguish the origin and maturity of a rule:

- **Source design:** stated in the supplied drafts. It may still be a tentative direction or an example; original qualifiers remain important.
- **Confirmed decision:** settled by the subsequent discussion and recorded in chapter 13. Implement this rule without reopening it because related tuning is unfinished.
- **Agreed direction:** the discussion establishes the feature's behavior or philosophy; the register names the remaining mechanics and tuning.
- **Delegated default:** a reasonable starting choice made under the user's permission to choose details. It is documented and adjustable during implementation or balance work.
- **Working interpretation:** a documented way to reconcile source ambiguity. It is a baseline for planning, not a claim that the author settled the question.
- **Proposal:** new detail added in this consolidation. It must be evaluated during implementation and playtesting.
- **Open:** a choice that needs a decision before the dependent feature can be accepted.
- **Future scope:** preserved design that is outside the first functional version.

Unlabelled descriptive system sections restate source design. The discussion clarification supplied on 2026-09-26 takes precedence where it settles an earlier ambiguity. Newly specified algorithms, contracts, acceptance criteria, screen plans, and development sequencing are proposals unless stated otherwise. Numbers remain tunable when the sources or discussion call them approximate.

The [decision register](13-decisions-and-open-questions.md) owns reconciliation across chapters. Shared engineering documents at the project root remain authoritative for code and workflow. Do not alter those shared documents to express Kestrum-specific design.

The earlier answers settle the ten formerly unanswered entries:

- The campaign has 80 world nodes, interpreted as major nodes with regional subnodes additional. Periodic effects resolve at the end of a full round.
- Armies can transfer formations and people whenever they share a node. Replenishment preserves veterancy. Progression requires relevant experiences and encounters.
- Economy defaults are adjustable JSON data. Saves are automatic at round end, manual during the player's turn, and have no game-imposed slot cap. Old narrative history may be forgotten.
- Initial content uses humans and ordinary classes. Display targets are 1920 × 1080 full screen and 1280 × 720 WebGL.

The earlier decisions about six formation slots, emergence, formation destruction, fixed graphs, named-character contributions, and 4–8 total factions remain in force. Remaining mechanics are listed in the register; vassalisation as a defeat outcome remains a proposal. The economy JSON is provisional content loaded and validated at startup by K01; economic simulation belongs to later packages.

The subsequent implementation-plan request selects concrete provisional defaults
for those remaining mechanics, including a narrow defeat-outcome vassal model.
Use the [packet authority rules](implementation-plan.md#authority-and-changes) and
[register mapping](13-decisions-and-open-questions.md#implementation-plan-defaults)
to distinguish those defaults from confirmed rules and earlier open questions.

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

The current `ui_*.png` images in `docs/verification/` show Kestrum's implemented title, atlas, and menus. The documentation-only verification record is historical.
