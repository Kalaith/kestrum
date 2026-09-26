# 11 — Simulation and data

[Documentation index](README.md) · [Code authority](../CODE_STANDARDS.md) · [Delivery](12-delivery-and-validation.md)

## Status

This chapter combines **confirmed behavior** with **proposed implementation design**, labelled where relevant. The title/empty-atlas shell now has basic chronology, settings and a single save; the strategic simulation below remains future work. The [implementation contracts](implementation/contracts.md) specify the current handoff against the actual code. Introduce types and files as their packages consume them; do not create unused scaffolding for every future system.

The required stack is Rust, Macroquad pinned exactly to `=0.4.16`, and macroquad-toolkit. Prefer existing toolkit capabilities for generic loading, input, layout, persistence, notifications, and capture. A Kestrum node graph and its semantic rules are game-specific; the template's grid demo does not require a square-tile game.

## Ownership and module responsibilities

| Responsibility | Proposed ownership |
| --- | --- |
| Content definitions | Immutable validated data loaded through the toolkit |
| Campaign state | `GameState`: calendar, world, factions, armies, people, knowledge, events, seeded randomness |
| Application coordination | `Game`: apply explicit actions, manage transitions, invoke services, coordinate save/load and presentation |
| Rules and calculations | Cohesive engine services taking state and returning validated results |
| UI | Read visible state and return `UiAction` intents; no simulation mutation while drawing |
| Settings | Separate persistent preferences, not mixed with campaign outcomes |
| Tests | Crate-local `tests/`, using intentional public seams |

Use named module files such as `data.rs`, `state.rs`, `engine.rs`, and `ui.rs`. Introduce focused child files only as needed. Every Rust file remains below 800 total physical lines, including tests and comments, with no exceptions. Target 200–400 and plan a cohesive split around 600.

The current Kestrum tests already live in `tests/` and use `src/lib.rs`. The
legacy test-placement warning applied to the original starter, not this checkout.

## Entity contracts

| Entity | Essential proposed facts | Important references and invariants |
| --- | --- | --- |
| Campaign | Seed, schema/content version, date, active faction, phase, RNG state | One calendar and one active action phase |
| Faction | Name, emblem, resources, headquarters/capital roles, diplomacy, knowledge | Valid faction IDs; inactive factions do not take turns |
| Region | World node, internal graph, entrances, anchor definition, political owner | Internal nodes and boundary endpoints resolve |
| Node | ID, geography, development layers, controller, damage, history links | Stable identity through renaming, decline, and capture |
| Route | Endpoints, traversal rules, terrain/road condition, boundary mapping if needed | Connectivity consistent across movement and supply |
| Army | Faction, location, movement state, up to six formation slots, assignments | One physical location and unique formation membership |
| Formation | Type, headcount/capacity, experience, specialization, origin, attached people | Zero headcount destroys the formation and its history; replacement gets a new ID; world battle facts may remain |
| Character | Birth date, service dates, roles/classes, compact participation evidence, recognition, lifecycle | Chronology valid; at most one incompatible active assignment; narrative pruning cannot change eligibility |
| Household/dependent | Sparse relationships, approximate or exact dates, place ties | No contradictory ancestry; deepen simulation when relevant |
| Mentorship | Teacher, learner, discipline, dates, opportunity | Valid participants and feasible active relationship |
| Equipment | Item identity, current holder, past custody and deeds | A historical item cannot have two current owners |
| Construction | Site, owner, order, duration/progress, costs, interruption state | Progress once per round; cancellation policy explicit |
| Siege | Node, sides, participants, start date, progress | End or revalidate when forces or diplomacy change |
| Event | Date, participants, place, kind, facts, visibility, causes | Facts immutable while retained; bounded narrative retention; application safety lives in authoritative state |
| Knowledge | Observer, subject, observed facts, date/location, confidence | Player view cannot read hidden live enemy state |

These are contracts rather than final serialization layouts. Split mutable campaign instances from reusable troop, class, trait, terrain, facility, and event definitions.

## Action handling

Potential commands include move army, recruit formation, transfer character, reorganize army, establish outpost, set development focus, improve a road, begin/maintain/assault a siege, attempt retreat or escape, propose peace, assign mentor, change role, and end turn. Only expose commands implemented for the current milestone.

For every command:

1. Validate actor ownership, IDs, and command-specific phase, access, movement, cost, and prerequisite rules. O09 transfers require shared-node presence and can occur between atomic actions without an active-faction restriction.
2. Return a clear reason if rejected, without partially spending resources or moving entities.
3. Compute consequences in a stable order.
4. Apply state changes once and emit relevant domain events.
5. Derive visible feedback through the current faction's knowledge.

For commands that depend on a route preview or selected target, revalidate at execution. UI previews never become a second authoritative rule implementation.

## Round-end resolution

**Confirmed boundary (O04):** periodic effects resolve at the end of a full round after every active faction has finished. Orders and resulting battles resolve as immediate action consequences. The following internal order is an implementation proposal:

1. Reconcile active armies, siege participation, territorial control, and current supply connectivity.
2. Settle income and upkeep under the chosen economic policy.
3. Progress eligible construction and existing sieges once.
4. Apply configured recovery and permitted ongoing siege/local effects.
5. Resolve development pressure, decline, and population movement.
6. Advance calendar and lifecycle checks using one explicit date boundary.
7. Evaluate progression, recognition, mentorship, and supported succession from eligible accumulated events.
8. Reconcile elimination and victory, update faction knowledge, prune eligible narrative history, automatically save the completed round, and present the next turn summary.

**Provisional ordering details now specified:** [P02](implementation/strategic-rules.md#p02--faction-phases-and-round-order) freezes the internal order, date labels, first progress step and next-round infrastructure eligibility for implementation. O11 supplies the existing income-before-upkeep and shortfall rules. These are reviewable defaults, while the full-round boundary and automatic save are confirmed. Never advance siege or age once per faction action.

If faction elimination occurs during an action phase, remove future turns safely without skipping the next valid faction or repeating a completed one. A round records which factions have already acted.

## Cross-system contracts

| Producer | Result consumed elsewhere |
| --- | --- |
| Movement/control | Updated positions and routes determine encounters, supply, anchor control, and knowledge |
| Battle resolver | Casualties, retreat, wounds, defended/captured sites, and enemy types feed progression and history |
| Supply | Recovery eligibility affects persistence and viable offensive length |
| Development | Facilities and local conditions affect recruitment, classes, income, and population |
| Character growth | Eligible classes, leaders, and mentors alter army capability and future opportunities |
| Lifecycle | Retirement/death releases duties and enables succession; narrative history follows the retention policy |
| Local threat resolution | Clearance/rewards create settlement and specialist opportunities |
| History | Contextual summaries and legacy links explain current state without modifying it |

The central integration case is Rosemarch: a route and defense create a leadership experience, medical service creates a specialization opportunity, and an escaped enemy retains identity for a later encounter.

## JSON content and validation

Keep balance values, content, configuration, and player-facing text in JSON under `assets/`. Load through toolkit embedded or typed runtime loading APIs. Kestrum owns typed schemas and semantic checks. Generic JSON parsing, loading, platform branching, source-labelled errors, and fallbacks belong to the toolkit; do not build duplicate local loaders or parse game-data files directly with `serde_json::from_str`.

O11 delegates initial economy choices to provisional JSON balance data. [economy.json](../assets/data/economy.json) now records those defaults; future economy implementation must consume it through the toolkit. It is not wired into the current template. Validate nonnegative resources/costs, positive capacities, percentages in range, known troop IDs, and supported policy fields. Keep its format version distinct from a campaign save's schema version.

Proposed content groups include campaign generation, geography/routes, troop definitions, classes, traits, experience tags, facilities, development, local threats, event text, and balance settings. Split by cohesive use rather than creating one huge configuration file.

Validation should check:

- Unique IDs and resolving references.
- Valid route endpoints, boundary mappings, reachable starts, and viable regional entrances.
- Anchor expressions with at least one feasible way to gain control.
- Troop capacities, costs, durations, and development thresholds in valid ranges.
- Valid class prerequisite expressions, experience tags, and facility references.
- No required starting ability gated behind an unreachable prerequisite cycle.
- Timeline and lifecycle constraints in authored scenarios.
- Exact runtime asset registry paths for externally loaded assets; embedded JSON does not belong in that registry.

Report a source-labelled content error rather than silently starting a broken campaign. User-facing errors should explain recovery; implementation detail can remain in development diagnostics.

## Determinism and bounded simulation

Persist the campaign seed and state-owned RNG. Sort iteration and event application where order matters. Isolate random draws for generation, combat, emergence, and development so cosmetic animations cannot change outcomes. The same supported save state and same action sequence should produce the same results.

Use integer seasonal dates and deliberate numeric rules. Cross-platform replay precision must be measured if floating point enters combat. Bound combat resolution and graph searches; a stalemate should terminate with an explicit outcome.

Keep ordinary population abstract. Preserve compact formation/character experience and participation facts for gameplay, with bounded narrative detail for biographies. A destroyed formation loses its own history under D05. O23 permits world memory to fade; use the [retention budgets](09-history-and-content.md#bounded-history-and-forgetting) and keep current state, necessary relationships, and applied outcomes independent from expiring stories. Cache graph calculations only with correct invalidation. Profile the 80-node world and long campaigns; tune retention budgets without making every historical event permanent.

## Persistence

**Confirmed decision (O22):** automatically save at the end of every full round. The player can also manually save during their turn. There is no game-imposed save-slot limit; actual storage capacity still applies.

**Implementation baseline:** each completed round creates a distinct automatic checkpoint, and each manual save creates a new named slot unless the player explicitly chooses an existing slot to overwrite. Do not rotate or delete slots automatically. Provide save deletion and clear storage-full errors. Manual saving uses a stable boundary during the player's turn; if requested while an action is resolving, finish that atomic action first. An autosave occurs after all periodic effects and history pruning, before the next round's orders are accepted. Save failure leaves the live campaign usable and reports that the round was not persisted.

Use available toolkit persistence support. Store authoritative state, active turn/round position, RNG state, stable IDs, knowledge, compact gameplay evidence, retained narrative, and ongoing orders/sieges. Settings remain separate. Load into a candidate state, validate, then replace the live campaign only after success. Loading a checkpoint cannot rerun its completed periodic effects or grant rewards again.

**Technical handoff:** [C06](implementation/contracts.md#c06--save-catalogue-and-compatibility) and K03 specify the indexed catalogue, interrupted-write recovery and shell-v1 compatibility policy. Inspection found the toolkit's WASM `get_save_slots` only checks five fixed names; shared indexed storage is a dependency, not an existing unlimited-list feature. Saves need explicit schema/content versions; unsupported versions must fail clearly rather than silently reinterpret a campaign. Template demo saves must not be treated as Kestrum saves. Cadence, slot policy and manual-save availability remain confirmed.

## Errors and recovery

Missing entities return clear errors or optional results, rather than panicking during ordinary play. Failed orders leave state intact. A failed save leaves the current session usable; invalid load data does not overwrite it. If an encounter cannot resolve, surface the failure and retain a recoverable boundary rather than pretending an outcome occurred.

## Platform and verification contracts

Target Windows native and browser WebGL/WASM. The template already demonstrates toolkit data loading, persistence, pointer input, camera transformation, scrolling, notifications, and capture. Adapt those integrations; do not claim its grid and demo actions are the Kestrum simulation.

Use the shared build pool from this actual checkout, preserve the exact Macroquad pin, and obey the central workspace policy. Test public behavior from `tests/`, strongly targeting five cases per major feature. Publishing after meaningful game changes uses the unparameterized project publisher. Source size, format, Clippy, tests, actual UI review, and publishing cover different risks and should be reported separately.
