# 07 — Living places

[Documentation index](README.md) · [World graph](02-world-time-and-control.md) · [History](09-history-and-content.md)

## Persistent places

**Implemented direction (O18–O19):** places grow, decline, recover and retain identity; capital and headquarters roles can move. Layered state, seasonal development pressure, damage, occupation, population migration, reclamation and relocation commands are implemented. [development.json](../assets/data/development.json) provides current thresholds and costs. The map does not yet express much of this change clearly; improving that is planned work.

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

The table preserves the founding vocabulary; it is not a list of implemented
tiers. Current habitation runs from Unsettled through Major City, military state
is None or Fort, and capital/headquarters are independent faction roles. Focus,
facilities, population, structural/fort damage, occupation, development pressure
and ruin state are separate facts. Multiple conditions can coexist. Citadels,
arcane institutions and the full civil-identity taxonomy remain future content.

## Growth from conditions

Growth comes mainly from favorable conditions held long enough: safe roads, food surplus, nearby population, protection, political importance, trade, religious importance, and stability. The player creates opportunities rather than pressing a button that guarantees the next city tier.

Development pressure can combine positive safety, trade, food, population, and administrative role with negative war damage, isolation, and raiding. Source plus/minus signs show relative direction, not a numerical equation.

**Implemented rule:** seasonal snapshots supply accumulated growth or decline pressure and eligible tier changes. Geography caps, population capacity, safety, supply, trade, food, capital role, focus, damage and occupation affect the result. Separate thresholds avoid immediate tier oscillation. The numerical defaults are tunable data, not unresolved implementation decisions.

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

Current decline follows implemented damage, isolation, occupation and population conditions. Famine, plague and broader political causes are source possibilities for future systems; they must not be narrated as events that already occurred without supporting state.

Example: Hawthorn City is besieged for three years, population collapses, Hawthorn Ruins remains, and bandits occupy the outer district. Later an expedition clears them, Hawthorn Hold is established, and slow resettlement begins. The rebuilt place retains its earlier history.

Ruins can host bandits, military camps, memorials, or reclaimed settlements; monsters and more elaborate uses are later content. Ruination preserves the node and current world state. Under O23, older stories and former identities may eventually be forgotten; permanent detailed place history is not required.

## War damage

Battles and occupation can create structural damage, population loss, supply disruption, damaged roads, destroyed fortifications, refugee pressure, and unrest. A border contested for twenty years should look different from untouched heartland.

**Implemented contract:** military and development consequences update authoritative site state once; round-end development consumes the resulting conditions. Reopening reports cannot apply damage again. Repair and reclamation use explicit conditions rather than restoring a location simply because an inspector was closed.

## Occupation

Winning control does not instantly produce cultural stability. A new occupation may face unrest, resistance, low supply, sabotage, and population flight. Long-term control can normalize gradually. This creates costs beyond the battle itself.

Occupation is an implemented numeric condition with seasonal stabilization; a garrison and safe conditions affect its decay, and occupation can reduce income and development. Broader civilian policies, rebellion and breakaway factions remain outside current scope. Existing ordinary local threats are distinct from an implied full resistance simulation.

## Refugees and population movement

Devastation can push population toward safer neighboring nodes. That can create growth, labor shortages at the origin, unrest at the destination, cultural mixing, and new recruits. One town's destruction may contribute to another city's rise.

**Implemented baseline:** aggregate population migration and explicit resettlement use legal destinations, route/range checks, capacity, costs and conserved arrivals. Outpost settlers also come from existing population. [Migration](../src/engine/development/migration.rs) and development commands own those rules; report viewing cannot create population. Cultural mixing and expanded refugee narratives are future content.

## Names, capitals, and local memory

Names can evolve: Red Plain → Red Plain Outpost → Redplain → Redplain Citadel. Places may be renamed for commanders, battles, noble houses, religious events, or new factions. Locals can remember an older name after conquest.

**Implementation baseline:** keep a stable ID and current display name. Retained events may carry dated names and link to that place, but old aliases and narrative records can expire under [history retention](09-history-and-content.md#bounded-history-and-forgetting). A missing old name must not invalidate current map references.

A new capital may be safer, wealthier, more central or more connected. Capital and headquarters relocation are separate implemented orders with costs, habitation/condition requirements and a headquarters cooldown in development data. The old place retains its identity, infrastructure and history; role changes do not create free population or a second headquarters bonus.

Compact place histories can show retained founding, fortification, sieges, growth, sack, and reclamation records. Selecting Hawthorn might show “Founded Year 6; survived three sieges; last captured Year 41,” with a visible path to available history. O23 permits older details to disappear, so a ruin may eventually be known only as a ruin.

## Evolving map presentation

**Current gap:** place growth, economic value and much damage are visible mainly
through inspection and management text. Faction-letter circles do not distinguish
a village from a major city. The region view also reuses the continental atlas,
so visible scenery does not reliably explain a local bridge or settlement.

**Planned:** the [map playability plan](map-playability-plan.md) gives settlements
recognizable development levels, independent fortifications, resource/facility
cues, construction progress and persistent damage/ruin states. The capital must
read as a capital; an important crossing must align with the river and legal
road. Safe heartland and a contested frontier should be visibly different using
real state. Contextual overlays expose supply or development pressure when they
help the current decision.

Early and developed campaigns should therefore look different for reasons the
player can explain. Keep changes attached to fixed sites and routes under D06;
new art cannot imply a new facility, road, resource or combat bonus. Distinct
regional topology and local geography are also planned; cosmetic variety alone
does not resolve the repeated regional chain.

## Acceptance cases

1. A protected, connected outpost can develop while a comparable isolated site remains small.
2. Sustained war damage can cause decline; reclaiming a ruin preserves its ID and current state even when older narrative history has expired.
3. Fortification and settlement size change independently, including a small heavily defended site.
4. Occupation and refugee movement affect neighboring places without duplicating population or history.
5. Renaming or relocating a capital preserves old references and does not reset development.
6. The normal map visibly distinguishes undeveloped land, a developed settlement, a fortified settlement, active work and a ruined place without requiring their management screens.
