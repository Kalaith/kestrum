# 03 — Kingdoms and economy

[Documentation index](README.md) · [Armies and logistics](04-armies-and-logistics.md) · [Decisions](13-decisions-and-open-questions.md)

## Founding a kingdom

The campaign starts as the player's kingdom forms. Other kingdoms are forming around the same world. Initial setup asks for a kingdom name, emblem, and total number of factions. More elaborate origins, traits, cultures, and starting bonuses are possible later; early identity should emerge mainly through play.

**Confirmed decision (O08):** the player chooses 4–8 total factions, including their own kingdom. This produces 3–7 rival kingdoms. Setup labels the total and defaults to four in [campaign_rules.json](../assets/data/campaign_rules.json). It also displays the deterministic campaign seed.

The current campaign is single-player. Human-controlled additional kingdoms and networking remain future scope.

## Victory, defeat, and continuity

**Implemented outcome:** victory requires all other independent kingdoms to be resolved by annexation or submission. War, Peace, truces, pending defeat decisions and saved victory/defeat endings are implemented. Submission is a narrow defeat outcome; a full vassal diplomacy simulation remains future scope.

When the player defeats a rival, the kingdom view offers annexation or submission. A submitted kingdom keeps its recorded identity under a sovereign and leaves the independent turn order. These delegated mechanics are implemented; their earlier proposal status is not an approval gate. A playable version must have a reachable victory using its supported actions. The current production victory script fails its 240-round deadline, as recorded in the [implementation README](../README.md); implementation completion does not establish settled campaign balance.

For v1, an eliminated faction is gone. Restoration wars, exiled governments, claimant factions, breakaway successor states, civil wars, separatist commanders, rebel kingdoms, succession disputes, and kingdom splintering are future scope. Individual displaced people may still have historical records; that does not restore their former faction as an active state.

Defeat eligibility requires both no controlled, non-ruined site of Outpost tier or higher and no nonempty army. Losing the capital title alone is insufficient. [Diplomacy state](../src/state/diplomacy.rs) defines this check and [outcome resolution](../src/engine/diplomacy/outcomes.rs) applies persistent defeat decisions, survivor displacement, cancelled orders and inheritance. Preserve those rules while improving map warnings and end-turn priorities.

## Pressure to grow

Remaining small is not punished by an artificial deadline. Rival kingdoms expand, gather resources, field armies, develop places, and produce experienced characters. The pressure is to grow enough to keep the kingdom viable.

Periods of peace still offer development, resource gathering, road and outpost construction, army training and reorganization, character reassignment, border preparation, local threat clearing, and nurturing future specialists.

## Economy

The explicit resources are **Gold, Wood, and Stone**. They support recruitment, upkeep, settlement development, roads, outposts, fortification, and specialist facilities. Population supports settlement growth and conserved Outpost settlers. Food availability, trade, safety, and population influence development without becoming additional stockpile resources.

| Resource or capacity | Source purpose | Presentation target |
| --- | --- | --- |
| Gold | Recruitment, upkeep, and development | Available balance, actual last-season income, upkeep paid/due and recovery spending |
| Wood | Roads, outposts, construction, and military support | Cost beside the chosen action |
| Stone | Construction and fortification | Cost beside the chosen action |
| Population | Settlement growth, displacement and conserved settlers | Local population and development conditions; recruitment currently makes no population deduction |
| Horses and rare opportunities | Access to specialist forces | Requirement or access status at the recruiting site |
| Infrastructure | Training and class opportunities | Physical site and whether access is currently valid |

**Confirmed decision (O11):** use reasonable provisional economy values stored in JSON and adjust them during balance work. The values are tuning defaults, not unresolved design blockers or fixed final balance.

### Provisional economy defaults

[assets/data/economy.json](../assets/data/economy.json) is the authoritative balance table. Toolkit JSON loading and Kestrum schema validation feed the implemented income, upkeep, recruitment, construction and recovery systems. These are tunable working values, not unfinished implementation decisions.

- Each faction starts with 500 Gold, 200 Wood, and 150 Stone. Settlement income and a single headquarters bonus of 40 Gold, 15 Wood, and 10 Stone accrue per full round. The bonus requires control of the headquarters site and adds to its settlement income; a capital title creates no extra income by itself.
- Recruitment costs and full-formation upkeep are listed for human Warriors, Spearmen, Archers, Riders, Medics, and Siege Engines. Upkeep charges the listed Gold rate for each surviving formation, regardless of current headcount. Named people add no separate upkeep initially.
- Recruitment completes immediately at a valid recruiting site after validation. Population deductions/pressure are disabled initially; enable a separate population model after economy playtesting.
- Income resolves before upkeep. Pay what is available, clamp Gold at zero, and record any shortfall without carrying debt. A shortfall blocks new recruitment and normal recovery until a later round's upkeep is fully paid. Existing forces remain; the player may disband formations without refund to reduce upkeep. Prepaid construction continues subject to its own conditions.
- Supplied formations may recover up to 20% of capacity at round end, capped by missing headcount. Restoring a full capacity costs 50% of that troop type's recruitment Gold, proportionally rounded up for the actual headcount restored; no Wood or Stone is charged. Process by stable army ID then slot, restoring only what available Gold can cover. Recovery keeps veterancy unchanged under O16.
- Outposts, roads, forts, and focus changes use the JSON order costs. Pay construction costs when an order is accepted; cancellation before any progress refunds 100%, while cancellation after progress or voluntary disbanding refunds nothing. Construction explicitly blocks or cancels when supply, control, army or target requirements are lost.

These defaults apply equally to player and AI before explicit difficulty modifiers. Rejected orders spend nothing. Show immediate costs separately from recurring upkeep and explain deficits at the affected controls. Balance changes belong in the JSON rather than hard-coded Rust values.

## Light settlement management

A settlement may have one active development focus:

- Growth.
- Fortification.
- Troop training.
- Gold production.
- Wood production.
- Stone production.

The player can also establish an outpost, encourage settlement, invest in trade, move administration, establish a temple, secure a road, or resettle refugees as suitable systems become available. These are high-level interventions; the player does not place individual houses or blacksmiths.

**Implemented rule:** focus and local conditions influence [development pressure](07-living-places.md), rather than guarantee an immediate settlement-tier upgrade. Geography caps, supply, damage, occupation and safety constrain growth. The available commands and their costs live in the construction and development data and validation.

**M01 map presentation:** the compact lower strip shows current owned Gold, Wood
and Stone balances separately from the last completed season's actual income,
upkeep paid/due and recovery Gold spent. It names the completed season and shows
an upkeep shortfall when present. Before the first receipt it explicitly says
that no season has completed. These values are saved simulation receipts, not a
forecast of the next season or a promise that current balances match the closing
receipt after later spending.

Settlement silhouettes show habitation tiers, with local work, damage and
Wood/Stone cues at appropriate scale. Full accounts, facilities, population and
growth causes remain in Manage. M02 brings common actions and their costs into
the selected-place inspector, plus map-linked seasonal consequences. No economy
rules or additional resource stockpiles are introduced by the overview.

## World-based military capability

For the initial human roster, ordinary training and recruitment depend on appropriate sites and resources. The examples below include initial cavalry and future magical specialists. Mages, magical Clerics, and Dragon Knights are deferred under O24; all capabilities should depend on suitable resources, facilities, experience, and people:

| Capability | Required opportunity from the sources |
| --- | --- |
| Cavalry | Horses and appropriate facilities |
| Mages | An arcane institution |
| Clerics | A temple or religious center |
| Dragon Knights | Suitable character, martial/riding background, dragon-related experience and opportunity, and dragon infrastructure such as a hatchery |

This replaces a conventional abstract technology tree where practical. If the kingdom loses its only Dragon Hatchery, it cannot train new Dragon Knights until another is built or captured.

**Current human rule:** existing specialists retain their recorded class when infrastructure is lost. New recruitment, courses and mentorship validate the required local facilities, supply, condition and resource access. Future rare mounts and hatcheries need content-specific rules when that scope is requested; they do not block current work.

## Diplomacy and long wars

Current diplomatic states are War and Peace, with submission as a defeat outcome. Alliances, marriages as diplomatic agreements, tribute, a full vassal diplomacy system, guarantees, negotiated borders, and prisoner exchanges are future extensions. Family relationships already exist independently of diplomatic marriage mechanics.

Temporary peace and truces can divide a historical conflict into separate active campaigns. The current truce is four rounds; recent losses and relative military strength inform peace acceptance through [diplomacy.json](../assets/data/diplomacy.json) and the diplomacy query rules. Narrative siege negotiation remains future scope.

## AI parity

**Agreed direction (O21):** AI kingdoms follow the same underlying game rules for resources, troop requirements, settlement systems, movement and supply, character development, classes, and sieges. Difficulty must not fabricate unavailable units or bypass world requirements.

The current Normal difficulty uses a zero-percent AI income bonus in campaign data. Additional difficulty presets remain future tuning. Recruitment efficiency is a source possibility, not an implemented free bonus.

**Implemented AI baseline:** rivals choose legal orders for movement, military composition, economy, progression and politics through shared command validation. Their objectives and priorities are data-driven. Map improvements must expose known rival pressure without bypassing those rules or inventing omniscient information.

War, peace and truce decisions and dated knowledge are implemented. Their effectiveness and campaign pacing remain balance work; use the actual planners and current regression results as the baseline rather than treating the original questions as unanswered.

## Fog of war and intelligence

**Agreed direction (O21):** the player receives little to no information about enemy armies until meeting them in combat. Any pre-combat presence or rough-size cue must preserve that limited knowledge. Combat can reveal troop types, composition, commanders, notable characters, and approximate strength. Scouts and spies remain later possibilities.

**Implemented knowledge contract:** records preserve what a faction observed, when, and where; history and projections filter by observer. A force seen years ago is not currently exact intelligence. M01 danger badges aggregate already-known regional threats, participant sieges and contact sites. They reveal no hidden commanders, composition, orders or undiscovered sites, and their counts never represent enemy strength.

## Acceptance examples

1. An unaffordable recruitment or development order explains its shortage without changing state.
2. Losing the only required facility blocks new specialist training for both player and AI.
3. Observing an army reveals only the configured knowledge level; a later battle adds legitimate detail.
4. Removing a faction neither accelerates the calendar nor leaves it taking active turns.
5. A complete conquest victory remains reachable using the diplomacy supported by that version.
