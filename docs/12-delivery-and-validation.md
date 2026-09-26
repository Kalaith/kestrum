# 12 — Delivery and validation

[Documentation index](README.md) · [Decisions](13-decisions-and-open-questions.md) · [Verification record](verification/documentation.md)

## Current milestone: title screen and empty atlas

The initial title/campaign shell is implemented. New Game opens empty geography as explicitly requested; nodes, factions, armies, and local regions remain later work. [Verification](verification/initial-map.md) records checks and limitations. This completes onboarding and presentation only, not every proposed M1 simulation feature below.

## Previous milestone: documentation and initial repository

This milestone consolidates the complete founding design, preserves the original drafts within `docs/reference/`, records conflicts and proposals, and commits the supplied template with the documentation. It makes no Kestrum gameplay or UI changes.

The discussion clarification and direct answers supplied on 2026-09-26 update that foundation. The ten formerly unanswered questions now have rules or delegated defaults. The implementation sequence below remains proposed; settled choices do not need fresh confirmation. Provisional economy data is stored in `assets/data/economy.json` for future toolkit loading.

The root copies of the three design drafts are retained in the initial commit. All their content is preserved in `docs/`; the documentation set remains usable after those root copies are deleted. Exact source hashes and section coverage are recorded in the [coverage ledger](source-coverage.md).

## Proposed implementation sequence

The current executable handoff is the [implementation plan](implementation-plan.md)
and its [K01–K18 work packages](implementation/work-packages.md). That plan expands
the broad M1–M6 stages below, moves save/evidence foundations earlier, and selects
explicit provisional mechanics under the user's request. Use its dependency order
and acceptance cases when assigning implementation work. This chapter retains the
high-level delivery intent and historical onboarding record.

Each milestone is independently useful and should be completed, validated, and committed before the next major change. This sequencing is a proposal; the source prototype goals remain the basis of scope.

### M1 — Template onboarding and strategic foundation

Adapt project/package identity, dependency path, assets/metadata, capture prefix, and the first screen composition. Coordinate legitimate shared workspace registration through the canonical management configuration; do not fabricate a workspace or edit unrelated projects to bypass validation.

Build the calendar, explicit actions, persistent node graph, world/region entrances, local control, and simple army movement. Start with a small test graph and one 8–12-site region with two entrances, a capital and fortress, and player/enemy headquarters. The campaign target is 80 major world nodes; internal sites are additional under the documented interpretation.

Exit evidence: touch-only map selection and movement work; an army can enter and leave the intended gate; partial regional control is visible; a full faction round advances one season.

### M2 — The first military loop

Implement six formation slots per army, variable headcount, several armies, composition, movement costs, automatic encounters, retreat, destruction, supply-gated recovery, and formation veterancy. Transfers are available whenever armies share a node, and reinforcement leaves surviving formations' veterancy unchanged. Load the provisional economy JSON through toolkit APIs and add enemy behavior sufficient to exercise the same rules.

Exit evidence: recruit → compose → move → fight → retain losses → recover or retreat works; destroyed formations do not reappear as veterans; leader presence has a clear, tunable effect; multiple fronts matter.

### M3 — People emerge from campaigns

Track a small number of human founders/juniors and permit grounded emergence from formations. Record participation and encounter facts, recognition, a limited trait set, ordinary classes, eligibility, and mentorship opportunities. Support an enemy who survives and can be recognized later.

The initial event vocabulary covers participation, victory, defeat, survival while outnumbered, commander wounded, assumed command, treatment of wounded, enemy types encountered, defense, capture, training, and non-combat encounters. O10 requires relevant participation; generic experience cannot bypass a missing encounter prerequisite.

Exit evidence: a player can explain a junior's development using actual participation. Tomas's leadership, Mira's development as a human Medic, and an escaped rival illustrate desired outcomes. Advanced classes and other races are deferred.

### M4 — Kingdom expansion and changing places

Expand Gold/Wood/Stone systems, development focuses, physical military prerequisites, outposts, roads, anchor control, basic growth/decline, occupation consequences, ordinary local threats, fog of war, and a persistent light siege with relief. Use the JSON economy placeholders as the balance starting point.

Build toward player-selected 4–8 total factions, minimal War/Peace diplomacy, faction elimination, and a reachable victory. Define exact conquest rules and reconcile D04/O07: conquest or vassalisation is the objective, while vassalisation as a defeat outcome remains a proposal for the initial version.

Exit evidence: an outpost can become useful; road access benefits both sides; an isolated force cannot replenish normally; a siege persists through turns and saves; a captured region can still contain hostile pockets; all rival kingdoms use valid capabilities.

### M5 — Generational continuity

Add age and career transitions, retirement, useful mentorship, sparse households/dependents, several kinds of heirs, inherited items or roles where supported, population movement, changing capitals, and contextual histories/eras.

Exit evidence: run a multi-decade scenario in which founders give way to students or relatives, places change unevenly, and the roster stays understandable. Demonstrate a legacy with no children and history pruning that preserves current state and progression facts while allowing old stories to be forgotten.

### M6 — Campaign balance and presentation

Tune emergence, combat, leader effects, economic pressure, recovery, construction, sieges, and growth. Test 4–8 factions on 80 world nodes, bounded histories, fog of war, dense states, and save compatibility. Verify round-end autosaves, unlimited slot lists, and manual saving during the player's turn. Complete native review at 1920 × 1080 full screen and WebGL review at 1280 × 720, with touch-only interaction and required publishing.

Exit evidence: a full campaign can conclude without an unavailable feature, and players remember particular forces, people, and places for reasons supported by the record.

## Prototype, first functional release, and long-term design

| Scope | Required focus |
| --- | --- |
| Earliest playable prototype | Small graph with one nested region, persistent armies/people, combat/retreat, partial control, evidence and recognition |
| First functional kingdom version | Human/ordinary-class roster; 80 world nodes; six formation slots; 4–8 total factions; JSON economy; supply/roads/outposts; light siege; round-end/manual saves with no slot cap; minimal diplomacy and reachable victory, with vassalisation scope under D04/O07 |
| Full generational promise | Useful aging/retirement, succession through family or mentorship, changing places/capitals, long histories and contextual memory |
| Explicit future scope | Other races and advanced/magical classes; multiplayer; extensive origins/cultures/bonuses; richer intelligence; alliances/tribute/full vassal diplomacy/guarantees/borders/prisoner exchanges; restoration, fragmentation, civil wars, claimant or separatist factions |

The generational game is the destination. Staging it after a small prototype does not remove it from the design. Later combat injury/capture/escape depth, weather, advanced bonds, complex civilian effects, and negotiation should be introduced when they strengthen the proven loop.

## Feature verification plan — proposal

Strongly target five meaningful tests per major feature, using table-driven cases for related variations. These are acceptance scenarios for future tests, not tests added by this documentation work.

| Feature | Five valuable cases |
| --- | --- |
| Graph and control | Connected movement; blocked route; correct regional entrance/exit; partial ownership; anchor capture with remaining enemies |
| Turn/calendar | Sequential turns; one season per round; four seasons per year; mid-round elimination; construction/siege/age advance once |
| Economy/development | Valid purchase; rejection without mutation; one active focus; facility access lost; deficit policy applied consistently |
| Army persistence | Six-slot validation; shared-node transfer without refreshing allowances; recovery preserving veterancy; destruction/new identity; specialization with participation prerequisites |
| Combat and siege | Ordinary outcome; retreat; wipeout consequences; continued siege/assault; relief or escape with multiple armies |
| Character development | Evidence-based trait; grounded emergence; class prerequisites; constrained enemy progression; repeated event does not grant duplicate credit |
| Living places | Sustained growth; decline/reclamation; independent military layer; occupation/refugees; stable identity across rename/capital move |
| Generations | Career aging; grounded service entry; retirement/death cleanup; non-blood succession; bounded active roster with retained history |
| History/knowledge | Chronological biography; unknown enemy remains hidden; stale observations; bounded pruning preserves gameplay facts and valid references; shared event rendered without duplicate effects |
| Persistence | Round-end autosave round-trip without duplicate effects; manual save preserves mid-turn RNG/order; supported migration; corrupt load preserves session; unbounded slot list with full-storage failure recovery |

Keep useful existing regressions. Additional distinct cases require an explanation, not removal of useful coverage to reach a count. No test may validate against a copied checkout, alternate manifest, dummy crate, or fabricated workspace.

## System acceptance playthroughs

### Rosemarch

Enter from the western gate, capture Milltown, commit forces toward the fort, face a counterattack, gain supported leadership/medical experiences, and encounter the surviving opponent again. Verify actual location, control, supply, and event records at each step. Example names are optional; the causal sequence matters.

### Encirclement and relief

Cut a supply route, observe blocked normal recovery, establish a siege, choose between early assault and waiting, bring relief, then confirm survivors and lasting local damage. Verify that co-located friendly armies and partial regional control remain coherent.

### Fifty years of continuity

Let an outpost develop, an old fort be damaged or ruined and later reclaimed, a founder retire or die, and a student or relative enter service. Move the capital if conditions support it. Inspect histories to confirm that names, dates, people, and places remain connected and that not every settlement becomes a metropolis.

## Initial template handoff

**Historical inspection before `3a2b5b4`:** the issues in the following list were
fixed by the title/atlas foundation. The current package identity, toolkit path,
workspace registration, test placement, artwork and capture/publish integrations
are correct. Do not repeat onboarding or alter workspace membership based on this
historical list; inspect the [current baseline](implementation-plan.md#actual-starting-point).

Read-only inspection of the supplied starter found:

- `Cargo.toml` still names the package `game_template` and points at `../../macroquad-toolkit`. In this checkout that resolves outside `RustGames`; the actual sibling toolkit is at `../macroquad-toolkit`.
- The deployed and canonical shared workspace manifests do not list Kestrum.
- Runtime config, page metadata, capture hooks, and scripts retain template identity. Do not publish that as a completed Kestrum game.
- Existing `src/**/tests.rs` files are legacy template placement; migrate separately before adding coverage.
- The root thumbnail and four verification screenshots are supplied template assets.

These were onboarding follow-ups preserved in the documentation-first initial
commit. Do not change shared workspace membership or unrelated projects just to
obtain a passing check. The original starter instructions remain in
[template provenance](reference/template_readme.md).

## Required engineering checks for implementation

Run from the actual Kestrum checkout using its real registered workspace:

```powershell
cargo fmt -- --check
..\rust_management\cargo.ps1 clippy --locked --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test --locked
.\publish.ps1
```

The tests include the toolkit source-size gate with an empty exception list. Formatting may use ordinary Cargo; other local build, check, test, Clippy, and run commands use the shared launcher. Publishing without parameters is required after meaningful game changes; an ordinary local run is not a substitute.

Do not create a branch for ordinary work; use `master` unless requested otherwise. Stage all project changes, including pre-existing files, after verification. Commit subjects use the game's voice and a clear parenthetical tag, honest validation notes, and AI co-authorship, following the shared commit guide. Verify a clean status and report the hash.

## Documentation milestone verification

Check exact archived bytes, complete source-heading coverage, local documentation links, absence of dependencies on root drafts, terminology and conflict consistency, and preservation of the starter implementation. Record actual checks and any checkout blockers in [documentation verification](verification/documentation.md).

This milestone does not warrant publishing an unchanged template or claiming a Kestrum visual review. Runtime checks, if attempted, must be reported accurately; documentation validation does not prove playable game behavior.
