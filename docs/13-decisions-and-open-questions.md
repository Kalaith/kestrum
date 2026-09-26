# 13 — Decisions and open questions

[Documentation index](README.md) · [Original sources](README.md#complete-source-preservation)

## How to use this register

This register makes ambiguity visible. A **working interpretation** is the proposed baseline used to keep these chapters coherent; it is not a new author-approved requirement. Preserve the source direction when changing it, record the reason, and update dependent chapters. Source examples and approximate values must not silently become hard-coded rules.

## Reconciliation register

| ID | Source tension | Working interpretation in this documentation | Confirmation needed |
| --- | --- | --- | --- |
| D01 | Concept uses roughly six characters per unit; kingdom design uses six troop slots per army | Six troop formations is the implementation baseline; earlier small companies remain concept examples | Confirm military scale before implementing composition |
| D02 | Concept tracks ordinary juniors; kingdom design can create a person only when a formation member stands out | Separate abstract members, tracked people, and recognized figures; support both emergence paths | Define when tracking and leader eligibility begin |
| D03 | Sequential faction turns and one strategic turn per season could age the world once per faction | A complete round through active factions advances one season | Confirm round boundaries and timing of all periodic systems |
| D04 | Conquest-or-vassal victory; vassals listed as future diplomacy | First functional victory is conquest-only; broader objective gains vassals when implemented | Confirm victory UI and exact conquest/elimination rule |
| D05 | Destroyed formation history is “lost”; world history should remain | Veteran identity and active benefits end; a compact chronicle can retain past service/destruction | Confirm archive retention and replacement behavior |
| D06 | Fixed world/subnode graph; living world shows new settlements and roads | Initial growth changes existing sites; road construction improves existing edges | Decide whether later graph expansion is needed |
| D07 | Roughly thirty important people; emergence examples use a faction's roster | Treat thirty as a soft active target per faction, pending confirmation; keep historical people | Confirm scope, active definition, curve, and treatment of retirees |
| D08 | Condition-driven growth; a chosen development focus | Focus and investment influence pressure and opportunity rather than purchase guaranteed tier changes | Set pressure, costs, and time thresholds |
| D09 | Capital appears as a settlement stage; capitals can move | Keep political capital role independent from physical settlement size | Decide relocation prerequisites and consequences |
| D10 | Wounds/captures are later combat scope; prototype includes commander wounded and assumed command | Limited leadership-trigger wound events can prove the loop before a full injury system | Define v1 wound consequences and recovery |
| D11 | Characters can become exiles/claimants; defeated factions are gone in v1 | Personal/historical displacement does not automatically spawn a restoration or claimant faction | Define surviving characters after elimination |
| D12 | Outpost construction is about three turns; each round is a season | Three seasonal progress steps, roughly nine months | Playtest pacing and first progress timing |
| D13 | Dragon Knight opportunities include drake/mentor/stable; kingdom rules add hatchery | Preserve both requirement sets as source possibilities; no final Boolean expression chosen | Decide mandatory versus alternative prerequisites |
| D14 | Supply means no normal recovery when cut off; winter/sieges mention attrition, disease, and supply consumption | Initial rule is recovery restriction; detailed attrition/civilian effects are separate staged features | Decide any later ongoing loss model explicitly |
| D15 | Sources use independent example dates and similar place/person names | Examples remain separate scenarios; persistent IDs must resolve actual content identities | Do not merge Thomas/Tomas or High Fort/Hawthorn labels implicitly |

## Source tuning targets

| Topic | Source target | Status |
| --- | --- | --- |
| Campaign factions | Recommended minimum 4, maximum 8 | Total versus rivals is unresolved |
| Army slots | Six troop formations | Current kingdom design; reconciled with concept by D01 |
| Formation scale | Warriors 100, Archers 80, Heavy Cavalry 40, Siege Engines 20, Wyverns 3, Dragon 1 | Illustrative headcounts |
| Leaderless army | Roughly 50% effectiveness | Tuning target; formula unresolved |
| Important roster | About thirty | Soft curve; scope unresolved under D07 |
| Outpost build | About three turns | D12 interpretation; costs/interruption unresolved |
| Strategic calendar | One season per strategic turn, four per year | D03 defines complete round |
| Campaign duration | Typically 20–50 years, potentially 60–100 | Experience target, no forced timer |
| Prototype region | About 8–12 nodes, two entrances, capital and fort | Initial test scale |
| Early characters | Three founders and six juniors in the fifty-year example | Illustration, not fixed setup |
| Age bands | 0–12, 13–16, 17–25, 26–40, 41–55, 56+ | Flexible by culture/species |
| Siege weakening | Five illustrative steps from prepared walls to vulnerability | No mandated five-turn victory |
| Stack count | No planned hard limit or direct penalty in v1 | Participation and resolution still need rules |

## Decisions needed before the first playable implementation

| ID | Question | Why it matters | Proposed starting point, where useful |
| --- | --- | --- | --- |
| O01 | How many world regions, simple nodes, and internal sites? | Pathfinding, pacing, readability | Small authored graph and one source-sized region first |
| O02 | What are movement budgets, costs, multi-edge rules, and boundary costs? | Strategic reach and encounter timing | One explicit route preview shared with execution |
| O03 | What provides supply and what blocks it? | Encirclement, recovery, outpost value | Friendly connected sources with consistent regional routing |
| O04 | When do periodic effects resolve and count their first step? | Fairness across faction order | Single round boundary; document order before testing |
| O05 | What resolves a battle, a stalemate, and multiple stacked armies? | Combat foundation and bounded execution | Small explainable deterministic resolver before depth |
| O06 | What is a named/qualified leader and how do multiple characters contribute? | Six-slot army balance and progression | Distinguish tracking, recognition, and command qualification |
| O07 | How is a faction conquered and the player defeated? | Achievable end state | Decide territory and surviving-army treatment together |
| O08 | Does 4–8 mean total factions or rivals? | Setup wording and world generation | Proposal: total count, default four; awaiting confirmation |
| O09 | Where and when may formations or people transfer? | Prevent teleportation and mid-battle exploits | Safe shared node with explicit cost/timing |
| O10 | Which battle facts and medical events create progression? | Earliest emergent-character proof | Source ten-event vocabulary with evidence references |

## Decisions for kingdom and generational expansion

| ID | Question | System affected |
| --- | --- | --- |
| O11 | Income/upkeep amounts, deficits, recruitment population pressure, and refunds? | Economy and army persistence |
| O12 | Construction interruption, supply access, and newly completed facility timing? | Outposts and specialist recruitment |
| O13 | Fort advantage, weakening, sortie/escape/relief participation, retreat destinations? | Sieges and multi-army battles |
| O14 | Recognition thresholds, emergence curve, trait contradictions/decay, and anti-farming limits? | Character development |
| O15 | Class expression semantics, training time, switching, and loss of infrastructure? | Opportunity-based progression |
| O16 | Reinforcement impact on veterancy and specialized formation identity? | Veteran preservation |
| O17 | Seasonal modifiers, weather, illness, casualties, and physical aging rates? | Pacing and career length |
| O18 | Settlement condition representation, growth limits, refugee routing, occupation stabilization? | Living places |
| O19 | Capital movement, political ownership reversal, and administrative roles? | Territorial strategy |
| O20 | Household creation, succession eligibility, mentorship duration/capacity, and property? | Generations |
| O21 | AI information limits, peace evaluation, and transparent difficulty bonuses? | Fairness and kingdom competition |
| O22 | Save boundaries, migration compatibility, autosaves, and data versioning? | Long-campaign reliability |
| O23 | Information freshness, biography discovery, eras, event retention, performance budgets? | History and fog of war |
| O24 | Species/cultures, naming data, class roster, visual art, and audio? | Content and presentation |
| O25 | Final viewport support and confirmation of proposed touch targets? | UI implementation and visual acceptance |

## Added proposals to evaluate

The source drafts do not specify a Rust entity schema, action transaction model, exact seasonal-resolution order, graph boundary contract, UI screen layout, save migration policy, or automated test plan. Chapters 10–12 and marked sections elsewhere propose those details to make the design implementable. They should be validated against actual toolkit capabilities and prototype behavior.

No gameplay implementation decision is considered complete solely because it appears in this documentation. When resolved, add the chosen rule and reason here and update the owning system chapter. Preserve unresolved alternatives in the original source references.
