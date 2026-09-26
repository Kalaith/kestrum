# 04 — Armies and logistics

[Documentation index](README.md) · [Combat](05-battles-and-sieges.md) · [Characters](06-character-development.md)

## Military identity and scale

A kingdom should field a handful of recognizable armies. Commander, composition, veterans, shared service, battles, and relationships give each army an identity. Exact army limits are open; the design prefers meaningful forces over dozens of disposable stacks.

The concept draft described persistent units of approximately six individuals, such as Briar Company: Captain Serai, Brother Edrin, two squires, and two temple neophytes. The kingdom draft instead specifies an army of six troop-formation slots, with named people supporting much larger bodies of troops.

**Confirmed decision (D01):** each army has six discrete troop-formation slots with variable headcounts. The earlier six-person company remains a preserved concept example. Individuals may emerge within formations, but ordinary headcount is abstract.

## Army composition

Example Briar Host:

| Slot | Formation |
| --- | --- |
| 1 | Warriors |
| 2 | Archers |
| 3 | Riders |
| 4 | Clerics |
| 5 | Squires |
| 6 | Siege Engines |

Each slot holds a formation whose size depends on type:

| Source example | Full headcount |
| --- | ---: |
| Warriors | 100 |
| Archers | 80 |
| Heavy Cavalry | 40 |
| Siege Engines | 20 |
| Wyverns | 3 |
| Dragon | 1 |

These examples communicate scale; fantasy formations are future content under O24. Initial human formation capacities and costs are provisional values in [economy.json](../assets/data/economy.json). Siege-engine counts and their accompanying crews need a consistent definition before casualty rules are implemented.

The player chooses troop types, character placement, support, and siege equipment before combat. This preparation is a principal tactical choice. Magical forces and advanced classes are later content.

**Confirmed decision (O09):** formations and named characters may transfer whenever their armies share the same physical node. A region's world-map marker does not make different internal sites the same node. A safe settlement, supply connection, or fresh movement allowance is not required.

**Implementation baseline:** transfers cost no movement or time and are not restricted to the active faction's phase. Apply them between atomic actions, never partway through a resolving battle or round-end calculation; revalidate co-location when applied. Allow empty slots while enforcing six slots, ownership, and unique membership. Transferring never refreshes movement or action allowances already spent. Exact movement bookkeeping belongs to O02.

## Named characters as force multipliers

**Confirmed decision (O06):** an army can exist without a named character and can keep operating after a leader dies or retires. It is substantially weaker. Roughly 50% effectiveness without a named character is a tuning idea, not a fixed formula.

Multiple important characters should improve an otherwise identical army. Concentrating them increases strength; distributing them improves coverage. Exact stacking, role compatibility, command appointment, and whether effects apply to one slot or the entire army are open. The first prototype must make the contribution visible and avoid runaway multiplication.

A newly emerging character may appear within a formation as “Squire Elian + Warriors” and can later leave Commander Teresa's Briar Host to lead the Frostmarch Guard. That transfer reduces Briar Host's concentration of talent. Appointment eligibility remains a mechanic to define; transfers follow O09's shared-node rule.

## Formation persistence

Formations maintain headcount across battles. Warriors at 100/100 may return at 72/100. They can recover if supply and other requirements permit, and may carry experience from their service.

Veterancy can progress from Warriors to Seasoned Warriors to Veteran Warriors. Restrained improvements may affect effectiveness, morale, resilience, discipline, and rout chance. Repeated survival provides more credible opportunities for individual emergence.

**Confirmed decision (D05):** at zero headcount the formation is destroyed. Its formation history and veteran identity are gone; a newly recruited formation of the same type is a new formation.

The wider world can still remember battles involving the lost formation, but there is no requirement to retain its own service archive. Those world events cannot restore the formation or pass its veteran benefits to replacements.

**Confirmed decision (O16):** reinforcement has no effect on a surviving formation's veterancy or specialization. Veteran Warriors at 21/100 remain fully veteran when restored to 100/100. Replacements neither dilute nor grant experience. At zero headcount, D05 still destroys the formation; recruitment cannot revive it.

## Formation specialization

Repeated service can create eligibility for a different troop type:

| Existing force | Evidence | Possible specialization |
| --- | --- | --- |
| Warriors | Repeated fortress defense | Shield Guard |
| Warriors | Repeated combat against cavalry | Pikemen |
| Riders | Extensive scouting and pursuit | Light Cavalry |

Experience creates eligibility; resources and facilities still determine whether the kingdom can support the change. Conversion cost, whether it is automatic or requested, and whether it changes capacity are open. Preserve identity and history when a surviving formation legitimately specializes.

## Bonds and shared service

Characters and troops that serve together may develop bonds. Initial effects should remain subtle. Later possibilities include morale, reduced rout chance, improved support, recovery, coordinated action, and one character saving another.

The purpose is army identity. A large relationship-management interface is outside the intended core. **Proposal:** begin with shared-service records and contextual biography entries before adding numerical bond effects.

## Movement

Armies use connected routes and a movement allowance. Composition, roads, terrain, and siege equipment affect speed; weather may matter later. Costs and movement amounts require prototyping.

**Proposal — movement order:** select army, preview destination and route, inspect movement cost and known supply consequences, confirm, then resolve the legal path. Hostile contact interrupts travel for encounter resolution. If a route becomes invalid, explain the interruption and retain the army at its last valid location.

Do not promise exact information beyond fog of war. The preview can identify known threats and uncertainty separately.

## Roads

Roads improve movement between nodes and help invaders as well as defenders. Damaging or destroying them is possible but should be harder than burning fields or destroying temporary structures.

Under confirmed decision D06, constructing a road improves an existing route. Road condition affects travel and potentially supply when that rule is defined. The current design keeps the graph fixed.

## Simple supply

**Agreed direction (O03):** an army with a valid supply connection can recover health and troop strength normally. A cut-off army cannot normally recover health or headcount and can be worn down by successive attacks. Initial logistics avoids detailed food inventories and supply-wagon control.

**Proposal — connectivity model:** supply queries follow usable routes to an eligible friendly source. Hostile-controlled nodes, blockades, contested boundaries, and siege isolation must be evaluated consistently at both world and regional scales. Cache results only if control and route changes invalidate them correctly.

**Open:** define valid sources, treatment of contested nodes, range limits, neutral passage, and supply through occupied territory. Recovery resolves at round end; provisional rate and cost are in [economy defaults](03-kingdoms-and-economy.md#provisional-economy-defaults). An outpost should extend an existing logistical position; treating every isolated new outpost as unlimited independent supply would undermine encirclement and requires a separate decision.

The v1 rule is restricted normal recovery. Starvation, automatic attrition, and detailed siege supplies are later possibilities. Show “Cut off: cannot replenish” at the selected army and identify a known broken route when possible.

## Outposts

Armies can establish outposts on suitable nodes. **Agreed direction (O12):** establishment takes roughly three turns. With the full-round seasonal boundary confirmed, this means approximately three seasonal progress steps, about nine months. This pacing must be tested; the first progress step, completion timing, supply needs, and interruption rules remain open.

Outposts create territorial presence, a forward recovery point, a future settlement seed, a defensive position, and supply extension. They are persistent places. They may become hamlets, villages, fortified settlements, and towns, or become abandoned, ruined, and occupied by bandits.

**Proposal — order states:** planned → under construction → complete; interruption can suspend or cancel the work according to an explicit policy. Show remaining seasons and why progress is blocked. Use O11's provisional cost/refund defaults; occupation and interruption rules still need definition. The source's “Village → Fort → Town” example describes changing habitation and fortification together; those remain separate state layers.

## Stacking and reinforcement

**Agreed direction (O05):** multiple friendly armies may occupy one node without a hard stacking limit. V1 has no planned direct stacking penalty. The cost is the territory left undefended elsewhere.

The source does not settle whether every co-located army automatically participates in a battle. Define deterministic participation and relief rules in the [combat system](05-battles-and-sieges.md); do not hide an arbitrary one-army limit behind the six-slot rule.

## Proposed invariants

- Slot count is at most six; headcount stays between zero and capacity.
- Destroyed formations cannot recover, specialize, or return through a simple type match.
- A character transfer preserves identity, biography, equipment ownership, and relationships.
- A lost leader does not implicitly delete surviving troops or invalidate every army action.
- Supply, movement, roads, and entrance mappings use the same world geography.
- Construction advances once per season, regardless of the number of factions.
- Roads and physical facility loss affect the player and AI through the same rule checks.
