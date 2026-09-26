# 07 — Living places

[Documentation index](README.md) · [World graph](02-world-time-and-control.md) · [History](09-history-and-content.md)

## Persistent places

**Agreed direction (O18–O19):** places grow, burn, decline, recover, change purpose, and outlive their founders; capitals can move as the world changes. A place retains its identity through those changes. Peace can encourage growth; prolonged war can leave scars that are still visible decades later. State representation, numerical thresholds, and relocation mechanics remain open.

Possible trajectories include Plain → Outpost → Hamlet → Village → Fortified Town → Regional Capital, and Fortified Town → Sacked → Ruins → Bandit Hold → Reclaimed Fort. These combine several state layers for readable storytelling; implementation should keep those layers distinct.

## Node state layers

| Layer | Source possibilities | Design meaning |
| --- | --- | --- |
| Geography | Plains, forest, coast, river crossing, hill, mountain pass, valley, marsh, island | Usually persistent; constrains useful development |
| Settlement | Unsettled, Camp, Outpost, Hamlet, Village, Town, City, Major City, Capital | Habitation and development |
| Military | None, Watchpost, Palisade, Fort, Stronghold, Citadel | Fortification independent of settlement size |
| Civil identity | Agricultural, military, religious, commercial, administrative, arcane, industrial, academic, frontier, criminal | What the place does and provides |
| Condition | Prosperous, stable, strained, occupied, damaged, devastated, abandoned, ruined, lawless | Current stress and viability |

A small settlement may hold a citadel; a large commercial city may be weakly defended. **Working interpretation (D09):** capital is a political role independent of settlement size, despite appearing at the end of a source growth ladder. Moving the government should not automatically change physical population or infrastructure.

**Open:** whether civil identities and conditions are exclusive categories or multiple simultaneous tags. Occupation and damage can coexist, so implementation should preserve both facts even if the UI chooses one leading summary.

## Growth from conditions

Growth comes mainly from favorable conditions held long enough: safe roads, food surplus, nearby population, protection, political importance, trade, religious importance, and stability. The player creates opportunities rather than pressing a button that guarantees the next city tier.

Development pressure can combine positive safety, trade, food, population, and administrative role with negative war damage, isolation, and raiding. Source plus/minus signs show relative direction, not a numerical equation.

**Proposal:** aggregate persistent local conditions once per season, accumulate development or decline pressure, then evaluate eligible changes. Require sustained conditions and suitable geography. Separate growth and decline thresholds can prevent a place switching between town and village every season. Numerical weights, timing, capacity, and change costs remain open.

High-level interventions include outposts, fortification, encouraging settlement, investing in trade, moving administration, temples, road security, and refugee resettlement. A settlement's active [development focus](03-kingdoms-and-economy.md) influences those conditions.

## Red Plain growth example

| Year | Place | Cause or role |
| --- | --- | --- |
| 1 | Red Plain | Unsettled plains |
| 4 | Red Plain Outpost | Military frontier position |
| 9 | Red Plain Hamlet | Population gathering around the garrison |
| 16 | Redplain | Fortified town and regional supply center |
| 29 | Redplain City | Major military and trade center |
| 42 | Redplain | Capital of the Northern March |

The frontier's outward movement makes the site safer and more central. These dates illustrate a possible history; they are not upgrade deadlines or guaranteed progression.

Not every site grows. Tiny villages, remote shrines, wilderness, and ruins should remain part of the world. Uneven development creates useful geographic differences.

## Decline and reclamation

Repeated attacks, lost trade routes, famine, plague, depopulation, political abandonment, a relocated frontier, and destroyed infrastructure can cause decline.

Example: Hawthorn City is besieged for three years, population collapses, Hawthorn Ruins remains, and bandits occupy the outer district. Later an expedition clears them, Hawthorn Hold is established, and slow resettlement begins. The rebuilt place retains its earlier history.

Ruins can host bandits, monsters, military camps, archaeology, memorials, religious sites, or reclaimed settlements. These uses are content possibilities of varying scope. Ruination must not remove the node or its past from the world.

## War damage

Battles and occupation can create structural damage, population loss, supply disruption, damaged roads, destroyed fortifications, refugee pressure, and unrest. A border contested for twenty years should look different from untouched heartland.

**Proposal:** military resolution emits explicit changes to the node and affected routes. Settlement simulation consumes those changes without independently applying the same damage twice. Short-term raid effects and long-term ruin should have distinct recovery conditions.

## Occupation

Winning control does not instantly produce cultural stability. A new occupation may face unrest, resistance, low supply, sabotage, and population flight. Long-term control can normalize gradually. This creates costs beyond the battle itself.

**Open:** exact stabilization rules, garrison requirements, civilian policies, and whether resistance begins as abstract disruption or physical forces. A minimal occupied condition can precede sophisticated rebellion; active breakaway factions remain outside v1.

## Refugees and population movement

Devastation can push population toward safer neighboring nodes. That can create growth, labor shortages at the origin, unrest at the destination, cultural mixing, and new recruits. One town's destruction may contribute to another city's rise.

**Proposal:** model aggregate groups initially, with source, destination, approximate size, and cause. Only promote individuals to deeper simulation when service, lineage, or politics makes them relevant. Migration must account for losses and arrivals consistently and should not create population merely because an event is reopened. Safe route selection, capacity, resettlement controls, and delay are open.

## Names, capitals, and local memory

Names can evolve: Red Plain → Red Plain Outpost → Redplain → Redplain Citadel. Places may be renamed for commanders, battles, noble houses, religious events, or new factions. Locals can remember an older name after conquest.

**Proposal:** keep a stable ID, current display name, and dated name history. Biographies can use the name appropriate to the event while linking to the same place.

A new capital may be safer, wealthier, more central, more connected, or politically important. The player may move the seat of government. The old capital remains a place with its own history. Relocation cost, prerequisites, and administrative effects need a decision.

Compact place histories can show founding, fortification, sieges, growth, sack, and reclamation. Selecting Hawthorn might show “Founded Year 6; survived three sieges; last captured Year 41,” with a visible path to the full chronology.

## Evolving map presentation

The early map can be sparse: empty land, crossroads, woods, hills, ancient ruins, and small settlements. Over decades, settlements, fortifications, important roads, regional capitals, and ruins make it visually denser.

Nested regions evolve too. A region beginning with Plains, Village, and Fort can later contain City, Citadel, Market Town, Monastery, Ruined Fort, and New Road. Under confirmed decision D06, these represent changed contents on fixed sites and improvements to existing routes.

## Proposed acceptance cases

1. A protected, connected outpost can develop while a comparable isolated site remains small.
2. Sustained war damage can cause decline; reclaiming a ruin preserves its ID and chronology.
3. Fortification and settlement size change independently, including a small heavily defended site.
4. Occupation and refugee movement affect neighboring places without duplicating population or history.
5. Renaming or relocating a capital preserves old references and does not reset development.
