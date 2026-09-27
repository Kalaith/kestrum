# 13 — Decisions and open questions

[Documentation index](README.md) · [Original sources](README.md#complete-source-preservation)

## How to use this register

The discussion clarification and direct answers supplied on 2026-09-26 resolve several ambiguities left by the founding drafts. The latest answers settle all ten items previously listed as unanswered; narrower implementation details remain where identified below. This register distinguishes **confirmed decisions**, **delegated defaults**, and **agreed directions with mechanics or tuning still open**. Confirmed decisions guide implementation; an unresolved formula does not reopen the underlying design.

Original D and O identifiers are retained so earlier references remain useful. A **working interpretation** or **proposal** is still a suggested implementation choice. In particular, vassalisation as a defeat outcome is a suggestion from the discussion, not a confirmed rule. Approximate values remain tunable. Update the owning chapter when a remaining choice is settled; preserve the founding references unchanged.

## Confirmed decisions

| ID | Confirmed rule | Remaining implementation detail and primary chapter |
| --- | --- | --- |
| D01 | Each army has six discrete troop-formation slots. Headcounts vary by troop type, such as 100 warriors or one dragon. The earlier six-person company does not define army capacity. | Exact troop capacities and roster are content/tuning work; [armies](04-armies-and-logistics.md) |
| D02 | Ordinary formation members remain abstract until someone distinguishes themselves and becomes a visible character attached to the formation. A plausible recent history can be generated retrospectively; simulation from birth is unnecessary. | O10 settles participation-based evidence; emergence/recognition thresholds remain O14; sparse juniors or family records can still serve the generational design; [characters](06-character-development.md) |
| D03 | A complete round through all active factions advances one season; periodic effects resolve at its end. | The internal effect order remains an implementation proposal under O04; [calendar](02-world-time-and-control.md#faction-turns-and-the-calendar) |
| D05 | At zero headcount, the formation is destroyed and its formation history and veteran identity are gone. A replacement is a new formation. The wider world can still remember battles. | No separate destroyed-formation archive is required. Reinforcement of a surviving formation is O16; [formation persistence](04-armies-and-logistics.md#formation-persistence) |
| D06 | Major nodes and internal subnodes are generated at world creation and remain fixed. Settlements and other contents change on those sites; roads improve existing connections. | O01 sets 80 world nodes; regional density and generation constraints remain implementation details; [world graph](02-world-time-and-control.md) |
| O01 | The world map has 80 nodes. | Interpretation: 80 major world nodes, with regional subnodes additional; internal density is a generation detail; [world](02-world-time-and-control.md#strategic-graph) |
| O04 | Periodic effects resolve at the end of one full round. | Player then NPC turns remains the sequence; detailed internal ordering is proposed in [round-end resolution](11-simulation-and-data.md#round-end-resolution) |
| O06 | Armies can operate without named characters at substantially reduced effectiveness. Multiple named characters strengthen the same army, creating a choice between concentration and coverage. People can emerge within formations and later move into leadership. | Roughly 50% leaderless effectiveness is a tuning idea. Contribution formulas and appointments still need mechanics under O05/O14; O09 settles transfers; [named characters](04-armies-and-logistics.md#named-characters-as-force-multipliers) |
| O08 | The player chooses 4–8 total factions, including the player's kingdom. | This means 3–7 rivals; the setup default is not selected; [kingdom setup](03-kingdoms-and-economy.md#founding-a-kingdom) |
| O09 | Formations and named people can transfer any time their armies share the same physical node. | Baseline: no movement/time cost or safe-site requirement; serialize between atomic actions and preserve spent allowances; [transfers](04-armies-and-logistics.md#army-composition) |
| O10 | Development uses experience and encounter types the unit actually participated in. No dragon flying without ever seeing a dragon. | Compact participation facts persist independently of narrative history; exact tags and class thresholds are content/mechanics work; [eligibility](06-character-development.md#class-eligibility-and-opportunities) |
| O11 | Use reasonable provisional economy values stored in JSON, to be adjusted for balance later. | Delegated defaults for costs, income, upkeep, recovery, deficits, population pressure, and refunds are now in [economy.json](../assets/data/economy.json) and explained in [economy defaults](03-kingdoms-and-economy.md#provisional-economy-defaults); not loaded by the template yet |
| O16 | Reinforcement has no effect on a surviving formation's veterancy or specialization. | Veteran Warriors at 21/100 remain fully veteran at 100/100; zero-headcount destruction still applies; [persistence](04-armies-and-logistics.md#formation-persistence) |
| O22 | Autosave at the end of every round, no game-imposed save-slot limit, and manual saving during the player's turn. | Baseline: distinct automatic checkpoints and new manual slots, atomic saves, explicit overwrite/deletion; storage/version migration details remain implementation work; [saves](11-simulation-and-data.md#persistence) |
| O23 | Use reasonable bounded retention; the world may forget old stories and only remember a ruin's current state. | Delegated budgets bound narrative events and summaries while preserving current state and compact progression facts; [history policy](09-history-and-content.md#bounded-history-and-forgetting) |
| O24 | Start with humans and ordinary classes. Advanced classes and other races come later. | Provisional ordinary human roster is recorded in [content scope](09-history-and-content.md#content-authoring-guidance--proposal); names, art, and audio remain authoring work |
| O25 | Aim primarily for full-screen 1920 × 1080, with 1280 × 720 for WebGL. | Use 720p as the initial minimum landscape canvas; touch controls remain required, while smaller portrait support is deferred; [viewport targets](10-interface-and-accessibility.md#viewport-targets) |

## Agreed directions with mechanics or tuning still open

These are implementation questions within an established design. They should not be presented as wholly unanswered features.

| ID | Agreed direction | What remains open | Primary chapter |
| --- | --- | --- | --- |
| O02 | Armies have movement speed; roads increase it for defenders and invaders alike. | Movement points, terrain and road costs/bonuses, multi-edge travel, regional boundary costs | [Armies](04-armies-and-logistics.md#movement) |
| O03 | Cut-off armies cannot normally recover health or headcount. | Eligible supply sources, graph tracing, contested/neutral passage, and range; O04/O11 settle recovery timing and provisional cost | [Supply](04-armies-and-logistics.md#simple-supply) |
| O05 | Friendly armies can stack without a hard limit. Entering an enemy-occupied node triggers automatic combat based on composition, characters, and other relevant factors. | Resolver, leader contribution formulas, stalemates, participation and casualty allocation across multiple armies | [Battles](05-battles-and-sieges.md) |
| O07 | Eliminated factions are gone in v1. The victory objective is conquest or vassalisation of all other factions. | Capital versus settlement requirements, surviving armies, player defeat, and how vassalisation fits War/Peace diplomacy (D04) | [Victory](03-kingdoms-and-economy.md#victory-defeat-and-continuity) |
| O12 | Establishing an outpost takes roughly three turns. | Attack/interruption policy, builder presence, supply requirement, and first/completion progress timing; O11 supplies provisional costs/refunds, and the seasonal calendar implies about nine months | [Outposts](04-armies-and-logistics.md#outposts) |
| O13 | Defenders begin with an advantage that weakens during a siege. They can sortie, try to escape, or receive reinforcements; relief armies can attack besiegers. | Advantage and weakening formulas, joint versus ordered encounters, retreat destinations, and edge cases | [Sieges](05-battles-and-sieges.md#persistent-siege) |
| O14 | Hero emergence is semi-rare, easier with few named heroes, and progressively rarer toward roughly 20–30 per faction. Veteran formations and meaningful experiences improve opportunities. | Probability curve, recognition thresholds, roster counting/retirees, trait evidence and decay, anti-farming rules | [Characters](06-character-development.md#emergence-from-formations) |
| O15 | Experience and opportunity create class eligibility, supported by world resources and infrastructure. Dragon training and Teresa's egg illustrate future advanced content under O24. | Exact prerequisite expressions, training time, switching, and interrupted access | [Classes](06-character-development.md#class-eligibility-and-opportunities) |
| O17 | Typical campaigns span about 20–50 years, allowing aging careers, changed roles, and retirement. | Seasonal modifiers, aging checks, illness, death probabilities, and species-specific rates | [Generations](08-generations-and-succession.md) |
| O18 | Settlements grow or decline with conditions; occupation and refugees change places over time. | State representation, numerical pressures, growth limits, refugee routing, and stabilization | [Living places](07-living-places.md) |
| O19 | Capitals can move as places change in safety, wealth, connectivity, and political importance. | Relocation prerequisites/costs, administrative effects, and political ownership reversal | [Capitals](07-living-places.md#names-capitals-and-local-memory) |
| O20 | Families, children, mentorship, and several kinds of heirs carry continuity; legacy does not require children. | Household and succession rules, mentorship duration/capacity, inheritance, and property | [Succession](08-generations-and-succession.md) |
| O21 | Players have little to no enemy-army information before combat. AI follows the same underlying game rules; difficulty may increase gold/resource income rather than create impossible units. | Primarily how AI evaluates war, peace, and truces; exact difficulty bonuses and visibility remain tuning work. AI knowledge representation is an implementation detail, not permission to bypass the agreed rules. | [AI and intelligence](03-kingdoms-and-economy.md#ai-parity) |

## Status of the latest answers

All ten previously unanswered entries now have a confirmed rule or delegated default. O16 is resolved: replenishment does not dilute veterancy. None of those ten needs to be reopened as a whole.

Remaining work includes regional site density, the internal order of round-end effects, exact progression tags/thresholds, save migrations/storage, content authoring, and visual validation. The agreed-direction table still records combat, logistics, victory, and generational mechanics to define. D04's vassalisation proposal remains unresolved.

## Delegated defaults

O11 authorizes reasonable provisional economy choices in JSON, and O23 authorizes reasonable history limits. Those chosen baselines are recorded in the owning chapters. They are adjustable implementation defaults, distinct from the user's fixed rules. The current template does not implement either system.

## Remaining reconciliation and source-handling notes

D01, D02, D03, D05, and D06 are resolved above. The other original D identifiers retain their narrower purposes below.

| ID | Status | Current direction or interpretation | Remaining decision |
| --- | --- | --- | --- |
| D04 | Open scope reconciliation | Conquest/vassalisation victory and early War/Peace diplomacy both remain intended. The earlier conquest-only plan was an interpretation, not a settled restriction. | Evaluate the discussion's proposal to make vassalisation a defeat outcome without a full diplomacy system; define its timing and relationship to elimination under O07. |
| D07 | Agreed direction; tuning open | Emergence responds to the faction's named roster and becomes rarer toward roughly 20–30 heroes. This is a soft curve. | Define counted roles, active status, retirees, and probabilities under O14; historical dead figures should not block new generations. |
| D08 | Working interpretation of agreed settlement direction | Focus and investment influence pressure and opportunity instead of purchasing guaranteed tier changes. | Set pressure, costs, and time thresholds under O18. |
| D09 | Working interpretation of agreed capital direction | The political capital role is independent from physical settlement size. | Define relocation prerequisites and consequences under O19. |
| D10 | Working interpretation | Limited leadership-trigger wound events can prove the prototype before a full injury system. | Define initial wound consequences and recovery. |
| D11 | Agreed v1 boundary; survivor rules open | Personal or historical displacement does not automatically create a restoration or claimant faction. | Define surviving characters after elimination under O07. |
| D12 | Agreed duration target; confirmed calendar boundary | About three turns to establish an outpost means three seasonal progress steps, roughly nine months, under D03. | Playtest pacing and define first progress/interruption rules under O12. |
| D13 | Future class content; expression open | Dragon opportunities require suitable experience and infrastructure; egg, drake, mentor, stable, and hatchery examples provide candidate requirements. | Decide mandatory versus alternative prerequisites under O15 when advanced classes enter scope; O24 defers them. |
| D14 | Agreed initial supply rule | Loss of supply blocks normal recovery. Detailed attrition and civilian effects are separate staged possibilities. | Select any later ongoing loss model explicitly. |
| D15 | Source-handling convention | Independent example dates and similar names remain separate scenarios. | Assign explicit IDs to actual content; do not implicitly merge Thomas/Tomas or High Fort/Hawthorn examples. |

## Fixed choices and tuning targets

| Topic | Current rule or target | Status |
| --- | --- | --- |
| World size | 80 major world nodes; regional subnodes additional under the documented interpretation | Confirmed node count O01 |
| Viewports | 1920 × 1080 full screen; 1280 × 720 WebGL | Confirmed O25; initial minimum landscape 720p |
| Initial content | Humans and ordinary classes | Confirmed O24; advanced classes and races later |
| Campaign factions | Player chooses 4–8 total, including the player | Confirmed O08; default unspecified |
| Army slots | Six discrete troop formations | Confirmed D01; exact headcounts vary |
| Formation scale | Initial human capacities and costs in economy.json; fantasy examples remain in source references | Provisional balance under O11/O24 |
| Leaderless army | Roughly 50% effectiveness | Tuning idea; formula unresolved |
| Important roster | Emergence increasingly rare toward roughly 20–30 named heroes per faction | Agreed soft curve; counted roles and probabilities open |
| Outpost build | About three turns | Agreed approximate duration; timing/interruption unresolved |
| Strategic calendar | One season per strategic turn, four per year | Confirmed D03/O04 full-round boundary; internal effect order remains proposed |
| Campaign duration | Typically 20–50 years, potentially 60–100 | Experience target, no forced timer |
| Prototype region | About 8–12 nodes, two entrances, capital and fort | Initial test scale, not world size |
| Early characters | Three founders and six juniors in the fifty-year example | Illustration, not fixed setup |
| Age bands | 0–12, 13–16, 17–25, 26–40, 41–55, 56+ | Flexible by culture/species |
| Siege weakening | Five illustrative steps from prepared walls to vulnerability | No mandated five-turn victory |
| Stack count | No hard limit; no planned direct penalty in v1 | Agreed direction; battle participation and resolution open |

## Added proposals to evaluate

The source drafts and answers do not settle a Rust entity schema, action transaction model, exact internal order of round-end effects, graph boundary contract, UI screen layout, save migration coverage, or automated test plan. Chapters 10–12 and marked sections elsewhere propose those details to make the design implementable. Validate them against actual toolkit capabilities and prototype behavior.

A confirmed design rule is not evidence of implemented or tested gameplay. Track implementation and verification separately, and keep remaining mechanics explicit without reopening settled choices.

## Implementation-plan defaults

On 2026-09-26 the user requested an implementation handoff suitable for a smaller
model and chose **“Specify concrete provisional defaults, clearly labelled for
review”** for the remaining mechanics. The [implementation plan](implementation-plan.md)
and its P01–P23 rules now supply those defaults. They are executable planning
choices, not newly confirmed author rules and not implemented gameplay. The
earlier “open” entries above retain their provenance; the mapping below identifies
the current provisional answer. A future agent follows these defaults unless a
review explicitly changes them, rather than inventing replacements.

| Previously open mechanics | Current provisional answer location |
| --- | --- |
| O01/O08 regional density, setup default and starts | [P01](implementation/strategic-rules.md#p01--setup-and-the-fixed-world) |
| O04 boundary effect order, event dates and first progress | [P02](implementation/strategic-rules.md#p02--faction-phases-and-round-order) |
| Anchor reversal, route scale and physical location | [P03](implementation/strategic-rules.md#p03--control-and-nested-boundaries) |
| O02/O09 movement, road cost and transfer bookkeeping | [P04](implementation/strategic-rules.md#p04--armies-movement-and-transfers) |
| O03/D14 roots, contested access and recovery | [P05](implementation/strategic-rules.md#p05--supply-and-recovery) |
| O11 local recruitment/facility access and added costs | [P06](implementation/strategic-rules.md#p06--economy-recruitment-and-facilities) |
| O12/D12 builder, supply, interruption and refunds | [P07](implementation/strategic-rules.md#p07--construction-roads-and-focus-orders) |
| D04/O07/D11 vassal outcome, exact defeat, survivors | [P08](implementation/strategic-rules.md#p08--war-peace-defeat-and-vassal-outcome) |
| O21 AI war/peace, knowledge and initial difficulty | [P08–P10](implementation/strategic-rules.md#p08--war-peace-defeat-and-vassal-outcome) |
| O05/O06 combat, leadership, stacking, retreat, survival and D10 wounds | [P11–P13](implementation/combat-rules.md#p11--participation-and-strength) |
| O13 siege advantage, relief, escape and third parties | [P14](implementation/combat-rules.md#p14--persistent-siege-escape-and-relief) |
| Local threats, damage and repeat-reward prevention | [P15](implementation/combat-rules.md#p15--ordinary-threats-and-lasting-damage) |
| O10/O14/D07 participation, emergence curve and recognition | [P16–P17](implementation/people-and-places.md#p16--participation-service-and-formation-veterancy) |
| O15/O24/D13 ordinary classes/training and later fantasy boundary | [P18](implementation/people-and-places.md#p18--ordinary-training-and-formation-specialization) |
| O18/D08/O19/D09 growth, refugees, capital and HQ recovery | [P19](implementation/people-and-places.md#p19--development-occupation-refugees-and-capitals) |
| O17 age, injury, retirement and mortality | [P20](implementation/people-and-places.md#p20--age-injury-career-change-and-death) |
| O20 mentoring, families, heirs and property | [P21–P22](implementation/people-and-places.md#p21--mentorship-and-useful-non-blood-continuity) |
| O23/D15 legacy, factual eras and retention integration | [P23](implementation/people-and-places.md#p23--items-institutional-memory-eras-and-retention) |
| O22 catalogue/storage recovery and save compatibility | [C06](implementation/contracts.md#c06--save-catalogue-and-compatibility) |
| O25 viewport/touch acceptance and existing limitations | [Acceptance](implementation/acceptance.md#visual-interaction-and-performance-evidence) |

The existing fixed rules, O11 economy table and O23 memory budgets are preserved.
Future advanced classes/races, richer diplomacy, restoration and multiplayer stay
outside this plan. Generations, families, non-blood mentorship/succession, refugees
and changing capitals have explicit packages; they have not been removed from the
game merely because the kingdom prototype precedes them.

## Full-release delegated implementation decisions

The 2026-09-26 full-release assignment authorizes implementation decisions and
automatic progression through K02–K18. These decisions remain delegated choices,
not newly confirmed author rules. Package evidence records their validation.

### I01 — Visible faction pacing and the old atlas

K02 follows P02's player-first seasonal round. The application presents one NPC
pass every 1.2 seconds, with Pause, Step and Resume controls. This delay affects
presentation only; the engine advances solely through atomic commands. Opening
an overlay or showing an error holds automatic progression. Step advances one
paused faction and preserves the pause for the following faction.

The existing v1 slot remains read-only and is labelled as an empty-atlas campaign.
K02 uses a separate v2 strategic checkpoint until K03 introduces its recoverable
catalogue. This preserves old bytes without inventing a strategic past for the
shell. K03 replaces the interim single strategic slot with unlimited named saves
and distinct round checkpoints; it is a required dependency, not deferred scope.

Affected contracts: C03, C04, C06; P02. Validation: K02 phase/save regressions and
the UI review recorded in `docs/verification/k02-campaign.md` when completed.

### I02 — Save identities, writer ownership and naming

K03 reserves campaign and save-request identities from the storage catalogue,
independent of simulation RNG. Each completed boundary reserves a new request:
loading an older save and reaching the same round can create a different timeline.
A failed write retains that request and its immutable payload for Retry. The
initial founding moment is a named manual save; later boundaries are checkpoints.
Requests commit in preparation order. An older uncommitted request is rejected
after a newer one commits, and an overwrite records the target revision it saw.
This prevents a delayed request from replacing newer work. A committed request
can still be retried idempotently. The application keeps one pending request.

One application holds the namespace's writer lease, through an OS file lock on
native or Web Locks in the browser. Other windows can discover committed entries
and show Retry Storage; writes wait until they acquire the lease. Browser storage
that cannot provide a writer lock reports its limitation instead of claiming a
successful save. This is an implementation choice for C06 serialization.
Validated loads remain available when storage cannot write the Continue choice;
the game opens the candidate and shows a warning while preserving the previous
persisted choice. A deliberate older-save load acknowledges the known catalogue,
so later journal cleanup does not silently choose a newer campaign instead.

Save names accept 1–40 Unicode characters without control characters. They are
labels only. A shared on-screen keyboard supplies letters, numbers, punctuation,
Space, Backspace and Clear; physical typing is supplemental. The earlier K02 slot
has an explicit import action that gives its copy a new catalogue campaign ID
and preserves the source. Empty-atlas saves remain read-only.

These are delegated implementation choices under the full-release assignment.
The K03 evidence records storage, compatibility and UI validation.

### I03 — Geographic inspection and additive v2 claims

K04 keeps map selection and remembered world/regional cameras in presentation
state. Loading or starting a campaign resets them; inspecting a site never changes
simulation state. A single inspector sits opposite the selected marker. The
48-pixel targets share the renderer's projected centers, and release selection
requires the same target with movement no greater than the toolkit drag threshold.

Earlier v2 saves lack both regional claims and contested-site fields. When both
are absent, migration initializes them and derives the first claim from saved
site control and HQ supply. Partial or explicitly invalid new fields are rejected.
Schema 2/content 1 and existing catalogue metadata remain valid; loading does not
overwrite the earlier bytes. No past claim is fabricated from missing history.

Rosemarch's mandatory city and fort permit only one qualifying owner. If future
authored alternative anchors allow simultaneous qualifiers, a qualifying incumbent
keeps the claim; otherwise the lowest qualifying faction ID resolves the tie.
This deterministic choice does not change local controllers or supply.

These delegated choices implement C02/C06/C07 and P03/P05. Validation and the
remaining platform limitations are recorded in `docs/verification/k04-geography.md`.

### I04 — Earlier military saves and recruitment receipts

K05 gives founding armies and Officers only to new campaigns. Earlier v2 saves
never simulated military service, so an absent military group migrates to empty
army, formation and person collections with fresh ID counters. The saved date,
resources and RNG remain unchanged; Recruit can establish an army at a legal
owned site. No founder, past income or service is invented. Partial new groups
remain invalid. Schema 2/content 1 and catalogue metadata remain compatible,
and loading leaves source bytes intact.

Current founder records contain only identity, birth/service dates, Officer class,
assignment and spent movement. Later career packages extend them from real
evidence. Disbanding retains named people at the same site or in a surviving
friendly formation, removes empty armies, never refunds costs and never reuses
destroyed IDs. A last-round economy statement records actual income, paid/due
upkeep, shortfall and closing resources; it does not promise future income.

These delegated choices implement C02/C03/C06 and P04–P06/P11. The K05 evidence
records recruitment, upkeep, migration and UI validation. Movement, transfers
and headcount recovery retain their scheduled K06 dependency.

### I05 — Route interruptions, appointments and recovery receipts

K06 finds the cheapest usable physical route with complete site-ID paths breaking
equal-cost ties. When no usable route reaches the destination, the preview shows
the physical route and its first public obstruction. Confirmation retains that
exact path. An obstruction at its first edge rejects without changing state;
after a legal prefix, the command commits only that prefix and reports the stop.
Before K07, hostile or fortified-neutral entry reports that an encounter is
unavailable and stops before entry. No victory, casualty or occupation is invented.

Transferring a whole formation carries its attached people. A moved commander
keeps the appointment only when the destination has no commander; an existing
destination appointment stays in place. Splitting carries that appointment to the
new army. The source remains vacant instead of inventing a promotion. Opening
Armies during rival phases explicitly pauses between atomic orders; transfers
are legal while paused, and Resume stays a visible map action.

Recovery snapshots current physical supply before economy and uses the resulting
post-upkeep Gold in faction/army/slot order. Only headcount and the exact Gold cost
change. A dated receipt retains historical troop kind/capacity so disbanding a
recovered formation cannot invalidate the save. Missing earlier recovery receipts
load as absent; no prior recovery is fabricated. Forecasts assume current control,
troops and balances remain until the boundary. Actual siege exclusions join the
shared supply query with K10's siege state.

These delegated choices implement C03/C04/C06/C07 and P03–P05. K06 evidence records
the acceptance cases, interactive movement/composition and platform limitations.

### I06 — Encounter observations, terrain and limited wounds

K07 implements P11–P13's bounded automatic field resolver with integer arithmetic.
An encounter ends the travelling group's order and exhausts its actual participants.
Other armies at the origin do not join. The report stores the observed participant
snapshot and final result; only the two participating factions can read it. A
BattleResolved fact identifies that committed encounter once. Reports never read
later enemy changes or reapply the result. K08 consumes this real participation
evidence for progression and bounded observation retention.
Each exchange recalculates leadership from fit adults still attached to a living
formation. Losing their host formation removes that contribution from later
exchanges; post-battle wound rolls do not change attacks already resolved. Reports
retain those exchange factors and each person's original participating formation.
Only an attacker victory captures the target. Defender victory or stalemate
preserves its previous controller, including a neutral site; mutual destruction
clears it. Defending an unclaimed site therefore grants no incidental conquest.

Bridge and Pass are explicit site tags, separate from broad geography. River
settlements do not all receive the bridge modifier. The shipped old Rosemarch
bridge gains its missing Bridge tag only through the grouped pre-combat migration.
Current saves validate the authored tags. Absent battle collection, battle counter
and occupation map together migrate to empty records and a fresh counter; partial
new groups are rejected. No earlier encounters or occupations are invented.

Peaceful armies may share a neutral site, as P08 permits. Such an arrival pays its
edge but leaves the site neutral: another peaceful force prevents an unopposed
capture. Peace-controlled foreign sites still deny access. Mixed peaceful and
hostile foreign occupants reject entry before an unsupported three-party battle;
the rejection reveals no names, counts or composition. Supply forecasts are
conditional on securing the destination, since hidden occupants can prevent it.

People gain Fit, Wounded and Dead states. Missing earlier status means Fit because
those versions had no injury or death simulation. Death releases the assignment
and command while retaining a factual dated person record for later history.
Wounded people can remain attached or move through legal local transfers, but do
not contribute field leadership. Two eligible supplied boundaries heal them. The
formation-wipe and limited commander rolls use the combat RNG and actual encounter
participants; no automatic founder immunity or replacement formation is granted.
For a side with several commanders, select the lowest person ID whose surviving
formation meets the loss threshold after combat. Roll at most once for that side,
in faction-ID order after the globally ordered formation-wipe rolls. An undamaged
lower-ID commander therefore does not suppress another eligible commander's check.

The Menu's redundant Resume entry is replaced by Battle Reports; its existing Back
control returns to play. The result opens automatically after a player encounter,
and reopening it remains read-only. These are delegated implementation choices,
not new author-confirmed rules. K07 verification records the acceptance
results and platform limitations.

### I07 — Genuine service, bounded histories and dated knowledge

K08 consumes committed facts at the seasonal progression boundary, once in stable
fact order. Formation and personal progression credit is deduplicated by physical
site, opposing faction and round. The first meaningful eligible encounter earns
credit; an earlier trivial contact does not prevent later qualifying service.
Witnessed troop-type facts still describe actual contact without granting generic
XP. Formation XP is capped at four per round and only surviving formations retain
it. Each person's host formation is the recorded starting formation, so a later
transfer cannot inherit another formation's deeds.

Movement receipts also retain the actual travelling formations and people before
any later reassignment. Successful physical edges supply distinct route evidence,
including the approach to a battle. An old receipt without that snapshot grants
no reconstructed route service. Victories against a retreating enemy retain their
own factual counter for P18's later Light Cavalry requirement. Neither record
grants a class, specialization or extra XP in K08.

Strategic anchor evidence uses actual sites in region anchor expressions,
including entrance supply alternatives. Successful defense requires the defending
faction's prior control to remain after the encounter. Capture requires a genuine
change to the attacking faction's control. Neither standing at an anchor nor
reading a report creates evidence. Ordinary named people physically present can
witness an encounter while wounded; P11 separately controls whether they contribute
field strength. K13 applies its own stated fitness and age requirements to courses.

Surviving people actually serving in a participating Medics formation can record
supervised treatment when their side suffers real casualties. This supplies P18's
entry requirement for an initial Medic course; requiring an existing Medic class
for every personal treatment fact would make that route circular. A remote person
or someone who died in the encounter receives no treatment credit. Recovery-site
treatment requires an assigned qualified Medic and actual restored headcount.
Combat treatment also requires recorded fitness at entry. A wounded passenger can
witness the battle without providing medical service. Earlier reports lack that
starting-status observation, so they cannot establish treatment eligibility.

Earlier K07 saves begin new progression counters at zero. Retained genuine battle
reports can seed dated narrative and knowledge records, but consumed facts are
never rewarded again. Still-pending facts remain eligible for the next boundary.
No absent pre-combat history is fabricated. Destroyed formations retain no service
archive or recoverable veteran identity.

Detailed history and linked battle reports share the 40-round/10,000-event budget.
Independent notable summaries retain up to twelve entries per extant subject for
80 rounds. Enemy-person observations expire after 80 rounds, with at most 10,000
across the campaign, ordered by encounter date and stable IDs. They are never
removed or updated because of an unseen death, move, promotion or rename. Retained
labels keep history understandable after a detail link expires. These observation
limits extend the existing P23 budgets as a delegated implementation decision.

P23's departed-person budget already applies to recorded deaths: retain their full
records for at most 80 rounds and at most 2,000 across the campaign, ordered by
death date and stable ID. Required pending receipts remain valid through seasonal
consumption. Battle, history and knowledge snapshots keep their own minimal labels.
K14 and K15 extend this cleanup for lifecycle and family references when those
systems exist.

Presence reveals only that a hostile force is at a friendly army's site or an
adjacent site. Enemy links open the last actual encounter's allowed facts, with
its date and location; they never inspect current remote rosters, resources,
biographies or families. UI history queries use the same observer filtering.

### I08 — Persistent work and conserved settlers

K09 keeps one open construction order per physical site or route. Active and
paused orders reserve their field builder; reassignment releases the old builder.
Facilities need a local builder to place the order and then release that army,
following P07's explicit exception for ongoing facility work. Terminal orders
cannot restart. Only the newest terminal receipt per target is retained alongside
its current open order; dated construction narratives use the existing history
budget. Gameplay completion therefore never depends on keeping an old story.

P19 says an Outpost draws **up to** fifty settlers. The implementation uses a
positive available surplus from the nearest eligible supplied friendly inhabited
site, ordered by legal route cost and then site ID. The donor keeps at least its
current habitation minimum. Existing Camp population stays at the destination;
arrivals add the same amount deducted from the donor. A smaller surplus can found
a smaller Outpost. With no eligible surplus, the final step pauses and cannot
manufacture people through repeated completion or reload. Initial headquarters
have 250 population and other sites their authored habitation's initial amount.

Focus selection records one priority and replaces the previous choice. K09 grants
no focus bonus; K11 installs P19's development, income and repair effects together.
The interface states the currently available effect. K09's paid repair is the
one-step improved-road repair from P07. Structural and fort repair remain P19's
K11 seasonal development behavior. This preserves package dependencies without
inventing an additional structural-repair purchase or promising passive growth.

Construction and focus changes create immutable, owner-visible history receipts.
Road records concern both physical endpoints. Completed work contributes a bounded
notable summary. Records preserve the builder and place labels available when the
action happened, without revealing another faction's private work orders.
Exact population and the selected focus are also private to the controlling
faction. Public habitation, geography and built site layers keep their existing
visibility; an unknown population is not displayed as zero.

Settler eligibility, available donor surplus and route costs are frozen before
construction progresses. Orders reserve that initial surplus in stable order-ID
order. Arrivals and newly completed roads cannot make a second founding order
eligible at the same boundary. This applies P02's snapshot timing and prevents
new settlers being moved repeatedly through a chain of completing Outposts.

These are delegated implementation decisions, not newly confirmed author rules.
[K09 verification](verification/k09-construction.md) records 81 passing tests,
native visual review, browser construction and reload, publication, and the
remaining platform limitations.

### I09 — Explicit siege outcomes and real participants

The P14 Escape table originally permitted escape on stalemate, while the next
paragraph explicitly returned a stalemated sortie or escape to the garrison.
K10 follows that explicit shared outcome rule: only victory reaches the chosen
escape destination. Defeat or stalemate leaves surviving selected defenders
inside; destroyed formations stay destroyed. The preview and report explain
this risk. This resolves a contradiction without adding a second fallback battle.

All surviving, nonempty original garrison armies join a relief encounter,
including armies that spent their movement earlier in the round. This uses P12's
rule that exhaustion never prevents defense. Wounded named people remain with
their formations but supply no P11 field leadership. Incoming relief and original
garrison identities remain distinct, so failure can retreat the former and keep
the latter inside without restoring casualties or granting garrison travel credit.

A failed or stalemated assault returns its surviving participants to their
existing outside camp. A successful escape moves the selected survivors to the
chosen exit; surviving besiegers retain their camp. If no garrison remains,
normal siege reconciliation gives the site to the besieger. These explicit
location outcomes distinguish taking a fort, breaking the siege, and escaping it.
All still pay the actual casualties and exhaust participants for the round.

P15's damaged-road threshold uses destroyed defender base power from both combat
and encirclement losses. Both remove actual formations or headcount in the assault
outcome; counting starting minus surviving power follows the stated destruction
threshold. The report keeps these casualty causes separate.

New Rosemarch prototype campaigns give Hawthorn Headquarters a Fort layer while
keeping its Village habitation and authored founding army. This provides a
reachable defended fort while the rival turn policy still passes before K12.
The empty High Fort remains a separate immediate-capture case. Existing saves
retain their saved layers; no fort or garrison is retroactively added. This is
an authored prototype challenge, not an additional production-faction grant.

A live siege requires two independent factions. Elimination or submission must
resolve that faction's participating armies in the same transaction before siege
reconciliation. A status-only change with unresolved armies is rejected; it cannot
leave an inactive faction fighting indefinitely or manufacture a withdrawal.
After military cleanup, reconciliation uses the actual remaining occupants to
lift the siege or transfer control. Historical receipts remain valid after either
faction ceases to be independent.

These are delegated implementation decisions. [K10 verification](verification/k10-sieges.md)
records 96 passing tests, publication and the remaining platform checks.

### I10 — Conserved population and local threats

K11 freezes seasonal conditions before construction and recovery, then reads
population after construction has applied its settlers. Natural changes are
planned together; migration uses fixed departure and arrival budgets. A newly
founded or reclaimed place first receives natural development at the following
boundary. This preserves P02 ordering without restoring spent settlers or moving
the same arrivals repeatedly. Civil identities describe existing geography,
facilities and roles; they do not introduce another progression currency.

A lawless ruin has no nonempty local army, functional fort or active ordinary
threat. A nominal claim alone does not police
an abandoned ruin. Eight consecutive eligible boundaries can create one Bandit
threat for that ruination. Each site keeps a monotonic ruination identity and a
creation flag independently of narrative retention. Later ruination may replace
an older cleared spawned record; historical reports keep weak threat IDs. Initial
authored occupants persist and cannot respawn from forgetting a report.

Threats have their own identities and a typed battle side. They use the shared
exchange arithmetic without creating a faction, army, formation or enemy person.
They cannot retreat; routed remnants are recorded as encirclement losses. Mutual
destruction clears the occupant with no reward. Actual survivors receive ordinary
significance-gated participation evidence, with distinct Bandit and Wildlife
encounter facts. Active threats block supply and civilian passage through even a
claimed site, as well as ordinary entry, recruitment and construction.

New campaigns place Bandits at Ruined Hold and Wildlife at the Hill. Earlier
strategic saves receive neutral current development values and no retroactive
occupants, growth, rewards or facts. Earlier faction battle reports retain their
actual sides in the new typed representation. Partial compatibility groups are
rejected. Older economy statements explicitly lack site inputs; new statements
save the operands and final one-floor income calculation from their boundary.

Place tier, ruin, rename and role changes create public dated place records.
Population movements remain private to their owner. Records retain the place
labels from their date; renaming never changes stable identities or old prose.
These are delegated implementation decisions, not newly confirmed author rules.

Current settlement forecasts require own-army observation of the site and its
neighbors. Without that coverage the current safety and pressure contribution
are unknown; actual saved pressure remains visible. Resettlement suggestions use
observed destinations. The engine still applies the real physical conditions to
an actual submitted order and to seasonal simulation. Forecasts do not expose
unseen enemy movement by changing a remote site's safety label.


### I11 — Bounded sovereign decisions and lasting outcomes

K12 uses one accepted ordinary command per automatic or paused NPC step. The
presentation delay does not enter simulation. A pure planner proposes an intent;
the engine applies it and its objective metadata in one candidate. Rejected
intents are bounded and the phase passes instead of repeating a failed command.
A phase accepts at most 64 strategic commands, then passes. Objectives last four
rounds unless invalid or an emergency overrides them. Each movement order takes
one physical edge so the next decision uses newly observed surroundings.

AI strength comparisons use its own current army and surviving enemy formations
from retained, actually witnessed battle reports. A presence marker supplies no
invented headcount. Observed empty enemy places remain legal capture targets.
Private resources and rosters do not enter travel or targeting. P08's explicit
peace evaluation alone uses aggregate enemy strength: the sum of headcount divided
by each formation's capacity, measured in full-formation equivalents. This avoids
treating one full formation as equal to six. Data validation bounds the exact
common denominator so the comparison cannot overflow. Losing any inhabited,
non-ruined place can motivate peace; surviving military independence requires a
functioning Outpost or higher, or a nonempty army. A ruined former town does not
provide that base.

Pending player peace and defeat decisions freeze further orders and calendar
advancement and form a stable manual-save boundary, even during an NPC phase. Once resolved, the same unfinished faction phase resumes, skipping
newly inactive kingdoms. Peace preplans every required adjacent withdrawal and
fails atomically if any force lacks a lawful exit. It does not teleport forces,
transfer civilian control or erase survivors. Public agreements and outcomes
produce public records; offers and rejections belong to the two parties, and
withdrawal rosters remain private to their owner.

Military defeat immediately displaces living survivors and cancels unfinished
orders. The later Annex/Submission decision cannot restore their military or
create free recruits. Vassals of a fallen sovereign follow its surviving victor;
without one, they become inactive. Residual minor claims do not revive a kingdom
or generate independent income. Simultaneous destruction resolves player defeat
before checking victory.

Ending a campaign consumes actual pending participation receipts once, including
the final battle, without adding a fictional full round, income or healing step.
The terminal partial-season service record can carry the current round. The
persisted result blocks simulation while records, named saves, menu and a new
campaign remain available. A terminal checkpoint is allowed even before the
first seasonal boundary. Earlier payloads missing the whole diplomacy/AI group
receive relation timers from their saved current state and empty policy history;
partial modern groups remain invalid.

These are delegated implementation decisions under the release assignment.
K12's controls use one dismissible kingdom decision view and a result view;
the map remains the ordinary play area. No permanent diplomacy dashboard is added.
Validation and publication results belong in the K12 verification record.

### I12 — Grounded careers and specialization

K13 applies the P17 emergence formula, candidate ordering, thresholds and P18
course requirements as written. A new person receives a stable ID, authored human
name, Recruit class, source-formation link and service start bounded by the latest
of formation creation, the eight-round history window and age seventeen. The
person occupies existing formation headcount. Career evidence is personal and
bounded; transferred people keep their own deeds but do not inherit formation
history. Recognition is factual and applied once. Traits change eligibility and
context only; they do not add undocumented combat strength.

Disposition is sampled once per tracked person from the campaign's people RNG.
Starting founders receive the same treatment. To migrate K12 payloads, missing
career records receive one deterministic disposition in ascending PersonId order;
the saved people RNG is advanced accordingly. A payload containing only some
people's career records is rejected as a mixed schema rather than guessed at.
Serialization and replay tests pin the current-schema boundary; migration tests
pin deterministic repeated loads.

Class eligibility and specialization options are shared between player UI and NPC
planning. NPCs propose the same validated training, riding, specialization and
commander commands; there is no officer-spawning shortcut. Course steps advance
only at eligible completed seasons. Cancellation returns Gold only before any
progress, consistent with construction. Light Cavalry uses its nine-point movement
allowance consistently in travel, combat exhaustion and siege reset paths.

Relationships record actual joint service and repeated mutual combat for living
tracked people, symmetrically and with a bounded ledger. Two mutual combats justify
the "Rival" label; the relationship adds no damage bonus or forced revenge. Service
links and career history remain factual labels, not inferred kinship. Native Career
and Formation Training screens show costs, facilities, prerequisites and action
availability with 48-pixel controls at both supported native sizes.

These are delegated implementation decisions, not newly confirmed author rules.
P18's direct evidence routes ship in K13. Its mentorship alternatives require the
qualified mentorship system assigned to K14; K14 must integrate those alternatives
into the same career-option rules and test them rather than marking them out of
scope. K13 verification records nine progression cases, the complete test suite,
native captures and publication.

### I13 — Aging, local recovery and mentorship

K14 implements the P20–P21 thresholds and keeps their effects in the real round,
combat, supply, development, transfer and progression paths. Ages advance on
four-season birthdays. Between 41 and 55, Light Cavalry personal movement is
capped at eight. At 56, an elder contributes to field leadership only when named
as commander; an existing command stays assigned until the player or rival
reassigns it. Manual retirement is available to a living adult. At 70, an
unretired field person moves to the nearest owned, unbesieged inhabited site by
graph distance; absent such a refuge, the person becomes retired and displaced
at their current site. Mortality is 2% at 60–69, 5% at 70–79, 15% at 80–89
and 35% at 90+, checked on birthdays in stable PersonId order. Aging produces
no combat experience.

Wounds heal through two eligible supplied, nonbesieged boundaries. A Medic earns
treatment evidence only when present at the real site of recovery. Mentors must
be age 26 or older, class/trait-qualified and have four service seasons in the
discipline. Learners may begin at 13 but remain site-assigned until age 17. Each
person teaches or studies one discipline link at a time. Lessons require shared
physical contact at an owned, supplied, safe, developed site with its required
facility; Riding also requires Horse Access. Wounds, separation, capture or loss
of qualification/access pause the link. Ending a paused link releases its slot.
Two completed seasons open a P18 class alternative; four complete a dated
mentorship record. Neither mentorship nor aging adds combat or encounter facts.

AI uses the same opportunity checks, commands and capacity limit when appointing
governors or planning lessons. New K14 person fields default safely when older
K13 campaign payloads omit them; they do not create relatives, items or historic
events. These implementation choices follow the documented provisional rules
and are delegated decisions, not newly confirmed author decisions. K14's
regressions and validation are recorded in
[the lifecycle verification](verification/k14-lifecycle.md).

### I15 — Sparse households and several routes to succession

K15 follows P22's age, season, population, and category rules. The validated
Form Household command is both adults' explicit acceptance; no extra marriage
confirmation or gender/genetics model is introduced. Partners may be wounded but
must be alive and not retired. A safe site is friendly, inhabited, nonbesieged,
without an active threat or battle, and without an at-war army at or beside it.

Raise Children makes one 20% attempt at each eligible Spring in stable household
ID order, using the people RNG once per attempt. Eight seasons separate births;
adopted wards share the two-dependent cap. Adoption names one adult guardian,
costs no Gold, and deducts exactly one unit from local population. A 13-year-old
dependent may train locally; at 17 a Recruit may enter local site or formation
service. A separate Village-or-larger apprentice costs 20 Gold and one population
unit, and is limited to one per faction per year while the faction has no
dependents. Family origin, not inherited statistics, connects each person.

Blood means biological ancestry; Adopted is the recorded guardian/ward link;
Martial is a real mentorship; Religious reuses that mentorship at a Temple;
Political requires four shared-service seasons and a local site role. Institutional
succession records an existing site. Command successors must already be fit adult
members of the named army. On vacancy, an eligible designation wins, then the
oldest local pupil, then highest command evidence and lowest PersonId; no candidate
leaves the army honestly vacant. K15 exposes an Item designation seam, while K16
owns item custody and estate transfer. Death, capture, retirement and defeat close
or prune only current duties; they do not teleport people, copy skills or revive
an eliminated faction. Rival planning proposes the same validated family and
continuity commands as the player. K14 payload migration adds empty K15 records
without advancing any RNG stream.

These are delegated implementation decisions, not newly confirmed author rules.
K15 validation and screenshots are recorded in
[the succession verification](verification/k15-succession.md).

### I16 — Factual heirlooms, chronicles and eras

K16 implements P23 without adding combat loot or inventing earlier history.
Each fresh founding Officer receives a named, non-combat Muster Sword whose
creation season is known. A custody transfer preserves its stable identity and
requires the living recipient to belong to the same faction and be at the current
holder's site. A living designated Item successor inherits that same item only
when physically present at the death site; otherwise the item stays as an estate
there for a local person to collect. Retirement does not transfer custody.

One factual custody deed is shared by the item, participating people and place,
and is private to the owning faction. Later custody deeds expire by bounded age
and count; current item custody remains. The same pruning removes successor and
completed-mentor references to a departed person while retaining the learner's
earned discipline seasons. Anniversary reminders select at most one eligible
service or site milestone per faction at each completed boundary and never repeat
the same subject and milestone.

Era labels use dated war declarations, peace agreements, active relations and
known site foundations. An older active war with no usable start date reports it
as unknown. Earlier saves get a current-round starting possession without a
fabricated deed, site foundation or war start. Defeat closes an active pair's war
date while retaining the historical diplomatic relation. These are delegated
implementation decisions under P23, not newly confirmed author rules. See
[the K16 verification](verification/k16-heirlooms.md).

### I17 — Authored production atlas and seeded founding

K17 keeps Rosemarch as a named four-faction fixture and adds a separate authored
production graph: 80 major markers, eight ten-site regions, 72 single-site
markers, and 152 physical sites connected by 191 explicit routes. Markers use
normalized placements on the supplied Kestrum atlas; crossings at the central
river are explicitly tagged as bridges. All eight neutral Village candidates
are simple world sites, have Horse Access, are at least four physical routes
apart, and have nearby wood and stone sources. The seed shuffles these candidates
and selects 4–8; two neutral adjacent sites receive one Bandit and one Wildlife
threat for every selected start.

The seed also assigns neutral settlement tiers with a 40/10/10/18/19/3 percent
distribution across Unsettled, Camp, Outpost, Hamlet, Village and Town. Other
sites begin without facilities or faction control. Every kingdom receives the
same economy grant, three full human formations and one age-24 Officer founder;
relations start at Peace. The player supplies the kingdom name and chooses one
botanical emblem. Rivals use distinct remaining emblems and names built from the
consumed human-name pool. These seeded weights and naming choices fill details
left open by P01; they do not grant hidden difficulty bonuses or fabricate
service history.

At world scale, the production atlas shows region names, faction emblems, compact
stationed-army counts, observed threats and selected site names; selecting any
marker opens its full name and facts. Regional maps show their ten site names.
This keeps the 80-node map readable while preserving the shared 48-pixel touch
targets. Production save validation compares fixed geography with
`world_layout.json`, while the prototype path continues to validate against
Rosemarch. These are delegated
implementation decisions under O01/O08/O24/O25 and P01, not new confirmed author
rules. Validation and map review are recorded in
[the K17 verification](verification/k17-production.md).

### I18 — Preserve a witnessed succession link

K18's full production run exposed a campaign failure when a mentor died while
their active pupil was the named heir to an item. A validated designation now
stores its accepted Martial or Religious link as witnessed evidence, so ending
the active mentorship does not invalidate a succession record that was legal
when made. Older designations without this field retain the existing live-link
validation; only designations accepted after normal command validation gain the
durable witness.
This preserves the named heir and item custody at the death site without
advancing mentorship or inventing a completed apprenticeship. This is a
delegated implementation decision under P22/P23, not a new confirmed author
rule. Regression evidence is recorded in
[the K18 verification](verification/k18-integrated.md).

### I19 — Keep phase-timing acceptance open when the tail exceeds the target

The isolated 8-faction/80-major-node/400-round replay measured NPC phase
planning at 64.112 ms median, 277.510 ms p95 and 452.659 ms maximum. The median is below the
provisional 250 ms review target; the p95 and maximum exceed it. K18 records both
instead of treating the median as proof that the full phase distribution meets
the target. Runtime command order remains unchanged, and `advance_npc` still
performs one atomic policy action per call. End-to-end frame pacing and the
minimum-WebGL browser remain unmeasured. This is a delegated implementation
decision about reporting and package status, not a new confirmed author rule.
See [the K18 verification](verification/k18-integrated.md).

### I20 — Reassign people when combat destroys their formation

A production-world battle exposed a wounded person still assigned to a
formation after combat cleanup removed its zero-headcount record. Combat cleanup
now moves remaining people to a surviving formation in the same army when one
exists, or to the army's battle site otherwise. Existing elder passengers who
are no longer field commanders continue to be placed at the site. This preserves
the wounded person's state and prevents later commands or saves from inheriting
a stale formation reference. The production regression validates the battle
result and exact save reload after 87 actual NPC actions; it does not claim a
production-AI victory. See
[the K18 verification](verification/k18-integrated.md).
