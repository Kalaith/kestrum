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

**Confirmed decision (O01):** the world map has 80 major nodes; regional internal sites are additional. The current authored production layout has 72 simple locations and eight regions of ten physical sites each, for 152 physical sites and 191 routes. Seeded setup assigns campaign contents to this geography. Each campaign preserves its graph and stable IDs while site contents change.

**Confirmed decision (D06):** a new settlement develops on an existing site, and a new road improves an existing connection. Visual density can increase substantially without changing graph topology. Persistent node and route identities support this rule.

## Headquarters and expansion

The player begins from a headquarters or equivalent core territory. It initially supplies recruits, military units, supplies, promotions, training, and command structure. Expansion follows connected routes, creating additional safe settlements and logistical positions.

Headquarters is a functional role. The capital is a separate, relocatable seat of government. Capital capture alone does not cause defeat: the implemented defeat check considers surviving settled holdings and military strength. See [kingdom outcomes](03-kingdoms-and-economy.md#victory-defeat-and-continuity).

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

**Implemented boundary contract:** each traversable regional boundary records its external route and internal endpoint. Movement validates that mapping in both directions. An army has exactly one physical location at a time. The world map aggregates its regional presence; it does not create a second copy of the army. World orders include internal travel to the departure gate, resolve a known accessible entrance and preserve each member's movement budget. Entering a regional view changes inspection scope, not army location.

### Prototype region

Rosemarch is the retained small authored scenario used by regression tests and captures. New Game uses the production world. The original 8–12-site regional sketch describes scale; it is not a requirement to repeat one local layout throughout the world.

### Planned geographic variety

All eight current production regions use the same ten-site chain, two entrances
and anchor pattern, with repeated local coordinates. Geography types and
development caps vary, but their route choices repeat. The
[map playability plan](map-playability-plan.md) replaces that repetition with
authored regional identities: branching approaches, defensible crossings,
resource concentrations and different relationships between an objective and
its supply route. Each region must present a distinct decision, not merely a new
name or terrain cost.

This is planned content and presentation work. Keep the confirmed 80-major-node
scale, fixed graph during a campaign, valid physical entrances and deterministic
movement. New geography needs revision-selected topology validation before
content changes; a layout-revision bump alone is insufficient. Existing saves
retain their established graph, routes and locations. The current world/region views reuse the
continental background; meaningful local terrain and roads remain plan work.

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

### M01 overview presentation

Political fill and boundaries describe the known claim; the site's ring describes
its current controller. Separate occupation and contested marks preserve local
exceptions. A regional marker combines discovered local control without asserting
that every internal site is secure. A partially explored region withholds its
political claim and displays an unknown-claim cue, distinct from known neutral land.

The territorial layer reads observer-filtered summaries and clips its display to
known geography. The world view additionally uses authored land/water masks in
`map_presentation.json`; regional fill uses the existing local coordinates.
The masks describe the atlas artwork and change no routes, terrain mechanics,
site identities, anchor rules or saved topology. M03 owns regional geography.

World-region danger marks aggregate already-visible internal local threats,
participant sieges and hostile-contact sites. Their counts describe known
conditions, never enemy armies, troop strength or orders. Attention entries focus
the specific discovered site, entering its region when needed, without moving a
force. The owned capital has a crown; undisclosed foreign capitals remain private.

## Strategic anchors

Regions can define important control requirements rather than demand every minor node. The source example for Rosemarch requires:

- Rosemarch City.
- High Fort.
- At least one external supply route.

Sufficient anchor control can transfer political ownership while isolated hostile forces remain. This supports surrounded armies, pockets of resistance, guerrilla activity, counterattacks, and negotiated withdrawal. Rich guerrilla and negotiation systems remain later extensions of the basic partial-control rule.

**Implemented rule:** anchor requirements are data with explicit all-of and any-of conditions, reevaluated after relevant control, conflict or headquarters changes. A contested or blocked anchor is not securely held. A qualifying claimant receives political ownership; when nobody qualifies, the previous claim remains and the region is contested. A political claim never captures the other physical sites automatically.

## Fronts and multiple wars

Fronts arise from army positions, hostile neighbors, defended routes, and supply. A kingdom can invade a province while defending a border, suppressing resistance, reinforcing a fortress, and counterattacking elsewhere.

The western Ashford approach may threaten player headquarters while an eastern force advances through Hawthorn Province toward the enemy capital. Scarce experienced people cannot be everywhere. Front lines also shift over decades; a famous frontier may eventually sit in the heartland.

## Faction turns and the calendar

Faction turns proceed sequentially: player, then NPC factions, then the round ends.

**Confirmed decision (O04/D03):** periodic effects resolve at the end of one full round through all active factions. That round advances one season. An individual faction turn is an action phase within the shared season; calendar advancement never depends on how many rivals remain.

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

Movement, combat, and control resolve as action consequences. Economy, recovery, construction progress, siege aging, settlement pressure, and biological time resolve at the end of the full round, followed by an automatic save. The order is implemented in [round.rs](../src/engine/round.rs); the [simulation chapter](11-simulation-and-data.md) describes its contract. Map presentation must not add a second timer or repeat these consequences.

## Seasonal rhythm

| Season | Source possibilities |
| --- | --- |
| Spring | Easier recruitment, improved movement, agricultural recovery |
| Summer | Main campaigning season, stronger supply production, more travel |
| Autumn | Harvest, strong logistics, preparations for winter |
| Winter | Slower movement, harsher sieges, attrition, fewer major offensives |

These are future possibilities, not current seasonal modifiers or a requirement for a weather simulator. The calendar and age tracking are implemented. Winter attrition must not silently replace the current supply rule of blocked normal recovery.

## Invariants and planned feedback

- A move must follow a connected, currently traversable route; preview cost and known danger.
- A regional entrance must resolve to a valid internal node and preserve the army's identity.
- Capturing a capital does not silently erase hostile armies in other nodes.
- Control badges and map marks distinguish occupied, contested, and politically owned territory.
- Advancing one full round changes time exactly once, including after faction elimination.
- History uses the same calendar for ages, wars, sieges, construction, and biographies.
- Routes remain legible at both map scales; unknown territory does not reveal hidden enemy details.
- Territorial display distinguishes political claim, occupation, contested control and withheld claims; a kingdom's boundary cannot imply ownership of hidden or hostile sites.
- World markers aggregate known regional threats, participant sieges and hostile contacts without revealing enemy rosters. Persistent route presentation remains M02 work.

## Current defaults and future scope

Routes are bidirectional, authored terrain costs and road adjustments determine travel, and member-owned movement budgets prevent transfer exploits. Regional control and round ordering are implemented defaults, not pending approvals. Naval travel and seasonal weather remain future scope. Regional variety and geographic readability follow the map plan; tune and test those changes without reopening settled campaign contracts.
