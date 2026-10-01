# 11 Simulation and data

[Documentation index](README.md) · [Code authority](../CODE_STANDARDS.md) ·
[Delivery](12-delivery-and-validation.md)

## Status

The strategic simulation, content loading, catalogue persistence, battle
transactions and generational systems are implemented. The
[map plan](map-playability-plan.md) extends their presentation, geographic content
and compatibility. Rust, Macroquad `=0.4.16` and macroquad-toolkit remain the stack.

Use current source for exact contracts. Earlier
[implementation contracts](implementation/contracts.md) describe the completed
foundation and must be read with later movement and battle behavior.

## Ownership and module responsibilities

| Responsibility | Current owner |
| --- | --- |
| Typed definitions and semantic validation | `src/data.rs` and `src/data/` |
| Campaign entities, invariants and compatibility | `src/state.rs` and `src/state/`; `StrategicCampaign` owns simulation state |
| Application input, transitions, storage and presentation coordination | `Game` in `src/game.rs` and `src/game/` |
| Commands, calculations, phases and observer projection | `src/engine.rs` and `src/engine/` |
| Picking, map scopes, camera and exploration geometry | `src/navigation.rs` and `src/navigation/` |
| Rendering and explicit `UiAction` intents | `src/ui.rs` and `src/ui/` |
| Behavioral tests | Crate-local `tests/` using the library's public API |

Keep UI drawing free of simulation mutation. Rendering consumes a visible
projection; it does not independently decide supply, costs, ownership or outcomes.
Use named modules and keep every Rust file within 800 total physical lines.

## Entity contracts

Stable faction, marker, site, route, army, formation and person identities tie
orders and history together. Each army occupies one physical site; world markers
can aggregate regions but do not become additional army locations. Region claims
and local site control remain distinct. Six formation slots and one named member
per formation are independent from commander appointment.

Construction, siege, battle, movement-plan, succession and history records retain
their real references and receipts. Renaming, capture or decline does not create
a new place identity. Destroyed formations are removed as active entities;
witnessed world/person records can retain their consequences. Narrative pruning
must preserve gameplay evidence and live invariants.

See the current owning [system chapters](README.md#current-documents) and
`src/state/validation/` for specific invariants.

## Action handling

The UI returns intents; the application enforces its screen/phase guards and
routes accepted commands to the engine. Commands validate actor, ownership,
references, access, resources, movement and prerequisites. Rejection must be
atomic, with a useful reason. Previews are read-only and execution revalidates.

Destination taps execute movement immediately. Saved remainders continue on
movement refresh, rechecking access and encounters. Transfers preserve spent
allowance and obey current co-location/phase rules.

Battles can pause an encounter at preparation. Start Battle commits one
deterministic receipt; playback cannot modify it and continuation cannot apply
the result twice. Test public application guards for flows crossing overlays,
as direct engine tests alone previously missed a blocked Start Battle action.

## Round-end resolution

One full round advances one season; four seasons make a year. The current
`src/engine/round.rs` boundary reconciles siege participation, snapshots supply
and development conditions, settles income/upkeep, progresses construction and
sieges, applies recovery/healing and development, advances the calendar, resolves
lifecycle/succession and reconciles relevant records. It then refreshes the
player phase and movement. Surrounding command coordination handles progression,
knowledge/history, continuing orders and checkpoint eligibility.

Follow the actual command pipeline when adding seasonal feedback. Capture
receipts from accepted changes; do not simulate a second boundary for the HUD,
infer last season from current totals, or resolve periodic effects per faction
action. Save retries reuse the resolved snapshot.

## Cross-system contracts

Movement and control affect encounters, supply, discovery and claims. Combat
feeds lasting losses, wounds, service and history. Supply constrains recovery;
development and facilities affect income, recruitment and career opportunities.
Aging and succession reconcile duties while preserving genuine continuity.

The planned map projection exposes those connections using owned/public/observed
facts. Cache invalidation must include changes from NPC actions, seasonal
boundaries, reload, construction, control and queued travel. Rendering and
animation consume no gameplay randomness.

## JSON content and validation

Load JSON through toolkit APIs. Kestrum owns typed schemas and semantic checks
for its campaign, world, economy, troops, combat, development, progression,
households, legacy and presentation data under `assets/data/`.

Validate unique/resolving IDs, route endpoints and reciprocal entrances,
reachable starts, feasible anchors, valid costs/durations and troop capacities,
progression prerequisites and asset paths. Invalid content must produce a
source-labelled error. Keep generic loading, pointer transforms and persistence
in the toolkit; game-specific geography/control belongs here.

Current production content has 80 major markers, eight ten-site regions,
152 physical sites and 191 routes. These are the current counts, not a mandate
to keep the repeated regional chains. M03 retains the site scope while changing
connections and local meaning.

## Determinism and bounded simulation

Persist the campaign's seed and state-owned RNG streams. Use stable application
order. Rendering, labels, terrain art and tutorials must not affect random
outcomes. Keep graph searches and combat bounded, with explicit termination.

Ordinary population stays abstract; only relevant people receive detailed
records. Keep bounded narrative history and compact mechanical evidence
separate. Unknown enemy data cannot become visible because a summary or alert
reads a live entity outside the observer boundary.

## Persistence

The toolkit catalogue supports named saves, round checkpoints, writer ownership,
recoverable publication and explicit overwrite/deletion. Current campaigns
preserve phase, RNG, pending encounters, orders and receipts. Older supported
schemas use explicit compatible decoding; malformed current fields remain
errors. Legacy atlas-only saves remain a separate read-only path.

A failed load must preserve the active session. A retry must not repeat economy,
movement, battle or lifecycle effects. New presentation acknowledgement state
needs additive defaults without invented historical notifications.

## Geography revisions

Current atlas revisions 1–3 support earlier geometry and metadata differences.
Saved-world validation still compares topology to authored production content:
route endpoints/costs, site facts, entrances and anchors matter. Changing a
revision number alone cannot preserve a different graph.

M03 must introduce revision-selected authored topology validation before new
regional connections ship. Existing campaigns keep their saved physical graph;
new campaigns use the new topology. Preserve history, pending movement paths,
road work, siege and battle-origin/retreat references, not only army locations.
Use strict recognized baselines and reject mixed/unknown revisions.

The compatibility requirements and validation scope are in
[the map plan](map-playability-plan.md#save-and-topology-compatibility).
Do not silently remap old campaigns or relax validation to accept arbitrary data.

## Testing

Test meaningful behavior through the appropriate public seam, including
application guards, projection secrecy, atomic rejection, deterministic
continuation and save roundtrips. Keep tests in `tests/` and preserve useful
regressions. Follow [delivery](12-delivery-and-validation.md) for actual-checkout
validation and the shared source-size gate.
