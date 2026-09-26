# 02 — World, time, and control

[Documentation index](README.md) · [Kingdoms](03-kingdoms-and-economy.md) · [Living places](07-living-places.md)

## Strategic graph

The world is a network of major locations connected by valid routes. Armies cannot travel freely across the background. Nodes may represent cities, villages, forts, castles, mountain passes, bridges, ports, temples, resource sites, headquarters, regional capitals, wilderness, or other strategic positions.

```text
                  Northern Fort
                       |
Capital ---- Farmland ---- Rosemarch ---- Eastern Pass
                       |
                  River Settlement
```

The top level has the conceptual scope of a small Stellaris galaxy. Exact world size is open. Top-level nodes are fixed at generation. Internal nodes may also be generated at the start and generally retain a stable graph while their state changes.

**Working interpretation:** the initial implementation uses persistent node and route identities. A new settlement develops on an existing site, and a new road improves an existing connection. Dynamic creation of graph connections is future scope unless separately approved. Visual density can increase substantially without changing graph topology.

## Headquarters and expansion

The player begins from a headquarters or equivalent core territory. It initially supplies recruits, military units, supplies, promotions, training, and command structure. Expansion follows connected routes, creating additional safe settlements and logistical positions.

Headquarters is a functional role. The starting capital can later lose importance or be replaced as the seat of government. Losing a capital is not yet specified as instant defeat; [elimination rules](13-decisions-and-open-questions.md) require a decision.

## Nested regions

Some world nodes contain a local graph. Entering the region places the army on its internal map rather than capturing the whole region.

```text
World: Player Territory ---- Rosemarch ---- Enemy Territory
                                |
                           Southern Road

Inside Rosemarch:
                    Northern Gate
                         |
                   Hawthorn Fort
                    /         \
               Milltown      Old Road
                   |             |
             Western Gate --- Rosemarch City
                                 |
                            Southern Pass
```

Rosemarch is simultaneously one major world location and an internal campaign. Internal maps support concentration, army splitting, multiple advances, bypasses, reinforcement, supply attacks, withdrawal, chokepoint defense, and defending captured ground.

### Entry points

Each external route can connect to a specific internal node. Arrival from the north may use North Gate; arrival from the west uses West Gate; arrival from the south uses South Pass. Geography changes the invasion, even when the regional destination is the same.

**Proposal — boundary contract:** each traversable regional boundary records its external route and internal endpoint. Movement validates that mapping in both directions. An army has exactly one physical location at a time. The world map aggregates its regional presence; it does not create a second copy of the army. Movement cost across scales must be previewed as one route cost, with no free jump through the interior.

### Prototype region

The original concept calls for roughly 8–12 internal nodes, two entrances, one regional capital, one fortress, and several minor locations. Additional simple world nodes connect player and enemy headquarters. Use a small authored region to prove the rules before pursuing a large generated world.

## Occupancy, control, and ownership

These concepts need separate treatment:

| Concept | Meaning |
| --- | --- |
| Occupancy | Which armies are physically at a node |
| Local control | Who currently holds a particular site; conflict may make it contested |
| Regional political ownership | Which kingdom claims or administers the region after anchor requirements are met |
| Stability | How successfully occupation has become normal civil control |

The first two emerge from movement and warfare. Political ownership can change before all enemies are cleared. Stability changes over time through the [living-world system](07-living-places.md).

Example: the player holds Western Gate and Milltown, High Fort is contested, and the enemy holds Rosemarch City and Southern Pass. A single solid player/enemy color cannot fully communicate this state.

## Strategic anchors

Regions can define important control requirements rather than demand every minor node. The source example for Rosemarch requires:

- Rosemarch City.
- High Fort.
- At least one external supply route.

Sufficient anchor control can transfer political ownership while isolated hostile forces remain. This supports surrounded armies, pockets of resistance, guerrilla activity, counterattacks, and negotiated withdrawal. Rich guerrilla and negotiation systems remain later extensions of the basic partial-control rule.

**Proposal:** define anchor requirements as data with explicit all-of and any-of conditions. Reevaluate them after a resolved control or route change. A contested anchor does not count as securely held. Losing an anchor should expose contested status immediately; whether it instantly reverses political ownership or requires sustained control is open.

## Fronts and multiple wars

Fronts arise from army positions, hostile neighbors, defended routes, and supply. A kingdom can invade a province while defending a border, suppressing resistance, reinforcing a fortress, and counterattacking elsewhere.

The western Ashford approach may threaten player headquarters while an eastern force advances through Hawthorn Province toward the enemy capital. Scarce experienced people cannot be everywhere. Front lines also shift over decades; a famous frontier may eventually sit in the heartland.

## Faction turns and the calendar

Source design specifies sequential faction turns and recommends one strategic turn per season. These need one unambiguous clock.

**Working interpretation:** one complete round through all active factions advances one season. An individual faction turn is that faction's action phase within the shared season. Calendar advancement never depends on how many rivals remain.

```text
Spring, Year 1
Player -> Faction 1 -> Faction 2 -> ... -> round resolution
Summer, Year 1
Player -> Faction 1 -> Faction 2 -> ... -> round resolution
```

| Duration | Complete rounds |
| --- | --- |
| One year | 4 |
| One decade | 40 |
| 20 years | 80 |
| 40 years | 160 |
| 50 years | 200 |
| 60–100 years | 240–400 |

Typical campaigns target 20–50 years. Longer histories of 60–100 years should be supported without making hundreds of years necessary. Starting characters need time to become meaningful before the campaign outlives them.

**Proposal — resolution boundary:** action consequences such as movement, combat, and control resolve during the acting faction's turn. Economy, recovery, construction progress, siege aging, settlement pressure, and biological time each resolve once per complete round in a defined order. The [simulation chapter](11-simulation-and-data.md) proposes that order and records unresolved fairness choices.

## Seasonal rhythm

| Season | Source possibilities |
| --- | --- |
| Spring | Easier recruitment, improved movement, agricultural recovery |
| Summer | Main campaigning season, stronger supply production, more travel |
| Autumn | Harvest, strong logistics, preparations for winter |
| Winter | Slower movement, harsher sieges, attrition, fewer major offensives |

These are possible light modifiers, not a requirement for a weather simulator. The calendar and age tracking can ship before seasonal modifiers. Winter attrition is future tuning and must not silently replace the initial supply rule of blocked normal recovery.

## Proposed invariants and feedback

- A move must follow a connected, currently traversable route; preview cost and known danger.
- A regional entrance must resolve to a valid internal node and preserve the army's identity.
- Capturing a capital does not silently erase hostile armies in other nodes.
- Control badges and map marks distinguish occupied, contested, and politically owned territory.
- Advancing one full round changes time exactly once, including after faction elimination.
- History uses the same calendar for ages, wars, sieges, construction, and biographies.
- Routes remain legible at both map scales; unknown territory does not reveal hidden enemy details.

## Open decisions

World and region counts, procedural-generation constraints, route directionality, terrain costs, naval travel, movement budgets, control reversal timing, and round-resolution order require prototyping. See [decisions D03, D06, D12, and O01–O04](13-decisions-and-open-questions.md).
