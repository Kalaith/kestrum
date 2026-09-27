# Ordered implementation work packages

[Plan and handoff prompt](../implementation-plan.md) · [Contracts](contracts.md) ·
[Verification and coverage](acceptance.md)

K01–K09 are **Done**; K10 is the next eligible task. See the completion record in
[acceptance](acceptance.md#completion-record). Numbered rule links
refer to the provisional packets; D/O identifiers refer to chapter 13. A package
is a reviewable feature, not permission for one oversized Rust file or commit.
If split for size, preserve the same acceptance contract and commit each useful
part before proceeding. Do not skip tests/UI/save integration to finish a label.

Every package inherits C01–C07 and the common completion checklist. Paths below are
relative to the project and are intended edit locations, not existing-file claims.
Keep normal player screens honest while a prerequisite feature is unavailable.

## K01 — Typed content and a durable strategic scenario

**Depends on:** existing shell. **Read:** chapters 02/03/11; C02/C05;
[P01](strategic-rules.md#p01--setup-and-the-fixed-world), P06.
**Files:** `src/data.rs`, focused `src/data/*.rs`, existing `economy.json`, new
`assets/data/campaign_rules.json`, `assets/data/scenarios/rosemarch.json`,
`tests/content.rs`.

Load and validate the existing economy through toolkit APIs. Add the physical
graph, ten-site region, four HQ markers, entrance mappings, anchors and faction
setup schema used by the small scenario. Keep major-marker IDs separate from
physical site IDs. Encode prototype geometry, initial control and starting
resources as durable data. Add only definitions consumed now; later packages
extend schemas when introducing their mechanics. Do not fill the atlas with
invented production content or create unused future modules.

**Exit:** the library loads a valid strategic scenario without a graphics context;
the existing title/atlas still opens. No claim of a playable strategic campaign.

**Five tests:** (1) toolkit loading resolves all economy/scenario references;
(2) duplicate IDs/missing endpoints fail with labelled errors; (3) gates map both
ways and all HQs can reach the region; (4) capacities/resources/policy bounds
reject malformed values; (5) anchor expressions resolve and are satisfiable.
Use table-driven variants within each concern.

## K02 — Campaign ownership, commands and seasonal phases

**Depends on:** K01. **Read:** chapters 02/11; C01–C04;
[P02](strategic-rules.md#p02--faction-phases-and-round-order).
**Files:** `src/state.rs`, `src/state/campaign.rs`, `src/engine.rs`,
`src/engine/actions.rs`, `src/engine/round.rs`, `src/lib.rs`, `src/game.rs`,
`src/ui/atlas.rs`, `tests/campaign.rs`.

Create the strategic campaign variant with stable IDs, faction resources, graph,
date, acted set and state-owned RNG streams. Keep the shell-v1 variant explicit.
Add atomic action outcomes/errors and pending domain facts. Initially NPC phases
only pass; display that prototype status rather than claim AI. Replace shell
End Turn semantics for strategic campaigns with player -> NPCs -> one boundary.
Define observer projections exposing public sites/own state, never raw enemy state.
Add the C03 visible between-action pause and off-turn transfer intent seam, but do
not display Transfer until armies exist. Round systems are called only when
implemented; no fake income/construction/history success records.

**Exit:** strategic phase/date and save state are coherent; existing shell saves
still load through the explicit legacy path. Update chronology regressions without
removing invalid-load and overlay protection coverage.

**Five tests:** (1) four/eight faction ordering and one seasonal increment;
(2) four rounds equal one year; (3) skipping an inactive mid-round faction never
repeats/skips another; (4) rejected commands preserve state/IDs/RNG/events;
(5) resuming a stable serialized phase matches uninterrupted command results.

## K03 — Unlimited save catalogue and safe version handling

**Depends on:** K02. **Read:** chapter 11 Persistence; O22;
[C06](contracts.md#c06--save-catalogue-and-compatibility).
**Files:** `src/state/persistence.rs`, `src/game.rs`, `src/ui/menus.rs`, focused
`src/ui/saves.rs`, `tests/persistence.rs`; shared toolkit indexed storage as a
separate coordinated dependency change if still missing.

Inspect current toolkit capabilities before coding. Implement the generic
recoverable index/journal in the toolkit if absent, with its own project guidance,
checks and commit; never hide generic browser storage in Kestrum. Integrate
Kestrum catalogue metadata, new named manual slots, round checkpoint identities,
explicit overwrite/delete, candidate-load validation and schema/content versions.
Preserve the empty-atlas v1 slot as C06 specifies. An autosave failure offers Retry
or Continue Unsaved without replaying the round. Store last-successful Continue
selection independently of the simulation RNG and do not reset it on failed load.

**Exit:** many distinct saves are discoverable after native restart and browser
reload; the list has no artificial cap. Long lists and storage-full recovery have
visible touch controls. Storage tests use a durable test seam/in-memory failure
store, never real campaign data or agent-created scratch directories.

**Five tests:** (1) unique round/manual slot identities and unlimited indexing;
(2) every interrupted journal/payload/index stage preserves previous saves;
(3) overwrite/delete/retry are explicit and idempotent; (4) supported version
loads and unsupported/corrupt loads preserve session/bytes; (5) mid-turn and
completed-round round-trips preserve order/RNG and never reapply effects.

## K04 — Selectable world and region geography

**Depends on:** K02, K03. **Read:** chapters 02/10; C02/C07;
[P03](strategic-rules.md#p03--control-and-nested-boundaries).
**Files:** `src/state/world.rs`, `src/navigation.rs`, `src/ui/atlas.rs`, focused
`src/ui/world.rs`, `src/ui/selection.rs`, `src/game.rs`, `tests/world.rs`.

Render actual scenario markers/routes over the existing atlas and a region view
with explicit entrances. Add tap selection, Enter Region/World Map, breadcrumbs,
controller/contested/political-owner distinction and a dismissible site inspector.
World markers aggregate internal state, never duplicate it. Implement graph and
anchor query services used by later movement/supply. Camera/picking share logical
coordinates; preserve drag/pinch suppression of accidental release actions.

**Exit:** a player can inspect the small graph and understand partial regional
control. Movement/army controls wait for K05/K06. Keep the map dominant, update
screen brief/Help, and capture selected world/region at both supported sizes.

**Five tests:** (1) external route maps to correct entrance and reverse exit;
(2) a region marker cannot be used as a physical site; (3) anchor ownership can
change with hostile pockets remaining; (4) losing an anchor marks contested
without erasing local controllers; (5) world/region picking transforms remain
consistent after zoom/resize (retain existing camera regressions).

## K05 — Six-slot armies, recruitment and the economy

**Depends on:** K04. **Read:** chapters 03/04; D01/D05/O11;
[P05–P06](strategic-rules.md#p05--supply-and-recovery), P11 founder contribution.
**Files:** `src/state/military.rs`, `src/state/people.rs` minimal founder record,
`src/engine/economy.rs`, `src/engine/supply.rs`, `src/ui/army.rs`, `tests/economy.rs`.

Instantiate the authored founding armies/people and implement six optional slots,
variable capacities, current headcount, recruit/disband, owned local facilities,
resource validation and round income/upkeep. Introduce current population only
when construction needs it in K09; recruitment population deduction remains
disabled. Implement supplied-site reachability against the physical graph so
recruitment cannot bypass its requirements. Empty armies are removed safely.
Display resources/costs where decisions need them, not as a large fixed dashboard.

**Exit:** recruit/disband is playable through visible controls; unaffordable,
foreign, seventh-slot or unsupported-facility orders explain rejection. Income
and upkeep apply only at the common boundary and are included in saves.

**Five tests:** (1) initial setup matches JSON grants and six-slot capacity;
(2) affordable valid recruitment creates a new exhausted formation;
(3) invalid cost/site/facility/slot attempts change nothing; (4) income precedes
full-formation upkeep with exact shortfall/block/recovery clearing policy;
(5) disband/zero-headcount deletion gives no refund or resurrected identity.

## K06 — Movement, composition and connected recovery

**Depends on:** K05. **Read:** chapter 04; O02/O03/O09/O16;
[P04–P05](strategic-rules.md#p04--armies-movement-and-transfers).
**Files:** `src/engine/movement.rs`, army action handlers, supply/economy services,
`src/ui/army.rs`, route preview/selection, `tests/movement.rs`, `tests/recovery.rs`.

Implement costed physical paths, preview/confirm, multi-edge interruption, group
movement, per-member spent allowances, whole-formation and person transfers, and
creating an army by splitting at a shared site. No partial-headcount merging.
Implement recovery rate/cost/priority from P05, preserving veteran metadata.
Allow transfers between NPC actions while progression is visibly paused. Different
internal sites never count as co-location. Before K07, reject hostile entry with
an honest unavailable-encounter reason; do not claim victories or permit overlap.

**Exit:** neutral travel and composition work entirely by tap controls. Show spent
movement, supply and reasons. A cut route blocks normal replenishment. A prototype
enemy garrison cannot be walked through while combat is not yet installed.

**Five movement tests:** legal cheapest route/gate costs; blocked/over-budget
interruption; road benefit for both owners; co-location/slot/ownership transfers;
split/transfer/recruit cannot refresh allowances, including off-turn transfers.
**Five recovery tests:** supplied cap/cost; affordable partial rounding; stable
priority for scarce Gold; cut-off/deficit/dead formation rejection; veteran and
specialization preservation across losses/recovery/save. These are two distinct
major features, not ten tests for one trivial responsibility.

## K07 — Automatic battles, retreat and explainable reports

**Depends on:** K06. **Read:** chapter 05; O05/O06;
[P11–P13](combat-rules.md#p11--participation-and-strength), P15 damage.
**Files:** `src/engine/combat.rs`, `src/engine/retreat.rs`, `assets/data/troops.json`,
`assets/data/combat_rules.json`, `src/ui/battle.rs`, `tests/combat.rs`.

Implement the exact bounded simultaneous resolver, force multipliers, counters,
multi-army participation, persistent casualties, retreat and no-route destruction.
Apply limited commander wounds and formation-wipe person outcomes, including
P20's two supplied recovery steps now so prototype wounds can heal before K14's
full lifecycle work. Emit real
participant facts and lasting damage without granting progression twice. Respect
the existing projection boundary in reports. Fortified hostile entry remains
explicitly unavailable until K10; ordinary encounters are now real and playable.

**Exit:** recruit -> move -> fight -> retreat/recover works. Reports show known
causes and persistent outcomes and are read-only. Enemy survivors keep identity;
a killed formation never returns with its old veteran data.

**Five tests:** (1) P12 worked arithmetic and simultaneous casualties;
(2) counters/leadership/multiple defenders keep separate identities;
(3) rout, stalemate, legal retreat and encirclement terminate;
(4) wipeout/wound survival and appointment cleanup use deterministic RNG;
(5) save/report reopening cannot repeat casualties, control or evidence.

## K08 — Participation ledger, veterancy and enemy knowledge

**Depends on:** K07. **Read:** chapters 06/09/11; O10/O21/O23;
[P10](strategic-rules.md#p10--observation-and-report-filtering),
[P16](people-and-places.md#p16--participation-service-and-formation-veterancy), P23.
**Files:** `src/engine/knowledge.rs`, `src/engine/history.rs`, participation state,
initial `assets/data/history_rules.json`, `src/ui/history.rs`,
`tests/knowledge.rs`, `tests/evidence.rs`.

Persist actual participation counters and recent formation service summaries.
Add Seasoned/Veteran progression, exactly-once fact consumption, bounded event
history and a basic selected-entity history view. Implement presence-only precombat
visibility, battle observations, dated last-known facts and filtering across map,
reports, links and history. Do not reveal a hidden enemy biography through a
convenient raw-state inspector. Begin pruning now, before histories accumulate.

**Exit:** veteran forces explain their service; unknown enemies remain unknown;
revisiting reports does not award anything. K13 can consume genuine evidence
without reconstructing imaginary battles.

**Five knowledge tests:** precontact redaction; legitimate battle reveal;
last-known snapshots stay stale; linked views/filter/search respect visibility;
pruning/load preserves valid knowledge references.
**Five evidence tests:** actual participant selection; significance/cap/dedup;
earned veterancy survives recovery; destruction loses formation evidence;
narrative pruning retains gameplay facts and does not enable duplicate rewards.

## K09 — Persistent construction and world-based facilities

**Depends on:** K08. **Read:** chapters 03/04/07; O12;
[P06–P07](strategic-rules.md#p06--economy-recruitment-and-facilities),
P19 initial population/settler transfer.
**Files:** `src/engine/construction.rs`, construction/site state, JSON facility
definitions, `src/ui/settlement.rs`, `tests/construction.rs`.

Implement outpost, road, Fort, ordinary facilities, repairs and one active focus.
Add the minimum population state needed for conserved settlement founding (250 at
starting HQ; no full growth model until K11). Order costs/refunds/progress/pause
and capture cancellation follow P07. Build on fixed sites/routes. Supply snapshot
timing prevents just-completed work helping recovery early. Ordinary training,
mounted, medical and siege recruitment must respect built facilities.

**Exit:** a supplied builder can found an Outpost after three eligible boundaries;
roads and facilities have real effects and explain interruption. A focus label
does not claim growth before K11; show its active effects only when installed.

**Five tests:** (1) exact prepaid cost and first/third progress timing;
(2) pause/resume/builder replacement and capture cancellation;
(3) started/unstarted refund and duplicate-order rejection;
(4) completion conserves settlers, fixed graph IDs and one-time effects;
(5) roads/facilities enable stated actions for both factions with next-boundary
supply/recovery timing.

## K10 — Siege decisions and multi-army relief

**Depends on:** K09. **Read:** chapter 05; O13;
[P14](combat-rules.md#p14--persistent-siege-escape-and-relief).
**Files:** `src/state/siege.rs`, `src/engine/siege.rs`, combat context integration,
`src/ui/siege.rs`, `tests/siege.rs`.

Implement defended-fort arrival, actual inside/outside participants, once-seasonal
weakening, assault, maintain, withdraw, sortie, escape and incoming relief.
Garrison fallback versus field retreat must be explicit outcomes. Handle new
besiegers, third hostile factions, diplomacy cleanup and permanent fort damage.
Extend supply for isolated defenders and endpoint-supplied besiegers. No five-turn
automatic capture, starvation simulation or free incoming reinforcement.

**Exit:** every side's legal siege choices works from touch UI, persists across
round saves and shows known risks. Dense participant lists remain inspectable
without obscuring relief routes on the map.

**Five tests:** establishment/unoccupied fort capture; seasonal weakening and
saved progress; assault/engine advantage with lasting damage; sortie/escape and
no-route rejection; multi-army relief/third faction/withdrawal/peace cleanup.

## K11 — Living settlements, ordinary threats and refugees

**Depends on:** K10. **Read:** chapters 05/07;
[P15](combat-rules.md#p15--ordinary-threats-and-lasting-damage),
[P19](people-and-places.md#p19--development-occupation-refugees-and-capitals).
**Files:** `src/engine/development.rs`, `src/engine/threats.rs`,
`assets/data/development.json`, threat definitions, settlement inspector,
`tests/development.rs`, `tests/threats.rs`.

Implement layered habitation/fortification/condition, condition pressure,
geographic caps, natural growth, occupation normalization, ruin and reclamation.
Complete actual focus effects. Implement conserved aggregate migration/resettling,
capital move, HQ relocation, rename and stable references. Add ordinary threats
and single-use rewards through the real combat resolver; ruins offer new context
without a new node or repeated farming reward. Road/battle damage is applied by
its producer once, not again by development.

**Exit:** safe and contested places diverge for explainable reasons. Refugee
arrivals are conserved and can help another place. A lost HQ has an explicit
recovery route. No guaranteed upgrade button or required political rebellion.

**Five development tests:** sustained conditional growth/caps; decline/ruin/repair
without military-tier conflation; occupation and income/focus arithmetic;
migration/resettling conservation/capacity; rename/capital/HQ move identity and
single-root effects.
**Five threat tests:** normal encounter/reward; defeat leaves unresolved threat;
clearance/reload pays once; reclamation prerequisite and stable site;
new ruination spawns at most one eligible bandit event and no low-risk XP farm.

## K12 — Legal AI, diplomacy and an achievable kingdom ending

**Depends on:** K11. **Read:** chapter 03; D04/O07/O21;
[P08–P10](strategic-rules.md#p08--war-peace-defeat-and-vassal-outcome).
**Files:** `src/engine/ai.rs`, `src/engine/diplomacy.rs`, campaign status,
`src/ui/kingdom.rs`, `src/ui/campaign_end.rs`, `tests/ai.rs`, `tests/diplomacy.rs`.

Replace passive NPC phases with the bounded priority policy. Use legal recruitment,
movement, construction, threats, sieges, recovery and HQ relocation, with observer
knowledge. Add War/Peace/truce and withdrawal constraints, explicit surrender
outcome choice for a defeated rival, symmetric defeat predicate and victory screen.
Store lost-site diplomacy facts independently of pruned narratives. No alliances,
tribute economy or vassal resurrection. End screen offers history, save/menu/new
campaign, with no unsupported post-victory simulation.

**Exit:** finish a small kingdom campaign using only real actions; AI performs
legal expansion and war. The screen calls this the kingdom milestone, not complete
generational Kestrum. Update historical M1–M4 status accurately.

**Five AI tests:** legal affordable orders; same prerequisites and knowledge;
objective persistence/emergency; bounded rejected-command handling;
seeded complete-phase replay independent of frame scheduling.
**Five diplomacy tests:** War/Peace/truce legality; agreement withdrawal failure;
capital loss versus actual defeat; Annex versus player Submission and no revival;
reachable victory/player defeat/mutual destruction with stable turn skipping.

## K13 — Grounded emergence, traits and ordinary careers

**Depends on:** K12. **Read:** chapters 04/06;
[P17–P18](people-and-places.md#p17--emergence-disposition-traits-and-recognition).
**Files:** `src/engine/progression.rs`, person state, `assets/data/progression.json`,
reusable human naming data, `src/ui/character.rs`, `tests/progression.rs`.

Implement emergence soft curve/candidate selection, age-compatible retrospective
service, hidden disposition, evidence-based traits, recognition and missing-class
requirements. Add ordinary class courses, riding practice, appointments and the
three formation specializations. No advanced/fantasy paths in the UI. Track
course progress from actual eligible seasonal states. Extend AI to train and
appoint from the same evidence, not spawn increasingly powerful officers.

**Exit:** the player can trace a named person's emergence/class opportunity to
actual relevant service. Enemy survivors can become recurring known rivals.
Transfer/formation destruction remains coherent with person identity.

**Five tests:** roster-sensitive deterministic emergence without hard cap;
retrospective dates/presence exclude impossible deeds; meaningful distinct evidence
produces traits/recognition once; every ordinary class's valid/missing/interrupted
prerequisites in table-driven cases; specialization preserves identity/capacity and
never grants retroactive experience or dilutes on reinforcement.

## K14 — Aging, recovery, retirement and mentorship

**Depends on:** K13. **Read:** chapter 08;
[P20–P21](people-and-places.md#p20--age-injury-career-change-and-death).
**Files:** `src/engine/lifecycle.rs`, `src/engine/mentorship.rs`,
`assets/data/lifecycle.json`, character/role UI, `tests/lifecycle.rs`,
`tests/mentorship.rs`.

Implement valid birthday/service chronology, limited wound recovery, useful elder
command/site roles, retirement, governors and birthday death checks. Add qualified
mentor/learner assignments with contact, interruptions and ordinary discipline
evidence. AI moves useful elders into mentor/governor roles and trains a replacement.
Death/retirement release duties once; item handling comes in K16, with no phantom
items added before then. Preserve living and necessary historical identities.

**Exit:** a veteran contributes through mentorship or governance and a student
can qualify for a career. No automatic combat XP for aging or remote/dead teachers.

**Five lifecycle tests:** birthday/age and service bounds; wound recovery/supply;
age/role transitions and governor effects; one seeded mortality check per birthday;
dead/retired assignment cleanup and stale-command rejection.
**Five mentorship tests:** qualification/capacity; actual shared location;
pause/resume/death; discipline credit without encounter fabrication;
equivalent family-independent career opportunity for player and AI.

## K15 — Sparse households and several routes to succession

**Depends on:** K14. **Read:** chapter 08;
[P22](people-and-places.md#p22--households-children-service-entry-and-succession).
**Files:** `src/state/relationships.rs`, `src/engine/succession.rs`, household and
successor UI, `tests/succession.rs`.

Implement optional familiar partnerships, household child/ward context, age-based
trainee/service entry and local apprentices. Keep the vast majority of population
abstract. Add blood, martial, religious, political and adopted successor links
with category eligibility, no skill copying and no remote command inheritance.
Integrate cleanup on death, retirement, site capture and eliminated factions.
Local households are not diplomatic marriage or a breeding/stat interface.

**Exit:** demonstrate both a family-based successor and a no-children pupil or
officer successor. Failed succession leaves an honest vacancy, not a fabricated
qualified heir. AI can maintain continuity under the same constraints.

**Five tests:** chronology/kinship/partnership validity; birth/adoption/apprentice
idempotency and source population accounting; trainee-to-service ages/opportunities;
all successor categories preserve eligibility/identity and no skill inheritance;
non-blood death/retirement succession, invalid designation and eliminated-faction
cleanup produce no duplicate roles or resurrected kingdoms.

## K16 — Heirlooms, contextual history and eras

**Depends on:** K15. **Read:** chapters 08/09;
[P23](people-and-places.md#p23--items-institutional-memory-eras-and-retention).
**Files:** `src/state/legacy.rs`, history/succession services, biography/chronicle
UI, existing history JSON, `tests/legacy.rs`.

Introduce mundane founding items and physical custody/estate transfer. Preserve
students, family-seat links, service traditions and supported memorial context.
Complete filtered chronicles/biographies, bounded notable summaries, era labels
and non-repeating reminders. Share one underlying event across all views. Handle
missing old records honestly while keeping live relationship and progression
facts intact. Do not create an endless departed-person archive or a new loot game.

**Exit:** the player can follow a weapon, pupil, place and war across changes of
generation, including forgotten history. Reports and search respect intelligence.

**Five tests:** item identity/single custody and local collection; valid inheritance
and site estate fallback; event shared views grant no duplicate effects; retention
age/count boundaries preserve live relationships/earned capabilities;
eras/reminders use real facts and do not repeat or reveal hidden enemy history.

## K17 — The 80-node production campaign and content pass

**Depends on:** K16. **Read:** chapters 01/02/09/10; O01/O08/O24/O25;
[P01](strategic-rules.md#p01--setup-and-the-fixed-world).
**Files:** `assets/data/world_layout.json`, human names/emblems and ordinary content,
generation service, setup UI, atlas/stack/roster views, `tests/generation.rs`.

Author/review the 80-major/152-physical initial layout and instantiate it from a
seed with eight viable HQ candidates. Use only consumed ordinary human content;
keep source examples separate by explicit IDs. Add the actual 4–8 total faction
setup and switch normal New Campaign to production; preserve the durable small
scenario for regression/capture. No scenario cheat settings in production controls.
Complete valid terrain/facility/threat coverage and presentation text. Reuse the
atlas; node placements require visual review, not arbitrary random coordinates.

**Exit:** normal New Campaign creates the advertised world and all systems remain
usable with dense stacks/long labels at both sizes. No fixed 4-faction assumption,
hard-coded Rosemarch ID or six-people army survives in production rules.

**Five tests:** exact major/physical counts and fixed topology; seeded generation
replay; valid connected entrances/anchors/HQ spacing for 4–8 factions; no missing
ordinary class/facility/threat prerequisite references; valid setup errors and
production save round-trip with independent faction identities.

## K18 — Integrated campaign, balance and release acceptance

**Depends on:** K17. **Read:** all coverage rows and scenarios in
[acceptance](acceptance.md); chapters 10/12; UI_STYLE.md.
**Files:** only demonstrated fixes/balance data, `tests/campaign_scenarios.rs`,
durable capture fixtures, README and verification/status documents.

Run Rosemarch, encirclement/relief, long continuity and full ending scenarios.
Exercise 80-node games with both 4 and 8 factions for 200/400 rounds. Profile
round latency, retained entities/events, save size and slot-list behavior; record
hardware/browser and numbers rather than claim unmeasured performance. Tune JSON
without breaking confirmed rules or bypassing failing checks. Add regression
coverage for actual cross-system failures rather than duplicate all unit tests.

Complete Windows/fullscreen and actual WebGL 720p review, touch-only input including
text entry, overlays, long lists, pinch/drag, Full Screen and recovery. Resolve or
explicitly retain the shell's browser fullscreen/physical-touch limitations;
screenshots cannot close them by assertion. Run normal publisher from this checkout.

**Exit:** every required coverage row is done or has a user-accepted explicit scope
change; all known material blockers are recorded. Only then call the generational
campaign complete. Commit all project changes and report clean status.

**Five integration tests:** Rosemarch causal development; isolation/siege/relief
across save; 200-round non-blood continuity with layered place changes;
400-round retention/replay and state invariants for 4/8 factions;
reachable ending plus supported save-version round-trips. Manual UI/platform
review and measured performance are additional acceptance evidence, not unit tests.
