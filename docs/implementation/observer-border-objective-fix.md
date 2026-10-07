# Stop AI armies oscillating between border sites

**Status:** Planned implementation handoff. The fix is not implemented.

## Symptom and evidence

In Observer mode, an army can move back and forth between neighboring sites for
many seasons without a war or a changing military situation. This comes from
the shared faction AI planner, so ordinary NPC turns can show the same behavior.

The audit inspected for this report is the machine-local file
`%LOCALAPPDATA%\kestrum\observer_logs\observer-260926-1791332235106.jsonl`.
It contains 1,940 `observer_step` records. The following accepted actions are
from the same run; each has a `Border` objective and the faction diagnostics
show no active wars:

| Audit record | Round | Army | Accepted movement | New objective |
| ---: | ---: | --- | --- | --- |
| 1855 | 164 | Rose Host | site 42 → 39 | Border at 39 |
| 1885 | 168 | Rose Host | site 39 → 42 | Border at 42 |
| 1912 | 172 | Rose Host | site 42 → 39 | Border at 39 |
| 1941 | 176 | Rose Host | site 39 → 42 | Border at 42 |

The same pattern appears for Ashen Lark's Army 9 between sites 25 and 26,
Bera Lark's Armies 3 and 7 between sites 48 and 51, and Ashen Wren's Army 4
along a longer sequence of adjacent sites. The audit records accepted movement
facts and the changed army locations, so this is actual campaign movement, not
only a map-marker animation.

## Current cause

The relevant implementation is the shared AI planner in
[`src/engine/ai/travel.rs`](../../src/engine/ai/travel.rs) and
[`src/engine/ai.rs`](../../src/engine/ai.rs):

1. `Planner::border` treats an owned site as a border target when any adjacent
   site belongs to another faction. That test does not require an active war or
   another military need.
2. The planner considers border movement only after earlier priorities such
   as expansion, attacks and diplomacy have not produced an order.
3. `toward` requires a route with at least one step. An army already on a
   candidate post cannot satisfy that target with a hold decision; the border
   planner can continue to another valid post instead.
4. Retained objectives expire after `objective_rounds`, currently four rounds
   in [`assets/data/ai.json`](../../assets/data/ai.json). When the objective
   expires, the planner can select the neighboring border site and reverse the
   army's route. The audit's four-round reversals match this lifetime.

Increasing `objective_rounds` alone would only make the reversals less
frequent. It would not remove the retargeting loop.

## Intended behavior

- A peaceful border, by itself, must not make an army march between friendly
  sites. Border movement should require a real strategic need, such as active
  war pressure, a known threat, or a specific defensive objective.
- When an army has reached a valid defensive post, the planner should treat
  that objective as satisfied and leave the army there until the situation
  changes. Expiring or completing an objective must not automatically select a
  neighboring post that sends the same army back.
- During an active war or known threat, AI armies must still be able to take
  useful, legal defensive and offensive routes. Keep movement, supply, and
  battle rules authoritative; preserve the faction's observer-knowledge
  boundary and deterministic planning.
- Apply the behavior in the shared AI planner so it is consistent in Observer
  mode and ordinary faction turns.

Use the existing `at_war` and observed-threat concepts where they fit. Do not
use a global increase to the objective lifetime as the fix, and do not create
an observer-only movement policy.

## Acceptance

1. Add a deterministic AI regression case with adjacent friendly border sites
   and a mobile army. With the factions at peace and no other military need,
   advancing past multiple objective-expiry intervals produces no repeated
   movement that alternates the army between those sites.
2. Cover the transition from war to peace: after the military need is gone, the
   army does not keep patrolling between friendly sites solely because the
   other side owns adjacent territory.
3. Retain a positive active-war or known-threat case showing the AI can still
   choose a useful legal movement or defensive order.
4. Keep the result deterministic across repeated runs and preserve ordinary
   Observer playback, legal-command validation, supply rules, and private AI
   projections.
5. Review the generated observer audit or an equivalent deterministic test
   trace. Confirm that movement stops once a valid post is reached and that a
   later order requires a changed strategic situation.

## Implementation and validation notes

Likely code touch points are `src/engine/ai/travel.rs` (border candidates and
movement toward targets), `src/engine/ai.rs` (objective completion/retention),
and the existing AI regression tests. Change `assets/data/ai.json` only if the
implementation has a reason to tune objective lifetime after fixing target
selection.

Follow [`PROJECT_AGENTS.md`](../../PROJECT_AGENTS.md) for the Kestrum validation
override. Because this changes simulation behavior, validate the focused AI
regression and the relevant AI, diplomacy, movement and Observer coverage, then
run the full suite at an integration boundary. Run formatting, strict
all-target/all-feature Clippy and the source-size gate. Report the inherited
seed-88 round-240 production-victory failure as a failure/baseline exception,
not a pass. Do not run `publish.ps1`.
