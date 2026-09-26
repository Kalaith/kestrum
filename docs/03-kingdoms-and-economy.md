# 03 — Kingdoms and economy

[Documentation index](README.md) · [Armies and logistics](04-armies-and-logistics.md) · [Decisions](13-decisions-and-open-questions.md)

## Founding a kingdom

The campaign starts as the player's kingdom forms. Other kingdoms are forming around the same world. Initial setup asks for a kingdom name, emblem, and total number of factions. More elaborate origins, traits, cultures, and starting bonuses are possible later; early identity should emerge mainly through play.

**Confirmed decision (O08):** the player chooses 4–8 total factions, including their own kingdom. This produces 3–7 rival kingdoms. Setup must label the total clearly; a default count has not been selected.

Initial play is single-player. Human-controlled additional kingdoms are future scope. Document state ownership so that possibility remains understandable, but do not build networking for the prototype.

## Victory, defeat, and continuity

**Agreed direction (O07):** victory requires every other faction to be conquered or vassalised. Initial diplomacy supports War and Peace. The founding drafts place vassals in future diplomacy, leaving the first version's victory scope to reconcile.

**Open (D04):** how vassalisation fits the initial version. The discussion suggests making it a defeat outcome without a full diplomacy system. That remains a proposal; the earlier conquest-only implementation plan was also an interpretation, not a confirmed restriction. Define whether and when a defeated kingdom can become a vassal, and distinguish that outcome from elimination. A playable version must have a reachable victory using its supported actions.

For v1, an eliminated faction is gone. Restoration wars, exiled governments, claimant factions, breakaway successor states, civil wars, separatist commanders, rebel kingdoms, succession disputes, and kingdom splintering are future scope. Individual displaced people may still have historical records; that does not restore their former faction as an active state.

**Open:** define conquest precisely, including last settlement lost, surviving field armies, isolated garrisons, and capital capture. Likewise decide the player's defeat rule, any surrender action, and treatment of defeated survivors. These must use consistent player and AI rules.

## Pressure to grow

Remaining small is not punished by an artificial deadline. Rival kingdoms expand, gather resources, field armies, develop places, and produce experienced characters. The pressure is to grow enough to keep the kingdom viable.

Periods of peace still offer development, resource gathering, road and outpost construction, army training and reorganization, character reassignment, border preparation, local threat clearing, and nurturing future specialists.

## Economy

The initial explicit resources are **Gold, Wood, and Stone**. They support recruitment, upkeep, settlement development, roads, outposts, fortification, and specialist facilities. Population may act as a soft military-growth limit. Food availability, trade, safety, and population can influence development without becoming additional detailed stockpile games.

| Resource or capacity | Source purpose | Proposed initial presentation |
| --- | --- | --- |
| Gold | Recruitment, upkeep, and development | Available balance and expected seasonal change |
| Wood | Roads, outposts, construction, and military support | Cost beside the chosen action |
| Stone | Construction and fortification | Cost beside the chosen action |
| Population | Potential soft limit on recruitment and growth | Local recruiting pressure, once modeled |
| Horses and rare opportunities | Access to specialist forces | Requirement or access status at the recruiting site |
| Infrastructure | Training and class opportunities | Physical site and whether access is currently valid |

Exact costs, income rates, recovery costs, and population conversion rules are open. Avoid inventing fixed numbers before the military loop establishes demand.

**Proposal — economic contract:** validate ownership, access, available resources, and prerequisites before accepting an order. Rejected orders do not deduct resources. Show recurring upkeep separately from immediate cost. An upkeep deficit needs an explicit policy; armies must not silently disappear or gain negative headcount.

## Light settlement management

A settlement may have one active development focus:

- Growth.
- Fortification.
- Troop training.
- Gold production.
- Wood production.
- Stone production.

The player can also establish an outpost, encourage settlement, invest in trade, move administration, establish a temple, secure a road, or resettle refugees as suitable systems become available. These are high-level interventions; the player does not place individual houses or blacksmiths.

**Working interpretation:** focus and investment influence [development pressure](07-living-places.md), rather than guarantee an immediate settlement-tier upgrade. Conditions still determine whether a place grows or declines. Focus rules must respect geography and access: an isolated, repeatedly raided town should not become prosperous merely because Growth is selected.

## World-based military capability

Advanced forces require resources, facilities, experience, and appropriate people. Examples:

| Capability | Required opportunity from the sources |
| --- | --- |
| Cavalry | Horses and appropriate facilities |
| Mages | An arcane institution |
| Clerics | A temple or religious center |
| Dragon Knights | Suitable character, martial/riding background, dragon-related experience and opportunity, and dragon infrastructure such as a hatchery |

This replaces a conventional abstract technology tree where practical. If the kingdom loses its only Dragon Hatchery, it cannot train new Dragon Knights until another is built or captured.

**Proposal:** existing specialists retain their recorded class when infrastructure is lost. New training is blocked, and any effect on replenishing rare mounts must be decided explicitly. Access should identify the supplying site so that a warning can explain “Hatchery access lost at Ashmere.” Ownership alone may be insufficient when the route is cut; that access rule is open.

## Diplomacy and long wars

Initial diplomatic states are War and Peace. Alliances, marriages as diplomatic agreements, tribute, a full vassal diplomacy system, guarantees, negotiated borders, and prisoner exchanges are future extensions. Whether vassalisation can first appear as a defeat outcome remains D04. Family relationships can exist without implementing all diplomatic marriage mechanics.

Temporary peace or truces can divide a historical conflict into separate active campaigns. The chronicle may group related wars into an era. **Open:** determine how a v1 peace offer is evaluated and whether a truce has a fixed duration. Narrative siege negotiation remains future scope while diplomacy stays minimal.

## AI parity

**Agreed direction (O21):** AI kingdoms follow the same underlying game rules for resources, troop requirements, settlement systems, movement and supply, character development, classes, and sieges. Difficulty must not fabricate unavailable units or bypass world requirements.

Difficulty may increase gold/resource income. Exact bonuses, their visibility, and difficulty presets remain open. Recruitment efficiency was another source possibility, not an additional confirmed bonus.

**Proposal — AI decision sequence:** assess threats and supply, protect important anchors, recover or retreat weak forces, choose feasible recruitment and development, select an offensive or local threat, then execute validated orders. Use the same action validation as player commands. Persistent strategic objectives can reduce repeated reversals without requiring an elaborate political simulation.

The remaining strategic design question is how AI evaluates war, peace offers, and truces. Knowledge records and information freshness still need an implementation policy consistent with the agreed rules; the clarification does not specify an exact AI observation algorithm.

## Fog of war and intelligence

**Agreed direction (O21):** the player receives little to no information about enemy armies until meeting them in combat. Any pre-combat presence or rough-size cue must preserve that limited knowledge. Combat can reveal troop types, composition, commanders, notable characters, and approximate strength. Scouts and spies remain later possibilities.

**Proposal — knowledge records:** store what a faction observed, when, and where. Distinguish unknown, observed, and last-known information. A force seen years ago should not be shown as currently exact. A biography or event notification must not accidentally reveal a hidden commander, location, or army composition.

## Proposed acceptance examples

1. An unaffordable recruitment or development order explains its shortage without changing state.
2. Losing the only required facility blocks new specialist training for both player and AI.
3. Observing an army reveals only the configured knowledge level; a later battle adds legitimate detail.
4. Removing a faction neither accelerates the calendar nor leaves it taking active turns.
5. A complete conquest victory remains reachable using the diplomacy supported by that version.
