# 13 — Decisions and open questions

[Documentation index](README.md) · [Original sources](README.md#complete-source-preservation)

## How to use this register

The discussion clarification supplied on 2026-09-26 resolves several ambiguities left by the founding drafts. This register distinguishes **confirmed decisions**, **agreed directions with mechanics or tuning still open**, and **unanswered questions**. Confirmed decisions guide implementation; an unresolved formula does not reopen the underlying design.

Original D and O identifiers are retained so earlier references remain useful. A **working interpretation** or **proposal** is still a suggested implementation choice. In particular, vassalisation as a defeat outcome is a suggestion from the discussion, not a confirmed rule. Approximate values remain tunable. Update the owning chapter when a remaining choice is settled; preserve the founding references unchanged.

## Confirmed decisions

| ID | Confirmed rule | Remaining implementation detail and primary chapter |
| --- | --- | --- |
| D01 | Each army has six discrete troop-formation slots. Headcounts vary by troop type, such as 100 warriors or one dragon. The earlier six-person company does not define army capacity. | Exact troop capacities and roster are content/tuning work; [armies](04-armies-and-logistics.md) |
| D02 | Ordinary formation members remain abstract until someone distinguishes themselves and becomes a visible character attached to the formation. A plausible recent history can be generated retrospectively; simulation from birth is unnecessary. | Emergence thresholds and event vocabulary remain O10/O14; sparse juniors or family records can still serve the generational design; [characters](06-character-development.md) |
| D05 | At zero headcount, the formation is destroyed and its formation history and veteran identity are gone. A replacement is a new formation. The wider world can still remember battles. | No separate destroyed-formation archive is required. Reinforcement of a surviving formation is O16; [formation persistence](04-armies-and-logistics.md#formation-persistence) |
| D06 | Major nodes and internal subnodes are generated at world creation and remain fixed. Settlements and other contents change on those sites; roads improve existing connections. | Graph size and generation constraints remain O01; [world graph](02-world-time-and-control.md) |
| O06 | Armies can operate without named characters at substantially reduced effectiveness. Multiple named characters strengthen the same army, creating a choice between concentration and coverage. People can emerge within formations and later move into leadership. | Roughly 50% leaderless effectiveness is a tuning idea. Contribution formulas, appointments, and transfers still need mechanics under O05/O09/O14; [named characters](04-armies-and-logistics.md#named-characters-as-force-multipliers) |
| O08 | The player chooses 4–8 total factions, including the player's kingdom. | This means 3–7 rivals; the setup default is not selected; [kingdom setup](03-kingdoms-and-economy.md#founding-a-kingdom) |

## Agreed directions with mechanics or tuning still open

These are implementation questions within an established design. They should not be presented as wholly unanswered features.

| ID | Agreed direction | What remains open | Primary chapter |
| --- | --- | --- | --- |
| O02 | Armies have movement speed; roads increase it for defenders and invaders alike. | Movement points, terrain and road costs/bonuses, multi-edge travel, regional boundary costs | [Armies](04-armies-and-logistics.md#movement) |
| O03 | Cut-off armies cannot normally recover health or headcount. | Eligible supply sources, graph tracing, contested/neutral passage, range, recovery timing and cost | [Supply](04-armies-and-logistics.md#simple-supply) |
| O05 | Friendly armies can stack without a hard limit. Entering an enemy-occupied node triggers automatic combat based on composition, characters, and other relevant factors. | Resolver, leader contribution formulas, stalemates, participation and casualty allocation across multiple armies | [Battles](05-battles-and-sieges.md) |
| O07 | Eliminated factions are gone in v1. The victory objective is conquest or vassalisation of all other factions. | Capital versus settlement requirements, surviving armies, player defeat, and how vassalisation fits War/Peace diplomacy (D04) | [Victory](03-kingdoms-and-economy.md#victory-defeat-and-continuity) |
| O12 | Establishing an outpost takes roughly three turns. | Attack/interruption policy, builder presence, supply requirement, costs, refunds, and first/completion progress timing; the seasonal interpretation implies about nine months | [Outposts](04-armies-and-logistics.md#outposts) |
| O13 | Defenders begin with an advantage that weakens during a siege. They can sortie, try to escape, or receive reinforcements; relief armies can attack besiegers. | Advantage and weakening formulas, joint versus ordered encounters, retreat destinations, and edge cases | [Sieges](05-battles-and-sieges.md#persistent-siege) |
| O14 | Hero emergence is semi-rare, easier with few named heroes, and progressively rarer toward roughly 20–30 per faction. Veteran formations and meaningful experiences improve opportunities. | Probability curve, recognition thresholds, roster counting/retirees, trait evidence and decay, anti-farming rules | [Characters](06-character-development.md#emergence-from-formations) |
| O15 | Experience and opportunity create class eligibility, supported by world resources and infrastructure. Dragon training needs appropriate infrastructure; awarding Teresa a recovered egg creates a path through play. | Exact prerequisite expressions, training time, switching, and interrupted access | [Classes](06-character-development.md#class-eligibility-and-opportunities) |
| O17 | Typical campaigns span about 20–50 years, allowing aging careers, changed roles, and retirement. | Seasonal modifiers, aging checks, illness, death probabilities, and species-specific rates | [Generations](08-generations-and-succession.md) |
| O18 | Settlements grow or decline with conditions; occupation and refugees change places over time. | State representation, numerical pressures, growth limits, refugee routing, and stabilization | [Living places](07-living-places.md) |
| O19 | Capitals can move as places change in safety, wealth, connectivity, and political importance. | Relocation prerequisites/costs, administrative effects, and political ownership reversal | [Capitals](07-living-places.md#names-capitals-and-local-memory) |
| O20 | Families, children, mentorship, and several kinds of heirs carry continuity; legacy does not require children. | Household and succession rules, mentorship duration/capacity, inheritance, and property | [Succession](08-generations-and-succession.md) |
| O21 | Players have little to no enemy-army information before combat. AI follows the same underlying game rules; difficulty may increase gold/resource income rather than create impossible units. | Primarily how AI evaluates war, peace, and truces; exact difficulty bonuses and visibility remain tuning work. AI knowledge representation is an implementation detail, not permission to bypass the agreed rules. | [AI and intelligence](03-kingdoms-and-economy.md#ai-parity) |

## Unanswered questions

These still need explicit choices. Their milestone dependencies differ; they are not all prerequisites for the first prototype.

| ID | Question | Why it matters / existing proposal |
| --- | --- | --- |
| O01 | What is the exact world-map size and number of major nodes/internal sites? | The target feel is a small Stellaris map. A small authored graph and one roughly 8–12-node test region are prototype proposals, not final world counts. |
| O04 | Where and in what order do all periodic effects resolve? | Player → NPC 1 → NPC 2 is agreed. One full round per season and a single resolution boundary remain the documented working interpretation/proposal. |
| O09 | Where, when, and at what movement/time cost can formations and named people transfer between armies? | Transfers are allowed; safe shared-node transfer is a proposal that needs a rule. |
| O10 | What exact event vocabulary drives character development? | Experience-based development is agreed. The ten source event examples are a candidate list, not a finalized implementation vocabulary. |
| O11 | What are income, upkeep, recruitment costs, deficit behavior, population pressure, and refund rules? | Economy values and failure policies determine army persistence and expansion. |
| O16 | How do replacements affect a surviving formation's veterancy and specialization? | If Veteran Warriors at 21/100 recover to 100/100, do they remain fully veteran? No dilution, retention, or replacement formula has been chosen. |
| O22 | What are save boundaries, slots, autosaves, migration compatibility, and long-campaign data versioning? | Chapter 11 offers technical proposals; the discussion has not decided a persistence policy. |
| O23 | How much history is retained, and what performance limits apply over decades? | Presentation direction is known; storage, evidence retention, information freshness, biography discovery, and era-generation budgets remain unspecified. D05 already settles destroyed-formation history. |
| O24 | What is the initial troop/class roster, species/cultures, naming data, art, and audio? | Source examples illustrate possibilities without fixing the launch content. |
| O25 | What resolutions and input targets are supported, including minimum viewport and tap sizes? | Touch accessibility and visual philosophy are established; chapter 10's numerical viewport/target sizes remain proposals. |

**Next design priority: O16.** Reinforcement changes the value of preserving a badly depleted veteran formation and therefore its identity. Resolve this before accepting veterancy and recovery behavior; this update selects no answer.

## Remaining reconciliation and source-handling notes

D01, D02, D05, and D06 are resolved above. The other original D identifiers retain their narrower purposes below.

| ID | Status | Current direction or interpretation | Remaining decision |
| --- | --- | --- | --- |
| D03 | Working interpretation | A complete round through active factions advances one season; individual faction phases do not each age the world. | Confirm calendar boundary and periodic order under O04. |
| D04 | Open scope reconciliation | Conquest/vassalisation victory and early War/Peace diplomacy both remain intended. The earlier conquest-only plan was an interpretation, not a settled restriction. | Evaluate the discussion's proposal to make vassalisation a defeat outcome without a full diplomacy system; define its timing and relationship to elimination under O07. |
| D07 | Agreed direction; tuning open | Emergence responds to the faction's named roster and becomes rarer toward roughly 20–30 heroes. This is a soft curve. | Define counted roles, active status, retirees, and probabilities under O14; historical dead figures should not block new generations. |
| D08 | Working interpretation of agreed settlement direction | Focus and investment influence pressure and opportunity instead of purchasing guaranteed tier changes. | Set pressure, costs, and time thresholds under O18. |
| D09 | Working interpretation of agreed capital direction | The political capital role is independent from physical settlement size. | Define relocation prerequisites and consequences under O19. |
| D10 | Working interpretation | Limited leadership-trigger wound events can prove the prototype before a full injury system. | Define initial wound consequences and recovery. |
| D11 | Agreed v1 boundary; survivor rules open | Personal or historical displacement does not automatically create a restoration or claimant faction. | Define surviving characters after elimination under O07. |
| D12 | Agreed duration target; calendar interpretation | About three turns to establish an outpost would mean three seasonal progress steps, roughly nine months, under D03. | Playtest pacing and define first progress/interruption rules under O12. |
| D13 | Agreed class philosophy; expression open | Dragon opportunities require suitable experience and infrastructure; egg, drake, mentor, stable, and hatchery examples provide candidate requirements. | Decide mandatory versus alternative prerequisites under O15. |
| D14 | Agreed initial supply rule | Loss of supply blocks normal recovery. Detailed attrition and civilian effects are separate staged possibilities. | Select any later ongoing loss model explicitly. |
| D15 | Source-handling convention | Independent example dates and similar names remain separate scenarios. | Assign explicit IDs to actual content; do not implicitly merge Thomas/Tomas or High Fort/Hawthorn examples. |

## Fixed choices and tuning targets

| Topic | Current rule or target | Status |
| --- | --- | --- |
| Campaign factions | Player chooses 4–8 total, including the player | Confirmed O08; default unspecified |
| Army slots | Six discrete troop formations | Confirmed D01; exact headcounts vary |
| Formation scale | Warriors 100, Archers 80, Heavy Cavalry 40, Siege Engines 20, Wyverns 3, Dragon 1 | Illustrative capacities |
| Leaderless army | Roughly 50% effectiveness | Tuning idea; formula unresolved |
| Important roster | Emergence increasingly rare toward roughly 20–30 named heroes per faction | Agreed soft curve; counted roles and probabilities open |
| Outpost build | About three turns | Agreed approximate duration; timing/interruption unresolved |
| Strategic calendar | One season per strategic turn, four per year | D03 interprets a strategic turn as a complete round; O04 still open |
| Campaign duration | Typically 20–50 years, potentially 60–100 | Experience target, no forced timer |
| Prototype region | About 8–12 nodes, two entrances, capital and fort | Initial test scale, not world size |
| Early characters | Three founders and six juniors in the fifty-year example | Illustration, not fixed setup |
| Age bands | 0–12, 13–16, 17–25, 26–40, 41–55, 56+ | Flexible by culture/species |
| Siege weakening | Five illustrative steps from prepared walls to vulnerability | No mandated five-turn victory |
| Stack count | No hard limit; no planned direct penalty in v1 | Agreed direction; battle participation and resolution open |

## Added proposals to evaluate

The source drafts and discussion do not settle a Rust entity schema, action transaction model, exact seasonal-resolution order, graph boundary contract, UI screen layout, save migration policy, or automated test plan. Chapters 10–12 and marked sections elsewhere propose those details to make the design implementable. Validate them against actual toolkit capabilities and prototype behavior.

A confirmed design rule is not evidence of implemented or tested gameplay. Track implementation and verification separately, and keep remaining mechanics explicit without reopening settled choices.
