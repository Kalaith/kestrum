# Kestrum implementation plan

[Design index](README.md) · [Decision register](13-decisions-and-open-questions.md)

Prepared 2026-09-26 against `3a2b5b4` on `master`. This is an implementation
handoff, not evidence that the planned systems exist. The user requested concrete
provisional defaults for unresolved mechanics, clearly labelled for review.

## How to use this plan

For the current full-release assignment, implement successive numbered packages,
validate and commit each one, then continue automatically through K18. The user's
2026-09-26 full-release instruction supersedes the earlier one-package-per-request
handoff and author-review gate. Remaining gaps are delegated implementation
decisions recorded in chapter 13; confirmed rules and acceptance gates remain.

The user's 2026-09-27 direction is **game first, UI second**, with regular commits.
Prioritize complete simulation, meaningful choices and playable campaign outcomes.
Add the controls and feedback those features need, and keep visual verification
focused on changed decisions and actual defects. Do not expand screenshot matrices
or polish secondary screens while required game systems remain unimplemented.

Give an implementation agent one numbered work package at a time, together with
this entry point. Each package names its prerequisites, source chapters, rule
sections, intended files, outputs, and five behavioral acceptance cases. Complete
and commit a package before beginning the next independently useful change.

| Read | Purpose |
| --- | --- |
| [Contracts](implementation/contracts.md) | State ownership, actions, events, persistence, data, and integration boundaries |
| [Strategic rules](implementation/strategic-rules.md) | Exact starting defaults for world creation, turns, movement, control, economy, supply, and construction |
| [Combat rules](implementation/combat-rules.md) | Automatic battles, multiple armies, losses, retreats, sieges, and local threats |
| [People and places](implementation/people-and-places.md) | Experience, emergence, classes, settlement change, aging, mentorship, succession, and history |
| [Work packages](implementation/work-packages.md) | Ordered implementation tasks K01–K18 and their tests |
| [Acceptance and coverage](implementation/acceptance.md) | System coverage, cross-system scenarios, platform checks, and completion record |

Read only the linked rule sections needed by the current package after the common
contracts. The source chapters explain intent; these packets resolve implementation
details. Do not send a smaller model the entire repository and ask it to “implement
the game” in one pass.

## Authority and changes

1. Current user instructions and the confirmed rules in chapter 13 take precedence.
2. `AGENTS.md`, `CODE_STANDARDS.md`, and `UI_STYLE.md` govern engineering and review.
3. Existing delegated economy and retention defaults remain authoritative starting
   values: `assets/data/economy.json` and chapter 09.
4. Rules marked **P** in these packets are **provisional implementation defaults**,
   selected for this plan under the user's instruction. Use them as written unless
   a subsequent review changes them. They are not newly confirmed author decisions.
5. Earlier open questions and proposals are resolved provisionally only to the
   extent specified here. Source illustrations are not mandatory protagonists,
   timelines, guaranteed outcomes, or additional launch systems.

A missing rule is not permission to invent one, omit the feature, or choose a
different game loop. First check this packet and its sources. If a genuine gap or
contradiction remains, document the exact case and a proposed amendment, then get
that case resolved before shipping dependent behavior. Routine file naming and
implementation details can follow existing style. Do not repeatedly ask whether
already specified provisional defaults may be implemented.

Balance changes must update the appropriate JSON, relevant rule, and behavioral
expectation together. Changes to mechanics require a recorded reason and review;
tests failing against the specified rule are not a reason to silently rewrite it.
Keep source archives unchanged. Preserve the distinction between “implemented,”
“tested,” “published,” and “playtested.”

## Actual starting point

| Present at the inspected commit | What to retain or extend |
| --- | --- |
| `src/state.rs`: title/campaign state, overlays, preferences, `Campaign { version, turn }` | Extend the existing ownership structure; replace shell chronology with faction rounds |
| `src/game.rs`: action dispatch, assets, input, save/load, capture scenes | Keep it as application coordination; extract domain work into library services |
| `src/navigation.rs`, `src/ui/atlas.rs` | Full-bleed atlas, bounded camera, mouse/touch pan and zoom; overlay strategic geometry in the same coordinate space |
| `src/ui.rs`, `src/ui/menus.rs`, `src/ui/components.rs` | Existing action-returning UI, typography, title, menus, settings, Help, confirmations, error feedback |
| `src/data.rs`, `assets/data/game_config.json` | Toolkit-loaded presentation data and semantic checks |
| `assets/data/economy.json` | Six human formation types and delegated balance values; currently not loaded by runtime |
| `tests/campaign.rs`, `tests/navigation.rs`, `tests/asset_registry.rs`, `tests/code_standards.rs` | Existing regressions and source-size gate; tests already live in `tests/` |
| Real workspace registration, correct sibling toolkit path, Macroquad `=0.4.16` | Onboarding is complete; do not recreate it or change workspace membership |
| `catalog_thumbnail.png`, `scripts/capture_ui.ps1`, `publish.ps1` | Keep publication/capture integrations and stable verification paths |

The shell has no strategic nodes, factions, formations, combat, economy,
generations, or simulation history. Terrain labels are decorative. Its single
`kestrum_campaign_v1` slot is not the final save system. The old “End Turn advances
one season” behavior must change when faction turns arrive.

Existing verification reports 13 passing tests and successful native/WASM preview
publication for the shell. Those are historical results, not checks performed for
this plan. Browser fullscreen behavior and physical touch/pinch review remain
explicit limitations in [initial-map verification](verification/initial-map.md).

## Delivery sequence and scope

| Stage | Packages | Independently useful outcome | Earlier chapter 12 milestone |
| --- | --- | --- | --- |
| Foundation | K01–K04 | Validated data, deterministic actions/rounds, save catalogue, selectable nested map | Remaining M1; persistence pulled forward from M6 |
| Military prototype | K05–K08 | Recruit, move, transfer, fight, retreat, recover, inspect outcomes and knowledge | M2 plus early M3/M4 integration |
| Kingdom campaign | K09–K12 | Construction, sieges, changing settlements, ordinary local threats, AI, peace, conquest/vassal outcome | M4 |
| People and continuity | K13–K16 | Grounded people, ordinary careers, meaningful aging, family and non-family legacy | M3 and M5 |
| Complete campaign | K17–K18 | Generated 80-node campaign, 4–8 factions, integrated history, long-run and platform acceptance | M6 |

This staging does not authorize calling K12 the complete Kestrum game. The first
functional kingdom milestone and the full generational promise are distinct.
Foundation and military work use a durable small scenario; the production setup
switches to 80 major nodes in K17. No unfinished system receives a working-looking
button or a placeholder success message.

The sequence deliberately establishes events, knowledge boundaries, and saves
before later systems depend on them. People are introduced as small explicit
founder records for combat; full development arrives in K13. Local threats and
sieges produce their real participation facts before progression consumes them.
Every package integrates its visible controls and Help, rather than postponing
all interface work until the end.

## Required systems and deliberate limits

The destination includes nested warfare and partial ownership; six formations
per army; unlimited friendly stacking; co-located transfers; automatic battles
and persistent sieges with relief; constrained recovery; world-based recruitment;
emergence and recognition; ordinary class paths; human families and non-blood
successors; changing settlements, refugees, capitals, items and institutional
memory; War/Peace and a reachable ending; equal underlying AI rules; limited enemy
intelligence; unlimited named save lists and round-end autosaves.

Future content remains recorded in the coverage table: advanced/magical classes,
other races, multiplayer, full vassal diplomacy, alliances, claimant/restoration
factions, civil wars, detailed disease/captivity, weather attrition, naval warfare,
and stronger numerical bond effects. None is needed to fake an ordinary Medic,
succession, or a siege. Conversely, families, mentorship, refugees, and capital
relocation are scheduled work, not silently deferred forever.

## Reusable implementation prompt

```text
Implement KXX from docs/implementation/work-packages.md in the actual Kestrum
checkout on master. Read AGENTS.md, CODE_STANDARDS.md, UI_STYLE.md,
docs/implementation-plan.md, docs/implementation/contracts.md, and the rule/source
sections named by KXX. Inspect the current code and completion record first.

Implement only this package and necessary integration with completed packages.
Use the packet's provisional defaults as specified; do not substitute mechanics,
invent missing systems, or downgrade scheduled requirements to future work.
Preserve existing useful regressions and real toolkit/workspace integration.
Implement state, rules, errors, save/load behavior, visible touch controls, Help,
and the package's five behavioral cases. Do not create unused future scaffolding.

Run the required checks in docs/implementation/acceptance.md, including the
no-argument publisher for meaningful game changes and actual UI review for changed
screens. Record limitations honestly. Update the package status, design deviations,
README, and durable evidence. Commit all project changes under the repository
instructions and report the hash, tests, publishing, remaining blockers, and next
eligible package. Do not claim success from a copied checkout or an untested UI.
```

## Current completion record

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
[K12 evidence](verification/k12-kingdoms.md). The user requested a stop after this
package and its commit; further implementation requires a later continuation.
K13–K18 remain **planned, not implemented**.
[Acceptance](implementation/acceptance.md#completion-record) owns the per-package
status table. The current atlas has faction-phase controls, selectable places and
army rosters. Minimum-browser display and physical-touch review remain open for K18.
K10 fixed WASM update caching through the existing publisher option.
