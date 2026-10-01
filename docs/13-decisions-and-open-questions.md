# 13 Decisions and current defaults

[Documentation index](README.md) · [Active map plan](map-playability-plan.md)

## How to use this register

This register separates fixed user decisions, implemented defaults and planned
map changes. The filename is retained for existing links; solved prototype
questions are no longer presented as pending decisions.

The user's current request takes precedence. Follow the active map plan for
delivery and the owning system chapter/source for behavior. Routine
implementation choices within that scope do not require a new confirmation
because an original draft called them provisional.

## Confirmed decisions

| Topic | Constraint |
| --- | --- |
| World scale (O01) | 80 major world markers; regional physical sites are additional. |
| Fixed geography | A campaign has an authored route graph. Regions can have different networks; identical chains are not a requirement. |
| Factions (O08) | Player selects 4–8 total factions, including the player. |
| Formation model (D01) | Six troop-formation slots; people serve within forces. Current membership is one named person per formation plus troops. |
| Formation destruction | A destroyed formation loses its active identity/veterancy; witnessed consequences can remain in people/place/world history. |
| Replenishment (O16) | Surviving formations retain veterancy when replenished. |
| Transfers (O09) | Co-located friendly forces can transfer under the implemented action guards without resetting spent movement. |
| Supply | Cut-off armies cannot normally recover headcount/health; current supply follows secure friendly physical routes. |
| Calendar (O04/D03) | Periodic effects occur once after all active factions finish a round; one season per round, four per year. |
| Careers (O10/O15) | Relevant service and encounters establish eligibility; generic XP cannot replace required experience. |
| Enemy information (O21) | Keep unobserved enemy details hidden; dated witnessed information remains distinct from live state. |
| Economy (O11) | Costs and balance are adjustable JSON defaults loaded through the toolkit. |
| Saves (O22) | Round-end autosaves, manual saves during player orders, no game-imposed slot cap, explicit recovery and overwrite/deletion flows. |
| History (O23) | Bounded narrative retention may forget old stories while preserving current state and gameplay evidence. |
| Content (O24) | Humans and ordinary classes first; other races and advanced classes are deferred. |
| Viewport (O25, corrected 2026-10-01) | 1920×1080 is the sole logical UI, design and acceptance spec, with visible touch controls. This user correction supersedes the earlier dual-resolution target. M01A implements the migration; acceptance is tracked separately. |

## Implemented defaults

These questions have working implementations. Their numeric tuning can change
with evidence; their earlier “open” labels are not blockers.

| Area | Current baseline and owner |
| --- | --- |
| Setup and starts | Seeded production layout revision 3; region-based headquarters candidates with a safe first route and nearby threats. [World](02-world-time-and-control.md), [generation source](../src/data/generation.rs). |
| Spatial scale and navigation | M01A uses a 1920×1080 UI, 5280×2970 world and 3360×1890 regional bounds at normal zoom 1. Three bands use separate entry/exit thresholds; Overview/Return View, compact force marks and group focus preserve context. [Interface](10-interface-and-accessibility.md), [scoped verification](verification/spatial-scale.md). |
| Movement and roads | Immediate destination orders, partial travel, saved continuation, access rechecks and preserved spent allowance. [Armies](04-armies-and-logistics.md), [movement source](../src/engine/movement.rs). |
| Supply and recovery | Physical connectivity, post-upkeep affordability and persisted outcomes. [Armies](04-armies-and-logistics.md). |
| Construction | Persistent prepaid work, builders, progress, interruption, cancellation and conserved settlers. [Living places](07-living-places.md). |
| Victory and submission (D04/O07) | Defeat/endings and inactive submission are implemented; submission adds no tributary economy or independent military. [Kingdoms](03-kingdoms-and-economy.md). |
| Combat and sieges | Prepared deterministic formation resolution, immutable playback, lasting losses and persistent siege orders. [Battles](05-battles-and-sieges.md). |
| People and generations | Evidence-based emergence, ordinary careers, aging, recovery, retirement, mentorship, households and succession. [Characters](06-character-development.md), [Generations](08-generations-and-succession.md). |
| Living places | Growth/decline, occupation, damage, refugees, changing roles and ordinary threats. [Places](07-living-places.md). |
| Legacy and memory | Factual custody/deeds, witnessed succession, bounded histories and known dates. [History](09-history-and-content.md). |
| AI and phases | Legal bounded rival orders and observable Pause/Step/Resume; no impossible units or hidden permission bypass. [Kingdoms](03-kingdoms-and-economy.md). |

A campaign lasting roughly 20–50 years and emergence becoming rarer toward
20–30 important people are experience/tuning targets, not mandatory timers or
hard roster caps. Exact prices, leadership effects, age rules and thresholds
belong to current validated data and the owning source. Earlier illustrative
values do not override implemented rules.

## Implementation-plan defaults

The P01–P23 packet supplied delegated defaults for K01–K18. It remains a detailed
mechanical reference, updated where later movement and battle changes supersede
it. It is not a second implementation queue or a list of unapproved systems.

- [Strategic rules](implementation/strategic-rules.md): setup, phases, control,
  movement, supply, economy, construction, diplomacy and AI.
- [Combat rules](implementation/combat-rules.md): encounters, losses, siege and threats.
- [People and places](implementation/people-and-places.md): evidence, careers,
  development, lifecycle and continuity.
- [Battle delivery](battle-system-implementation-plan.md): completed B01–B07
  formation combat and presentation.

Historical I01–I20 fix narratives are represented by the completed acceptance
and verification records. They should not be copied into new plans as unresolved
problems. The [evidence index](verification/README.md) identifies current limits.

## Implemented and planned map decisions

M01's political territory, local control, symbols, accounts and known attention
are implemented with recorded limits. M01A completes the spacious-world request
under its [recorded scope](verification/spatial-scale.md): engineering checks,
14 native captures, automated native actions, background browser gameplay,
publishing and reload, with a complete 1920×1080 browser capture.

| Choice | Implemented default |
| --- | --- |
| Presentation bounds | World 5280×2970, region 3360×1890; at normal zoom 1, respectively 2.75 and 1.75 screen widths. Authored normalized geometry and movement costs are unchanged. |
| Information bands | Overview enters at 0.68 and exits at 0.78; detail enters at 1.45 and exits at 1.30; campaign fills the middle. Maximum zoom is 2.4. Values are validated JSON tuning defaults. |
| Camera actions | New and loaded campaigns focus home. Overview fits known land; Return View restores the working camera. Recenter focuses the selected force or home. World and individual regions retain independent session contexts. |
| Symbols and forces | Compact marks replace large unselected banners. Crowded place and nearby-force groups focus before exact selection; regional grouping remains distinct from armies sharing one site. Full details remain on selection. |
| Secondary screens | Management sheets keep pixel-sized content centered within 1920×1080, with matching camera/pointer translation. Map Key holds symbol and navigation explanations. Battlefield small controls remain an inherited limitation. |

These map recommendations remain planned:

1. Keep economic context and a compact attention list with the map; use one
   contextual inspector for common actions and defer full management detail.
2. Keep direct movement and show relevant saved orders and seasonal consequences.
3. Replace repeated regional chains with distinct connected geography. Retain
   the existing world/region scopes and initial physical-site count for delivery.
4. Preserve existing campaign topology; establish versioned authored validation
   before introducing new graphs. New worlds and old worlds remain truthful.
5. Teach the opening through a useful action, consequence and investment.
   Introduce careers/households when relevant to events in play.

The [map plan](map-playability-plan.md) owns milestone detail and acceptance.
This updates the earlier sparse-HUD, fit-known-world and screen-tour assumptions.

## Remaining choices within the plan

Resolve these while implementing the relevant milestone and record the result:

- Further map extent, band, icon and label tuning where actual 1920×1080 review
  supplies evidence; the M01A defaults above are implemented rather than open.
- Region-specific connections, local terrain and anchor expressions within the
  distinct strategic briefs and save-compatibility contract.
- Which legal opening opportunities best demonstrate a payoff for different
  starting regions, assessed through actual interaction and player observation.

These are bounded design tasks, not reasons to postpone delivery or recreate a full
GDD. Numerical balance remains tunable; avoid unrelated system expansion.

## Deferred work and validation limits

The map plan does not add richer vassal diplomacy, alliances, tribute, other
races, advanced classes, multiplayer, claimant factions or additional world
locations. A larger navigable presentation of the existing world is in scope.
Existing recorded campaign-balance and battlefield issues remain visible in the
[evidence index](verification/README.md).

Physical-touch testing was waived and further platform/performance/balance work
deferred on 2026-09-28. Keep that scope decision, report untested behavior
honestly and retain visible touch controls. Completed K18 status does not mean
all platforms or campaign balance passed.

## Source handling

The original drafts and template README remain unchanged as provenance.
Illustrative people, timelines, counts and proposed algorithms in those files
are examples, unless adopted in current rules. See
[source coverage](source-coverage.md) for preserved hashes and source spans.
