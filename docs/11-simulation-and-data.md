# 11 — Simulation and data

[Documentation index](README.md) · [Code authority](../CODE_STANDARDS.md) · [Delivery](12-delivery-and-validation.md)

## Status

This chapter is a **proposed implementation design**. It connects the source gameplay systems without claiming they exist in the template. Select concrete types and files as features are implemented; do not create unused scaffolding for every future system.

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

The template currently has legacy tests under `src/`. Migrate those as a separate implementation change before expanding coverage; this documentation milestone preserves them.

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
| Character | Birth date, service dates, roles/classes, evidence, recognition, lifecycle | Chronology valid; at most one incompatible active assignment |
| Household/dependent | Sparse relationships, approximate or exact dates, place ties | No contradictory ancestry; deepen simulation when relevant |
| Mentorship | Teacher, learner, discipline, dates, opportunity | Valid participants and feasible active relationship |
| Equipment | Item identity, current holder, past custody and deeds | A historical item cannot have two current owners |
| Construction | Site, owner, order, duration/progress, costs, interruption state | Progress once per round; cancellation policy explicit |
| Siege | Node, sides, participants, start date, progress | End or revalidate when forces or diplomacy change |
| Event | Date, participants, place, kind, facts, visibility, causes | Immutable outcome facts; unique application |
| Knowledge | Observer, subject, observed facts, date/location, confidence | Player view cannot read hidden live enemy state |

These are contracts rather than final serialization layouts. Split mutable campaign instances from reusable troop, class, trait, terrain, facility, and event definitions.

## Action handling

Potential commands include move army, recruit formation, transfer character, reorganize army, establish outpost, set development focus, improve a road, begin/maintain/assault a siege, attempt retreat or escape, propose peace, assign mentor, change role, and end turn. Only expose commands implemented for the current milestone.

For every command:

1. Validate phase, actor ownership, IDs, access, movement, costs, and prerequisites.
2. Return a clear reason if rejected, without partially spending resources or moving entities.
3. Compute consequences in a stable order.
4. Apply state changes once and emit relevant domain events.
5. Derive visible feedback through the current faction's knowledge.

For commands that depend on a route preview or selected target, revalidate at execution. UI previews never become a second authoritative rule implementation.

## Proposed seasonal resolution

During a faction's action phase, orders and resulting battles resolve immediately. After every active faction has finished, the shared round resolves once:

1. Reconcile active armies, siege participation, territorial control, and current supply connectivity.
2. Settle income and upkeep under the chosen economic policy.
3. Progress eligible construction and existing sieges once.
4. Apply configured recovery and permitted ongoing siege/local effects.
5. Resolve development pressure, decline, and population movement.
6. Advance calendar and lifecycle checks using one explicit date boundary.
7. Evaluate progression, recognition, mentorship, and supported succession from eligible accumulated events.
8. Reconcile elimination and victory, update faction knowledge, persist a stable checkpoint, and present the next turn summary.

**Open ordering details:** whether completed infrastructure helps recovery immediately or next season; whether an order placed late in a round receives a full progress step; the exact date assigned to boundary events; upkeep deficits; and who can act after a new season begins. Adopt a written rule before tests pin these behaviors. Never advance siege or age once per faction action.

If faction elimination occurs during an action phase, remove future turns safely without skipping the next valid faction or repeating a completed one. A round records which factions have already acted.

## Cross-system contracts

| Producer | Result consumed elsewhere |
| --- | --- |
| Movement/control | Updated positions and routes determine encounters, supply, anchor control, and knowledge |
| Battle resolver | Casualties, retreat, wounds, defended/captured sites, and enemy types feed progression and history |
| Supply | Recovery eligibility affects persistence and viable offensive length |
| Development | Facilities and local conditions affect recruitment, classes, income, and population |
| Character growth | Eligible classes, leaders, and mentors alter army capability and future opportunities |
| Lifecycle | Retirement/death releases duties and enables succession without erasing history |
| Local threat resolution | Clearance/rewards create settlement and specialist opportunities |
| History | Contextual summaries and legacy links explain current state without modifying it |

The central integration case is Rosemarch: a route and defense create a leadership experience, medical service creates a specialization opportunity, and an escaped enemy retains identity for a later encounter.

## JSON content and validation

Keep balance values, content, configuration, and player-facing text in JSON under `assets/`. Load through toolkit embedded or typed runtime loading APIs. Kestrum owns typed schemas and semantic checks. Generic JSON parsing, loading, platform branching, source-labelled errors, and fallbacks belong to the toolkit; do not build duplicate local loaders or parse game-data files directly with `serde_json::from_str`.

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

Keep ordinary population abstract. Retain enough history in surviving formations to ground emerging people without simulating every soldier. When a formation is destroyed, its own history ends under D05; world battle records may remain. Cache graph calculations only with correct invalidation. Event retention must respect that distinction and preserve required evidence while preventing UI work from scanning the entire campaign every frame. Storage policy and performance budgets for large worlds and hundred-year campaigns remain open under O23.

## Persistence

Reuse toolkit versioned save/load and migration support. A save should include all authoritative campaign state, active round position, RNG state, persistent IDs, knowledge, event history, and ongoing orders or siege states that the supported save boundary permits. Settings remain separate.

**Proposal:** start with saving at stable action or round boundaries. If mid-resolution saving is unsupported, explain when saving becomes available and never create a half-applied encounter. Preserve long-term chronology and identity through migrations. Load into a candidate state, validate it, then replace the current campaign only after success.

**Open:** save-slot count, autosave cadence, migration policy, old content compatibility, and supported interruption points. Template demo saves must not be mistaken for Kestrum campaign saves; game identity and schema need a deliberate onboarding change.

## Errors and recovery

Missing entities return clear errors or optional results, rather than panicking during ordinary play. Failed orders leave state intact. A failed save leaves the current session usable; invalid load data does not overwrite it. If an encounter cannot resolve, surface the failure and retain a recoverable boundary rather than pretending an outcome occurred.

## Platform and verification contracts

Target Windows native and browser WebGL/WASM. The template already demonstrates toolkit data loading, persistence, pointer input, camera transformation, scrolling, notifications, and capture. Adapt those integrations; do not claim its grid and demo actions are the Kestrum simulation.

Use the shared build pool from this actual checkout, preserve the exact Macroquad pin, and obey the central workspace policy. Test public behavior from `tests/`, strongly targeting five cases per major feature. Publishing after meaningful game changes uses the unparameterized project publisher. Source size, format, Clippy, tests, actual UI review, and publishing cover different risks and should be reported separately.
