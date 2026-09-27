# Acceptance, coverage and implementation record

[Plan](../implementation-plan.md) · [Packages](work-packages.md)

## Definition of done for every package

1. All named rules and package outputs are implemented through real state and
   actions, including valid, rejected, interrupted and resumed cases. No placeholder
   button, comment, fake event or bypassed prerequisite counts as completion.
2. New state round-trips at a stable boundary; schema/content compatibility is
   deliberate. Rejected actions preserve resources/RNG/IDs and invalid loads
   preserve the live session. Player and AI use the same constraints.
3. The package's five meaningful behavior cases pass through library APIs in
   `tests/`. Preserve useful regressions. Explain distinct extra coverage rather
   than deleting tests to reach five. No new tests/test helpers under `src/`.
4. Changed player flows have visible tap controls, disabled reasons, Help and
   feedback. Update the screen brief; review normal/minimum and relevant dense,
   failure and urgent states. Do not defer all UI to K18.
5. Required checks run in the real checkout, followed by the no-argument publisher
   for meaningful game changes. A blocked command is reported with its actual
   failing dependency/path; no copied project, dummy crate or alternate manifest.
6. Update this status table with commit/evidence and limitations; update README
   and any changed rule. Commit all project files according to AGENTS.md and
   verify `git status --short` is empty before reporting completion.

Recommended commands from the actual Kestrum directory:

```powershell
cargo fmt -p kestrum -- --check
..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test -p kestrum --locked --all-features
# The normal test run includes tests/code_standards.rs with no size exceptions.
.\scripts\capture_ui.ps1 -Fullscreen
.\scripts\capture_ui.ps1 -Scenes title_minimum,gameplay_minimum,help_minimum -WindowWidth 1280 -WindowHeight 720
.\publish.ps1
git status --short
```

Extend the existing capture harness with each implemented scene and pass those
scene names explicitly when reviewing it; the three minimum scenes above cover
only the current shell. Use the shared wrapper's hidden default, wait for completion
and verify its launched game exits. Save screenshots directly in
`docs/verification/`, overwriting equivalent scene names. No scratch captures,
backup folders, ad hoc logs or temporary source copies.

Pure documentation changes use link/anchor, coverage, consistency, source-size and
whitespace checks; they do not warrant republishing unchanged gameplay. Do not
present historical runtime results as a fresh validation run.

## Traceability and omission check

Rule packets: [strategic P01–P10](strategic-rules.md),
[combat P11–P15](combat-rules.md), [people/place P16–P23](people-and-places.md).
Every row must have evidence before final acceptance. An earlier prototype may
leave a later row planned; it cannot silently delete or relabel it as future scope.

| Required behavior / source decisions | Rule or contract | Owning packages | Acceptance evidence |
| --- | --- | --- | --- |
| Kingdom identity, 4–8 total factions (O08) | P01 | K02, K17 | Valid/invalid setup and 4/8-faction campaigns |
| 80 major nodes, fixed sites/routes (O01/D06) | P01/C02 | K01, K04, K17 | Counts, saved topology and atlas review |
| Nested entries, multiple fronts, partial anchors | P03 | K04, K06, K10 | Actual entrances and hostile pockets after ownership change |
| One season per full round (D03/O04) | P02/C04 | K02 | Phase and boundary tests, including eliminated faction |
| Six formation slots, variable headcounts (D01) | P04/P11 | K05 | Six-slot UI, capacities and rejected seventh |
| Multiple armies/no stack cap (O05) | P04/P11 | K06, K07, K10 | Multi-army field/relief and dense stack list |
| Co-located free transfers at stable boundaries (O09) | P04/C03 | K06 | Off-turn transfer without movement refresh |
| Road speed for everyone, movement/composition (O02) | P04/P07 | K06, K09 | Costed route and invader/defender symmetry |
| Supply, recovery, no initial starvation (O03/D14) | P05 | K05, K06, K10 | Cut route, siege isolation and recovery arithmetic |
| Gold/Wood/Stone/deficits/refunds (O11) | P06 | K05, K09, K11 | Existing JSON fields consumed and costs explained |
| Facilities constrain recruit/train availability (O15) | P06/P18 | K05, K09, K13 | Local opportunity loss blocks new capability |
| Outpost duration and interruption (O12/D12) | P07 | K09 | Three steps, blocked progress and settlers conserved |
| Named people strengthen legal leaderless armies (O06) | P11 | K05, K07, K13 | Visible contribution and loss of a leader |
| Combat/counters/multi-army casualty allocation (O05) | P11/P12 | K07 | Bounded deterministic exchanges and clear report |
| Destruction erases formation identity/history (D05) | P13/P16 | K05, K07, K08 | Replacement cannot recover destroyed veterancy |
| Reinforcement never dilutes survivors (O16) | P05/P16 | K06, K08 | Veteran 21/100 -> recovery remains veteran |
| Retreat, recurring enemy, limited leadership wound (D10) | P13 | K07, K13 | Persistent survivor and assumed-command facts |
| Persistent siege/sortie/escape/relief (O13) | P14 | K10 | Save across siege and multi-army relief |
| Ordinary threats, one-time rewards, reclamation | P15/P19 | K11 | Human/wildlife clearance, no repeat payout |
| Abstract soldiers -> grounded people (D02) | P16/P17 | K08, K13 | Valid emergence dates and service provenance |
| Actual encounter eligibility (O10) | P16/P18 | K08, K13 | No remote/pre-service/fabricated encounter credit |
| Soft emergence curve, traits/recognition (O14/D07) | P17 | K13 | Few/many-roster curve, no hard cap, deeds visible |
| Ordinary classes and specialization (O15/O24) | P18 | K13 | All ordinary paths, conversions and interruptions |
| Shared service, subtle bonds, enemy careers | P17/P23 | K13, K16 | Repeated known rival and actual service links |
| Layered growth/decline/forts/occupation (O18/D08) | P19 | K09, K11 | Safe/war-damaged divergence and independent layers |
| Refugees, settlement/recruiting context (O18/O20) | P19/P22 | K11, K15 | Population conservation and contextual entry |
| Rename, HQ and political capital (O19/D09) | P19 | K11 | Stable IDs, no double HQ income or automatic city growth |
| Aging, injury, retirement, death (O17) | P20 | K14 | Birthday checks and useful non-frontline roles |
| Mentorship, family and no-children succession (O20) | P21/P22 | K14, K15 | Eligible student and several successor categories |
| Households/dependents/adoption and local institutions | P22 | K15 | Optional family path without stat inheritance |
| Items, legacy, eras, factual histories (D15) | P23 | K16 | Single custody, factual chronology, distinct examples |
| Bounded memory without lost gameplay facts (O23) | P23/C03 | K08, K16, K18 | Retention boundaries and 200/400-round coherence |
| War/Peace and constrained AI (O21) | P08/P09 | K12 | Same commands, no fabricated units or omniscience |
| Limited enemy information (O21) | P10/C07 | K02, K08, K18 | Precombat/report/biography/search redaction |
| Conquest/vassal outcome, defeat and no restoration (D04/O07/D11) | P08 | K12, K18 | Reachable ending and permanent elimination |
| Unlimited named manual/round saves (O22) | C06 | K03, all later state, K18 | Native/browser reload, quota/retry, interrupted writes |
| Human-only initial content, botanical identity (O24/D13) | P01/P18/P23 | K13, K16, K17 | No advanced paths required by ordinary classes |
| 1080p/720p touch UI and tutorial (O25) | C07/chapter 10 | Every visible package, K18 | Actual canvas/interaction review and honest device limits |
| Native/WASM publishing and source standards | AGENTS.md/C01 | Every gameplay package | Real checkout checks, publisher and commit |

## Explicit later scope

| Preserved design | Boundary for this plan |
| --- | --- |
| Dragon Knight/Plague Cleric, magic, dragon eggs/hatcheries, other races (D13) | After ordinary classes; do not fabricate encounters/resources to enable them |
| Weather modifiers, detailed starvation, disease, captivity and negotiation | Calendar/light siege first; no hidden periodic attrition |
| Numerical combat bonds, heroic rescues and large relationship management | Shared-service facts/context first; no unspecified multipliers |
| Rich cultures/origins/bonuses, spies, scouting intelligence and naval warfare | Initial human land campaign; every later rule needs its own packet |
| Alliances, tribute, diplomatic marriage, full vassal administration | Only P08 defeat-outcome vassal status initially |
| Restoration, claimant/separatist kingdoms, civil wars and splintering | Eliminated factions stay gone; personal displaced records are permitted |
| Multiplayer | Single-player state ownership; no speculative networking |
| Portrait/mobile canvas below 1280 × 720 | Not part of current minimum; touch remains mandatory at supported sizes |

## Integrated scenario A — Rosemarch causes people to change

Use the durable small scenario and an explicit seed. Enter west_gate, capture
milltown, send one force toward high_fort while a second holds milltown. Script
the opponent through normal legal commands to counterattack; a controlled seed
can exercise the limited wound/assumed-command case. Medical evidence needs real
casualties and a participating Medic/Medics formation. Resolve fort combat with a
surviving enemy who legally retreats; meet that same ID later.

Assert actual site IDs, troop slots/headcounts, movement costs, control and supply.
Only present participants receive evidence. After sufficient qualifying service,
recognition and an ordinary Medic/Officer path explain their real causes. Inspect
the same battle through army, person and place views without repeating rewards.
No guaranteed named protagonist, dragon path, plague simulator or scripted free
promotion is needed. Complete this via touch controls as well as library tests.

## Integrated scenario B — Isolation, construction, siege and relief

Build a forward outpost, then cut its route to the HQ with legal hostile control.
Wounded veteran formations cannot replenish despite occupying an Outpost. Move a
builder away during construction: progress pauses, cancellation after progress
does not refund. Restore supply and verify only the stated recovery Gold is spent.

Besiege a fort with several armies, save before round resolution and again after
it, reload and compare one seasonal weakening step. Exercise an early assault,
an escape with a legal exit, a rejected escape with none, and joint relief.
Separate garrison fallback from incoming relief retreat. Check persistent fort
damage, formation destruction and person survival once. A new road helps the
invader too; a siege does not continue after its last besieger leaves.

## Integrated scenario C — Fifty years and a legacy without children

Run 200 complete rounds in a scenario with long-lived factions/peace intervals.
Found an outpost, develop a safe connected site while a geographically constrained
site stays small, damage and reclaim a fort, and send conserved refugees to a
safer place. Move the capital independently of fort/population tiers; relocate a
lost HQ through the explicit recovery action.

Train a junior through a non-related mentor, give an elder a useful role, retire
or lose the founder and appoint a qualified successor already at the army.
Transfer a mundane named item once. Separately exercise a sparse dependent's valid
entry into service. Test dates, relationships and class requirements; do not force
every random birth/death or fixed story date to occur naturally in one test.

Advance narrative pruning beyond ten/twenty years. Old stories may vanish while
earned traits, current ruins, item holder, living relatives and command eligibility
remain valid. Add a 400-round run to find growth/memory/ID-overflow assumptions.
These tests simulate the real engine; a durable test scenario may prevent early
victory through its authored initial conditions, not by disabling end logic in
production. Distinguish controlled scenarios from ordinary AI-played campaigns.

## Integrated scenario D — Real endings and durable saves

Complete a normal campaign with four factions, then exercise eight-faction
boundaries. Losing just a capital must not delete remaining armies. Defeat the
last independent rival and test Annex and Submission paths in separate runs.
Player elimination and mutual destruction produce honest defeat. Save/reload the
terminal state without restarting NPC turns or issuing the reward/event again.

For storage, exceed the old five-name browser discovery list, reload, inspect
every new manual/checkpoint entry, deliberately fill/fail the test storage, retry
the same checkpoint, and verify no older save was silently removed. Test a declared
supported migration plus an unsupported version and a corrupt candidate; the
active campaign remains usable after each failed load. Use the toolkit's test
store for injected failures, not the player's real saves.

## Visual, interaction and performance evidence

Review at actual 1920 × 1080 native fullscreen and 1280 × 720 WebGL canvas, covering:
setup/text entry, world/region selection, dense multi-army stacks, route preview,
transfer comparison, combat report, all siege sides, supply loss, construction
pause, long character/household histories, large values/long labels, long save
lists, invalid load/full storage, campaign ending and reopened Help. Use stable
scene names such as `ui_region.png`, `ui_siege.png`, `ui_character.png` and their
`_minimum` forms directly in `docs/verification/`; create only implemented states.

Use touch alone to discover and perform all actions, including text entry with
the platform's input route, Close/Back, scrolling, pause/off-turn transfer and
Full Screen. Verify pointer picking after drag, pinch, resize, zoom and display
scaling. Physical pinch limitations must remain explicitly unverified if suitable
hardware/tools are unavailable. The shell's fullscreen issue is a regression to
investigate, not a reason to change the supported minimum silently.

Provisional performance review targets on the recorded test hardware: ordinary
input/draw remains responsive at 60 Hz; typical phase planning/boundary work stays
below 250 ms at 8 factions/80 nodes; if exceeded, use bounded work between frames
without changing simulation order. These are measurements and tuning targets,
not guarantees. Record peak entity/event counts, rounds tested, native/browser
timings and save sizes. Keep gameplay state bounded by actual modeled population
and relationships; narrative caps alone do not prove total memory is bounded.

## Completion record

Replace Planned only with In progress, Blocked (specific reason), or Done (commit
and evidence). A dependency package must be Done before its consumers start.
“Done with limitation” must name the limitation and cannot close a required
acceptance criterion without explicit scope acceptance.

| Package | Status | Commit / evidence / remaining limitations |
| --- | --- | --- |
| K01 | Done | Commit subject: `Rosemarch gains its sites and founding kingdoms (K01 typed content)`; [verification](../verification/k01-content.md). 18 tests, format, Clippy, source gate and Windows/WebGL Preview publish pass. Roost tracking unavailable; no strategic play or new browser/touch claim. |
| K02 | Done | Commit subject: `Kestrum's kingdoms share one seasonal round (K02 campaign phases)`; [verification](../verification/k02-campaign.md). 23 tests, format, Clippy, source gate, both-size native review, browser phase/reload checks and Windows/WebGL Preview publish pass. Inherited fullscreen/display scaling and physical touch remain K18 limitations. |
| K03 | Done | Commit subject: `Kestrum keeps each turning of its history (K03 recoverable saves)`; [verification](../verification/k03-saves.md). 28 tests, format, Clippy, source gate, both-size native review, browser paging/reload/Unicode/overwrite/Busy-writer retry and Windows/WebGL Preview publish pass. Toolkit/shared bridge commits and native file restart evidence are linked there. Physical device and inherited minimum-browser display review remain K18 limitations. |
| K04 | Done | Commit subject: `Kestrum reveals the gates beneath its borders (K04 geography)`; [verification](../verification/k04-geography.md). Five geography/navigation cases, 33 total tests, formatting, strict Clippy, source gate, both-size native captures, browser selection/camera/reload and Windows/WebGL Preview publish pass. Minimum-browser display and physical-touch limitations remain open for K18. |
| K05 | Done | Commit subject: `Kestrum musters its founding armies (K05 armies and economy)`; [verification](../verification/k05-armies.md). Five economy/military cases, 38 total tests, formatting, strict Clippy, source gate, both-size native review, browser recruitment/economy/reload and Windows/WebGL Preview publish pass. Inherited minimum-browser display and physical-touch limitations remain open for K18. |
| K06 | Done | Commit subject: `Kestrum follows its roads and supply lines (K06 movement and recovery)`; [verification](../verification/k06-logistics.md). Five movement and five recovery cases, 48 total tests, formatting, strict Clippy, source gate, both-size native review, browser group movement/transfer/reload and Windows/WebGL Preview publish pass. Minimum-browser display and physical-touch limitations remain open for K18. |
| K07 | Done | Commit subject: `Kestrum remembers the price of battle (K07 combat and retreat)`; [verification](../verification/k07-combat.md). Five combat and five wound cases, 58 total tests, formatting, strict Clippy, source gate, 26 normal/minimum native captures, published browser battle/recovery/reload and Windows/WebGL Preview publish pass. Inherited minimum-browser display and physical-touch limitations remain open for K18. |
| K08 | Done | Commit subject: `Kestrum remembers the service behind its veterans (K08 evidence and knowledge)`; [verification](../verification/k08-service.md). Five evidence and five knowledge cases, 68 total tests, formatting, strict Clippy, source gate, 42 normal/minimum native captures and browser battle/service/search/filter/reload pass. Windows/WebGL Preview publish succeeded; Project Roost tracking service was unavailable. Inherited minimum-browser display and physical-touch limitations remain open for K18. |
| K09 | Done | Commit subject: `Kestrum raises lasting works along its roads (K09 construction)`; [verification](../verification/k09-construction.md). Five construction, five content and three history cases; 81 total tests, formatting, strict Clippy and source gate pass. 46 normal/minimum native captures reviewed; published browser costs/refunds/focus/progress/reload/completion pass. Windows/WebGL Preview publish and Project Roost tracking succeeded. Minimum-browser sizing, WASM cache invalidation and physical-touch limitations remain open for K18. |
| K10 | Done | `71159d1`: persistent siege/escape/relief, 96 tests and publication; [evidence](../verification/k10-sieges.md). Minimum WebGL/physical-touch limits remain. |
| K11 | Done | `1d9ddcd`: living places and threats; [evidence](../verification/k11-living-places.md). 111 tests, formatting, strict Clippy, source gate, eight native captures, published browser development/save/reload and Windows/WebGL Preview publishing pass. Minimum WebGL/physical-touch limits remain. |
| K12 | Done | Commit subject: `Kestrum's rivals wage war and accept their fate (K12 kingdom campaign)`; [evidence](../verification/k12-kingdoms.md). 127 tests, formatting, strict Clippy, source gate, eight native captures, published browser war/AI/save/reload and Windows/WebGL Preview publishing pass. Inherited minimum WebGL scaling and physical-touch acceptance remain K18 checks. User requested a stop after this commit. |
| K13 | Done | `10ab4c6`: grounded emergence, traits, ordinary careers and specialization; nine progression cases and 136 tests, formatting, strict Clippy, source-size gate, native 1920×1080/1280×720 review, Windows/WebGL Preview publishing; [evidence](../verification/k13-careers.md). Browser interaction, physical-touch acceptance and minimum-WebGL campaign scaling remain K18 checks. |
| K14 | Planned | Lifecycle/mentorship |
| K15 | Planned | Households/succession |
| K16 | Planned | Items/history/eras |
| K17 | Planned | Production world/content |
| K18 | Planned | Integrated/platform acceptance |

## Plan review priorities

The most consequential provisional choices to review in play are P08's narrow
vassal defeat outcome, P11–P14's combat/retreat/siege mathematics, P17's emergence
frequency, P19's growth/population pace and P20–P22's aging/household defaults.
Their provisional status does not authorize an implementation agent to replace
them silently. Review actual scenarios, record an amended rule and data values,
and keep coverage intact. No open mechanics gate remains merely because the
original GDD had not yet selected a formula; genuine newly discovered conflicts
follow the plan's amendment procedure.
