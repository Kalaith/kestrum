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
