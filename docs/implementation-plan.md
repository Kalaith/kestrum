# Kestrum implementation plan

[Design index](README.md) · [Active map playability plan](map-playability-plan.md) · [Decision register](13-decisions-and-open-questions.md)

Originally prepared 2026-09-26 against `3a2b5b4` on `master`. Status reconciled
2026-10-01 against the current source, README and delivery records. This document
records the delivered K01–K18 baseline. The
[map playability plan](map-playability-plan.md) owns the next work; do not restart
completed packages or use this original handoff as the current task list.

## Current delivery status — 2026-10-01

K01–K18 are **Done under the user's amended testing scope**. The
[full implementation review](implementation-review.md) maps R01–R13 to their
fixes. Normal New Campaign creates the 80-major/152-physical production world
with deterministic 4–8 faction setup; Rosemarch remains the regression fixture.

On 2026-09-28 the user waived physical-touch testing and ended further testing
while gameplay and balance may still change. Remaining browser, rendered
performance and multi-seed balance acceptance is deferred. The serial 4/8-faction
400-round replay passed; eight-faction phase p95 improved to 114.63 ms. The
K18 closeout recorded a seed-88 victory deadline failure at round 120. Later
B01–B07 delivery passed its then-current suite and reached a scripted victory at
round 198. Subsequent reaction/rout fixes leave the unchanged 240-round production
victory assertion failing. The [battle review](verification/battle-review.md)
records that current balance blocker and remaining battlefield presentation
issues. Historical passes do not establish a current full-suite pass, settled
balance or full platform acceptance. [K18 verification](verification/k18-integrated.md)
preserves the earlier platform and performance evidence.

## How to use this plan

The [battle system implementation plan](battle-system-implementation-plan.md)
records completed B01–B07 delivery. Formation positioning, ordered tactics,
pending preparation, visible battlefield playback, leaders and rival doctrines
are implemented. Its integration supersedes the original non-positional P11/P12
resolver. Preserve campaign consequences and existing regression coverage when
working on the map; a standalone tactical game is not pending work.

Read the active map plan first, then use these packets to understand existing
state, rules, persistence and tests. The user's playability review changes the
next priority to readable geography, kingdom state and actions on the map.
Feature delivery and a usable strategic experience are separate acceptance
questions. Implement and commit each independently useful active-plan change;
do not interpret the completed K sequence as an instruction to repeat it.

| Read | Purpose |
| --- | --- |
| [Contracts](implementation/contracts.md) | State ownership, actions, events, persistence, data, and integration boundaries |
| [Strategic rules](implementation/strategic-rules.md) | Exact starting defaults for world creation, turns, movement, control, economy, supply, and construction |
| [Combat rules](implementation/combat-rules.md) | Historical P11/P12 resolver and retained campaign consequences; current formation combat is in the battle plan |
| [People and places](implementation/people-and-places.md) | Experience, emergence, classes, settlement change, aging, mentorship, succession, and history |
| [Work packages](implementation/work-packages.md) | Completed delivery contracts K01–K18 and their regression cases |
| [Acceptance and coverage](implementation/acceptance.md) | System coverage, cross-system scenarios, platform checks, and completion record |

Read the linked sections needed by the current change after the common contracts.
Where a packet describes an earlier delivery stage, its dated status note and
later implementation amendment take precedence over the original imperative text.

## Authority and changes

1. Current user instructions and the confirmed rules in chapter 13 take precedence.
2. `AGENTS.md`, `CODE_STANDARDS.md`, and `UI_STYLE.md` govern engineering and review.
3. Existing delegated economy and retention defaults remain authoritative starting
   values: `assets/data/economy.json` and chapter 09.
4. Rules marked **P** are delegated implementation defaults used for the delivered
   campaign, subject to documented later amendments. The label is not an outstanding
   approval gate or a claim that the founding draft supplied those values.
5. Earlier open questions and proposals are resolved provisionally only to the
   extent specified here. Source illustrations are not mandatory protagonists,
   timelines, guaranteed outcomes, or additional launch systems.

First check the active plan, implemented rules and recorded amendments when a
gap appears. Resolve routine implementation choices within the user's delegated
scope and document the reason. Ask only when a genuine scope conflict or missing
user decision blocks the work; the word provisional does not require approval.

Balance changes must update the appropriate JSON, relevant rule, and behavioral
expectation together. Changes to mechanics require a recorded reason and review;
tests failing against the specified rule are not a reason to silently rewrite it.
Keep source archives unchanged. Preserve the distinction between “implemented,”
“tested,” “published,” and “playtested.”

## Historical starting point — 2026-09-26

This table describes the pre-K01 shell at `3a2b5b4`, not the current game.
The current baseline includes the systems in the completion record below,
direct map orders with saved routes, exploration and the integrated battlefield.

| Present at the inspected commit | What to retain or extend |
| --- | --- |
| `src/state.rs`: title/campaign state, overlays, preferences, `Campaign { version, turn }` | Extend the existing ownership structure; replace shell chronology with faction rounds |
| `src/game.rs`: action dispatch, assets, input, save/load, capture scenes | Keep it as application coordination; extract domain work into library services |
| `src/navigation.rs`, `src/ui/atlas.rs` | Full-bleed atlas, bounded camera, mouse/touch pan and zoom; overlay strategic geometry in the same coordinate space |
| `src/ui.rs`, `src/ui/menus.rs`, `src/ui/components.rs` | Existing action-returning UI, typography, title, menus, settings, Help, confirmations, error feedback |
| `src/data.rs`, `assets/data/game_config.json` | Toolkit-loaded presentation data and semantic checks |
| `assets/data/economy.json` | Six human formation types and delegated balance values; not loaded by the pre-K01 runtime |
| `tests/campaign.rs`, `tests/navigation.rs`, `tests/asset_registry.rs`, `tests/code_standards.rs` | Existing regressions and source-size gate; tests already live in `tests/` |
| Real workspace registration, correct sibling toolkit path, Macroquad `=0.4.16` | Onboarding is complete; do not recreate it or change workspace membership |
| `catalog_thumbnail.png`, `scripts/capture_ui.ps1`, `publish.ps1` | Keep publication/capture integrations and stable verification paths |

That shell had no strategic nodes, factions, formations, combat, economy,
generations or simulation history. Terrain labels were decorative. K01–K18
replaced its single `kestrum_campaign_v1` slot with the strategic save catalogue
and its season-per-click chronology with faction phases and round boundaries.

Existing verification reports 13 passing tests and successful native/WASM preview
publication for the shell. Those are historical results, not checks performed for
this plan. Browser fullscreen behavior and physical touch/pinch review remain
explicit limitations in [initial-map verification](verification/initial-map.md).

## Completed delivery sequence and scope

| Stage | Packages | Independently useful outcome | Earlier chapter 12 milestone |
| --- | --- | --- | --- |
| Foundation | K01–K04 | Validated data, deterministic actions/rounds, save catalogue, selectable nested map | Remaining M1; persistence pulled forward from M6 |
| Military prototype | K05–K08 | Recruit, move, transfer, fight, retreat, recover, inspect outcomes and knowledge | M2 plus early M3/M4 integration |
| Kingdom campaign | K09–K12 | Construction, sieges, changing settlements, ordinary local threats, AI, peace, conquest/vassal outcome | M4 |
| People and continuity | K13–K16 | Grounded people, ordinary careers, meaningful aging, family and non-family legacy | M3 and M5 |
| Complete campaign | K17–K18 | Generated 80-node campaign, 4–8 factions, integrated history, long-run and platform acceptance | M6 |

This was the original delivery sequence. K17 introduced the current 80-major
production content; K18 closed under the amended testing scope. The active map
plan can revise authored geography with explicit save compatibility; these counts
describe delivered content rather than an immutable visual design requirement.

The sequence established events, knowledge boundaries and saves before later
systems depended on them. Founders preceded full personal development in K13;
threats and sieges produced participation facts before progression consumed them.
Each delivery included its controls and Help. The active map plan addresses the
remaining gap between those feature screens and a readable strategic experience.

## Required systems and deliberate limits

Delivered systems include nested warfare and partial ownership; six formations
per army; unlimited friendly stacking; co-located transfers; automatic battles
and persistent sieges with relief; constrained recovery; world-based recruitment;
emergence and recognition; ordinary class paths; human families and non-blood
successors; changing settlements, refugees, capitals, items and institutional
memory; War/Peace and a reachable ending; equal underlying AI rules; limited enemy
intelligence; unlimited named save lists and round-end autosaves.

Future content remains recorded in the coverage table: advanced/magical classes,
other races, multiplayer, full vassal diplomacy, alliances, claimant/restoration
factions, civil wars, detailed disease/captivity, weather attrition, naval warfare,
and stronger numerical bond effects. Families, mentorship, refugees and capital
relocation are implemented baseline capabilities, not work awaiting a package.

## Handoff for current work

```text
Start with docs/map-playability-plan.md and the current user request. Read
AGENTS.md, CODE_STANDARDS.md, UI_STYLE.md and the relevant implemented rules.
Work in the actual Kestrum checkout on master. K01–K18 and B01–B07 are delivered
baseline, with disclosed balance and presentation limitations; do not reimplement
them from historical work-package text. Preserve state and save compatibility,
useful regressions, existing toolkit integration and observer boundaries.
Complete the active plan's relevant gameplay and visual acceptance, record
actual checks and blockers, update current documentation, and commit under
the repository instructions when implementation is requested.
```

## Current completion record

The checks below are delivery-time evidence. They are retained with their dates
and do not override the later balance blocker or K18 testing-scope amendment above.

K01 is **Done**: toolkit-loaded typed economy and the durable Rosemarch scenario
pass their behavioral checks. See [K01 evidence](verification/k01-content.md).
K02 is **Done**: atomic strategic commands, faction rounds and v1/v2 compatibility
pass the checks in [K02 evidence](verification/k02-campaign.md).
K03 is **Done**: unlimited indexed saves, compatibility, retry and touch naming
pass the checks in [K03 evidence](verification/k03-saves.md).
K04 is **Done**: selectable nested geography, entrances, supply and territorial
anchors pass the checks in [K04 evidence](verification/k04-geography.md).
K05 is **Done**: six-slot armies, founding Officers, recruitment/disbanding and
round economy pass [K05 evidence](verification/k05-armies.md).
K06 is **Done**: costed group movement, preserved transfer allowances and supplied
recovery pass [K06 evidence](verification/k06-logistics.md).
K07 is **Done**: automatic encounters, casualties, retreat, limited wounds and
recorded reports pass [K07 evidence](verification/k07-combat.md).
K08 is **Done**: participation, veterancy, bounded history and observer-safe records
pass [K08 evidence](verification/k08-service.md).
K09 is **Done**: persistent construction, local facilities and conserved settlers
pass [K09 evidence](verification/k09-construction.md).
K10 is **Done**: persistent sieges and multi-army relief pass
[K10 evidence](verification/k10-sieges.md).
K11 is **Done**: living settlements, refugees, administration and ordinary threats
pass [K11 evidence](verification/k11-living-places.md).
K12 is **Done**: legal AI, diplomacy and reachable kingdom endings pass
[K12 evidence](verification/k12-kingdoms.md).
K13 is **Done**: `10ab4c6` implements evidence-grounded emergence, ordinary class
courses, appointments and formation specialization. Nine progression cases and
136 tests pass, with strict Clippy, formatting, the source-size gate, native
screen review and Windows/WebGL Preview publishing. See
[K13 evidence](verification/k13-careers.md). Its remaining browser, physical-touch
and minimum-WebGL checks were later covered by K18's disclosed scope amendment.
K14 is **Done**: aging, birthday mortality, wounded-person recovery, retirement,
governors and qualified local mentorship integrate with player and rival actions.
Fifteen lifecycle/mentorship regressions plus the existing suite pass (151 total);
formatting, strict Clippy, source-size gate, twelve normal/minimum native captures and
Windows/WebGL Preview publishing pass. See
[K14 evidence](verification/k14-lifecycle.md). Its remaining browser, physical-touch
and minimum-WebGL checks were later covered by K18's disclosed scope amendment.
K15–K17 are **Done** as delivered: households/succession, heirlooms/history and
production setup are recorded in [K15 evidence](verification/k15-succession.md),
[K16 evidence](verification/k16-heirlooms.md) and
[K17 evidence](verification/k17-production.md). The full implementation review
records corrective follow-ups to these and earlier deliveries.
[Acceptance](implementation/acceptance.md#completion-record) owns the per-package
status table. The current atlas has faction-phase controls, selectable places and
army rosters. K18's remaining browser testing is deferred and physical-touch
testing is waived under the user's 2026-09-28 scope amendment.
K10 fixed WASM update caching through the existing publisher option.
