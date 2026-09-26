# 12 — Delivery and validation

[Documentation index](README.md) · [Decisions](13-decisions-and-open-questions.md) · [Verification record](verification/documentation.md)

## Current milestone: documentation and initial repository

This milestone consolidates the complete founding design, preserves the original drafts within `docs/reference/`, records conflicts and proposals, and commits the supplied template with the documentation. It makes no Kestrum gameplay or UI changes.

The root copies of the three design drafts are retained in the initial commit. All their content is preserved in `docs/`; the documentation set remains usable after those root copies are deleted. Exact source hashes and section coverage are recorded in the [coverage ledger](source-coverage.md).

## Proposed implementation sequence

Each milestone is independently useful and should be completed, validated, and committed before the next major change. This sequencing is a proposal; the source prototype goals remain the basis of scope.

### M1 — Template onboarding and strategic foundation

Adapt project/package identity, dependency path, assets/metadata, capture prefix, and the first screen composition. Coordinate legitimate shared workspace registration through the canonical management configuration; do not fabricate a workspace or edit unrelated projects to bypass validation.

Build the calendar, explicit actions, persistent node graph, world/region entrances, local control, and simple army movement. Include a player headquarters, several simple world nodes, one 8–12-node region with two entrances, a regional capital and fortress, and an enemy headquarters.

Exit evidence: touch-only map selection and movement work; an army can enter and leave the intended gate; partial regional control is visible; a full faction round advances one season.

### M2 — The first military loop

Implement six formation slots per army, variable headcount, several armies, basic composition, movement costs, automatic encounters, retreat, destruction, supply-gated recovery, and formation veterancy. Add minimal resource costs and enemy behavior sufficient to exercise the same rules.

Exit evidence: recruit → compose → move → fight → retain losses → recover or retreat works; destroyed formations do not reappear as veterans; leader presence has a clear, tunable effect; multiple fronts matter.

### M3 — People emerge from campaigns

Track a small number of founders/juniors and permit grounded emergence from formations. Record reusable experiences, recognition, a limited trait set, basic classes, eligibility, and mentorship opportunities. Support an enemy who survives and can be recognized later.

Prototype events: participation, victory, defeat, survival while outnumbered, commander wounded, assumed command, treatment of wounded, fighting a particular enemy type, defense, and capture of a strategic node.

Exit evidence: a player can explain a junior's development using actual events. Tomas's leadership, Mira's medical opportunities, and an escaped rival illustrate desired outcomes without requiring scripted missions or every advanced class.

### M4 — Kingdom expansion and changing places

Add Gold/Wood/Stone, development focuses, physical military prerequisites, outposts, roads, anchor control, basic growth/decline, occupation consequences, local threats, fog of war, and a persistent light siege with relief.

Build toward several rival kingdoms and minimal War/Peace diplomacy, faction elimination, and a reachable conquest victory. Decide faction counts and conquest rules first.

Exit evidence: an outpost can become useful; road access benefits both sides; an isolated force cannot replenish normally; a siege persists through turns and saves; a captured region can still contain hostile pockets; all rival kingdoms use valid capabilities.

### M5 — Generational continuity

Add age and career transitions, retirement, useful mentorship, sparse households/dependents, several kinds of heirs, inherited items or roles where supported, population movement, changing capitals, and contextual histories/eras.

Exit evidence: run a multi-decade scenario in which founders give way to students or relatives, places change unevenly, losses remain historical, and the roster stays understandable. Demonstrate a meaningful legacy with no children.

### M6 — Campaign balance and presentation

Tune emergence, combat, leader effects, economic pressure, recovery, construction, sieges, and growth. Test supported faction counts, long histories, fog of war, dense states, and save compatibility. Complete native and browser review at normal and minimum viewports, with touch-only interaction and required publishing.

Exit evidence: a full campaign can conclude without an unavailable feature, and players remember particular forces, people, and places for reasons supported by the record.

## Prototype, first functional release, and long-term design

| Scope | Required focus |
| --- | --- |
| Earliest playable prototype | Small graph with one nested region, persistent armies/people, combat/retreat, partial control, evidence and recognition |
| First functional kingdom version | Six formation slots, headcounts, basic economy and AI, supply/roads/outposts, light siege, minimal diplomacy, elimination and achievable conquest victory |
| Full generational promise | Useful aging/retirement, succession through family or mentorship, changing places/capitals, long histories and contextual memory |
| Explicit future scope | Multiplayer; extensive origins/cultures/bonuses; richer intelligence; alliances/tribute/vassals/guarantees/borders/prisoner exchanges; restoration, fragmentation, civil wars, claimant or separatist factions |

The generational game is the destination. Staging it after a small prototype does not remove it from the design. Later combat injury/capture/escape depth, weather, advanced bonds, complex civilian effects, and negotiation should be introduced when they strengthen the proven loop.

## Feature verification plan — proposal

Strongly target five meaningful tests per major feature, using table-driven cases for related variations. These are acceptance scenarios for future tests, not tests added by this documentation work.

| Feature | Five valuable cases |
| --- | --- |
| Graph and control | Connected movement; blocked route; correct regional entrance/exit; partial ownership; anchor capture with remaining enemies |
| Turn/calendar | Sequential turns; one season per round; four seasons per year; mid-round elimination; construction/siege/age advance once |
| Economy/development | Valid purchase; rejection without mutation; one active focus; facility access lost; deficit policy applied consistently |
| Army persistence | Six-slot validation; valid transfer; cumulative casualties/recovery; destruction/new identity; specialization with prerequisites |
| Combat and siege | Ordinary outcome; retreat; wipeout consequences; continued siege/assault; relief or escape with multiple armies |
| Character development | Evidence-based trait; grounded emergence; class prerequisites; constrained enemy progression; repeated event does not grant duplicate credit |
| Living places | Sustained growth; decline/reclamation; independent military layer; occupation/refugees; stable identity across rename/capital move |
| Generations | Career aging; grounded service entry; retirement/death cleanup; non-blood succession; bounded active roster with retained history |
| History/knowledge | Chronological biography; unknown enemy remains hidden; stale observations; alias/history links; shared event rendered without duplicate effects |
| Persistence | Deterministic save round-trip; supported migration; corrupt load preserves session; RNG/order preserved; no repeated battle/reward after load |

Keep useful existing regressions. Additional distinct cases require an explanation, not removal of useful coverage to reach a count. No test may validate against a copied checkout, alternate manifest, dummy crate, or fabricated workspace.

## System acceptance playthroughs

### Rosemarch

Enter from the western gate, capture Milltown, commit forces toward the fort, face a counterattack, gain supported leadership/medical experiences, and encounter the surviving opponent again. Verify actual location, control, supply, and event records at each step. Example names are optional; the causal sequence matters.

### Encirclement and relief

Cut a supply route, observe blocked normal recovery, establish a siege, choose between early assault and waiting, bring relief, then confirm survivors and lasting local damage. Verify that co-located friendly armies and partial regional control remain coherent.

### Fifty years of continuity

Let an outpost develop, an old fort be damaged or ruined and later reclaimed, a founder retire or die, and a student or relative enter service. Move the capital if conditions support it. Inspect histories to confirm that names, dates, people, and places remain connected and that not every settlement becomes a metropolis.

## Initial template handoff

Read-only inspection of the supplied starter found:

- `Cargo.toml` still names the package `game_template` and points at `../../macroquad-toolkit`. In this checkout that resolves outside `RustGames`; the actual sibling toolkit is at `../macroquad-toolkit`.
- The deployed and canonical shared workspace manifests do not list Kestrum.
- Runtime config, page metadata, capture hooks, and scripts retain template identity. Do not publish that as a completed Kestrum game.
- Existing `src/**/tests.rs` files are legacy template placement; migrate separately before adding coverage.
- The root thumbnail and four verification screenshots are supplied template assets.

These are onboarding follow-ups, preserved in place for this documentation-first initial commit. Do not change shared workspace membership or unrelated projects just to obtain a passing check. The full original starter instructions are retained in [template provenance](reference/template_readme.md).

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
