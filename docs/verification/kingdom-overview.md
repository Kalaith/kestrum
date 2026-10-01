# M01 — Readable kingdom overview

2026-10-01. Implementation follows `3073c18` directly on the actual `master`
checkout. M01 changes presentation and observer summaries; regional topology,
movement rules, seasonal accounting and enemy intelligence rules are preserved.

## Implemented behavior

- Political tint and boundaries partition known atlas land using projected
  claims. An authored coast/water mask clips the artwork, not the route graph.
  Unknown regional claims have no fill and use `?`; local rings still show
  discovered control. `/` indicates occupation and `X` contested control.
- Settlement silhouettes distinguish habitation tiers; forts have battlements,
  the owned capital has a crown, and important resource/work/damage cues appear
  at useful zoom. Collision-aware labels prioritize selection, capital, danger,
  region and owned places. Known kingdom names support political orientation.
- Owned army banners show identity (or an explicit regional stack count), total
  owned troops and siege, cut-off, queued-route or idle state. Regional stacks
  do not imply physical co-location. Rendering and picking share banner geometry
  and whole-banner occlusion rules.
- The lower strip shows Gold/Wood/Stone, the actual last completed season's
  income, upkeep paid/due and recovery cost. Before a completed season it says
  there is no receipt; it never presents a forecast as earned income.
- Attention starts collapsed, opens upward with three paged rows, and collapses
  during selection. Its highest-priority condition per known site is siege,
  hostile contact, local threat, contested control or occupation; cut-off owned
  armies follow. A row focuses its actual site or owned army without issuing an
  order. Hidden targets are rejected.
- World region warnings aggregate only already-visible internal threat sites,
  participating sieges and hostile contact sites. Counts describe known
  conditions, never enemy forces or strength. Foreign capital roles remain
  private under the existing observer contract.

The initial top accounts strip obscured northern capitals in the dense scene.
Visual review moved it below the map and shortened inspectors to end above it.
The map retains its camera and direct destination-tap movement. An attention
selection clears only the transient order card; stored movement plans remain.

## Behavioral verification

Five `map_overview` cases cover hidden-change invariance, mixed regional control,
known danger aggregation and disappearance, live development/ownership/army
facts and actual receipts, plus siege-specific supply. A separate navigation
case covers both viewport sizes, three zooms, label priority/collisions, unchanged
place/banner picking and artwork land/water clipping. The sixth case is distinct
geometry coverage; it does not duplicate the observer/simulation cases.

The capture action harness focuses a known internal site and an owned army,
checks that simulation state is unchanged, and rejects a real undiscovered site.
Stress snapshots are explicitly authored, pass normal campaign validation, and
produce their economy statement through actual seasonal commands. They are not
claims of earned campaign progress. `midgame_map` uses the existing earned
developed campaign. `overview_siege` reuses the command-established siege fixture.

## Visual evidence

Shared hidden captures write directly to stable paths in this directory.

| State | Normal | Minimum |
| --- | --- | --- |
| Early fog and home force | [Early](ui_production_world.png) | [Early](ui_production_world_minimum.png) |
| Eight factions, long names, mixed control, known danger | [Urgent](ui_overview_urgent.png) | [Urgent](ui_overview_urgent_minimum.png) |
| Regional local control | [Region](ui_overview_region.png) | [Region](ui_overview_region_minimum.png) |
| Focused attention object | [Attention](ui_overview_attention.png) | [Attention](ui_overview_attention_minimum.png) |
| Known siege aggregation | [Siege](ui_overview_siege.png) | [Siege](ui_overview_siege_minimum.png) |
| Actual unpaid upkeep and cut-off force | [Deficit](ui_overview_deficit.png) | [Deficit](ui_overview_deficit_minimum.png) |
| Earned developed realm | [Midgame](ui_midgame_map.png) | [Midgame](ui_midgame_map_minimum.png) |
| Immediate partial movement | [Movement](ui_world_move_partial.png) | [Movement](ui_world_move_partial_minimum.png) |
| Dense army selection | [Stack](ui_world_move_stack.png) | [Stack](ui_world_move_stack_minimum.png) |

Both nine-scene capture runs completed through the shared hidden wrapper and
their launched game processes exited. The normal request was 1920×1080 (native
client capture 1920×1061); the minimum captures are exactly 1280×720. Review
covered early fog, eight factions and long names, the earned Year-16 realm,
occupation/contest, known siege/contact/threats, unpaid upkeep, attention focus,
partial movement and dense army selection. The map remains dominant; accounts,
Attention, zoom/recenter, inspector close and End Turn remain readable and
separate. Long labels intentionally truncate, with complete names on selection.
Important crowded labels use leader lines. Banners avoid their own capital and
warning symbols, including when clamped against a map edge.

## Validation results

All checks used this actual checkout and its shared workspace configuration.

- `cargo fmt --check`: passed.
- Shared Cargo strict Clippy, `-p kestrum --all-targets --all-features -- -D warnings`:
  passed after the final display refinements.
- Shared Cargo focused tests (`map_overview`, `map_overview_navigation`,
  `map_exploration`, `world_army_orders`, `movement_plans`, `code_standards`):
  23 passed after the final refinements, including the 800-line source limit.
- Shared Cargo full suite, `test -p kestrum --all-features --no-fail-fast`:
  only the inherited
  `k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`
  failed (seed 88, no victory by round 240). Its assertion is unchanged.
  The four/eight-faction 400-round continuity cases passed; one release profiling
  test remains intentionally ignored. This full run preceded the last banner,
  label and receipt refinements, which the final focused checks and captures cover.

- No-argument `.\publish.ps1`: passed Windows release, WebGL release, packaging
  and deployment to the local Preview at
  `\\wsl.localhost\Ubuntu\home\kalai\dev\games\kestrum`. The publisher refreshed
  the catalogue and recorded the publish in Project Roost.

## Published browser review

Exercised `http://127.0.0.1/games/kestrum/` after publication and reload. A new
four-faction Rose campaign used the ordinary default seed 260926 and preserved
existing saved campaigns. In the normal full-screen view (DOM canvas 1936×1048):

- Closed the opening guide; opened Attention and selected Riverfold Crossing.
  Its actual inspector appeared without travel or a changed season/balance.
  [Attention focus](ui_overview_browser_attention.jpg).
- Closed selection and recentered, selected Rose Host, then tapped Riverfold
  Bridge. It arrived immediately, remaining movement changed from 6 to 3, and
  the banner followed the army. Dragging and the visible + control changed the
  view while preserving that location and allowance.
- End Turn remained reachable with the army selected. After rival turns, Summer
  Year 1 / Round 2 showed Gold 530, Wood 223, Stone 164; actual season 1 income
  was +60G/+23W/+14S, upkeep paid/due 30/30G and recovery 0G.
- World Map showed the owned capital crown, known internal danger and an unknown
  political claim; undiscovered land remained fogged. The 1920×1080 override
  rendered the overview and its close action correctly.
- After resetting the viewport and reloading, Continue restored Round 2, the
  army at Riverfold Bridge, resources and the same receipt.
  [Restored campaign](ui_overview_browser.jpg). Browser warning/error logs were
  empty at startup and after this check.

The 1280×720 override reported matching DOM viewport, canvas backing dimensions
and canvas bounds. Its first frame filled the view; selecting the attention
entry focused Riverfold Orchard but reproduced the inherited smaller upper-left
image with black remainder. [Minimum browser limitation](ui_overview_browser_minimum.jpg).
Minimum WebGL acceptance therefore remains unverified; native minimum review
and successful click equivalents do not replace it. The override was reset.

## Retained scope limits

The physical-touch waiver remains in effect; click equivalents do not establish
device or pinch acceptance. M02 owns broader map actions, persistent route
presentation and seasonal consequence receipts; M03 owns regional terrain and
topology. Existing battle presentation and production-victory balance findings
remain independent limitations. This milestone does not claim a first-time
human playtest or integrated M05 acceptance.
