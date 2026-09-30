# Battle system implementation plan

[Design index](README.md) · [Battle design](05-battles-and-sieges.md) · [Current implementation](../README.md)

Prepared 2026-09-30 against `f49be5a` on `master`. This plan turns the supplied
[battle system notes](reference/battle-system-notes.txt) and the subsequent
[battlefield mockup](reference/battle-system-mockup.png) into an implementation
sequence for Kestrum. The mockup and the user's request for game-focused
presentation govern the visual direction. This plan proposes gameplay and
engineering choices; it does not claim they are implemented or approved balance
values. The source is preserved
byte for byte. The original K01–K18 plan remains the record of the earlier release.

The target experience is to **prepare an army, program its tactics, then watch
that plan meet an opposing formation on a visible battlefield**. Soldiers and
their actions occupy the main play area; formation slots are underlying rules,
not twelve dashboard cards. The first playable version must let the player see
a front line break and cavalry exploit
the opening. Animation of the existing arithmetic alone would not establish
that experience.

## Battlefield direction from the mockup

Build a single illustrated battlefield, with the player's force on the left and
the enemy on the right, facing inward. Use terrain, depth, banners, weapons and
distinct troop silhouettes to establish the armies and their roles. Front and
rear ranks must be spatially legible within that scene. A logical formation
array does not need visible slot boxes, a checkerboard or a separate arena panel.

The mockup's strongest elements are the visible troop groups, shared landscape,
army identity, readable unit condition and a reaction such as Brace appearing
over the soldiers who execute it. Translate actual combat events into movement:
cavalry accelerates toward an exposed unit, spears lower to meet it, archers
release a volley, a broken unit withdraws, and the vacated space stays visible.
The player should understand the exchange by watching the battlefield before
opening any explanation.

Keep the interface subordinate to that scene. Aim for at least 75% of the default
execution view to remain battlefield. Use a shallow top display for army names,
aggregate troop counts/morale and round number. Place compact unit names,
headcounts and morale indicators beside their troop groups. Emphasize the current
actor and target through the group itself, using brief ability callouts and
restrained effects. Avoid a permanent event feed, six equal cards per side or a
full tactics table beside the fight.

The mockup's bottom inspection area is contextual. Selecting a group can reveal
its leader portrait, current condition, triggered tactic and concise ability
description in one compact strip. Full rule editing and all statistics belong
to preparation or an expanded inspector. Deselecting closes the strip; the
battlefield remains dominant when it is open. Terrain is visible in the scene,
with its precise modifier available on inspection rather than in a permanent
extra panel. Playback controls stay in a quiet lower edge.

The mockup supplies visual composition and tone, not mandatory characters,
fantasy classes, army sizes or balance numbers. Retain Kestrum's real troop
counts and current human roster. Derive aggregate displays from participating
units; define a weighted morale summary explicitly rather than copying the
example's percentages. The selected-unit stat labels must distinguish troop
strength from attack, morale and resistance.

## Starting point and integration boundaries

| Existing system | Required change |
| --- | --- |
| `Army.slots` already holds six optional formations in `src/state/military.rs` | Give those indices front/rear and column meaning; preserve formation IDs and actual troop counts. |
| Formations already retain capacity, service, veterancy and specialization | Add saved tactics and a selected battle leader; keep temporary combat state separate. |
| Several named people can share a formation; `Army.commander` is a separate appointment | Preserve every assignment. A hero remains embedded in troops and consumes no extra slot. |
| `src/engine/combat/arithmetic.rs` resolves up to eight simultaneous exchanges with rotating targets | Replace targeting and simultaneous exchanges with ordered activations, reactions, exposure and morale. This changes outcomes and requires new behavioral fixtures. |
| `src/engine/combat.rs` and its context/threat modules apply casualties, retreats, person consequences and site results | Keep the encounter transaction and adapt it to the new resolver result. |
| Movement, siege actions and threat clearing resolve encounters inside accepted commands | Introduce a serializable pending encounter for player preparation, with explicit command blocking and turn continuation. |
| `BattleReport` stores immutable participant snapshots, losses and consequences | Add versioned opening snapshots and typed combat events for visual playback. Keep older reports readable. |
| `src/ui/battle.rs` presents paginated Outcome, Forces, People and Factors text | Keep this as historical inspection; make a shared landscape with visible troops the primary encounter view. |
| AI already uses the same transactional commands and separate saved RNG streams | Run the same combat resolver and legal tactics for rivals; playback time must never affect simulation. |

Relevant dependencies include [P11–P15](implementation/combat-rules.md), movement,
siege relief, ordinary threats, person wounds, formation specialization,
participation evidence and save compatibility. P11's lack of tactical positioning
and P12's rotating targets are the old model. The proposed expansion replaces
those parts when its campaign integration ships. P13–P15's campaign consequences
remain the baseline unless a milestone explicitly records an amendment.

## Proposed first playable scope

Use the existing six human troop kinds: Warriors, Spearmen, Archers, Riders,
Medics and Siege Engines. Heavy infantry can use the existing Shield Guard
specialization rather than introducing an overlapping troop roster. Mages,
dragons, axes, assassins and new weapon families remain later content; their
examples in the notes do not require them in the prototype.

Each army supplies one three-column, two-row formation. Each unit represents
aggregate troops and has three to five ordered tactics, with safe defaults.
Combat has an opening, one activation per eligible unit per round, bounded
reactions, then morale and formation updates. The battlefield stages representative
soldier groups in a shared landscape, with troop counts, morale, actions and open
lanes integrated into the scene.
Named leaders later add tactical abilities grounded in their existing careers.

The initial implementation excludes freely editing tactics during execution,
individual soldier simulation, a separate tactical movement map, persistent
combat morale, and a commander intervention. A limited intervention remains a
later option because it would require authoritative execution between player
inputs, rather than playback of an already committed battle.

## Proposed formation and unit rules

The saved array retains its ordering. Player labels use slots 1–6; Rust uses 0–5.

```text
Logical deployment within EACH army

Front:  [1] [2] [3]
Rear:   [4] [5] [6]

Battlefield presentation

PLAYER GROUPS  ->  shared engagement space  <-  ENEMY GROUPS
```

The array defines protection and reach; it is not the execution screen layout.
Stage both forces in an oblique battlefield view, with the front rank nearer
the engagement space and the rear behind it. Preserve three distinguishable
lanes through depth and spacing. The enemy faces the player, while slot numbers
stay relative to its own army. Slot 1 protects slot 4, slot 2 protects slot 5,
and slot 3 protects slot
6. An empty, destroyed or routed front slot exposes its rear partner. Battle
position changes affect the runtime board; they do not silently rewrite the
player's saved deployment after the encounter.

**Proposed reach rules:** ordinary melee targets occupied enemy front slots,
preferring its own column before other columns. It can reach the rear once that
enemy army has no active front units. Ranged attacks can select either row.
Breakthrough can target a rear unit with an empty front partner in that same
enemy army. Reach is checked before a target preference is applied. A visually
charging Rider returns to its occupied slot unless the action explicitly changes
position. An Advance action moves a rear unit into its empty front partner and
uses its activation; it grants no free additional attack.

For multiple armies, **six slots remains a per-army limit**. Every participating
army keeps its own board, morale and identity, and all eligible units enter the
same round's initiative order. Do not discard formations or flatten twenty-four
units into six slots. Exposure belongs to the target's board; another friendly
army does not invisibly fill its gaps. Target selection may choose any legal
enemy board, with stable IDs resolving ties. Existing group movement and joint
relief must preserve all participating armies and casualty ownership.

| Unit input | Proposed meaning |
| --- | --- |
| Strength | Actual surviving headcount. Display count/capacity; strength percentage means count/capacity, avoiding a bias toward small cavalry headcounts. |
| Morale | Temporary 0–100 battle state. Starts at 100 in the prototype; casualties, allied routs and documented abilities change it through JSON rules. |
| Attack and defence | Extend existing attack/resistance data and use checked integer arithmetic. Keep leadership, veterancy, terrain and relevant siege factors; avoid applying them twice. |
| Initiative | New battle ordering statistic. It does not reuse or reset strategic movement allowance. |
| Range | A small reach category such as melee or ranged, evaluated by the formation rules. |
| Armour | Initially represented by resistance and tags needed by abilities. Add a separate numeric stat only when penetration mechanics justify it. |
| Leader | At most one selected, eligible attached person supplies the unit's active hero ability set. Other attached people retain their identities and existing army contributions. |

Morale reaches zero before a unit routes. A routed unit leaves its battle slot,
retains surviving troops, and cannot act, react or protect the rear. It does not
teleport to another strategic node individually. Resolve the army's eventual
location with the encounter's existing retreat rules. Track both current morale
and original troop counts so losses, routing and destruction remain distinct.

Keep wound and death checks at encounter conclusion initially. An existing
Wounded person cannot supply an ability, but an end-of-battle wound does not
retroactively cancel earlier actions. Defer mid-battle commander wounds and
their reaction triggers until their timing and RNG contract are implemented.

## Proposed tactics and action resolution

A tactic contains `trigger`, `action`, `condition`, `target_filter` and
`target_priority`. Offer named building blocks, bounded thresholds and one
condition per row initially. Players never write expressions. A preset creates
three rows; the editor supports at most five per activation list and five per
reaction list. Most units need only an activation list and one reaction default.

| Building block | Initial options |
| --- | --- |
| Trigger | Activation; incoming attack; end of round. Opening actions come from a separate once-per-battle capability. |
| Condition | Always; self strength below threshold; eligible ally below threshold; enemy cavalry present; reachable exposed rear present; first activation only. |
| Target filter | Enemy or ally as required by the action; front/rear; cavalry; exposed; eligible leader present. |
| Priority | Lowest/highest strength percentage; lowest resistance; own column first. Ties use board/slot/unit ID. |
| Action | Attack, Volley, Charge/Breakthrough, Guard, Brace, Wait, Advance and support abilities introduced by later milestones. |

On activation, evaluate rows from the top. Skip a row if its condition is false,
the action is unavailable, its resource is exhausted, or it has no legal target.
Execute the first valid row. If none qualifies, use a documented basic Attack
against a reachable front target; if none is reachable, Wait. An explicit valid
Wait consumes the activation and prevents that fallback. Record the chosen row,
target and concise reasons higher rows did not qualify for later inspection.

**Brace is a reaction**, not a normal activation that predicts a future attack.
When Charge selects a target, the defending unit evaluates incoming-attack
rules before damage. Brace may reduce charge damage and retaliate within a
single resolved effect. Start with one reaction budget per unit per round;
reaction damage cannot trigger another reaction. Guard has an explicit duration
and expiry. Each unit has one normal activation, and opening and once-per-battle
abilities have separate bounded charges. These limits prevent reaction loops,
permanent defence and repeated free rallies.

Resolve rounds in this order:

1. Snapshot all participants, deployment, eligible leaders, rules and resources.
   Opening capabilities fire once in stable initiative/ID order; ordinary units
   without such a capability receive no invented opening attack.
2. Build initiative from active units: descending initiative, then a documented
   alternating side priority by round, army ID, slot and unit ID. No frame-time
   or uncontrolled randomness enters ordering.
3. Before each activation, revalidate that actor, position and resources still
   permit it. Units destroyed or routed before their turn do not act. Targets
   and conditions use the current runtime state.
4. Resolve the selected action, permitted reaction, damage and resulting events.
   Death clears a slot immediately; scheduled morale and rout checks occur at
   round end. A later activation can exploit an already cleared slot.
5. Apply bounded end-of-round effects, morale changes and routs in stable order.
   Position changes must name the unit and destination, and validate occupancy.
6. Stop when one side has no combat-capable units, both sides are destroyed or
   routed, or the round limit is reached. Preserve mutual destruction; if both
   sides route, the attacker withdraws. At the limit, retain P12's normalized
   remaining-power comparison and stalemate withdrawal as the first default.

Use eight rounds as the provisional starting limit, keeping it data-driven.
Replace the old automatic rout at 40% remaining base power with the specified
unit morale system when it is integrated. Damage and morale numbers need
scenario tuning; initial values must be authored in JSON, explained in the rules
and pinned by behavioral tests rather than silently inferred from animation.

## Campaign transaction and playback contract

Use two separate notions of state: authoritative pending preparation in the
campaign, and cosmetic playback in `Game`. The resolver itself advances a plain
runtime snapshot and returns a `BattleResolution`; it never calls Macroquad.

```text
Accepted movement or combat order
    -> Pending encounter and locked continuation
    -> Player prepares or accepts current tactics
    -> Start Battle resolves and commits once
    -> Cosmetic battlefield playback
    -> Aftermath and report
    -> Resume the saved strategic continuation
```

For an order that creates a player encounter, movement already traversed and its
cost remain accepted; the encounter boundary blocks further orders. Store the
origin, site, context, participants, active actor and continuation needed to
resume. Repeated loading must not recharge that edge, advance an extra season,
consume another battle ID or replay NPC planning. Validate the temporary hostile
co-location as a pending encounter, rather than a normal uncontested site.

During pending preparation, allow changes only to the player's participating
deployment and tactics. Reject unrelated movement, transfers, recruitment,
diplomacy, End Turn and NPC advancement. Create a narrow preparation command
permission for a player attacked during an NPC turn; it grants no ordinary player
turn. Rival-only encounters resolve automatically from saved legal presets.
For repeat player encounters, a visible Use Current Plan action can skip editing
while still respecting the pending boundary.

Start Battle revalidates the pending encounter, resolves on a candidate campaign,
then applies casualties, exhaustion, retreat, person consequences, control,
siege/threat updates, evidence and the immutable receipt in one accepted
transaction. A failed command leaves the pending encounter intact. Duplicate
confirmation must be rejected after consumption. Keep the strategy paused while
the player watches; resume its continuation only after the visible Continue
action. If the player reloads after resolution, show the committed aftermath
without reapplying it.

Pause, step, speed changes, Skip to Result and replay control **presentation**.
They cannot change tactics or outcomes. The event stream holds sufficient
opening state, ordered events and deltas to reproduce positions, counts, morale
and chosen rules without consulting current formations or current balance data.
Reopening a report cannot grant experience, wounds, loot or control a second time.

Use a resolver version and rules/content revision in new receipts. Bound event
volume through round, action and reaction caps; retain detailed playback only
within a documented recent-report budget. Older or pruned reports retain their
summary and explicitly offer no detailed replay. Never fabricate events from
legacy exchange totals.

## Proposed architecture and persistence

| Responsibility | Suggested existing extension or new module |
| --- | --- |
| Typed ability, condition, priority and doctrine definitions | Extend `src/data/combat.rs`; introduce `src/data/battle_tactics.rs` and authored `assets/data/battle_tactics.json`. Load through toolkit JSON APIs and validate all IDs, references, limits and action/target compatibility. |
| Persistent formation configuration | Extend `src/state/military.rs` with a cohesive configuration type, defined in `src/state/tactics.rs`. Tactics follow `FormationId` through transfer, replenishment and army splitting. |
| Pending encounter and runtime snapshot | Extend `src/state/battle.rs` through named children such as `pending.rs`, `runtime.rs` and `events.rs`. Separate permanent deployment from temporary combat positions. |
| Pure simulation | Extend `src/engine/combat.rs` through `formation.rs`, `tactics.rs`, `rounds.rs`, `reactions.rs` and `resolution.rs`. Replace obsolete arithmetic when integrated; retain shared consequence handling. |
| Transactional editing and encounter commands | Extend `src/engine/actions/types.rs`, command validation, movement/siege/threat entry points and NPC continuation. Suggested intents: SetFormationTactics, SetBattleLeader, SetDeployment, PrepareEncounter and StartBattle. |
| Battlefield staging and contextual inspection | Add named modules under `src/ui/battle/` for terrain, troop groups, actor/target feedback and inspection, with playback orchestration under `src/game/battle/`. The renderer maps logical positions into a shared scene, reads projected state and returns `UiAction`; cosmetic animation never mutates campaign state. |
| Compatible saves and reports | Extend `src/state/campaign/compatibility.rs` through a battle-specific child and battle validation. Preserve the persistence catalogue and recovery workflow. |
| Behavioral regression coverage | Add focused `tests/battle_*.rs`; extend existing combat, siege, threat, progression and persistence regressions. Keep test helpers under `tests/support/`. |

Persist tactics, explicit leader selection and army deployment. Initialize older
formations with type-appropriate legal defaults and preserve their saved slot
indices. For an older save, its eligible army commander can initialize the battle
leader of their containing formation; other formations default to no selected
leader. Do not detach other people or manufacture recognition. Multiple attached
people continue to contribute under existing leadership rules, while only the
selected leader unlocks that formation's hero actions.

Validate leader eligibility at snapshot time. Transferring, retiring or losing
that person must leave the saved configuration understandable: show unavailable
ability rows and their reason, skip them safely, and retain a basic fallback.
Structural errors and unknown IDs remain save/data errors; ordinary loss of an
ability prerequisite is not corrupted data. Old reports still deserialize and
render their original text. Pending encounters require an explicit serializable
schema upgrade; missing legacy fields can receive defaults, malformed present
fields must not be repaired silently.

Follow the shared Rust/toolkit standards. Consider generic selection, reorder,
viewport or playback helpers as toolkit candidates before writing local copies;
the battle rules themselves belong in Kestrum. Keep named module files, target
200–400 lines and enforce the 800-total-line limit without exceptions. Keep
Macroquad pinned to `=0.4.16`, with no new dependency anticipated for the prototype.

## Battlefield composition and controls

Record the implementation screen briefs in the [project README](../README.md),
following [UI_STYLE.md](../UI_STYLE.md). Normal target is 1920 × 1080; minimum is
a 1280 × 720 landscape canvas. Use the supplied mockup's opposing forces and
landscape as the visual reference, with a lighter default inspection area.

| Phase | Main focus and primary action | Supporting and deferred information |
| --- | --- | --- |
| Preparation | Stage the player's groups on the battlefield; selecting one reveals its deployment and tactic choices. Start Battle advances play after showing known terrain and retreat risk. | A compact editor replaces the selected-unit strip when needed. Enemy detail respects existing fog and scouting; no hidden tactics or perfect outcome prediction. |
| Execution | Troop groups exchange attacks in the shared landscape, with the contested lane drawing attention. Pause/Resume, Step, Speed and Skip to Result remain visible at the edge. | Compact labels accompany groups. Selection opens one contextual strip; the active ability appears briefly over its actor. Full transcript and statistics open on demand. |
| Aftermath | The same battlefield shows survivors, empty positions and withdrawing troops. Continue returns to the campaign. | Add a concise result overlay and a few explanatory moments. Detailed factors, people and history remain in the existing report. |

Use roughly 6–12 representative figures for an infantry unit, with appropriate
smaller groups or equipment for Riders and Siege Engines. These are illustrative
figures, not simulated individuals. Prototype recognizable troop silhouettes,
weapons, banners, facing and an illustrated terrain backdrop before expanding
the editor. Include a short approach, action, impact and recovery sequence for
each supported action family. Even provisional art must read as troops fighting
in a place, rather than colored rectangles changing numbers. Reuse coherent
troop sprite sets, and commission missing assets once the staging is proven.
Leader portraits are optional context, not a prerequisite for understanding play.

At the minimum size, reframe the landscape and shorten labels before shrinking
soldier groups or sacrificing engagement space. The selected inspector can expand
when paused, but must not permanently divide the battle into panels. Select a
group and destination to change deployment; provide visible ordering controls
and optional keyboard shortcuts in the preparation editor. Selection, charge,
reaction and rout feedback use motion, posture, icons and labels as well as color.

Multi-army encounters remain one battle scene. Use reserve groups and banners to
establish additional participants, and shift the camera toward the acting and
targeted groups when needed. An overview can expose all participating formations;
do not substitute a pair of boards that visually suggests other armies stopped
fighting. Pause freezes camera focus for inspection. Disclose enemy information
only to the extent the encounter and existing knowledge rules permit; a prepared
replay payload must be filtered too.

## Delivery sequence and acceptance gates

Complete, validate and commit each useful milestone before beginning the next.
B01–B02 prove the combat and its visible battlefield before a substantial
management interface is built. B01–B04 form the first complete campaign
version; B05–B07 complete the intended system. No calendar estimate is asserted
before the new resolver and pending-command changes have been measured.

### Implementation status — 2026-09-30

| Milestone | Status | Verification record |
| --- | --- | --- |
| B01 Deterministic formation combat | Complete; headless resolver and authored rules | [Battle system verification](verification/battle-system.md#b01--deterministic-formation-combat) |
| B02 Battlefield presentation | Complete; deterministic fixture, event projection and playback view | [Battle system verification](verification/battle-system.md#b02--battlefield-presentation) |
| B03 Campaign encounter integration | Complete; deterministic field, threat and siege receipts integrate with campaign consequences | [Battle system verification](verification/battle-system.md#b03--campaign-encounter-integration) |
| B04 Deployment and tactics authoring | Complete; saved per-formation rules, contextual editor and slot swaps | [Battle system verification](verification/battle-system.md#b04--deployment-and-tactics-authoring) |
| B05 Leaders and support roles | Complete; saved leader selection, bounded class abilities, Medic stabilization and siege roles | [Battle system verification](verification/battle-system.md#b05--leaders-and-support-roles) |
| B06 Doctrines and rival preparation | Not started | — |
| B07 Balance and campaign readiness | Not started | — |

The full Kestrum test run currently reaches round 284 before an NPC attempts to
end its phase while a pending battle remains. The focused battle, campaign,
movement, recovery, siege, threat and source-size suites pass. This remaining
long-campaign integration case is tracked for B07; results are recorded in the
milestone evidence.

### B01 Prototype deterministic formation combat

Implement the runtime model, JSON rules, formation reach, ordered activations,
activation tactics, Brace reactions, bounded morale/routs and typed events through
a public headless resolver. Keep the live campaign on the current resolver until
B03 makes the replacement transactional. The snapshot already supports several
six-slot army boards and tagged threat combatants rather than fabricated IDs.

Acceptance cases: protected rear rejects ordinary melee; a cleared centre allows
Breakthrough; invalid higher tactic falls through correctly; Brace fires before
charge damage with no reaction chain; repeated identical input produces identical
events and outcomes, including a bounded all-Wait stalemate.

### B02 Prove the battlefield and visible action

Stage the B01 fixture in the real game's existing capture/runtime harness, using
the mockup's left/right opposing forces and shared terrain. Animate troop groups,
charge approaches, Brace posture, volleys, impact, casualties and withdrawal.
Build only compact identity/condition labels, quiet playback controls and the
selected-unit strip. Keep the tactics editor out of this milestone. Prove that
the centre breaking and cavalry exploiting it read through the scene itself.

Acceptance cases: the default battlefield occupies at least 75% of the viewport;
all twelve groups read as troops with clear allegiance and front/rear protection;
a broken centre, charge and Brace are understandable with the inspector closed;
pause/step/speed/skip preserve the same result; both supported sizes retain
readable groups, labels and engagement space without a permanent panel grid.

### B03 Integrate encounters and campaign consequences

Route field, threat and siege encounter contexts through the new resolver with
legal default tactics. Add pending encounters, save/load, preparation permissions
and exact turn continuation. Connect the B02 battlefield to committed receipts
and a concise Start Battle/Use Current Plan preparation step in that same scene.
Adapt ordinary threats as tagged aggregate combatants;
preserve their actual total headcount and payout. Their visual form need not
pretend to be an army with six full units.

Acceptance cases: player movement stops at contact and charges the edge once;
an NPC attack waits for player acceptance without advancing the NPC phase; pending
and resolved saves reload correctly; multi-army relief/assault applies each loss
and wall factor to the correct participants; repeated confirmation/report access,
reload and playback cannot duplicate casualties, person events, experience, site
damage or payout. Replays must remain usable after formations disappear or
balance data changes.

### B04 Add compact deployment and tactics authoring

Extend the selected-group strip during preparation with three-row defaults, a
five-row cap, trigger-specific rules, legal block selection and explicit fallback
display. The player configures one group at a time within the battlefield scene;
do not add a separate dashboard carrying every unit's rules and statistics.
Persist configuration with the formation. Add group selection, slot swaps and
visible ordering controls. Use the same command validation for all actors.

Acceptance cases: ordering changes which valid action runs; explicit Wait delays
a Rider until a later exposed-rear opportunity; selection and swapping preserve
all six formation IDs; transfer/replenishment/save-load retain unit tactics;
unavailable actions show a reason and fall back safely without granting a missing
ability or revealing hidden enemy information. Verify long names and full rule
lists without expanding the default execution interface.

### B05 Give leaders and support troops tactical roles

Add selected battle leaders and evidence/class-based capability grants. Prototype
Officer Rally, Cavalry Exploit Opening and an Infantry Hold the Line ability;
illustrative Elian and Mara are not mandatory authored characters. Rally restores
morale once per battle. Hold the Line moves into a legal vacant front slot with
a bounded defence benefit. Medics stabilize morale or reduce a documented future
morale penalty; they do not resurrect lost headcount or bypass seasonal treatment.
Siege Engines receive a bounded opening bombardment and explicit close-combat
weakness while retaining the existing assault-wall interaction.

Acceptance cases: a formation with no named leader remains legal; a leader unlocks
an action rather than taking a seventh slot; other attached people retain their
assignments and leadership contributions; a transferred/wounded/retired leader
cannot supply an unavailable action; Rally, Hold the Line, Medic support and
bombardment respect charges, timing, casualties and existing progression evidence.

### B06 Add doctrines and rival preparation

Create Defensive Line, Ranged Support and Breakthrough presets using the same
legal blocks as custom tactics. Apply a doctrine as a snapshot of defaults with
explicit unit overrides, rather than competing layers evaluated during combat.
Store personal templates within the campaign first; cross-campaign libraries are
later scope. Mark customized hero rows as explicit unit overrides so applying a
new doctrine preserves them.
Rivals select legal doctrine/deployment from information their faction knows,
without special combat bonuses or reading the player's hidden rule list.

Acceptance cases: a doctrine creates legal rules for all six existing troop
types; an override survives later doctrine application; saved templates roundtrip
without retaining another formation's PersonId; the same rival knowledge/input
produces the same preparation; a rival can exploit a gap and be countered by
prepared spears under the same action/resource rules as the player.

### B07 Tune and verify campaign readiness

Tune matchup modifiers, casualty pacing, morale and initiative in JSON using
small representative scenarios, then regression campaigns. Compare equal armies,
soft counters, exposed rear, leaderless forces, veteran wounded armies and siege
relief. Document the expected strategic consequences when new combat intentionally
changes an old formula test. Bound replay memory and check native/WASM behavior.

Acceptance cases: preparation choices change outcomes predictably; no troop type
dominates every fixture; multi-army and threat headcounts are conserved;
formation loss, retreats, person survival, service and succession remain valid;
a complete witnessed encounter saves, reloads, replays and resumes a campaign
without duplicate effects or stalled turns. Add further focused regressions for
any defects discovered, without weakening older useful coverage.

## Later tactical depth

After B01–B07 are playable, extend the existing action and event model with
formation swaps, forced displacement, flanking, Intercept/Cover and disruption.
Each needs explicit reach, occupancy, duration and reaction-budget rules before
it becomes an editor option. Add support relationships when existing bond
evidence can unlock a specific cooperation effect, rather than granting an
unexplained universal multiplier. More conditions and target tags can follow
observed player needs; avoid an unrestricted expression language.

Advanced troop types, magic, expanded portrait sets and a cross-campaign template
library are separate content or presentation milestones. Battlefield art and
clear action animation are part of B02, not deferred until the management screens
are complete. One commander order
per battle can be evaluated later, but must resolve through authoritative battle
state between activations, with saved charges and resumable input boundaries.
It cannot be layered onto cosmetic replay as though the player changed an
already committed outcome.

## First playable demonstration

Use a deterministic fixture with Spearmen, Warriors and Warriors in the front,
and Archers, Riders and Medics in the rear. Give the enemy a front unit tuned to
fall before its protected ranged unit. The player's Rider rules are Breakthrough
if a reachable rear unit is exposed, then Wait. Configure the Spearmen to Brace
against incoming cavalry. The Archer targets an eligible rear enemy.

The demonstration passes when a player can change deployment or tactic order,
start the encounter, see the centre collapse, see the Rider exploit that specific
lane, inspect the chosen rule, and return to persistent losses and retreats on
the strategic map. Also show a deliberately poor plan failing visibly. Do not
force an exact round-two kill into the production balance to reproduce the
illustrative story from the notes.

## Verification and known limitations

Each major feature targets five meaningful behavioral cases in `tests/`, plus
existing regressions for affected campaign systems. Use the actual checkout and
shared Cargo launcher. Required implementation checks are:

```powershell
cargo fmt -p kestrum -- --check
..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test -p kestrum --all-features
..\rust_management\cargo.ps1 test -p kestrum --test code_standards
.\publish.ps1
```

Run `publish.ps1` without parameters after meaningful gameplay changes. UI
milestones also use the existing shared hidden-window capture wrapper and stable
`docs/verification/ui_battle_*.png` filenames. Replace equivalent states in place;
wait for captures and verify the game process exits. Review both normal and
minimum native/browser sizes, first preparation, dense twelve-unit combat,
multi-army selection, a formation gap, reactions, rout, wounded leaders and
aftermath. Capture supported states only. Verify camera framing, group selection,
ordering and playback controls; report what was actually tested. Also review the
default scene with the inspector closed: an uninformed observer should be able
to identify who attacked, who defended and which lane opened from the action.

The latest [formation membership verification](verification/formation-members.md)
records an existing 1280 × 720 WebGL scaling/picking problem. This remains a
release limit until corrected and exercised. The
recent commit history also records existing long-campaign failures; new combat
must report their actual status rather than claiming a clean inherited baseline.
Do not copy the project, change workspace membership or invent placeholder crates
to bypass checks. Keep all project work, validate and commit on `master`, and
finish with an empty `git status --short` or a specific blocker.

This planning change is documentation only. Its verification covers source
preservation, local references and diff hygiene; runtime, UI, balance and
publication acceptance belong to the implementation milestones above.
