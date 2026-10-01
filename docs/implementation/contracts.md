# Implementation contracts

[Plan](../implementation-plan.md) · [Active map playability plan](../map-playability-plan.md) · [Work packages](work-packages.md)

These contracts describe the delivered campaign's engineering boundaries, with
recommended module shapes from the original K01–K18 plan. **P** records delegated
defaults, not outstanding approval. Status reconciled 2026-10-01: the
[active map plan](../map-playability-plan.md) owns next work. Preserve these
ownership, observer, determinism and persistence boundaries during that work.
Concrete existing modules take precedence over illustrative filenames below.

The user's 2026-10-01 correction makes 1920×1080 the sole design and acceptance
spec. The [current viewport contract](../10-interface-and-accessibility.md#viewport-specification)
supersedes the earlier 1280×720 logical/minimum guidance retained below.

## C01 — Ownership and module boundaries

Keep `GameState` as the owner of the optional `Campaign`. `Game` owns runtime
assets, input, UI navigation, storage coordination, and capture mode. `Campaign`
owns authoritative simulation state. A screen, camera, selection, or open report
does not own armies, people, resources, or time.

| Module family | Responsibility and public seam |
| --- | --- |
| `data.rs`, `data/economy.rs`, `data/world.rs`, `data/rules.rs` | Immutable typed definitions and semantic validation through toolkit loaders |
| `state.rs`, `state/campaign.rs`, `state/world.rs`, `state/military.rs`, `state/people.rs` | Serializable authoritative entities and invariants |
| `engine.rs`, `engine/actions.rs`, `engine/round.rs` | Command validation/application and turn sequencing |
| `engine/movement.rs`, `engine/supply.rs`, `engine/economy.rs`, `engine/construction.rs` | Strategic rules |
| `engine/combat.rs`, `engine/retreat.rs`, `engine/siege.rs`, `engine/threats.rs` | Bounded encounter calculations and lasting consequences |
| `engine/progression.rs`, `engine/development.rs`, `engine/lifecycle.rs`, `engine/succession.rs` | People and place evolution |
| `engine/knowledge.rs`, `engine/history.rs`, `engine/ai.rs` | Observation projection, retained narratives, legal AI commands |
| `state/persistence.rs` | Kestrum schema validation/migration and save catalogue metadata; toolkit owns generic storage |
| Existing `ui.rs` and focused `ui/*.rs` | Read projected state, return intents; no simulation changes during drawing |
| `tests/*.rs`, `tests/support.rs` when needed | Behavioral tests and small reusable fixtures through intentional library APIs |

Expose testable services through `src/lib.rs`. Keep every Rust file at or below
800 physical lines and functions within the shared limits. Split cohesive domains
near 600 lines; do not use `mod.rs` or compress formatting to meet the gate.

## C02 — Identity, graph, and entity shape

Use typed stable IDs around monotonic integer values, with separate counters per
entity family. Never reuse deleted IDs, infer identity from names, or serialize
vector positions as references. Persist counters and validate them above extant
IDs. Cosmetic map positions use floats; simulation amounts and dates use integers.

| Entity | Minimum authoritative facts introduced by its owning package |
| --- | --- |
| Campaign | Schema/content version, campaign ID, seed, RNG streams, next IDs, completed rounds, active faction, acted set, phase, entities, pending evidence |
| Faction | Name/emblem, active/eliminated/vassal status, resources, deficit flag, HQ/capital site IDs, diplomatic relations, knowledge |
| Major location / region | World marker, optional internal sites and gate mapping, political owner and contested flag, anchor expression |
| Site | Physical ID, major-location parent, geography/tags, controller, habitation, military layer, damage, occupation, focus, facilities, population, development counters |
| Route | Fixed endpoint site IDs, major-map relation if crossing a boundary, terrain cost, road condition |
| Army | Faction, physical site, six optional formation IDs, optional command appointment |
| Formation | Type, headcount, cumulative service/evidence, veterancy, specialization, creation/service dates, movement spent, attached person IDs |
| Person | Birth/service dates, status, assignment, class/traits, disposition, participation counters, recognition, relationship IDs, movement spent |
| Order / siege | Owner, site/route, participants or builder, creation date, progress, status and blocking reason |
| Event / observation | Stable ID/date, supported facts, participants/place labels, visibility; observed enemy facts are snapshots |
| Household / mentorship / item | Stable participants/holder, role or discipline, dates, current status, compact necessary relationships |

An army has one physical `SiteId`. A regional world marker is not an occupiable
site. World and regional views render the same army. A person has one primary
assignment: formation, site role, dependent, recovering, retired, or dead.
Command/mentorship are validated links to that assignment, not additional copies.

Events may retain a destroyed formation's label, but cannot retain a recoverable
formation object or restore veteran benefits. Dangling gameplay references are
invalid; expired narrative references render a saved label or “older history
unavailable.” Equipment has exactly one current holder or estate/site custodian.

## C03 — Actions and transaction boundaries

Recommended public API shapes (names may follow the local style):

```text
preview(state, data, observer, command) -> Preview or RuleError
apply(state, data, actor, command) -> ActionOutcome or RuleError
project(state, observer) -> VisibleCampaign
validate_campaign(candidate, data) -> valid or diagnostic
```

Preview is read-only and consumes no RNG. Execution revalidates. `RuleError` carries
a stable reason and relevant IDs/amounts for localized UI text. It must not expose
hidden enemy state. UI and AI submit the same domain commands; a report never
applies an outcome. Existing `UiAction` keeps navigation separate from commands.

Validate ownership, phase, membership, location, costs and prerequisites before
mutation. Compute an outcome against a candidate state and candidate RNG; commit
only if valid. Candidate cloning is acceptable initially for this small simulation;
profile before replacing it. Rejection preserves resources, IDs, RNG and events.

Multi-edge movement commits one legal edge/encounter at a time. A changed or
blocked later edge stops at the last committed site and produces an interrupted
order result, not a rollback of earlier travel. Map orders persist their remaining
route and continue after movement refreshes; ordinary destination taps execute
the legal affordable part immediately. Preview is a read-only engine boundary,
not a mandatory confirmation screen for every move.

B03 added a serializable pending battle preparation boundary. Starting a pending
battle commits its resolution and campaign consequences atomically; playback
reads the committed receipt. Round-end effects commit as one boundary. Save only
stable boundaries, including complete pending preparation; never half a battle
commit or half a round.

Ordinary commands require the actor's faction turn. Transfers are the confirmed
exception: allow the owning faction to transfer between co-located armies at any
stable boundary, including between NPC actions, preserving spent allowances.
The player cannot issue commands for another faction. Provide a visible pause of
NPC progression between actions so the transfer exception is actually usable;
pause does not undo a completed command or allow unrelated off-turn orders.

Use one accepted-action sequence number and a pending domain-fact queue. Consume
facts exactly once, then retain narrative copies if desired. Reports, history
views, serialization, and UI animation cannot enqueue fresh rewards. Ended local
threats and completed construction have terminal state independent of history.

## C04 — Determinism and dates

Use `macroquad_toolkit::rng::SeededRng`, already present in the sibling toolkit,
not a new RNG implementation/dependency. Create named generation, combat,
development, and people streams by drawing four seeds from a root seeded stream
in that fixed order; persist their full states. Cosmetic randomness is separate.
Restore serialized state, not `SeededRng::new(saved_state)`.

Process factions, sites, armies, slots, people and events in stable ID order where
rules do not specify another priority. Never depend on hash-map traversal. Use
integer permille or rational arithmetic, wide intermediates, explicit rounding,
and checked resource/date arithmetic. Overflow rejects an action/load clearly.
Use no frame delta, wall-clock time, thread order, or UI RNG in simulation.

`completed_rounds = 0` means Spring of the starting year. Season is remainder by
4; elapsed years are quotient by 4. Person dates may be signed seasonal offsets
before campaign creation. Age is floor of elapsed seasons divided by 4. A faction
turn is not a date. Round timing is specified in strategic rule P02.

## C05 — Data and validation

Keep `game_config.json` for presentation and `economy.json` for existing economic
defaults. Add cohesive definition files as consumed: `campaign_rules.json`,
`world_layout.json`, `scenarios/rosemarch.json`, `troops.json`, `combat_rules.json`,
`progression.json`, `development.json`, `lifecycle.json`, and `history_rules.json`.
These are planned assets, not files this documentation change creates.

Use `macroquad_toolkit::include_json!` or typed `data_loader` APIs. A combined
Kestrum `GameData` assembles and semantically validates definitions; it does not
duplicate the toolkit's parser, I/O, platform branching or fallback system.
Reference troop costs/capacities in `economy.json`; do not copy competing values
into `troops.json`. Embedded JSON is not a runtime asset-registry entry.

Validate unique IDs, resolving references, positive capacities/cost divisors,
nonnegative resources, bounds on percentages, reachable starts, reverse gate
mapping, feasible anchors and class prerequisites, chronology, compatible
assignments, and known supported policy values. Reject unknown schema versions
and unsupported enum/policy values. Error text identifies the source and field.
The production 80-major-node check is distinct from explicitly named test scenarios.

Each new persisted field requires a version/migration decision in its package.
Supported saves identify the content rules version; changing balance under an old
save must be an explicit compatible migration, not an unnoticed replay change.

## C06 — Save catalogue and compatibility

Confirmed requirements are automatic checkpoints after every completed round,
manual saves during the player's turn, and no game-imposed slot cap. Each automatic
checkpoint and each new manual save has a distinct ID. Only explicit overwrite or
deletion changes an existing slot. Names are labels, never raw file paths/keys.

Inspected toolkit APIs already include `save_to_slot_with_version`,
`load_from_slot_with_migration`, `delete_slot`, `slot_exists`, `save_json_key`,
`load_json_key`, `RawSaveStore`, and `SlotSaveStore`. Native slot writes use atomic
replacement. `get_save_slots` enumerates native files but its WASM branch only
probes `slot_1`, `slot_2`, `slot_3`, `autosave`, `quicksave`. It cannot satisfy this
design as-is. Multi-key writes are not transactional.

K03 must first add or adopt a **shared toolkit indexed-save capability**, with
failure injection coverage, rather than create Kestrum-specific browser I/O.
Kestrum owns catalogue metadata: internal ID, display name, kind, campaign ID,
season, schema/content version and last successful save. The storage index owns
monotonic allocation, a recoverable pending-operation record, and committed entry
references. The exact public toolkit API is implementation work; do not assume it
already exists or edit unrelated toolkit functionality.

Required storage protocol: durably reserve a unique internal key and journal the
pending operation; write the payload to that fresh key; atomically publish the new
catalogue reference; then clear the journal. A restart reconciles a pending write:
valid payload can be published, absent/invalid payload leaves previous entries
untouched and reports failure. Overwrite uses a new payload key and switches the
existing catalogue entry only after success. Explicit deletion removes the entry
through the same recoverable protocol before deleting its payload. A failed cleanup
can leave an internal orphan, never silently lose a listed good save. Serialize
writers; test interrupted stages. Internal tool-managed staging is not permission
to create agent scratch files or campaign backup clutter.

New/overwrite/delete errors remain visible and leave the session usable. A failed
autosave does not roll back the completed round; offer Retry (same checkpoint
identity) or Continue Unsaved. Retrying must not rerun seasonal effects. Unbounded
lists scroll/page; never rotate old saves because narrative history is bounded.
Validate candidate loads completely before replacing the live session.

**P compatibility choice:** retain read-only access to shell version 1 in its
original slot, explicitly identified as an empty-atlas campaign. Do not invent
armies, historical events, or resources to “migrate” its chronology into a real
kingdom. Offer return to that shell or start a new strategic campaign in a new
slot; never overwrite it automatically. Strategic schemas begin at version 2.
Each later package migrates the preceding strategic version with defensible
defaults or clearly marks an incompatible development version; K18 tests the
declared support matrix. Unsupported loads preserve the active game and old bytes.

Capture mode and automated tests must use their established isolated storage or
in-memory test store, never the player's catalogue. Preferences stay separate.

## C07 — Information and UI boundaries

Map topology and public settlement names/controllers are visible. Enemy army
composition, exact headcount, character identity, private resources and careers
are not read directly by player UI or AI decision logic. P10 defines observations.
Rendering a report/biography must use its viewer-filtered facts.

Reuse the existing atlas camera/virtual coordinates for draw and pick. Preserve
the 1280 × 720 logical minimum, 1920 × 1080 target, and existing 48-pixel controls
where applicable. A selected inspector is dismissible; ordinary play has no
permanent dashboard for every simulated system. Long stack, roster, report and
save lists scroll with reachable Back/Close/confirm actions.

For each package, update the chapter 10 screen brief, contextual Help, legal and
disabled action feedback, and both-size capture scenes it actually implements.
Tap selection and confirmation must cover dragging, right-click or shortcuts.
Do not add debugging statistics to the normal player experience.
